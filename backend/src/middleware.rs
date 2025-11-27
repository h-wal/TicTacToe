use actix_web::{
    Error, FromRequest, HttpMessage, HttpRequest, body::MessageBody, dev::{Payload, ServiceRequest, ServiceResponse}, error::ErrorUnauthorized, middleware::Next, web
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::{env, future::Ready, future::ready};

use crate::routes::user::Claims;

pub struct JwtClaims(pub Claims);

impl FromRequest for JwtClaims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        if let Some(header_value) = auth_header {
            if let Ok(token) = header_value.to_str() {
                let secret = env::var("SECRET_KEY").expect("JWT_SECRET must be set");
                let decoding_key = DecodingKey::from_secret(secret.as_bytes());
                let validation = Validation::default();

                match decode::<Claims>(token, &decoding_key, &validation) {
                    Ok(token_data) => {
                        return ready(Ok(JwtClaims(token_data.claims)));
                    }
                    Err(e) => {
                        eprintln!("JWT decoding error: {:?}", e);
                        return ready(Err(ErrorUnauthorized("Invalid JWT token")));
                    }
                }
            }
        }
        ready(Err(ErrorUnauthorized("Authorization header missing or invalid")))
    }
}


pub async fn check_user(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>, //it is something which implementws the Message Body 
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let claims = req.extract::<JwtClaims>().await.map_err(|_| ErrorUnauthorized("Invalid or missing token"))?;
    req.extensions_mut().insert(claims.0); // store Claims so handlers can read them
    next.call(req).await
}