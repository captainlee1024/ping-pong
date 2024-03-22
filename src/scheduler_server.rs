use tonic::{transport::Server, Request, Response, Status};
use std::{pin::Pin, time::Duration};
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tokio::sync::mpsc;
use pingpong::{PongResponse, PingRequest, ping_pong_server::PingPong};

pub mod pingpong {
    tonic::include_proto!("pingpong");
}

#[derive(Default)]
pub struct PingPongService;

#[tonic::async_trait]
impl PingPong for PingPongService {
    type PingStreamStream = Pin<Box<dyn Stream<Item = Result<PongResponse, Status>> + Send + Sync + 'static>>;

    async fn ping_stream(&self, request: Request<tonic::Streaming<PingRequest>>) -> Result<Response<Self::PingStreamStream>, Status> {
        let mut stream = request.into_inner();
        let (mut tx, rx) = mpsc::channel(10);

        // Spawn a task to process the incoming stream and send responses to the channel
        tokio::spawn(async move {
            while let Some(request) = stream.next().await {
                match request {
                    Ok(request) => {
                        // Sleep for 10 seconds
                        tokio::time::sleep(Duration::from_secs(2)).await;

                        println!("recv from client: {}", request.message);

                        let response = PongResponse {
                            message: "pong".into(),
                        };
                        if let Err(_) = tx.send(Ok(response)).await {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(Status::internal(format!("Error: {}", e)))) .await;
                        break;
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let svc = pingpong::ping_pong_server::PingPongServer::new(PingPongService::default());

    println!("PingPong server listening on {}", addr);

    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}
