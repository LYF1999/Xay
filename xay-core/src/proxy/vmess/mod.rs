use std::{convert::TryFrom, fmt::Debug};

use bytes::{Bytes, BytesMut};
use convert_case::{Case, Casing};
use futures::{Sink, Stream};
use nix::sys::socket::InetAddr;
use pin_project::pin_project;
use serde::Deserialize;
use tokio_tungstenite::tungstenite::{
    handshake::client::Request,
    http::{header::HeaderName, HeaderValue},
};

use crate::transport::ws::WsTransport;

mod account;
mod session;

use super::crypto::Encrypt;

#[pin_project]
pub struct VMessProxy<C> {
    #[pin]
    conn: C,
    uuid: String,
    alter_id: Option<i32>,
    encrypt: Encrypt,
    target_addr: InetAddr,
}
#[derive(Deserialize, Debug, Clone)]
pub struct WSSettings {
    headers: fxhash::FxHashMap<String, String>,
    path: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum StreamSettings {
    #[serde(rename = "ws")]
    WS(WSSettings),
}

#[derive(Deserialize, Debug, Clone)]
pub struct VMessEndpoint {
    addr: String,
    uuid: String,
    alter_id: Option<i32>,
    encrypt: Encrypt,
    stream_settings: StreamSettings,
}

impl VMessProxy<WsTransport> {
    pub async fn new(endpoint: &VMessEndpoint, target_addr: InetAddr) -> anyhow::Result<Self> {
        let addr = &endpoint.addr;
        let ws_settings = {
            match endpoint.stream_settings {
                StreamSettings::WS(ref v) => v,
                _ => {
                    panic!("not supported network type")
                }
            }
        };

        let mut req = Request::default();
        *req.uri_mut() = format!("ws://{}{}", addr, ws_settings.path).parse()?;
        let headers = req.headers_mut();
        ws_settings.headers.iter().try_for_each(|(key, val)| {
            headers.append(
                &HeaderName::try_from(&key.to_case(Case::Camel))?,
                HeaderValue::try_from(val)?,
            );
            Ok::<_, anyhow::Error>(())
        })?;

        tracing::info!("connect to {:?} headers: {:?}", req.uri(), req.headers());
        let (conn, ..) = tokio_tungstenite::connect_async(req).await?;
        Ok(VMessProxy {
            conn: WsTransport::new(conn),
            uuid: endpoint.uuid.clone(),
            alter_id: endpoint.alter_id,
            encrypt: endpoint.encrypt.clone(),
            target_addr,
        })
    }
}

impl<C, Err> Stream for VMessProxy<C>
where
    C: Stream<Item = std::result::Result<Bytes, Err>>,
{
    type Item = std::result::Result<Bytes, Err>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        todo!()
    }
}

impl<C> Sink<Bytes> for VMessProxy<C>
where
    C: Sink<Bytes>,
{
    type Error = C::Error;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.conn.poll_ready(cx)
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        let is_aead = self.alter_id.is_none();

        todo!()
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.conn.poll_flush(cx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.conn.poll_flush(cx)
    }
}

impl<Err, C> super::Proxy<Err> for VMessProxy<C> where
    C: Sink<Bytes, Error = Err> + Stream<Item = std::result::Result<Bytes, Err>>
{
}
