use cdbc::io::Encode;
use crate::io::MssqlBufMutExt;
use crate::protocol::header::{AllHeaders, Header};

#[derive(Debug)]
pub struct SqlBatch<'a> {
    pub transaction_descriptor: u64,
    pub sql: &'a str,
}

impl Encode<'_> for SqlBatch<'_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) {
        AllHeaders(&[Header::TransactionDescriptor {
            outstanding_request_count: 1,
            transaction_descriptor: self.transaction_descriptor,
        }])
        .encode(buf);

        // SQLText
        buf.put_utf16_str(self.sql);
    }
}
