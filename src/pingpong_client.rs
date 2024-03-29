use pingpong::pingpongcs::client::run_with_addr;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "http://[::1]:50051".to_string();
    run_with_addr(addr).await
}
