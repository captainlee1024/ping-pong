use pingpong::pingpongcs::server::launch_scheduler_server_with_addr;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".to_string();
    launch_scheduler_server_with_addr(addr).await
}
