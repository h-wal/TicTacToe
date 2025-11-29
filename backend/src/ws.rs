use actix_web::{HttpRequest, rt, web};
use actix_ws::Message;
use db::models::room;
use serde::Deserialize;
use snowflake::ProcessUniqueId;
use std::collections::{HashMap, HashSet};
use std::fmt::format;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};

pub type ClientId = ProcessUniqueId; //declaring types for websocket
pub type RoomId = String;

#[derive(Clone)]
pub struct ClientInfo {
    //this is a clientInfo struct representing the user along with its connected rooms and the tx (mpsc) sender (added later)
    pub id: ClientId,
    pub rooms: HashSet<RoomId>, //all the rooms the user is connected to 
	pub tx: mpsc::UnboundedSender<String>
}

pub struct RoomManager {
    //global state of our web socket shared accross all the threads.
    pub clients: HashMap<ClientId, ClientInfo>, // this hashmap has all the clinetinfo as value and the clientId as its key
    pub rooms: HashMap<RoomId, HashSet<ClientId>>, // this hashmap has all the users a room has as a client set as the value and the roomId is the key
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
	
	pub fn register_client(&mut self, id: ClientId, tx: mpsc::UnboundedSender<String>){
		let info = ClientInfo{
			id,
			rooms: HashSet::new(),
			tx
		};
		self.clients.insert(id, info);
	}

	pub fn unregister_client(&mut self, id: ClientId){
		if let Some(client) = self.clients.remove(&id){ //remove the given id from the clients hash map if this exists if will return the clientInfo struct the value stored in the hash map
			for room_id in client.rooms{ // we are iterating the array stroed in the struct which was returned earlier the feild rooms is an array which contains all the room id the user was connected to and we are iterating over it.
				if let Some(set) = self.rooms.get_mut(&room_id){ //now for every room id which was given to us we use it to get the  rooms feild in the room manager which contains all the user in that room and reomve that user from that room
					set.remove(&id); //removing the client from every room which it was part (which we got from the clienrs feild)
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

	pub fn leave_room(&mut self, client_id: ClientId, room_id: &String){
		
		if let Some(set) = self.rooms.get_mut(room_id){
			set.remove(&client_id);
			if set.is_empty() { // check if it was the lasy user to leave the room
				self.rooms.remove(room_id); // remove that room from the room manager
			}
		}

		if let Some(client) = self.clients.get_mut(&client_id){
			client.rooms.remove(room_id);
		}

	}
}


pub struct AppState {
    pub manager: Arc<RwLock<RoomManager>>,
}

impl AppState { //factory function on app state
    pub fn new() -> Self {
        Self {
            manager: Arc::new(RwLock::new(RoomManager::new())),
        }
    }
}


/////////////////////////// //////////////////
////////////////////////// 
/// 
/// parsing messages from the client to the ws
/// 
////////////////////////// ///////////////////
////////////////////////// ///////////////////

#[derive(Deserialize, Debug)] // deserialize so that json -> rust , debug to make it printable
#[serde(tag = "action")] //the action feild acts like the tag to decide which enum variant to use 
pub enum ClientAction{

	#[serde(rename="join_room")] //this matches the join_room
	JoinRoom {
		room: String
	},
	#[serde(rename="leave_room")] //this matches the leave_room
	LeaveRoom {
		room: String
	},
	#[serde(rename="chat")] ////this matches the chat
	Chat{
		room: String,
		message: String
	}

}

pub async fn ws_handler(
    request: HttpRequest,
    body: web::Payload,
	state: web::Data<AppState>,
) -> Result<actix_web::HttpResponse, actix_web::error::Error> {
	
    // we are upgrading the http requewt into a web socket...
    let (response, mut session, mut stream) = actix_ws::handle(&request, body)?;
    // session is used to send messages to the ws client
    // stream is used to receive messages from the client
    // response is what will be sent to the client

	let (tx, mut rx) = mpsc::unbounded_channel::<String>(); //making a mpsc channel to talk to send and rcv messages to and from the client
	session.text("Welcome").await.unwrap(); //send a confimation message back
	 //send a welcome message to clinet , session.text is a future hence has to be awaited
	let client_id = ProcessUniqueId::new(); //creating a new process id to identify the client
	
	let mut manager = state.manager.write().await; //taking write access from manager
	manager.register_client(client_id, tx);
	
	let manager_state = state.manager.clone();
	let mut session_out = session.clone();

	let mut session_write = session.clone();
	let manager_state2 = manager_state.clone();

	rt::spawn(async move {
		while let Some(message) = rx.recv().await {
			if session_write.text(message).await.is_err(){

				let mut mgr = manager_state2.write().await;
				mgr.unregister_client(client_id);
				break;
			}
		} 
	});

    rt::spawn(async move {
        while let Some(message) = stream.recv().await {
            match message {
                Ok(Message::Ping(data)) => {
                    let _ = session.pong(&data).await;
                }
                Ok(Message::Text(message)) => {

					handle_incoming_message(&manager_state, client_id, &message, session.clone()).await;
					println!("Rcvd {} , {} from client ", client_id, message);
                }
				Ok(Message::Close(reason)) => {
					eprintln!("Clinet {:?} disconnected {:?}", client_id, reason);
					break;
				}
                Ok(_) => {},
				Err(e) => {
					eprint!("WebSocket Error {:?} {:?}", client_id, e);
					break;
				}
            }
        }

		let mut manager = manager_state.write().await;
    	manager.unregister_client(client_id);
    });

    Ok(response)
}

async fn handle_incoming_message(
	manager_state: &Arc<RwLock<RoomManager>>,
	client_id: ProcessUniqueId,
	message: &str,
	mut session: actix_ws::Session,

){
	match serde_json::from_str::<ClientAction>(&message){
		Ok(ClientAction::JoinRoom { room }) => {
			println!("client {} requested to join room {}", client_id, room);

			let mut manager = manager_state.write().await;
			manager.join_room(client_id, room.clone());

			let _ = session.text(
				format!("{{\"status\": \"joined\", \"room\": \"{}\"}}", room)
			).await;
		}

		Ok(ClientAction::LeaveRoom { room }) => {
			println!("client {} requested to leave room {}", client_id, room);

			let mut manager = manager_state.write().await;
			manager.leave_room(client_id, &room);

			let _ = session.text(
				format!("{{\"status\": \"left\", \"room\": \"{}\"}}", room)
			).await;
		}

		Ok(ClientAction::Chat { room, message }) => {
			println!("client {} requested to send {} in room {}", client_id,message, room);


			broadcast_to_room(
				&manager_state,
				&room, 
				&client_id,
				&format!("{{\"chat\": \"{}\"}}", message),
			).await;

			let _ = session.text(
				format!("{{\"status\": \"sent message {} \", \"room\": \"{}\"}}",message, room)
			).await;
		}

		Err(e) => {
			let _ = session.text(
				format!("{{\"error\": \"invalid_json\", \"details\": \"{}\"}}", e)
			).await;
		}
	}
}

async fn broadcast_to_room(
	manager_state: &Arc<RwLock<RoomManager>>,
	room_id: &str,
	sent_by_client: &ProcessUniqueId,
	message: &str
) {
	let manager = manager_state.read().await;

	if let Some(clients) = manager.rooms.get(room_id){
		for client_id in clients{
			if let Some(client_info) =  manager.clients.get(client_id){
				//send a message to the client
				let _ = client_info.tx.send(message.to_string());
				println!("Send message: {:?} to client {} from {}", message, client_id, sent_by_client)
			}
		}
	}
}

#[actix_web::main]
async fn main() {}
