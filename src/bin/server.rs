use redb::{Database, Result};
use sylvan_row::{common, const_params::*, database::{self, FriendShipStatus, PlayerData}, filter, gamedata::Character, mothership_common::*, network} ;
use std::{sync::{Arc, Mutex}, thread::JoinHandle};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::mpsc, net::{TcpListener}};
use ring::hkdf;
use opaque_ke::{ServerLoginStartResult};
use rand::{rngs::OsRng};
use opaque_ke::{
  RegistrationResponse, ServerLogin,
  ServerLoginParameters, ServerRegistration,
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
      loop {
        // this thing is really cool and handles whichever branch is ready first
        tokio::select! {
          // wait until we recieve packet, and write it to buffer.
          socket_read = socket.read(&mut buffer) => {
            //std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
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
              let packets: Result<Vec<ClientToServerPacket>, bincode::Error> = network::tcp_decode::<ClientToServerPacket>(buffer[..len].to_vec());
              let packets: Vec<ClientToServerPacket> = match packets {
                Ok(packets) => packets,
                Err(_err) => {
                  // write an internal error here maybe?
                  println!("hi");
                  continue;
                }
              };
              for packet in packets {
                match packet.information {                    
                  // MARK: Registration
                  ClientToServer::RegisterRequestStep1(recieved_username, client_message) => {
                    username = recieved_username.clone();
                    let valid = filter::valid_username(username.clone());
                    if !valid {
                      let _ = socket.write_all(&network::tcp_encode(ServerToClientPacket {
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
                        let _ = socket.write_all(&network::tcp_encode(&ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                        //.expect() is ok to use on serialize because we control what gets serialized.
                        }).expect("hi1")).await;
                        continue;
                      }
                    };
                    if username_taken_real {
                      let _ = socket.write_all(&network::tcp_encode(&ServerToClientPacket {
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
                        let _ = socket.write_all(&network::tcp_encode(&ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                        }).expect("hi3")).await;
                        continue;
                      }
                    };
                    let response: RegistrationResponse<DefaultCipherSuite> = server_registration_start_result.message;
                    // reply to the client
                    // this doesnt reply
                    let _ = socket.write_all(&network::tcp_encode(&ServerToClientPacket {
                      information: ServerToClient::RegisterResponse1(response),
                    }).expect("hi4")).await;
                  }
                  ClientToServer::RegisterRequestStep2(client_message) => {
                    if username == String::new() {
                      let _ = socket.write_all(&network::tcp_encode(&ServerToClientPacket {
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
                        let _ = socket.write_all(&network::tcp_encode(&ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                        }).expect("hi7")).await;
                        continue;
                      }
                    }
                    if !user_exists_real {
                      let _ = socket.write_all(&network::tcp_encode(&ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(RefusalReason::UsernameInexistent),
                      }).expect("hi7")).await;
                      continue;
                    }
                    match password_file {
                      Ok(playerdata) => password_file_real = playerdata.password_hash,
                      Err(_err) => {
                      let _ = socket.write_all(&network::tcp_encode(&ServerToClientPacket {
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
                        let _ = socket.write_all(&network::tcp_encode(&ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                        }).expect("hi8")).await;
                        continue;
                      },
                    };
                    server_login_start_result = Some(result);
                    let response = server_login_start_result.as_ref().unwrap().message.clone();
                    let _ = socket.write_all(&network::tcp_encode(&ServerToClientPacket {
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

                      // MARK: Init after login

                      // login is successful, so append the user to the player list.
                      // however, if they already exist, disconnect the old session.
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
                            queued_with: Vec::new(),
                            is_party_leader: true, // pary leader by default. demoted if invited. promoted back if you leave the party.
                            invited_by: Vec::new(),
                            is_ready: false,
                          }
                        );
                      }
                    }
                  }
                  _ => {
                    // Ignore packet. Invalid.
                  }
                }
              }
            }
            // logged in, so use cipher
            // MARK: ===============
            else {

              let packets = network::tcp_decode_decrypt::<ClientToServerPacket>(buffer[..len].to_vec(), cipher_key.clone(), &mut last_nonce);
              let packets = match packets {
                Ok(packets) => {packets},
                Err(_err) => {
                  continue;
                }
              };
              for packet in packets {

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
                      let p_index = match from_user(&username, players.clone()) {
                        Ok(index) => index,
                        Err(()) => {
                          // this has no reason to happen lowkey.
                          continue;
                        }
                      };
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
                      // debug
                      let player_count_1v1: usize = if MATCHMAKE_ALONE { 1 } else { 2 };

                      if queued_2v2.len() >= 4 {
                        queued_2v2.truncate(4);
                      players_to_match = queued_2v2;
                      }
                      else if queued_1v1.len() >= player_count_1v1 {
                        queued_1v1.truncate(player_count_1v1);
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
                      let p_index = match from_user(&username, players.clone()) {
                        Ok(index) => index,
                        Err(()) => {
                          // this has no reason to happen lowkey.
                          continue;
                        }
                      };
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
                            println!("3 Error: {:?}", err);
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
                  // MARK: Chat Message
                  ClientToServer::SendChatMessage(peer_username, message) => {
                    println!("Chat message: {} : {}", peer_username, message);
                    let mut peers_are_friends: bool = false;
                    let mut internal_error_occurred: bool = false;
                    {
                      let database = local_database.lock().unwrap();
                      match database::get_friend_status(&database, &peer_username, &username) {
                        Ok(status) => {
                          if status == FriendShipStatus::Friends {
                            peers_are_friends = true;
                          }
                        }
                        Err(err) => {
                          match err {
                            redb::Error::Corrupted(reason) => {
                              if reason == "norelation" {
                                //peers_are_friends = false;
                              } else {
                                internal_error_occurred = true;
                              }
                            }
                            _ => {
                              internal_error_occurred = true;
                            }
                          }
                        }
                      }
                    }
                    if internal_error_occurred {
                      tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(
                          RefusalReason::InternalError
                        ),
                      })).await.unwrap();
                      continue;
                    }
                    // if they indeed are friends, we can proceed.
                    if peers_are_friends {
                      let mut peer_user_online = true;
                      let peer_channel;
                      {

                        let players = local_players.lock().unwrap();
                        let index_of_peer: usize = match from_user(&peer_username, players.clone()) {
                          Ok(index) => {index},
                          Err(()) => {
                            peer_user_online = false;
                            0 // return a gibberish value who cares it won't be read.
                          }
                        };
                        peer_channel = players[index_of_peer].channel.clone();
                      }
                      if !peer_user_online {
                        tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(
                            RefusalReason::UserNotOnline
                          ),
                        })).await.unwrap();
                        continue;
                      }
                      peer_channel.send(
                        PlayerMessage::SendPacket(
                          ServerToClientPacket { information: 
                          ServerToClient::ChatMessage(username.clone(), message, ChatMessageType::Private) }
                        )
                      ).await.unwrap();
                    }
                    // peers are not friends.
                    else {
                      tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(
                          RefusalReason::NotFriends
                        ),
                      })).await.unwrap();
                    }
                  }
                  // MARK: Lobby invite
                  ClientToServer::LobbyInvite(other_player) => {
                    let mut player_not_found = false;
                    let mut not_friends = false;
                    let mut other_player_channel: Option<tokio::sync::mpsc::Sender<PlayerMessage>> = None;
                    {
                      let mut players = local_players.lock().unwrap();
                      match from_user(&other_player, players.clone()) {
                        Ok(index) => {
                          // check if they're friends.
                          let friends;
                          {
                            let database = local_database.lock().unwrap();
                            match database::get_friend_status(&database, &username.clone(), &other_player) {
                              Ok(friendship_status) => {
                                friends = friendship_status == FriendShipStatus::Friends;
                              }
                              Err(_err) => {
                                friends = false;
                              }
                            }
                          }
                          if friends {
                            // updated invited_by list for the other player.
                            players[index].invited_by.push(username.clone());
                            // now send a packet to the player
                            // the code a few lines below will perform this duty.
                            other_player_channel = Some(players[index].channel.clone());
                          }
                          // if not friends
                          else {
                            not_friends = true;
                          }
                        }
                        // no exist >:(
                        Err(_) => {
                          player_not_found = true;
                        }
                      };
                    }
                    // no exist >:(
                    if player_not_found {
                      tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(
                          RefusalReason::UserNotOnline
                        ),
                      })).await.unwrap();
                      continue;
                    }
                    if not_friends {
                      tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(
                          RefusalReason::NotFriends
                        ),
                      })).await.unwrap();
                      continue;
                    }
                    // since everything else went well, operation inform other client is go.
                    if let Some(channel) = other_player_channel {
                      channel.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::LobbyInvite(username.clone())
                      })).await.unwrap();
                    }
                  }
                  // MARK: Lobby accept
                  ClientToServer::LobbyInviteAccept(other_player) => {
                    let mut users_to_inform: Vec<tokio::sync::mpsc::Sender<PlayerMessage>> = Vec::new();
                    let mut lobby_info_update: Vec<LobbyPlayerInfo> = Vec::new();
                    let mut user_not_online = false;
                    let mut already_in_party = false;
                    {
                      let mut players = local_players.lock().unwrap();
                      let other_index_result = from_user(&other_player, players.clone());
                      match other_index_result {
                        Ok(other_index) => {
                          let own_index = from_user(&username, players.clone()).expect("oops");

                          // check if already in party
                          if !players[own_index].queued_with.is_empty() {
                            already_in_party = true;
                          }
                          else {

                            // check if was invited
                            let other_username = players[other_index].username.clone();
                            let was_invited = players[own_index].invited_by.contains(&other_username);
                            if was_invited {

                              // remove the invitation
                              players[own_index].invited_by.retain(|element| (element != &other_username));

                              // update own player
                              players[own_index].queued_with = vec![other_username.clone()];
                              players[own_index].is_party_leader = false;
                              players[own_index].queued = false;
                              
                              // update other players
                              players[other_index].is_party_leader = true;
                              players[other_index].queued_with.push(username.clone());
                              users_to_inform.push(players[other_index].channel.clone());
                              lobby_info_update.push(
                                LobbyPlayerInfo {
                                  username: other_username.clone(),
                                  is_ready: false,
                                }
                              );
                              for player in players[other_index].queued_with.clone() {
                                match from_user(&player, players.clone()) {
                                  Ok(index) => {
                                    users_to_inform.push(players[index].channel.clone());
                                    lobby_info_update.push(
                                      LobbyPlayerInfo {
                                        username: players[index].username.clone(),
                                        is_ready: players[index].is_ready.clone(),
                                      }
                                    );
                                  }
                                  Err(_err) => {
                                    // don't bother.
                                  }
                                }
                              }
                            }
                          }
                        }
                        Err(_err) => {
                          user_not_online = true;
                        }
                      }
                    }
                    // send error packets
                    if user_not_online {
                      tx.send(
                        PlayerMessage::SendPacket(
                          ServerToClientPacket { 
                            information: ServerToClient::InteractionRefused(
                              RefusalReason::UserNotOnline
                            )
                          }
                        )
                      ).await.unwrap();
                      continue;
                    }
                    if already_in_party {
                      tx.send(
                        PlayerMessage::SendPacket(
                          ServerToClientPacket { 
                            information: ServerToClient::InteractionRefused(
                              RefusalReason::AlreadyInPary
                            )
                          }
                        )
                      ).await.unwrap();
                      continue;
                    }
                    for user in users_to_inform {
                      user.send(
                        PlayerMessage::SendPacket(
                          ServerToClientPacket {
                            information: ServerToClient::LobbyUpdate(
                              lobby_info_update.clone(),
                            )
                          }
                        )
                      ).await.unwrap();
                    }
                  }
                  // MARK: Lobby leave
                  ClientToServer::LobbyLeave => {
                    let mut users_to_inform: Vec<tokio::sync::mpsc::Sender<PlayerMessage>> = Vec::new();
                    let mut lobby_info_update: Vec<LobbyPlayerInfo> = Vec::new();
                    {
                      let mut players = local_players.lock().unwrap();
                      let own_index = from_user(&username, players.clone()).expect("oops");

                      let is_party_leader = players[own_index].is_party_leader;
                      if is_party_leader {
                        // we are the party leader so we need to transfer ownership.
                        if !players[own_index].queued_with.is_empty() {
                          for player_name in players[own_index].queued_with.clone() {

                            match from_user(&player_name, players.clone()) {
                              Ok(new_owner_index) => {
                                players[new_owner_index].is_party_leader = true;
                                players[new_owner_index].queued_with = players[own_index].queued_with.clone();
                                players[new_owner_index].queued_with.retain(|element| (element != &username));
                                for player_name in players[new_owner_index].queued_with.clone() {
                                  match from_user(&player_name, players.clone()) {
                                    Ok(index) => {
                                      players[index].queued_with = players[new_owner_index].queued_with.clone();
                                    }
                                    Err(_) => {

                                    }
                                  }
                                }
                                

                                break;
                              }
                              Err(_err) => {
                                // this user encountered an error, let's try again.
                              }
                            }
                          }
                        }
                        else {
                          // we are alone in this party, whatever bro
                        }
                      }
                      else {
                        // we are not a party leader so we're only storing the name of our party leader.
                        if !players[own_index].queued_with.is_empty() {
                          let other_player = players[own_index].queued_with[0].clone();
                          
                          
                          // inform everybody
                          match from_user(&other_player, players.clone()) {
                            Ok(other_index) => {
                              
                              // remove ourselves from the party leader's party
                              players[other_index].queued_with.retain(|element| (element != &username));

                              // for every username in the other party leader's party, inform of the departure.
                              for other_username in players[other_index].queued_with.clone() {
                                // if this isn't us (leaving player)
                                if other_username != username {
                                  // inform everybody
                                  match from_user(&other_username, players.clone()) {
                                    Ok(index) => {
                                      users_to_inform.push(players[index].channel.clone());
                                      lobby_info_update.push(LobbyPlayerInfo {
                                        username: other_username,
                                        is_ready: players[index].is_ready,
                                      })
                                    }
                                    Err(_err) => {
                                      // ignore this.
                                    }
                                  }
                                }
                              }
                            }
                            Err(_err) => {
                              // the other user is offline, or some other shenanigan. Don't worry about them.
                            }
                          }
                        }
                        else {
                          // this shouldn't really happen, the user is asking to leave the lobby when they're
                          // not in one.
                        }

                        // reset flags
                        players[own_index].is_party_leader = true;
                        players[own_index].queued = false;
                        players[own_index].queued_with = Vec::new();
                      }
                    }
                    // the leaving client will not be informed of their departure, they already know it happened.
                  }
                  // packets that shouldn't arrive.
                  _ => {
                    // ignore
                  }
                }
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
                    &network::tcp_encode_encrypt(packet, cipher_key.clone(), nonce).expect("oops")
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

fn from_user(username: &String, players: Vec<PlayerInfo>) -> Result<usize, ()> {
  for p_index in 0..players.len() {
    if &players[p_index].username == username {
      return Ok(p_index);
    }
  }
  return Err(())
}