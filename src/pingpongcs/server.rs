use pingpong::{ping_pong_server::PingPong, PingRequest, PongResponse};
use std::collections::HashMap;
use std::sync::Arc;
use std::{pin::Pin, time::Duration};
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::{async_trait, transport::Server, Request, Response, Status};

pub mod pingpong {
    tonic::include_proto!("pingpong");
}

pub struct PingPongService {
    scheduler: Arc<TokioMutex<Scheduler>>,
}

#[derive(Default)]
pub struct Scheduler {
    pub name: String,
    // TODO: TaskName -> TaskType
    pub task_sender_table:
        HashMap<TaskName, Arc<mpsc::Sender<(BatchContext, mpsc::Sender<String>)>>>,
    pub task_receiver_table: HashMap<
        TaskName,
        Arc<tokio::sync::Mutex<mpsc::Receiver<(BatchContext, mpsc::Sender<String>)>>>,
    >,
    // TODO: TaskName -> ServiceType
    pub service_table: HashMap<TaskName, HashMap<String, String>>,
}

pub type BatchContext = String;
pub type TaskName = String;

impl Scheduler {
    pub fn new(name: String) -> Self {
        let mut s = Scheduler::default();
        s.name = name;
        s
    }
}

impl PingPongService {
    pub fn new(scheduler: Arc<TokioMutex<Scheduler>>) -> Self {
        PingPongService { scheduler }
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
        let current_scheduler = Arc::clone(&self.scheduler);
        let handler = PingPongServiceHandler::default();

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
                                    println!("scheduer: {:?}", current_scheduler.lock().await.name);

                                    println!("recv from client: {}", request.message);
                                    if request.message == "registry" {
                                        // registry
                                         println!("registry done: {}", request.message);

                                    if let Err(_) = tx.send(Ok(PongResponse{
                                        message: "registry done".into(),})).await {
                                        // If sending fails, it means the receiver has been dropped, so we should stop processing
                                        break;
                                    }
                                        continue;
                                    }


                                // let response  = handle_ping().await;
                                let response = handler.handle_ping(request).await;

                                    if let Err(_) = tx.send(Ok(response)).await {
                                        // If sending fails, it means the receiver has been dropped, so we should stop processing
                                        break;
                                    }
                            }
                            Err(e) => {
                                println!("client exit: {}", e.to_string());
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
                message: format!("unexpect message {}", ping_request.message).into(),
            };
        }

        PongResponse {
            message: format!("pong").into(),
        }
    }
}

pub async fn launch_scheduler_server_with_addr(
    addr: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket_addr = addr.parse()?;
    let scheduler = Arc::new(TokioMutex::new(Scheduler::new("scheduler".into())));
    let svc = pingpong::ping_pong_server::PingPongServer::new(PingPongService::new(scheduler));

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
