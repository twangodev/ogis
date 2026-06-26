use super::{registry::Registry, WireError};
use crate::{params::OgParams, templates::TemplateMap};
pub fn pack_body(_p: &OgParams, _reg: &Registry, _t: &TemplateMap) -> Result<Vec<u8>, WireError> { todo!() }
pub fn unpack_body(_b: &[u8], _reg: &Registry, _max: usize) -> Result<OgParams, WireError> { todo!() }
