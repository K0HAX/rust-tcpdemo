use std::net::SocketAddr;

pub struct UserData {
    pub addr: SocketAddr,
}

pub struct Request {
    pub data: String,
}

pub struct Response {
    pub msg: String,
}

