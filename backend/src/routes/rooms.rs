use actix_web::web::{Data, Json};
use db::{Db, models::room};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateRoomRequestStruct {
    pub roomSlug: String,
    pub createdBy: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateRoomResponseStruct {
    pub slug: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetRoomsResponse {
    pub rooms: Vec<String>,
}

pub async fn create_room(
    db: Data<Db>,
    body: Json<CreateRoomRequestStruct>,
) -> Result<Json<CreateRoomResponseStruct>, actix_web::error::Error> {
    let response = db
        .create_room(&body.roomSlug, &body.createdBy)
        .await
        .map_err(|e| actix_web::error::ErrorConflict(e.to_string()))?;

    Ok(Json(CreateRoomResponseStruct { slug: response.slug }))
}

pub async fn get_rooms(
    db: Data<Db>,
) -> Result<Json<GetRoomsResponse>, actix_web::error::Error> {
    let response = db
        .get_rooms()
        .await
        .map_err(|e| actix_web::error::ErrorConflict(e.to_string()))?;

    Ok(Json(GetRoomsResponse { rooms: response.rooms }))
}
