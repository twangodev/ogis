use crate::wire::WireError;
pub const SIG_LEN: usize = 6;
pub fn sign(_secret: &[u8], _version: u8, _body: &[u8]) -> String { todo!() }
pub fn verify(_secret: &[u8], _version: u8, _body: &[u8], _seg: &str) -> Result<(), WireError> { todo!() }
