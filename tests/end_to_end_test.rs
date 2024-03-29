#[tokio::test]
async fn test_ping_stream_e2e() {
    // Start the server
    let server_addr = "[::1]:50051".to_string();
    let server_future = tokio::spawn(async move {
        pingpong::pingpongcs::server::launch_scheduler_server_with_addr(server_addr).await.unwrap();
    });

    // Give the server a little time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Run the client
    let client_addr = "http://[::1]:50051".to_string();
    let client_future = tokio::spawn(async move {
        pingpong::pingpongcs::client::run_with_addr(client_addr).await.unwrap();
    });

    // Wait for the client to finish
    let client_result = client_future.await;
    assert!(client_result.is_ok(), "Client encountered an error");

    // Stop the server
    server_future.abort();
}