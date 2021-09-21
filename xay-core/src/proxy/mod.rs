use bytes::Bytes;
use futures::{Sink, Stream};
use serde::Deserialize;
pub mod crypto;
pub mod vmess;

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum NetworkType {
    #[serde(rename = "ws")]
    WS,
    #[serde(rename = "tcp")]
    TCP,
    #[serde(rename = "udp")]
    UDP,
}

pub trait Proxy<Err>:
    Stream<Item = std::result::Result<Bytes, Err>> + Sink<Bytes, Error = Err>
{
}
