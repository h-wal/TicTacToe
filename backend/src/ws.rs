use actix_web::{HttpRequest, web};
use actix_ws::{Message, Session};
use tokio::runtime::Handle as RtHandle;

pub async fn ws_handler(request: HttpRequest, body: web::Payload) -> Result<actix_web::HttpResponse, actix_web::error::Error> {
    let (response, mut session, mut stream) = actix_ws::handle(&request, body)?; //upgrade to ws

    actix_web::rt::spawn(async move {
        while let Some(message) = stream.recv().await {
            match message {
                Ok(Message::Ping(data)) => {
                    let _ = session.pong(&data).await;
                }
                Ok(Message::Text(message)) => {
                    let _ = session.text(message).await;
                }
                _ => {

                }
            }
        }
    });
    Ok(response)
}