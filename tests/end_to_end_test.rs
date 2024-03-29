use pingpong::pingpongcs::client::{MyPingPongClientHandler, PingPongClient, PingPongClientHandler};
use pingpong::pingpongcs::server::{launch_scheduler_server_with_addr, PingPongHandler, PingPongServiceHandler};
use std::sync::Arc;
use tonic::async_trait;
use pingpong::pingpongcs::client::pingpong::{PingRequest as ClientPingRequest, PongResponse as ClientPongResponse};
use pingpong::pingpongcs::server::pingpong::{PingRequest as ServerPingRequest, PongResponse as ServerPongResponse};


#[tokio::test]
async fn test_ping_pong_stream_e2e() {
    // Start the server
    let server_addr = "[::1]:50051".to_string();
    let ping_pong_handler = Arc::new(MockPingPongServiceHandler::default());
    let server_future = tokio::spawn(async move {
        launch_scheduler_server_with_addr(server_addr, ping_pong_handler)
            .await
            .unwrap();
    });

    // Give the server a little time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Run the client
    let client_addr = "http://[::1]:50051".to_string();
    let handler = Arc::new(MockPingPongClientHandler::default());
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

#[derive(Default)]
pub struct MockPingPongServiceHandler {}


#[async_trait]
impl PingPongHandler for MockPingPongServiceHandler {
    async fn handle_ping(&self, ping_request: ServerPingRequest) -> ServerPongResponse {
        if ping_request.message != "ping" {
            return ServerPongResponse {
                message: format!("unexpect message {}", ping_request.message),
            };
        }

        ServerPongResponse {
            message: "pong".to_string(),
        }
    }
}

#[derive(Default)]
pub struct MockPingPongClientHandler {}

#[async_trait]
impl MyPingPongClientHandler for MockPingPongClientHandler {
    async fn handle_ping(&self, i: i32, pong_response: ClientPongResponse) -> ClientPingRequest {
        if pong_response.message != "pong" {
            return ClientPingRequest {
                message: format!("unexpect message {}", pong_response.message),
            };
        }

        ClientPingRequest {
            message: format!("ping{}", i),
        }
    }
}