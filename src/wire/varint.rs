use super::WireError;
pub fn write_varint(_out: &mut Vec<u8>, _v: u64) { todo!() }
pub fn read_varint(_input: &mut &[u8]) -> Result<u64, WireError> { todo!() }
pub fn read_bytes<'a>(_input: &mut &'a [u8], _n: usize) -> Result<&'a [u8], WireError> { todo!() }
