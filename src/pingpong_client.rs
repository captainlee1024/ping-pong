use tonic::transport::Channel;

pub mod ping_pong {
    tonic::include_proto!("pingpong");
}

use ping_pong::{PongResponse, PingRequest, ping_pong_client::PingPongClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "http://[::1]:50051"; // 添加协议方案
    let channel = Channel::from_static(addr)
        .connect()
        .await?;

    let mut client = PingPongClient::new(channel);

    let request = PingRequest {
        message: "Ping".into(),
    };

    let response = client.ping(request).await?;

    println!("Response received: {:?}", response);

    Ok(())
}
