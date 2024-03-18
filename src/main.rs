use std::{pin::Pin, sync::Arc};

use tokio::sync::Mutex;
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

    Server::builder()
        .add_service(reflection_service)
        .add_service(DrawingServer::new(TestService {
            canon: Arc::new(Mutex::new(canvas::blank()))
        }))
        .serve(addr)
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct TestService {
    canon: Arc<Mutex<DrawingCanvas>>
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

        let output = async_stream::try_stream! {
            let lock = arc_clone.lock().await;
            yield (*lock).clone();
            drop(lock);

            while let Some(canvas) = stream.message().await? {
                let mut lock = arc_clone.lock().await;
                *lock = canvas::merge(&*lock, &canvas);
                yield canvas::clamp(&*lock);
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
