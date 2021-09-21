use std::task;

use bytes::Bytes;
use futures::{ready, Sink, Stream};
use pin_project::pin_project;
use tokio::net::TcpStream;

pub type WsError = tokio_tungstenite::tungstenite::Error;
pub type Message = tokio_tungstenite::tungstenite::Message;

#[pin_project]
pub struct WsTransport {
    #[pin]
    conn: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
}

impl WsTransport {
    pub fn new(
        conn: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
    ) -> Self {
        WsTransport { conn }
    }
}

impl Sink<Bytes> for WsTransport {
    type Error = WsError;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.conn.poll_ready(cx)
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        let this = self.project();
        this.conn.start_send(Message::Binary(item.to_vec()))
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
        this.conn.poll_close(cx)
    }
}

impl Stream for WsTransport {
    type Item = Result<Bytes, WsError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        let msg = ready!(this.conn.poll_next(cx));
        match msg {
            Some(Ok(Message::Binary(v))) => task::Poll::Ready(Some(Ok(Bytes::from(v)))),
            Some(Err(e)) => task::Poll::Ready(Some(Err(e))),
            _ => todo!(),
        }
    }
}
