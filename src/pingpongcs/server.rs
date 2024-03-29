use pingpong::{ping_pong_server::PingPong, PingRequest, PongResponse};
use std::sync::Arc;
use std::{pin::Pin, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::{async_trait, transport::Server, Request, Response, Status};

pub mod pingpong {
    tonic::include_proto!("pingpong");
}

pub struct PingPongService {
    pub ping_pong_handler: Arc<dyn PingPongHandler + Send + Sync>,
}

impl PingPongService {
    pub fn new(ping_pong_handler: Arc<dyn PingPongHandler + Send + Sync>) -> Self {
        PingPongService { ping_pong_handler }
    }
}

#[tonic::async_trait]
impl PingPong for PingPongService {
    type PingStreamStream =
        Pin<Box<dyn Stream<Item = Result<PongResponse, Status>> + Send + Sync + 'static>>;
    // type PingStreamStream: futures_core::Stream<
    //     Item = Result<crate::pingpong::PongResponse, tonic::Status>,
    // >
    // + Send
    // + 'static;
    async fn ping_stream(
        &self,
        request: Request<tonic::Streaming<PingRequest>>,
    ) -> Result<Response<Self::PingStreamStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(10);
        let handler = self.ping_pong_handler.clone();

        // Spawn a task to process the incoming stream and send responses to the channel
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Wait for the next request to arrive
                    Some(result) = stream.next() => {
                        match result {
                            Ok(request) => {
                                // Spawn a task to handle each request
                                // tokio::spawn(async move {
                                    // Sleep for 10 seconds
                                    tokio::time::sleep(Duration::from_secs(2)).await;

                                    println!("recv from client: {}", request.message);
                                    if request.message == "registry" {
                                        // registry
                                         println!("registry done: {}", request.message);

                                    if tx.send(Ok(PongResponse{
                                        message: "registry done".into(),})).await.is_err() {
                                        // If sending fails, it means the receiver has been dropped, so we should stop processing
                                        break;
                                    }
                                        continue;
                                    }


                                // let response  = handle_ping().await;
                                let response = handler.handle_ping(request).await;

                                    if tx.send(Ok(response)).await.is_err() {
                                        // If sending fails, it means the receiver has been dropped, so we should stop processing
                                        break;
                                    }
                            }
                            Err(e) => {
                                println!("client exit: {}", e);
                                let _ = tx.send(Err(Status::internal(format!("Error: {}", e)))).await;
                                break;
                            }
                        }
                    }
                    // 加一些定时器，退出信号channel etc...
                    else => {
                        // The stream has been closed, so we should stop processing
                        break;
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}

#[async_trait]
pub trait PingPongHandler {
    async fn handle_ping(&self, ping_request: PingRequest) -> PongResponse;
}

#[derive(Default)]
pub struct PingPongServiceHandler {}
#[async_trait]
impl PingPongHandler for PingPongServiceHandler {
    async fn handle_ping(&self, ping_request: PingRequest) -> PongResponse {
        if ping_request.message != "ping" {
            return PongResponse {
                message: format!("unexpect message {}", ping_request.message),
            };
        }

        PongResponse {
            message: "pong".to_string(),
        }
    }
}

pub async fn launch_scheduler_server_with_addr(
    addr: String,
    ping_pong_handler: Arc<dyn PingPongHandler + Send + Sync>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket_addr = addr.parse()?;
    // let ping_pong_handler = Arc::new(PingPongServiceHandler::default());
    let svc =
        pingpong::ping_pong_server::PingPongServer::new(PingPongService::new(ping_pong_handler));

    println!("PingPong server listening on {}", socket_addr);

    Server::builder()
        .add_service(svc)
        .serve(socket_addr)
        .await?;
    Ok(())
}
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "[::1]:50051".parse()?;
//
//     let scheduler = Arc::new(TokioMutex::new(Scheduler::new("scheduler".into())));
//     let svc = pingpong::ping_pong_server::PingPongServer::new(PingPongService::new(scheduler));
//
//     println!("PingPong server listening on {}", addr);
//
//     Server::builder().add_service(svc).serve(addr).await?;
//
//     Ok(())
// }
