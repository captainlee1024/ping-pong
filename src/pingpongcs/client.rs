use mockall::automock;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::async_trait;
use tonic::transport::Channel;

pub mod pingpong {
    tonic::include_proto!("pingpong");
}

use pingpong::{ping_pong_client, PingRequest, PongResponse};

pub async fn run_with_addr(addr: String) -> Result<(), Box<dyn std::error::Error>> {
    // let channel = Channel::from_static(addr);

    // let mut client = pingpong::ping_pong_client::PingPongClient::new(channel);

    let mut client = ping_pong_client::PingPongClient::connect(addr).await?;
    let ping_pong_client = PingPongClient {
        client: client.clone(),
    };

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
        let ping_request = ping_pong_client.handle_ping(i, recv_msg).await;
        tx.send(ping_request).await?;
    }

    Ok(())
}

pub struct PingPongClient {
    pub client: ping_pong_client::PingPongClient<Channel>,
}

#[async_trait]
impl PingPongHandler for PingPongClient {
    async fn handle_ping(&self, i: i32, pong_response: PongResponse) -> PingRequest {
        if pong_response.message != "pong" {
            return PingRequest {
                message: format!("unexpect message {}", pong_response.message).into(),
            };
        }
        // TODO: update ping_pong's pr

        PingRequest {
            message: format!("ping{i}").into(),
        }
    }
}

#[async_trait]
#[automock]
pub trait PingPongHandler {
    async fn handle_ping(&self, i: i32, pong_response: PongResponse) -> PingRequest;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pingpong::PingRequest;
    use std::future;

    #[tokio::test]
    async fn test_handle_ping() {
        let mut mock = MockPingPongHandler::new();
        mock.expect_handle_ping().returning(|_, _| {
            Box::pin(future::ready(PingRequest {
                message: "mock ping".into(),
            }))
        });

        // 使用mock进行测试...
    }
}
