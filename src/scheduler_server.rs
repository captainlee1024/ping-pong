use pingpong::pingpongcs::server::{launch_scheduler_server_with_addr, PingPongServiceHandler};
use std::sync::Arc;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".to_string();
    let ping_pong_handler = Arc::new(PingPongServiceHandler::default());
    launch_scheduler_server_with_addr(addr, ping_pong_handler).await
}
