pub struct Host {
    pub name: String,
    pub port: u16,
}

impl Host {
    pub fn new(name: &str, port: u16) -> Self {
        Self {
            name: String::from(name),
            port,
        }
    }
}
 