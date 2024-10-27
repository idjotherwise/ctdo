#[derive(Debug, Clone)]
pub struct Category {
    pub name: String,
    pub color: String,
}
impl Category {
    pub fn default() -> Self {
        Self {
            name: "General".to_string(),
            color: "White".to_string(),
        }
    }
}
