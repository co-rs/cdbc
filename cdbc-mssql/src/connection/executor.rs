use cdbc::describe::Describe;
use cdbc::executor::{Execute, Executor};
use crate::connection::prepare::prepare;
use crate::protocol::col_meta_data::Flags;
use crate::protocol::done::Status;
use crate::protocol::message::Message;
use crate::protocol::packet::PacketType;
use crate::protocol::rpc::{OptionFlags, Procedure, RpcRequest};
use crate::protocol::sql_batch::SqlBatch;
use crate::{
    Mssql, MssqlArguments, MssqlConnection, MssqlQueryResult, MssqlRow, MssqlStatement,
    MssqlTypeInfo,
};
use either::Either;
use std::borrow::Cow;
use std::sync::Arc;
use cdbc::Error;
use cdbc::io::chan_stream::{ChanStream, TryStream};

impl MssqlConnection {
    fn run(&mut self, query: &str, arguments: Option<MssqlArguments>) -> Result<(), Error> {
        self.stream.wait_until_ready()?;
        self.stream.pending_done_count += 1;

        if let Some(mut arguments) = arguments {
            let proc = Either::Right(Procedure::ExecuteSql);
            let mut proc_args = MssqlArguments::default();

            // SQL
            proc_args.add_unnamed(query);

            if !arguments.data.is_empty() {
                // Declarations
                //  NAME TYPE, NAME TYPE, ...
                proc_args.add_unnamed(&*arguments.declarations);

                // Add the list of SQL parameters _after_ our RPC parameters
                proc_args.append(&mut arguments);
            }

            self.stream.write_packet(
                PacketType::Rpc,
                RpcRequest {
                    transaction_descriptor: self.stream.transaction_descriptor,
                    arguments: &proc_args,
                    procedure: proc,
                    options: OptionFlags::empty(),
                },
            );
        } else {
            self.stream.write_packet(
                PacketType::SqlBatch,
                SqlBatch {
                    transaction_descriptor: self.stream.transaction_descriptor,
                    sql: query,
                },
            );
        }

        self.stream.flush()?;

        Ok(())
    }
}

impl Executor for MssqlConnection {
    type Database = Mssql;

    fn fetch_many<'q, E: 'q>(
        &mut self,
        mut query: E,
    ) -> ChanStream<Either<MssqlQueryResult, MssqlRow>>
        where
            E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        chan_stream! {
            self.run(sql, arguments)?;

            loop {
                let message = self.stream.recv_message()?;

                match message {
                    Message::Row(row) => {
                        let columns = Arc::clone(&self.stream.columns);
                        let column_names = Arc::clone(&self.stream.column_names);

                        r#yield!(Either::Right(MssqlRow { row, column_names, columns }));
                    }

                    Message::Done(done) | Message::DoneProc(done) => {
                        if !done.status.contains(Status::DONE_MORE) {
                            self.stream.handle_done(&done);
                        }

                        if done.status.contains(Status::DONE_COUNT) {
                            r#yield!(Either::Left(MssqlQueryResult {
                                rows_affected: done.affected_rows,
                            }));
                        }

                        if !done.status.contains(Status::DONE_MORE) {
                            break;
                        }
                    }

                    Message::DoneInProc(done) => {
                        if done.status.contains(Status::DONE_COUNT) {
                            r#yield!(Either::Left(MssqlQueryResult {
                                rows_affected: done.affected_rows,
                            }));
                        }
                    }

                    _ => {}
                }
            }

            Ok(())
        }
    }

    fn fetch_optional<'q, E: 'q>(
        &mut self,
        query: E,
    ) ->  Result<Option<MssqlRow>, Error>
        where
            E: Execute<'q, Self::Database>,
    {
        let mut s = self.fetch_many(query);
        while let Some(v) = s.try_next()? {
            if let Either::Right(r) = v {
                return Ok(Some(r));
            }
        }
        Ok(None)
    }

    fn prepare_with<'q>(
        &mut self,
        sql: &'q str,
        _parameters: &[MssqlTypeInfo],
    ) -> Result<MssqlStatement<'q>, Error> {
        let metadata = prepare(self, sql)?;

        Ok(MssqlStatement {
            sql: Cow::Borrowed(sql),
            metadata,
        })
    }

    fn describe<'q>(
        &mut self,
        sql: &'q str,
    ) -> Result<Describe<Self::Database>, Error>
    {
        let metadata = prepare(self, sql)?;

        let mut nullable = Vec::with_capacity(metadata.columns.len());

        for col in metadata.columns.iter() {
            nullable.push(Some(col.flags.contains(Flags::NULLABLE)));
        }

        Ok(Describe {
            nullable,
            columns: (metadata.columns).clone(),
            parameters: None,
        })
    }
}
