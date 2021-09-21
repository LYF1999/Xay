use bytes::Buf;
use bytes::Bytes;
use bytes::BytesMut;
use futures::Sink;
use futures::SinkExt;
use futures::Stream;
use futures::TryStreamExt;
use pin_project::pin_project;
use std::marker::PhantomData;
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Framed};

use crate::proxy::Proxy;
use crate::BoxError;

#[pin_project]
pub struct Duplex<I, O, Err> {
    #[pin]
    input: I,
    out: O,
    _marker: PhantomData<fn(Err)>,
}

impl<O, Err> Duplex<Framed<TcpStream, BytesCodec>, O, Err> {
    pub fn create(conn: TcpStream, proxy: O) -> anyhow::Result<Self> {
        let codec = BytesCodec::new();

        let framed = Framed::new(conn, codec);

        Ok(Duplex {
            input: framed,
            out: proxy,
            _marker: PhantomData,
        })
    }
}

impl<I, O, Err, InErr> Duplex<I, O, Err>
where
    O: Proxy<Err> + Unpin,
    I: Stream<Item = Result<BytesMut, InErr>> + Sink<Bytes, Error = InErr> + Unpin,
    <I as Sink<Bytes>>::Error: Into<BoxError>,
    Err: Send + 'static + Into<BoxError>,
    InErr: Into<BoxError>,
{
    pub async fn run(self) -> Result<(), BoxError> {
        let mut input = self.input;
        let mut output = self.out;

        loop {
            tokio::select! {
                r = output.try_next() => {
                    match r {
                        Ok(buf) => if let Some(buf) = buf {
                            if let Err(err) = input.send(buf).await {
                                return Err(err.into())
                            };
                        },
                        Err(err) => return Err(err.into()),
                    }
                }
                r = input.try_next() => {
                    match r {
                        Ok(buf) => if let Some(mut buf) = buf {
                            let buf = buf.copy_to_bytes(buf.len());
                            if let Err(err) =  output.send(buf).await {
                                return Err(err.into())
                            };
                        },
                        Err(err) => return Err(err.into()),
                    }
                }
            }
        }
    }
}
