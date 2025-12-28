//! Sandbox

#[derive(Debug, Clone)]
pub struct Sandbox {
    pub id: u32,
}

impl Sandbox {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}
