use std::sync::{Arc, Mutex};

use proto::{
    drawing_server::{Drawing, DrawingServer}, CreateRoomRequest, CreateRoomResponse, DrawingCanvas, HealthCheckRequest, PullCanvasRequest, QueryRoomsRequest, QueryRoomsResponse, RoomDetails, UploadCanvasRequest, UploadCanvasResponse
};
use tonic::{server::NamedService, transport::Server, Request, Response, Status};

use crate::proto::HealthCheckResponse;

mod proto {
    tonic::include_proto!("drawing");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("drawing_descriptor");
}

mod canvas;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:7878".parse().unwrap();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let service = TestService {
        rooms: Arc::new(Mutex::new(Vec::new())),
    };

    let drawing_server = DrawingServer::new(service);

    Server::builder()
        .add_service(reflection_service)
        .add_service(drawing_server)
        .serve(addr)
        .await
        .unwrap();
}

pub struct TestService {
    rooms: Arc<Mutex<Vec<DrawingCanvas>>>,
}

#[tonic::async_trait]
impl Drawing for TestService {

    async fn create_room(&self, request: Request<CreateRoomRequest>) -> Result<Response<CreateRoomResponse>, Status> {

        let initial = if let Some(canv) = request.into_inner().initial {
            let err = |msg| Err(Status::invalid_argument(msg));

            use canvas::CheckInvariantsResult as C;
            match canvas::check_invariants(&canv) {
                C::Fine => canv,
                C::NegativeWidth => return err(format!("Given canvas width cannot be negative! Value given: {0}.", canv.width)),
                C::NegativeHeight => return err(format!("Given canvas height cannot be negative! Value given: {0}.", canv.height)),
                C::OverflowingSize => return err(format!("Given canvas dimensions would cause a 32-bit integer overflow! Given canvas dimensions width {0} and height {1}, when multiplied, exceed the representation limit of 32 bits.", canv.width, canv.height)),
                C::MismatchedSize => return err(format!("Given canvas dimensions do not match the actual size of the given contents! Given canvas dimensions width {0} and height {1}, when multiplied, do not match the number of pixels in the canvas contents ({2}).", canv.width, canv.height, canv.contents.len()))
            }
        } else {
            canvas::blank(50, 50)
        };

        let mut lock = self.rooms.lock().unwrap();
        lock.push(initial);
        Ok(Response::new(CreateRoomResponse { room_id: (lock.len() - 1) as i32 }))
    }

    async fn query_rooms(&self, _request: Request<QueryRoomsRequest>) -> Result<Response<QueryRoomsResponse>, Status> {
        let lock = self.rooms.lock().unwrap();
        let result: Vec<_> = lock.iter().enumerate().map(|(i, c)| RoomDetails { id: i as i32, width: c.width, height: c.height }).collect();

        Ok(Response::new(QueryRoomsResponse { rooms: result }))
    }

    async fn upload_canvas(
        &self,
        uploaded: Request<UploadCanvasRequest>
    ) -> Result<Response<UploadCanvasResponse>, Status> {

        let UploadCanvasRequest { canvas: Some(canvas), room } = uploaded.into_inner() else {
            let msg = format!("A canvas must be included with the request!");
            return Err(Status::invalid_argument(msg));
        };

        let mut lock = self.rooms.lock().unwrap();

        let Some(target_room) = lock.get_mut(room as usize) else {
            let msg = format!("There is no room numbered {room}!");
            return Err(Status::out_of_range(msg));
        };

        canvas::try_merge_into(target_room, &canvas).map_err(|e| Status::invalid_argument(e))?;

        Ok(Response::new(UploadCanvasResponse {}))
    }

    async fn pull_canvas(&self, request: Request<PullCanvasRequest>) -> Result<Response<DrawingCanvas>, Status> {
        let room = request.into_inner().room;

        let mut lock = self.rooms.lock().unwrap();

        if lock.len() == 0 && room == 0 {
            let initial = canvas::blank(50, 50);
            lock.push(initial.clone());
            return Ok(Response::new(initial));
        }

        let Some(target_room) = lock.get_mut(room as usize) else {
            let msg = format!("There is no room numbered {room}!");
            return Err(Status::out_of_range(msg));
        };

        Ok(Response::new(target_room.clone()))
    }

    async fn health_check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        println!("Got a request: {:?}", request.metadata());
        Ok(Response::new(HealthCheckResponse {
            status: "OK".to_string(),
        }))
    }
}

impl NamedService for TestService {
    const NAME: &'static str = "test";
}

impl Clone for TestService {
    fn clone(&self) -> Self {
        Self {
            rooms: Arc::clone(&self.rooms),
        }
    }
}

#[allow(dead_code)]
fn get_room_or_status(rooms: &mut [DrawingCanvas], idx: usize) -> Result<&mut DrawingCanvas, String> {
    rooms.get_mut(idx).ok_or_else(|| format!("No such room: {idx}! Send a query to get information about available rooms."))
}