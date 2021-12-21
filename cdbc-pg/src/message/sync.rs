use cdbc::io::IoEncode;

#[derive(Debug)]
pub struct Sync;

impl IoEncode<'_> for Sync {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) {
        buf.push(b'S');
        buf.extend(&4_i32.to_be_bytes());
    }
}
