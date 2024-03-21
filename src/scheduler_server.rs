use tonic::{transport::Server, Request, Response, Status};

pub mod ping_pong {
    tonic::include_proto!("pingpong");
}

use ping_pong::{PongResponse, PingRequest, ping_pong_server::PingPong};

#[derive(Default)]
pub struct PingPongService;

#[tonic::async_trait]
impl PingPong for PingPongService {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PongResponse>, Status> {
        println!("Received request: {:?}", request);

        // Echo back the received message
        let response = PongResponse {
            message: "pong".into(),
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let svc = ping_pong::ping_pong_server::PingPongServer::new(PingPongService::default());

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}
