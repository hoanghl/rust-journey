pub struct Address {
    pub ip: String,
    pub port: u32,
}

impl Address {
    pub fn to_str(self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
