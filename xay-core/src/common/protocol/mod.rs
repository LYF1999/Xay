use bytes::BufMut;

pub type RequestCmd = u8;

pub type SecurityType = i32;

pub struct MemoryUser<A> {
    pub email: String,
    pub level: u32,
    pub account: A,
}

pub struct RequestHeader<A> {
    pub version: u8,
    pub req_cmd: RequestCmd,
    pub option: u8,
    pub security: SecurityType,
    pub port: u16,
    pub addr: std::net::IpAddr,
    pub user: MemoryUser<A>,
}

pub struct Id {
    pub uuid: [u8; 16],
    pub cmd_key: [u8; 16],
}
