use pingpong::pingpongcs::client::{PingPongClient, PingPongClientHandler};
use pingpong::pingpongcs::server::{launch_scheduler_server_with_addr, PingPongServiceHandler};
use std::sync::Arc;

#[tokio::test]
async fn test_ping_pong_stream_e2e() {
    // Start the server
    let server_addr = "[::1]:50051".to_string();
    let ping_pong_handler = Arc::new(PingPongServiceHandler::default());
    let server_future = tokio::spawn(async move {
        launch_scheduler_server_with_addr(server_addr, ping_pong_handler)
            .await
            .unwrap();
    });

    // Give the server a little time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Run the client
    let client_addr = "http://[::1]:50051".to_string();
    let handler = Arc::new(PingPongClientHandler::default());
    let client = PingPongClient::new(client_addr, handler);
    let client_future = tokio::spawn(async move {
        client.launch_service().await.unwrap();
    });

    // Wait for the client to finish
    let client_result = client_future.await;
    assert!(client_result.is_ok(), "Client encountered an error");

    // Stop the server
    server_future.abort();
}
