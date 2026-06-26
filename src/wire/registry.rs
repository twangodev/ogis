pub struct Registry;
impl Registry {
    pub fn load() -> &'static Registry { todo!() }
    pub fn template_id(&self, _n: &str) -> Option<u16> { todo!() }
    pub fn template_name(&self, _id: u16) -> Option<&str> { todo!() }
    pub fn color_id(&self, _n: &str) -> Option<u16> { todo!() }
    pub fn color_name(&self, _id: u16) -> Option<&str> { todo!() }
}
