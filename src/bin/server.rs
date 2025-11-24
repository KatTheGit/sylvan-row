use redb::Database;
use sylvan_row::{common, const_params::*, database::{self, PlayerData}, gamedata::Character, gameserver, mothership_common::*} ;
use std::{sync::{Arc, Mutex}, thread::{JoinHandle}};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::mpsc, net::{TcpListener}};

use opaque_ke::{CipherSuite, ServerLoginStartResult};
use rand::RngCore;
use rand::rngs::OsRng;
use opaque_ke::{
  ClientLogin, ClientLoginFinishParameters, ClientRegistration,
  ClientRegistrationFinishParameters, CredentialFinalization, CredentialRequest,
  CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload, ServerLogin,
  ServerLoginParameters, ServerRegistration, ServerRegistrationLen, ServerSetup,
};

#[tokio::main]
async fn main() {

  let listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", SERVER_PORT)).await.unwrap();

  let players: Vec<PlayerInfo> = Vec::new();
  // Arc allows for shared access, and Mutex makes it mutually exclusive.
  let players = Arc::new(Mutex::new(players));
  
  // Contains all threads running game servers
  let fleet: Vec<JoinHandle<()>> = Vec::new();
  let fleet = Arc::new(Mutex::new(fleet));
  
  // Database
  let database: Database = database::load().expect("Couldn't load database");
  let database = Arc::new(Mutex::new(database));

  let main_players = Arc::clone(&players);
  loop {

    // Accept a new peer.
    let (mut socket, _addr) = listener.accept().await.unwrap();
    // Create the channels to communicate to this thread.
    let (tx, mut rx): (mpsc::Sender<PlayerMessage>, mpsc::Receiver<PlayerMessage>)
      = mpsc::channel(32);
    
    // for simplicity's sake these will be referred to as threads
    // in code and comments.
    let local_players = Arc::clone(&players);
    let local_fleet = Arc::clone(&fleet);
    let local_database = Arc::clone(&database);
    let mut logged_in: bool = false;
    tokio::spawn(async move {
      // Username the client claims to be.
      let mut username: String = String::from("");
      let mut buffer = [0; 2048];
      let mut rng = OsRng;
      let server_setup = ServerSetup::<DefaultCipherSuite>::new(&mut rng);
      let mut server_login_start_result: Option<ServerLoginStartResult<DefaultCipherSuite>> = None;
      loop {
        // this thing is really cool and handles whichever branch is ready first
        tokio::select! {
          // wait until we recieve packet, and write it to buffer.
          socket_read = socket.read(&mut buffer) => {
            let len: usize = match socket_read {
              Ok(0) => {
                return
              }
              Ok(len) => { len }
              Err(err) => {
                println!("ERROR: {:?}", err);
                return; // An error happened. We should probably inform the client later, and log this.
              }
            };
            // handle the packet
            let packet = bincode::deserialize::<ClientToServerPacket>(&buffer[..len]);
            match packet {
              Ok(packet) => {
                match packet.information {
                  // MARK: Match Request
                  ClientToServer::MatchRequest(data) => {

                  }

                  // MARK: Match Cancel
                  ClientToServer::MatchRequestCancel => {
                    
                  }

                  // MARK: Registration
                  ClientToServer::RegisterRequestStep1(recieved_username, client_message) => {
                    println!("Trying to run step 1");
                    username = recieved_username.clone();
                    let username_taken: bool;
                    {
                      let mut database = local_database.lock().unwrap();
                      username_taken = database::username_taken(&mut database, &recieved_username).expect("oops");
                    }
                    if username_taken {
                      let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                        information: ServerToClient::AuthenticationRefused(RefusalReason::UsernameTaken),
                      }).expect("hi")).await;
                      username = String::new();
                      continue;
                    }
                    let server_registration_start_result = ServerRegistration::<DefaultCipherSuite>::start(
                      &server_setup,
                      client_message,
                      username.clone().as_bytes(),
                    ).expect("oops");
                    let response: RegistrationResponse<DefaultCipherSuite> = server_registration_start_result.message;
                    // reply to the client
                    // this doesnt reply
                    let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                      information: ServerToClient::RegisterResponse1(response),
                    }).expect("hi")).await;
                  }
                  ClientToServer::RegisterRequestStep2(client_message) => {
                    if username == String::new() {
                      let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                        information: ServerToClient::AuthenticationRefused(RefusalReason::InternalError),
                      }).expect("hi")).await;
                      continue;
                    }
                    let password_file = ServerRegistration::<DefaultCipherSuite>::finish(client_message);
                    println!("Registered user {:?}", username.clone());
                    let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                      information: ServerToClient::RegisterSuccessful,
                    }).expect("hi")).await;
                    {
                      let mut database = local_database.lock().unwrap();
                      database::create_player(&mut database, username.clone().as_str(), PlayerData::new(password_file)).expect("oops");
                    }

                  }
                  // MARK: Login
                  ClientToServer::LoginRequestStep1(username, client_message) => {
                    let password_file: ServerRegistration<DefaultCipherSuite>;
                    {
                      let database = local_database.lock().unwrap();
                      password_file = database::get_player(&database, &username).expect("oops").password_hash;
                    }
                    server_login_start_result = Some(ServerLogin::start(
                      &mut rng,
                      &server_setup,
                      Some(password_file),
                      client_message,
                      username.as_bytes(),
                      ServerLoginParameters::default(),
                    ).expect("oops"));
                    let response = server_login_start_result.as_ref().unwrap().message.clone();
                    let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                      information: ServerToClient::LoginResponse1(response),
                    }).expect("hi")).await;
                  }
                  ClientToServer::LoginRequestStep2(client_message) => {
                    if let Some(server_login_start_result) = server_login_start_result.take() {
                      let server_login_finish_result = server_login_start_result.state.finish(
                        client_message,
                        ServerLoginParameters::default(),
                      ).expect("oops");
                      let session_key = server_login_finish_result.session_key;
                      println!("KEY: {:?}", session_key);
                    }
                  }
                  _ => panic!()
                }
              }
              Err(err) => {
                println!("ERROR: {:?}", err)
              }
            }
          }

          thread_message = rx.recv() => {
            if let Some(message) = thread_message {
              match message {
                PlayerMessage::GameAssigned(data) => {
                  socket.write_all(
                    &bincode::serialize::<ServerToClientPacket>(
                      &ServerToClientPacket {
                        information: ServerToClient::MatchAssignment(
                          MatchAssignmentData { port: data.port }
                        )
                      }
                    ).expect("oops")
                  ).await.unwrap();
                },
              }
            }
          }
        }
      }
    });
  }
}