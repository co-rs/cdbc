use cdbc::io::IoEncode;
use crate::protocol::Capabilities;

// https://dev.mysql.com/doc/internals/en/com-ping.html

#[derive(Debug)]
pub(crate) struct Ping;

impl IoEncode<'_, Capabilities> for Ping {
    fn encode_with(&self, buf: &mut Vec<u8>, _: Capabilities) {
        buf.push(0x0e); // COM_PING
    }
}
