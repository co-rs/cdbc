use cdbc::io::Encode;

pub struct AllHeaders<'a>(pub &'a [Header]);

impl Encode<'_> for AllHeaders<'_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) {
        let offset = buf.len();
        buf.resize(buf.len() + 4, 0);

        for header in self.0 {
            header.encode_with(buf, ());
        }

        let len = buf.len() - offset;
        buf[offset..(offset + 4)].copy_from_slice(&(len as u32).to_le_bytes());
    }
}

pub enum Header {
    TransactionDescriptor {
        // number of requests currently active on the connection
        outstanding_request_count: u32,

        // for each connection, a number that uniquely identifies the transaction with which the
        // request is associated; initially generated by the server when a new transaction is
        // created and returned to the client as part of the ENVCHANGE token stream
        transaction_descriptor: u64,
    },
}

impl Encode<'_> for Header {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) {
        match self {
            Header::TransactionDescriptor {
                outstanding_request_count,
                transaction_descriptor,
            } => {
                buf.extend(&18_u32.to_le_bytes()); // [HeaderLength] 4 + 2 + 8 + 4
                buf.extend(&2_u16.to_le_bytes()); // [HeaderType]
                buf.extend(&transaction_descriptor.to_le_bytes());
                buf.extend(&outstanding_request_count.to_le_bytes());
            }
        }
    }
}