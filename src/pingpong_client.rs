use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub mod pingpong {
    tonic::include_proto!("pingpong");
}

use pingpong::{ping_pong_client, PingRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "http://[::1]:50051";

    // let channel = Channel::from_static(addr);

    // let mut client = pingpong::ping_pong_client::PingPongClient::new(channel);

    let mut client = ping_pong_client::PingPongClient::connect(addr).await?;

    // Create a channel for sending requests
    let (tx, rx) = mpsc::channel(10);

    // 握手
    tx.send(PingRequest {
        message: "ping".into(),
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

        tx.send(PingRequest {
            message: format!("ping{i}").into(),
        })
        .await?;
    }

    Ok(())
}
