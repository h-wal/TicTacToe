use std::env;

use actix_web::{
    App, Error, HttpMessage, HttpRequest, HttpResponse,
    dev::{ServiceRequest, ServiceResponse},
    middleware::{Next, from_fn},
    web::{Data, Json},
};
use db::Db;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct SigninResponse {
    token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    pub fn new(sub: String) -> Self {
        Self {
            sub,
            exp: 10000000000000000000,
        }
    }
}

pub async fn create_user(
    db: Data<Db>,
    body: Json<UserRequest>,
) -> Result<Json<UserResponse>, actix_web::error::Error> {
    let user = db
        .create_user(&body.username, &body.password)
        .await
        .map_err(|e| actix_web::error::ErrorConflict(e.to_string()))?;

    Ok(Json(UserResponse { id: user.id }))
}

pub async fn sign_in(
    db: Data<Db>,
    body: Json<UserRequest>,
) -> Result<Json<SigninResponse>, actix_web::error::Error> {
    let user = db
        .get_user_by_username(&body.username)
        .await
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if user.password != body.password {
        return Err(actix_web::error::ErrorUnauthorized("Incorrect password"));
    }

    let token = encode(
        &Header::default(),
        &Claims::new(user.id),
        &EncodingKey::from_secret(
            env::var("SECRET_KEY")
                .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
                .as_bytes(),
        ),
    )
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(Json(SigninResponse { token }))
}

pub async fn me(req: HttpRequest) -> Result<HttpResponse, Error> {
    let extensions = req.extensions();
    let claims = extensions
        .get::<Claims>()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("No Claims From Jwt"))?;

    Ok(HttpResponse::Ok().body(format!("Hello User, {}", claims.sub)))
}

// #[actix_web::main]
// async fn main() {
//     let app = App::new().wrap(from_fn(my_middleware));
// }