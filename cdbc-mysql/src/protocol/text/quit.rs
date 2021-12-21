use cdbc::io::IoEncode;
use crate::protocol::Capabilities;

// https://dev.mysql.com/doc/internals/en/com-quit.html

#[derive(Debug)]
pub(crate) struct Quit;

impl IoEncode<'_, Capabilities> for Quit {
    fn encode_with(&self, buf: &mut Vec<u8>, _: Capabilities) {
        buf.push(0x01); // COM_QUIT
    }
}
