use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub name: String,
    pub privacy_options: Vec<Privacy>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Privacy {
    ShowName,
    HideName,
    AcceptConnections,
    RefuseConnections,
}

impl Session {
pub fn new(name: &str, privacy_options: Vec<Privacy>) -> Self {
    Self {
        name:String::from(name),
        privacy_options
    }
}
}