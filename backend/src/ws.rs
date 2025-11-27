use actix_web::{HttpRequest, rt, web};
use actix_ws::Message;
use db::models::room;
use snowflake::ProcessUniqueId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type ClientId = ProcessUniqueId; //declaring types for websocket
pub type RoomId = String;

#[derive(Clone)]
pub struct ClientInfo {
    //this is a clientInfo struct representing the user along with its connected rooms and the tx (mpsc) sender (added later)
    pub id: ClientId,
    pub rooms: HashSet<RoomId>,
}

pub struct RoomManager {
    //central state of our web socket shared accross all the threads (different room threads spawn different threads)
    pub clients: HashMap<ClientId, ClientInfo>,
    pub rooms: HashMap<RoomId, HashSet<ClientId>>,
}

impl RoomManager {
    //defining a factory function on the room manager
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            rooms: HashMap::new(),
        }
    }

	//Entering a new client
	pub fn register_client(&mut self, id: ClientId){
		let info = ClientInfo{
			id,
			rooms: HashSet::new()
		};
		self.clients.insert(id, info);
	}

	pub fn unregister_client(&mut self, id: ClientId){
		if let Some(client) = self.clients.remove(&id){ //remove the given id from the clients hash map if this exists if will return the clientInfo struct the value stored in the hash map
			for room_id in client.rooms{ // we are iterating the array stroed in the struct which was returned earlier the feild rooms is an araawy and we are iteratign over it also we the array contains all the rooms the user is part of..
				if let Some(set) = self.rooms.get_mut(&room_id){ //now for every room id which was given to us we iterate over the rooms feild in the room manager and reomve that user from that room
					set.remove(&id);

					if set.is_empty() { //also if it was the last user to leave the room we completely remove the room (no longer needed)
						self.rooms.remove(&room_id);
					}
				}
				
			}
		}
	}

	pub fn join_room(&mut self, client_id: ClientId, room_id: String){

		let room  = self.rooms.entry(room_id.to_string()).or_insert_with(HashSet::new);

		room.insert(client_id);

		if let Some(client) = self.clients.get_mut(&client_id) {
			client.rooms.insert(room_id.to_string());
		}

	}

	pub fn leave_room(&mut self, client_id: ClientId, room_id: &str){
		
		if let Some(set) = self.rooms.get_mut(room_id){
			set.remove(&client_id);
			if set.is_empty() {
				self.rooms.remove(room_id);
			}
		}

		if let Some(client) = self.clients.get_mut(&client_id){
			client.rooms.remove(room_id);
		}

	}
}



start from here

// pub struct AppState {
//     pub manager: Arc<RwLock<RoomManager>>,
// }

// impl AppState {
//     pub fn new() -> Self {
//         Self {
//             manager: Arc::new(RwLock::new(RoomManager::new())),
//         }
//     }
// }


pub async fn ws_handler(
    request: HttpRequest,
    body: web::Payload,
) -> Result<actix_web::HttpResponse, actix_web::error::Error> {
    // we are upgrading the http requewt into a web socket...
    let (response, mut session, mut stream) = actix_ws::handle(&request, body).unwrap();
    // session is used to send messages to the ws client
    // stream is used to receive messages from the client
    // response is what will be sent to the client

    rt::spawn(async move {
        while let Some(message) = stream.recv().await {
            match message.unwrap() {
                Message::Ping(data) => {
                    let _ = session.pong(&data).await;
                }
                Message::Text(message) => {
                    let _ = session.text("sent from web sockets").await;
                }
                _ => {}
            }
        }
    });

    Ok(response)
}

#[actix_web::main]
async fn main() {}
