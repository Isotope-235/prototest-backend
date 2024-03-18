use std::{pin::Pin, sync::Arc};

use tokio::sync::{watch, Mutex};
use tokio_stream::Stream;
use tonic::{server::NamedService, transport::Server, Request, Response, Status, Streaming};

mod proto {
    tonic::include_proto!("drawing");

  pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("drawing_descriptor");
}

pub mod drawing {
    tonic::include_proto!("drawing");
}

use drawing::{drawing_server::{Drawing, DrawingServer}, DrawingCanvas};
use crate::drawing::{HealthCheckRequest, HealthCheckResponse};

mod canvas;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:7878".parse().unwrap();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build().unwrap();

    let canon = Arc::new(Mutex::new(canvas::blank()));
    let service = TestService {
        canon,
        channel: Arc::new(Mutex::new(watch::channel(canvas::blank())))
    };

    let drawing_server = DrawingServer::new(service);

    Server::builder()
        .add_service(reflection_service)
        .add_service(drawing_server)
        .serve(addr)
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct TestService {
    canon: Arc<Mutex<DrawingCanvas>>,
    channel: Arc<Mutex<(watch::Sender<DrawingCanvas>, watch::Receiver<DrawingCanvas>)>>
}

#[tonic::async_trait]
impl Drawing for TestService {
    type OpenConnectionStream = Pin<Box<dyn Stream<Item = Result<DrawingCanvas, Status>> + Send + 'static>>;

    async fn open_connection(
        &self,
        request: Request<Streaming<DrawingCanvas>>,
    ) -> Result<Response<Self::OpenConnectionStream>, Status>
    {
        println!("Got a request: {:?}", request.metadata());
        let mut stream = request.into_inner();

        let arc_clone = self.canon.clone();
        let chan_clone = Arc::clone(&self.channel);


        let channel_arc_clone = Arc::clone(&self.channel);

        let output = async_stream::try_stream! {
            let ac = arc_clone.clone();
            let lock = ac.lock().await;
            yield (*lock).clone();
            drop(lock);


            let arc_clone = arc_clone.clone();
            let arc_clone2 = arc_clone.clone();
            tokio::spawn(async move {
                while let Some(message) = stream.message().await.unwrap() {
                    let mut lock = arc_clone.lock().await;

                    let updated_canvas = canvas::merge(&*lock, &message).1;
                    *lock = canvas::clamp(&updated_canvas);

                    let chan_lock = chan_clone.lock().await;

                    chan_lock.0.send((*lock).clone()).unwrap();
                }
            });


            let mut channel_lock = channel_arc_clone.lock().await;

            while (channel_lock).1.changed().await.is_ok() {
                let lock = arc_clone2.lock().await;
                yield (*lock).clone();
                drop(lock);
            }

        };

        Ok(Response::new(Box::pin(output) as Self::OpenConnectionStream))
    }

  async fn health_check(&self, request: Request<HealthCheckRequest>) -> Result<Response<HealthCheckResponse>, Status> {
    println!("Got a request: {:?}", request.metadata());
    Ok(Response::new(HealthCheckResponse {
      status: "OK".to_string()
    }))
  }
}

impl NamedService for TestService {
    const NAME: &'static str = "test";
}
