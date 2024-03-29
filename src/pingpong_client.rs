use pingpong::pingpongcs::client::{PingPongClient, PingPongClientHandler};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "http://[::1]:50051".to_string();
    let handler = Arc::new(PingPongClientHandler::default());
    let client = PingPongClient::new(addr, handler);
    client.launch_service().await
}
