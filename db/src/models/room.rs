use anyhow::Result;
use sqlx::prelude::FromRow;

use crate::Db;

#[derive(Debug, FromRow)]
pub struct CreateRoomResponse {
    pub slug: String,
}

#[derive(Debug, FromRow)]
pub struct GetRoomResponse {
    pub rooms: Vec<String>,
}

impl Db {
    pub async fn create_room(
        &self,
        room_slug: &String,
        username: &String,
    ) -> Result<CreateRoomResponse> {
        let response = sqlx::query_as::<_, CreateRoomResponse>(
            "INSERT INTO rooms (slug, creator_id) VALUES ($1, $2) RETURNING slug",
        )
        .bind(room_slug)
        .bind(username)
        .fetch_one(&self.pool)
        .await?;

        println! {"{:?}", response}
        Ok(CreateRoomResponse {
            slug: response.slug,
        })
    }

    pub async fn get_rooms(&self) -> Result<GetRoomResponse> {
        let rooms = sqlx::query_scalar::<_, String>("SELECT slug FROM rooms")
            .fetch_all(&self.pool)
            .await?;

        println!("{:?}", rooms);

        Ok(GetRoomResponse { rooms })
    }
}
