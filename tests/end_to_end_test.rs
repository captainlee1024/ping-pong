use pingpong::pingpongcs::client::{MyPingPongClientHandler, PingPongClient};
use pingpong::pingpongcs::server::{launch_scheduler_server_with_addr, PingPongHandler};
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

    // Run the first client
    let client_addr1 = "http://[::1]:50051".to_string();
    let handler1 = Arc::new(MockPingPongClientHandler::default());
    let client1 = PingPongClient::new(client_addr1, handler1);
    let client_future1 = tokio::spawn(async move {
        client1.launch_service().await.unwrap();
    });

    // Run the second client
    let client_addr2 = "http://[::1]:50051".to_string();
    let handler2 = Arc::new(MockPingPongClientHandler::default());
    let client2 = PingPongClient::new(client_addr2, handler2);
    let client_future2 = tokio::spawn(async move {
        client2.launch_service().await.unwrap();
    });


    // Wait for the clients to finish
    let client_result1 = client_future1.await;
    assert!(client_result1.is_ok(), "Client 1 encountered an error");

    let client_result2 = client_future2.await;
    assert!(client_result2.is_ok(), "Client 2 encountered an error");

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