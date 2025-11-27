use actix_web::{App, HttpServer, dev::ServiceRequest, middleware::from_fn, web};
use db::Db;

use crate::{middleware::check_user, routes::{rooms::{create_room, get_rooms}, user::{create_user, me, sign_in}}};

pub mod routes;
pub mod middleware;
pub mod ws;

#[actix_web::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    let db = Db::new().await.unwrap();
    let _ = HttpServer::new(move || {
        App::new()
            .service(web::resource("/signup").route(web::post().to(create_user)))
            .service(web::resource("/signin").route(web::post().to(sign_in)))
            .service(
                web::resource("/me")
                    .wrap(from_fn(check_user))
                    .route(web::get().to(me))
                )
            .service(web::resource("/createroom")
                .wrap(from_fn(check_user))
                .route(web::post().to(create_room))
            )
            .service(web::resource("/getrooms")
                .wrap(from_fn(check_user))
                .route(web::get().to(get_rooms))
            )
            .app_data(actix_web::web::Data::new(db.clone()))

    })
    .bind("0.0.0.0:3000")
    .unwrap()
    .run()
    .await;

}
