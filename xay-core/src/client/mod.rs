use std::os::unix::prelude::AsRawFd;

use crate::config::Endpoint;
use crate::future_err::WithResultLog;
use crate::BoxError;
use crate::{config, proxy::vmess::VMessProxy};
use nix::sys::socket::InetAddr;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_stream::StreamExt;

mod codec;
mod duplex;

#[derive(Default)]
pub struct Server {}

impl Server {
    pub fn new() -> Server {
        Server::default()
    }

    pub async fn run(&self) -> Result<(), BoxError> {
        let config = config::get_config();
        let mut listener = TcpListenerStream::new(
            tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?,
        );

        let vmess_endpoint = {
            match config.endpoints[0] {
                Endpoint::VMess(ref v) => v,
                _ => {
                    panic!("not supported endpoint")
                }
            }
        };

        loop {
            match listener.try_next().await? {
                None => return Ok(()),
                Some(conn) => tokio::spawn(
                    async move {
                        let remote_addr = conn.peer_addr()?;
                        tracing::info!("handle connection for remote addr: {}", remote_addr);
                        let dest = nix::sys::socket::getsockopt(
                            conn.as_raw_fd(),
                            nix::sys::socket::sockopt::OriginalDst,
                        )?;
                        let inet = InetAddr::V4(dest);
                        let dest = inet.to_str();
                        tracing::info!("original dst: {:?}", dest);

                        let proxy = VMessProxy::new(vmess_endpoint, inet).await?;
                        let proxy_pipe = duplex::Duplex::create(conn, proxy)?;
                        proxy_pipe.run().await?;

                        Ok::<_, BoxError>(())
                    }
                    .with_log_err(),
                ),
            };
        }
    }
}
