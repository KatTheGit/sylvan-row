use redb::{Database, Result};
use sylvan_row::{common, const_params::*, database::{self, FriendShipStatus, PlayerData}, filter, gamedata::Character, gameserver, mothership_common::*} ;
use std::{sync::{Arc, Mutex}, thread::JoinHandle, time::Duration};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::mpsc, net::{TcpListener}};
use ring::hkdf;
use opaque_ke::{generic_array::GenericArray, ServerLoginStartResult};
use rand::{rngs::OsRng};
use opaque_ke::{
  RegistrationResponse, ServerLogin,
  ServerLoginParameters, ServerRegistration,
};
use chacha20poly1305::{
  aead::{Aead, KeyInit},
  ChaCha20Poly1305, Nonce
};

#[tokio::main]
async fn main() {

  let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", SERVER_PORT)).await.unwrap();

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

  // the server is now started so none of the code below should use .expect() or .unwrap(), unless
  // it is perfectly safe to do so.
  loop {
    // Accept a new peer.
    let (mut socket, _addr) = match listener.accept().await {
      Ok(info) => info,
      Err(_) => {continue;}
    };
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
      let mut last_nonce: u32 = 0;
      let mut nonce: u32 = 1;
      let mut server_login_start_result: Option<ServerLoginStartResult<DefaultCipherSuite>> = None;
      // cipher key, also session key.
      let mut cipher_key: Vec<u8> = Vec::new();
      let mut packet_queue: Vec<ServerToClientPacket> = Vec::new();
      loop {
        // this thing is really cool and handles whichever branch is ready first
        tokio::select! {
          // wait until we recieve packet, and write it to buffer.
          socket_read = socket.read(&mut buffer) => {
            let len: usize = match socket_read {
              Ok(0) => {
                // disconnect
                // remove player from players.
                if logged_in {
                  let mut players = local_players.lock().unwrap();
                  for p_index in 0..players.len() {
                    if players[p_index].username == username {
                      players.remove(p_index);
                      return;
                    }
                  }
                }
                return
              }
              Ok(len) => { len }
              Err(err) => {
                if err.kind() == std::io::ErrorKind::WouldBlock {
                  continue;
                }
                println!("ERROR: {:?}", err);
                return; // An error happened. We should probably inform the client later, and log this.
              }
            };
            // if the packet is too big, skip it.
            if len > buffer.len() {
              continue;
            }
            // handle the packet

            // not logged in, register, login, and get cipher key.
            if !logged_in {
              let packet = bincode::deserialize::<ClientToServerPacket>(&buffer[..len]);
              match packet {
                Ok(packet) => {
                  match packet.information {                    
                    // MARK: Registration
                    ClientToServer::RegisterRequestStep1(recieved_username, client_message) => {
                      username = recieved_username.clone();
                      let valid = filter::valid_username(username.clone());
                      if !valid {
                        let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::InvalidUsername),
                        }).expect("hi1")).await;
                        continue;
                      }
                      let username_taken: Result<bool, redb::Error>;
                      let username_taken_real: bool;
                      {
                        let mut database = local_database.lock().unwrap();
                        username_taken = database::username_taken(&mut database, &recieved_username);
                      }
                      match username_taken {
                        Ok(taken) => { username_taken_real = taken; },
                        Err(_) => {
                          let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                            information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                          //.expect() is ok to use on serialize because we control what gets serialized.
                          }).expect("hi1")).await;
                          continue;
                        }
                      };
                      if username_taken_real {
                        let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::UsernameTaken),
                        }).expect("hi2")).await;
                        username = String::new();
                        continue;
                      }
                      let server_registration_start_result = match ServerRegistration::<DefaultCipherSuite>::start(
                        &local_server_setup,
                        client_message,
                        username.clone().as_bytes(),
                      ) {
                        Ok(result) => {result},
                        Err(_err) => {
                          let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                            information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                          }).expect("hi3")).await;
                          continue;
                        }
                      };
                      let response: RegistrationResponse<DefaultCipherSuite> = server_registration_start_result.message;
                      // reply to the client
                      // this doesnt reply
                      let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                        information: ServerToClient::RegisterResponse1(response),
                      }).expect("hi4")).await;
                    }
                    ClientToServer::RegisterRequestStep2(client_message) => {
                      if username == String::new() {
                        let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                        }).expect("hi5")).await;
                        continue;
                      }
                      let password_file = ServerRegistration::<DefaultCipherSuite>::finish(client_message);
                      println!("Registered user {:?}", username.clone());
                      {
                        let mut database = local_database.lock().unwrap();
                        database::create_player(&mut database, username.clone().as_str(), PlayerData::new(password_file)).expect("hi6");
                      }
                    }
                    // MARK: Login
                    ClientToServer::LoginRequestStep1(recv_username, client_message) => {
                      username = recv_username;
                      let password_file: Result<PlayerData, redb::Error>;
                      let password_file_real: ServerRegistration<DefaultCipherSuite>;
                      let user_exists: Result<bool, redb::Error>;
                      let user_exists_real: bool;
                      {
                        let database = local_database.lock().unwrap();
                        user_exists = database::username_taken(&database, &username);
                        password_file = database::get_player(&database, &username);
                      }
                      match user_exists {
                        Ok(exists) => user_exists_real = exists,
                        Err(_err) => {
                          let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                            information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                          }).expect("hi7")).await;
                          continue;
                        }
                      }
                      if !user_exists_real {
                        let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::UsernameInexistent),
                        }).expect("hi7")).await;
                        continue;
                      }
                      match password_file {
                        Ok(playerdata) => password_file_real = playerdata.password_hash,
                        Err(_err) => {
                        let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                        }).expect("hi7")).await;
                          continue;
                        }
                      }
                      let result = match ServerLogin::start(
                        &mut rng,
                        &local_server_setup,
                        Some(password_file_real),
                        client_message,
                        username.as_bytes(),
                        ServerLoginParameters::default(),
                      ) {
                        Ok(result) => result,
                        Err(_err) => {
                          let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                            information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                          }).expect("hi8")).await;
                          continue;
                        },
                      };
                      server_login_start_result = Some(result);
                      let response = server_login_start_result.as_ref().unwrap().message.clone();
                      let _ = socket.write_all(&bincode::serialize(&ServerToClientPacket {
                        information: ServerToClient::LoginResponse1(response),
                      }).expect("hi9")).await;
                    }
                    ClientToServer::LoginRequestStep2(client_message) => {
                      if let Some(server_login_start_result) = server_login_start_result.take() {
                        let server_login_finish_result = server_login_start_result.state.finish(
                          client_message,
                          ServerLoginParameters::default(),
                        ).expect("hi10");
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

                        //// login successful, add user to user list
                        //{
                        //  // Make sure we avoid duplicates, if a player struct was left by this player
                        //  // beforehand (ungraceful exit)
                        //  let mut players = local_players.lock().unwrap();
                        //  if logged_in {
                        //    for p_index in 0..players.len() {
                        //      if players[p_index].username == username {
                        //        println!("{:?}, {:?}", players[p_index].username, username);
                        //        players.remove(p_index);
                        //        drop(players);
                        //        return;
                        //      }
                        //    }
                        //  }
                        //}
                        let mut channel_copy: Option<mpsc::Sender<PlayerMessage>> = None;
                        {
                          let mut players = local_players.lock().unwrap();
                          for p_index in 0..players.len() {
                            if players[p_index].username == username {
                              channel_copy = Some(players[p_index].channel.clone());
                              players.remove(p_index);
                              //return;
                            }
                          }
                        }
                        if let Some(channel) = channel_copy {
                          channel.send(PlayerMessage::ForceDisconnect).await.unwrap();
                        }
                        {
                          let mut players = local_players.lock().unwrap();
                          players.push(
                            PlayerInfo {
                              username: username.clone(),
                              session_key:cipher_key.clone(),
                              channel: tx.clone(),
                              queued: false,
                              queued_gamemodes: Vec::new(),
                              selected_character: Character::Hernani,
                              assigned_team: common::Team::Blue,
                            }
                          );
                        }
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
            // MARK: ===============
            else {
              // this code is a bit redundant for TCP, but will work particularily well for UDP.
              let recv_nonce = &buffer[..4];
              let recv_nonce = match bincode::deserialize::<u32>(&recv_nonce){
                Ok(nonce) => nonce,
                Err(_) => {
                  continue;
                }
              };
              if recv_nonce <= last_nonce {
                continue;
              }
              let nonce_num = recv_nonce;
              let mut nonce_bytes = [0u8; 12];
              nonce_bytes[8..].copy_from_slice(&nonce.to_be_bytes());
              let nonce_formatted = Nonce::from_slice(&nonce_bytes);
              
              let key = GenericArray::from_slice(cipher_key.as_slice());
              let cipher = ChaCha20Poly1305::new(key);
              
              let raw_packet = &buffer[4..len];
              let deciphered = match cipher.decrypt(&nonce_formatted, raw_packet.as_ref()) {
                Ok(decrypted) => {
                  if nonce_num <= last_nonce {
                    continue; // this is a parroted packet, ignore it.
                  }
                  // this is a valid packet, update last_nonce
                  last_nonce = nonce_num;
                  decrypted
                },
                Err(_err) => {
                  continue; // this is an erroneous packet, ignore it.
                },
              };
              let packet = match bincode::deserialize::<ClientToServerPacket>(&deciphered) {
                Ok(packet) => packet,
                Err(_err) => {
                  continue; // ignore invalid packet
                }
              };
              // this is the nonce we use to *send* data
              // incrementing it once we recieve a valid packet
              // allows us to make sure our reply always has a correct
              // nonce.
              match packet.information {
                // MARK: Match Request
                ClientToServer::MatchRequest(data) => {
                  let players_copy: Vec<PlayerInfo>;
                  let mut players_to_match: Vec<usize> = Vec::new();

                  {
                    // add the player to the queue
                    let mut players = local_players.lock().unwrap();
                    // this player's index
                    let p_index = from_user(&username, players.clone());
                    players[p_index].queued = true;
                    if data.gamemodes.len() <= 2{ players[p_index].queued_gamemodes = data.gamemodes; }
                    players[p_index].selected_character = data.character;

                    // finally
                    players_copy = players.clone();
                    // do matchmaking checks
                    let mut queued_1v1: Vec<usize> = Vec::new();
                    let mut queued_2v2: Vec<usize> = Vec::new();
                    for player_index in 0..players_copy.len() {
                      if !players_copy[player_index].queued { continue; }
                      if players_copy[player_index].queued_gamemodes.contains(&GameMode::Standard2V2) {
                        queued_2v2.push(player_index);
                      }
                      if players_copy[player_index].queued_gamemodes.contains(&GameMode::Standard1V1) {
                        queued_1v1.push(player_index);
                      }
                    }
                    if queued_2v2.len() >= 4 {
                      queued_2v2.truncate(4);
                    players_to_match = queued_2v2;
                    }
                    else if queued_1v1.len() >= 2 {
                      queued_1v1.truncate(2);
                      players_to_match = queued_1v1;
                    }
                    for matched_player in players_to_match.clone() {
                      players[matched_player].queued = false;
                    }
                  }
                  // Create a game
                  if !players_to_match.is_empty() {
                    let port = common::get_random_port();
                    {
                      let mut fleet = local_fleet.lock().unwrap();
                      let mut player_info = Vec::new();
                      let mut team_counter = 0;
                      for player_index in 0..players_copy.len() {
                        if players_to_match.contains(&player_index) {
                          let mut player = players_copy[player_index].clone();
                          if team_counter < (players_to_match.len() / 2) {
                            player.assigned_team = common::Team::Red;
                            team_counter += 1;
                          }
                          player_info.push(player);
                        }
                      }
                      // MARK: Game Server
                      let thread_database = Arc::clone(&local_database);
                      fleet.push(
                        std::thread::spawn(move || {
                          let players = player_info.clone();
                          match std::panic::catch_unwind(|| {sylvan_row::gameserver::game_server(player_info.len(), port, player_info)}){
                            Ok(winning_team) => {
                              println!("Winning team: {:?}", winning_team);
                              let mut database = thread_database.lock().unwrap();
                              // assign victories.
                              for player in players {
                                if player.assigned_team == winning_team {
                                  // put the victory in the database
                                  let mut player_data: PlayerData = match database::get_player(&database, &player.username) {
                                    Ok(data) => data,
                                    Err(_err) => {continue;}
                                  };
                                  player_data.wins += 1;
                                  match database::create_player(&mut database, &player.username, player_data) {
                                    Ok(_) => {},
                                    Err(_err) => {},
                                  }
                                }
                              }
                            },
                            Err(error) => {
                              println!("Game server crashed: {:?}", error);
                            }
                          };
                        }
                      ));
                    }
                    for pm_index in players_to_match {
                      players_copy[pm_index].channel.send(PlayerMessage::SendPacket(
                        ServerToClientPacket {
                          information: ServerToClient::MatchAssignment(
                            MatchAssignmentData { port: port }
                          )
                        }
                      )).await.unwrap();
                    }
                  }
                }
                // MARK: Match Cancel
                ClientToServer::MatchRequestCancel => {
                  {
                    let mut players = local_players.lock().unwrap();
                    // player's index
                    let p_index = from_user(&username, players.clone());
                    players[p_index].queued = false;
                  }
                }
                // MARK: Data Request
                ClientToServer::PlayerDataRequest => {
                  // client wants to see their stats!!!
                  let player_stats: PlayerStatistics;
                  {
                    let database = local_database.lock().unwrap();
                    let player_data = match database::get_player(&database, &username) {
                      Ok(data) => data,
                      Err(_err) => {
                        continue;
                      }
                    };
                    player_stats = PlayerStatistics {
                      wins: player_data.wins as u16,
                    };
                  }
                  tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                    information: ServerToClient::PlayerDataResponse(player_stats),
                  })).await.unwrap();
                }
                // MARK: Get Friend List
                // expensive operation
                ClientToServer::GetFriendList => {
                  println!("getting friend list...");
                  let friend_list: Result<Vec<(String, FriendShipStatus)>, redb::Error>;
                  {
                    let database = local_database.lock().unwrap();
                    friend_list = database::get_status_list(&database, &username);
                  }
                  match friend_list {
                    Ok(friend_list) => {
                      let mut final_friend_list: Vec<(String, FriendShipStatus, bool)> = Vec::new();
                      {
                        let players = local_players.lock().unwrap();
                        for friend in friend_list {
                          // only tell the user the peer is online if they're friends.
                          let mut online: bool = false;
                          if friend.1 == FriendShipStatus::Friends {
                            for player in players.clone() {
                              if player.username == username {
                                continue;
                              }
                              let split: Vec<&str> = friend.0.split(":").collect();
                              if split[0] == player.username
                              || split[1] == player.username {
                                online = true;
                                break;
                              }
                            }
                          } else {
                            online = false;
                          }
                          final_friend_list.push((friend.0, friend.1, online));
                        }
                      }
                      println!("{:?}", final_friend_list);
                      tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::FriendListResponse(final_friend_list),
                      })).await.unwrap();
                    }
                    Err(err) => {
                      println!("{:?}", err);
                      tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                      })).await.unwrap();
                    }
                  }
                }
                // FR = Friend Request
                // MARK: FR / FR Accept
                ClientToServer::SendFriendRequest(other_user) => {
                  println!("{:?}", other_user);
                  if username == other_user {
                    tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                      information: ServerToClient::InteractionRefused(RefusalReason::ThatsYouDummy),
                    })).await.unwrap();
                    continue;
                  }
                  let username_exists: Result<bool, redb::Error>;
                  let current_status: Result<FriendShipStatus, redb::Error>;
                  // ends up being true if the request was successful, and false if
                  // there was an internal error (database error).
                  let mut request_successful = false;
                  // true if the interaction led to a friendship grant.
                  // (Happens if FR is sent to someone who already sent you an FR)
                  let mut friendship_achieved = false;
                  {
                    let database = local_database.lock().unwrap();
                    username_exists = database::username_taken(&database, &other_user);
                    current_status = database::get_friend_status(&database, &username, &other_user);
                  }
                  match username_exists {
                    Ok(exists) => {
                      if !exists {
                        // the requested user doesn't exist.
                        tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::UsernameInexistent),
                        })).await.unwrap();
                        continue;
                      }
                    }
                    Err(err) => {
                      println!("1 Error: {:?}", err);
                      // database error (internal server error)
                      tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                      })).await.unwrap();
                      continue;
                    }
                  }
                  match current_status {
                    Ok(status) => {
                      match status {
                        FriendShipStatus::Friends => {
                          // if they're already friends, ignore
                          tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                            information: ServerToClient::InteractionRefused(RefusalReason::AlreadyFriends),
                          })).await.unwrap();
                          continue;
                        }
                        FriendShipStatus::PendingForA | FriendShipStatus::PendingForB => {
                          let predicted_pending_status: FriendShipStatus = database::get_friend_request_type(&username, &other_user);
                          if predicted_pending_status == status {
                            // this friend request was already made.
                            tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                              information: ServerToClient::InteractionRefused(RefusalReason::FriendRequestAlreadySent),
                            })).await.unwrap();
                            continue;
                          } else {
                            // the other user also sent a friend request, which means either they
                            // both want to be friends, or one is accepting a request.
                            // either way, let's grant the friendship.
                            {
                              let database = local_database.lock().unwrap();
                              match database::set_friend_status(&database, &username, &other_user, FriendShipStatus::Friends) {
                                Ok(_) => {
                                  // friendship successful by mutual request.
                                  request_successful = true;
                                  friendship_achieved = true;
                                }
                                Err(err) => {
                                  println!("2 Error: {:?}", err);
                                  // friendship failed (internal error).
                                  request_successful = false;
                                }
                              }
                            }
                          }
                        }
                        FriendShipStatus::Blocked => {
                          // blocked, so ignore.
                          tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                            information: ServerToClient::InteractionRefused(RefusalReason::UsersBlocked),
                          })).await.unwrap();
                          continue;
                        }
                      }
                    }
                    Err(err) => {
                      println!("3 Error: {:?}", err);
                      match err {
                        redb::Error::Corrupted(reason) => {
                          if reason == "norelation" {
                            // the users have no entry in the database, set their status as pending.
                            let pending_status: FriendShipStatus = database::get_friend_request_type(&username, &other_user);
                            {
                              let database = local_database.lock().unwrap();
                              match database::set_friend_status(&database, &username, &other_user, pending_status) {
                                Ok(_) => {
                                  // friend request successfully sent
                                  request_successful = true;
                                }
                                Err(err) => {
                                  println!("4 Error: {:?}", err);
                                  // friend request failed (internal error).
                                  request_successful = false;
                                }
                              }
                            }
                          }
                        }
                        _ => {
                          // some other error happened in the database.
                          tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                            information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                          })).await.unwrap();
                          continue;
                        }
                      }
                    }
                  }
                  if request_successful {
                    if friendship_achieved {
                      // users are now friends.
                      tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::FriendshipSuccessful,
                      })).await.unwrap();
                    } else {
                      // the friend request was successfully sent to the peer.
                      tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::FriendRequestSuccessful,
                      })).await.unwrap();
                    }
                  }
                  else {
                    // request unsuccessful due to an internal error (db error).
                    tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                      information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                    })).await.unwrap();
                  }
                }
                _ => {}
              }
            }
          }

          thread_message = rx.recv() => {
            if let Some(message) = thread_message {
              match message {
                PlayerMessage::ForceDisconnect => {
                  return;
                }
                PlayerMessage::SendPacket(packet) => {
                  nonce += 1;
                  socket.write_all(
                    &packet.cipher(nonce, cipher_key.clone())
                  ).await.unwrap();
                }
              }
            }
          }
        }
      }
    });
  }
}

fn from_user(username: &String, players: Vec<PlayerInfo>) -> usize {
  for p_index in 0..players.len() {
    if &players[p_index].username == username {
      return p_index;
    }
  }
  return usize::MAX;
}