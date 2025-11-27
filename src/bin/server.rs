use redb::Database;
use sylvan_row::{const_params::*, database::{self, PlayerData}, gamedata::Character, gameserver, mothership_common::*} ;
use std::{sync::{Arc, Mutex}, thread::{JoinHandle}};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::mpsc, net::{TcpListener}};
use ring::hkdf;
use opaque_ke::{generic_array::GenericArray, ServerLoginStartResult};
use rand::{rngs::OsRng};
use opaque_ke::{
  RegistrationResponse, ServerLogin,
  ServerLoginParameters, ServerRegistration, ServerSetup,
};
use chacha20poly1305::{
  aead::{Aead, KeyInit},
  ChaCha20Poly1305, Nonce
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

  let server_setup = database::load_server_setup().expect("Failed to load server setup");
  let server_setup = Arc::new(server_setup);

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
    let local_server_setup = Arc::clone(&server_setup);
    tokio::spawn(async move {
      // Username the client claims to be.
      let mut username: String = String::from("");
      let mut buffer = [0; 2048];
      let mut rng = OsRng;
      let mut server_login_start_result: Option<ServerLoginStartResult<DefaultCipherSuite>> = None;
      // cipher key, also session key.
      let mut cipher_key: Vec<u8> = Vec::new();
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

            // not logged in, register, login, and get cipher key.
            if !logged_in {
              let packet = bincode::deserialize::<ClientToServerPacket>(&buffer[..len]);
              match packet {
                Ok(packet) => {
                  match packet.information {                    
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
                        &local_server_setup,
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
                      //let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                        //  information: ServerToClient::RegisterSuccessful,
                        //}).expect("hi")).await;
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
                        &local_server_setup,
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
                        let session_key = server_login_finish_result.session_key.to_vec();

                        // Shrink PAKE key
                        // put this in a function later
                        let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, &[]);
                        let prk = salt.extract(&session_key);
                        let okm = prk.expand(&[], hkdf::HKDF_SHA256).unwrap();
                        let mut key_bytes = [0u8; 32];
                        okm.fill(&mut key_bytes).unwrap();
                        let key = Vec::from(&key_bytes);
                        // save the new key for this thread
                        cipher_key = key;
                        logged_in = true;
                      }
                    }
                    _ => {
                      // Ignore packet. Invalid.
                    }
                  }
                // if the user is logged in
                }
                Err(err) => {
                  println!("ERROR: {:?}", err)
                }
              }
            }
            // logged in, so use cipher
            else {

              let nonce = &buffer[..4];
              println!("{:?}", nonce);
              let nonce = bincode::deserialize::<u32>(&nonce).expect("oops");
              println!("{:?}", nonce);
              let mut nonce_bytes = [0u8; 12];
              nonce_bytes[8..].copy_from_slice(&nonce.to_be_bytes());
              let nonce = Nonce::from_slice(&nonce_bytes);
              
              let key = GenericArray::from_slice(cipher_key.as_slice());
              let cipher = ChaCha20Poly1305::new(key);
              
              let raw_packet = &buffer[4..len];
              let deciphered = cipher.decrypt(&nonce, raw_packet.as_ref()).expect("shit");
              let packet = bincode::deserialize::<ClientToServerPacket>(&deciphered);
              println!("{:?}", packet);
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