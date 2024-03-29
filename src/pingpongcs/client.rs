use mockall::automock;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::async_trait;

pub mod pingpong {
    tonic::include_proto!("pingpong");
}

use pingpong::{ping_pong_client, PingRequest, PongResponse};

pub struct PingPongClient {
    addr: String,
    pub ping_pong_client_handler: Arc<dyn MyPingPongClientHandler + Send + Sync>,
}

impl PingPongClient {
    pub fn new(
        addr: String,
        ping_pong_client_handler: Arc<dyn MyPingPongClientHandler + Send + Sync>,
    ) -> Self {
        PingPongClient {
            addr,
            ping_pong_client_handler,
        }
    }

    pub async fn launch_service(&self) -> Result<(), Box<dyn std::error::Error>> {
        // let channel = Channel::from_static(addr);

        // let mut client = pingpong::ping_pong_client::PingPongClient::new(channel);

        let mut client = ping_pong_client::PingPongClient::connect(self.addr.clone()).await?;
        let ping_pong_client_handler = self.ping_pong_client_handler.clone();

        // Create a channel for sending requests
        let (tx, rx) = mpsc::channel(10);

        // 握手
        tx.send(PingRequest {
            message: "registry".into(),
        })
        .await?;

        // 创建请求
        let request = ReceiverStream::new(rx);

        // 发送请求
        let response = client.ping_stream(request).await?;

        // 处理响应
        let mut inbound = response.into_inner();
        let mut i = 1;
        while let Some(recv_msg) = inbound.message().await? {
            println!("recv from server: {}", recv_msg.message);
            i += 1;
            if i > 3 {
                break;
            }

            // let ping_request = handle_pong(i).await;
            let ping_request = ping_pong_client_handler.handle_ping(i, recv_msg).await;
            tx.send(ping_request).await?;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct PingPongClientHandler {}

#[async_trait]
impl MyPingPongClientHandler for PingPongClientHandler {
    async fn handle_ping(&self, i: i32, pong_response: PongResponse) -> PingRequest {
        if pong_response.message != "pong" {
            return PingRequest {
                message: format!("unexpect message {}", pong_response.message),
            };
        }
        // TODO: update ping_pong's pr

        PingRequest {
            message: format!("ping{i}"),
        }
    }
}

#[async_trait]
#[automock]
pub trait MyPingPongClientHandler {
    async fn handle_ping(&self, i: i32, pong_response: PongResponse) -> PingRequest;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pingpong::PingRequest;
    use std::future;

    #[tokio::test]
    async fn test_handle_ping() {
        let mut mock = MockMyPingPongClientHandler::new();
        mock.expect_handle_ping().returning(|_, _| {
            Box::pin(future::ready(PingRequest {
                message: "mock ping".into(),
            }))
        });

        // 使用mock进行测试...
    }
}
