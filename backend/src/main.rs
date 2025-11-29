use actix_web::{App, HttpServer, dev::ServiceRequest, middleware::from_fn, web};
use db::Db;

use crate::{
    middleware::check_user,
    routes::{
        rooms::{create_room, get_rooms},
        user::{create_user, me, sign_in},
    },
    ws::ws_handler,
};

pub mod middleware;
pub mod routes;
pub mod ws;

#[actix_web::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    let db = Db::new().await.unwrap();
    let app_state = actix_web::web::Data::new(crate::ws::AppState::new());
    let _ = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone()) //websocket room manager state (global state holdng all the clients, thier tx, rooms , etc)
            .app_data(actix_web::web::Data::new(db.clone()))
            .service(web::resource("/signup").route(web::post().to(create_user)))
            .service(web::resource("/signin").route(web::post().to(sign_in)))
            .service(
                web::resource("/me")
                    .wrap(from_fn(check_user))
                    .route(web::get().to(me)),
            )
            .service(
                web::resource("/createroom")
                    .wrap(from_fn(check_user))
                    .route(web::post().to(create_room)),
            )
            .service(
                web::resource("/getrooms")
                    .wrap(from_fn(check_user))
                    .route(web::get().to(get_rooms)),
            )
            .service(
                web::resource("/ws")
                    .route(web::get().to(ws_handler))) //written in http hander later upgraded to web socket connection
    })
    .bind("0.0.0.0:3000")
    .unwrap()
    .run()
    .await;
}
