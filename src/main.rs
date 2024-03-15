use std::pin::Pin;

use tokio_stream::Stream;
use tonic::{server::NamedService, transport::Server, Request, Response, Status, Streaming};

pub mod drawing {
    tonic::include_proto!("drawing");
}

use drawing::{drawing_canvas::Row, drawing_server::{Drawing, DrawingServer}, DrawingCanvas};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:7878".parse().unwrap();

    Server::builder()
        .add_service(DrawingServer::new(TestService))
        .serve(addr)
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct TestService;

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


        let canv = DrawingCanvas {
            rows: std::iter::repeat_with(|| Row { cols: vec![0; 50] }).take(50).collect(),

        };

        let output = async_stream::try_stream! {
            let init = DrawingCanvas {
                rows: std::iter::repeat_with(|| Row { cols: vec![0; 50] }).take(50).collect(),
            };

            yield init;

            while let Some(canvas) = stream.message().await? {
                yield canvas;
            }
        };

        Ok(Response::new(Box::pin(output) as Self::OpenConnectionStream))
    }
}

impl NamedService for TestService {
    const NAME: &'static str = "test";
}
