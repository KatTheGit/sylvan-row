use redb::{Database, Result};
use sylvan_row::{const_params::*, database::{self, FriendShipStatus, PlayerData}, filter::{self, ProfanityLevel, contains_profanity}, gamedata::*, mothership_common::*, network} ;
use std::{collections::HashMap, io::Write, sync::{Arc, Mutex}, thread::JoinHandle, time::{Instant, SystemTime, UNIX_EPOCH}, vec};
use tokio::{io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt}, net::TcpListener, sync::mpsc};
use ring::hkdf;
use opaque_ke::{ServerLoginStartResult};
use rand::{rngs::OsRng};
use opaque_ke::{
  RegistrationResponse, ServerLogin,
  ServerLoginParameters, ServerRegistration,
};
use pollster::FutureExt as _;
use log::{info, warn, error};
use std::panic;
use std::backtrace::Backtrace;
use rand::Rng;
use std::fs::File;


#[tokio::main]
async fn main() {

  // start logger
  let default_file = include_bytes!("../../log4rs.yaml");
  match File::create_new("log4rs.yaml") {
    // create a log4rs.yaml file if it can't be found.
    Ok(mut file) => {
      file.write_all(default_file).expect("oops");
    },
    // otherwise do nothing.
    Err(_err) => {},
  };
  // start logger
  log4rs::init_file("log4rs.yaml", Default::default()).expect("Could not initialise log4rs.");

  // make sure all panics are fully logged.
  panic::set_hook(Box::new(|info| {
    let backtrace = Backtrace::force_capture();
    let location = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_else(|| String::from("<unknown>"));
    if let Some(s) = info.payload().downcast_ref::<&str>() {
      error!("PANIC at {}: {}\n{}", location, s, backtrace);
      println!("PANIC at {}: {}\n{}", location, s, backtrace);
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
      error!("PANIC at {}: {}\n{}", location, s, backtrace);
      println!("PANIC at {}: {}\n{}", location, s, backtrace);
    } else {
      error!("PANIC at {} (unknown payload)\n{}", location, backtrace);
      println!("PANIC at {} (unknown payload)\n{}", location, backtrace);
    }
  }));

  let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", SERVER_PORT)).await.unwrap();

  let players: Vec<PlayerInfo> = Vec::new();
  // Arc allows for shared access, and Mutex makes it mutually exclusive.
  let players = Arc::new(Mutex::new(players));

  // Current set of gamemodes proposed by the matchmaker.
  let gamemode_rotation: Vec<GameMode> = vec![
    GameMode::Standard1V1,
    GameMode::Standard2V2,
    GameMode::Ctp2V2,
    GameMode::Ctp1V1,
  ];
  let gamemode_rotation = Arc::new(Mutex::new(gamemode_rotation));
  
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
  // MARK: Console input
  let commandline_players = Arc::clone(&players);
  let commandline_database = Arc::clone(&database);
  tokio::spawn(async move {
    let stdin = tokio::io::stdin();
    let mut lines = tokio::io::BufReader::new(stdin).lines();
    while let Ok(Some(line)) = lines.next_line().await {
      let tokens: Vec<&str> = line.split_whitespace().collect();
      println!("{:?}", tokens);
      if let Some(token_0) = tokens.get(0) {
        match *token_0 {
          // chat announcement for every player.
          // announce <message>
          "announce" => {
            // ignore if no parameters.
            if let Some(token_1) = tokens.get(1) { } else {
              println!("Enter message.");
              continue;
            }
            let players_copy;
            {
              let players = commandline_players.lock().unwrap();
              players_copy = players.clone()
            }
            let message: String = line[9..].to_string();
            for player in players_copy.clone() {
              match player.channel.send(
                PlayerMessage::SendPacket(ServerToClientPacket { information: ServerToClient::ChatMessage(String::from("Server Announcement"), message.clone(), ChatMessageType::Administrative) })
              ).await {
                Ok(_) => {}
                Err(_) => {}
              };
            }
          }
          // ban <user> <days>
          "ban" => {
            if let Some(token_1) = tokens.get(1) {
              if let Some(token_2) = tokens.get(2) {
                // get the current time as seconds.
                let current_time = SystemTime::now()
                  .duration_since(UNIX_EPOCH)
                  .unwrap()
                  .as_secs() as i64;
                match token_2.parse::<i64>() {
                  Ok(ban_days) => {
                    let banned_until: i64 = current_time + ban_days * 24 * 60 * 60;
                    // if the player is online, get them out of here.
                    let players_copy;
                    {
                      let players = commandline_players.lock().unwrap();
                      players_copy = players.clone()
                    }
                    for player in players_copy {
                      if player.username == *token_1 {
                        // inform of ban
                        match player.channel.send(
                          PlayerMessage::SendPacket(ServerToClientPacket { information: ServerToClient::InteractionRefused(RefusalReason::Ban(banned_until)) })
                        ).await {
                          Ok(_) => {}
                          Err(_) => {}
                        };
                        // force disconnect
                        match player.channel.send(
                          PlayerMessage::ForceDisconnect
                        ).await {
                          Ok(_) => {}
                          Err(_) => {}
                        };
                        break;
                      }
                    }
                    // save the ban in the database
                    {
                      let mut database = commandline_database.lock().unwrap();
                      let player_data_result = database::get_player(&database, token_1);
                      match player_data_result {
                        Ok(mut player_data) => {
                          player_data.ban = banned_until;
                          let _ = database::create_player(&mut database, token_1, player_data);
                          println!("{} banned.", token_1);
                        }
                        Err(_) => {
                          println!("Database error");
                        }
                      }
                    }
                  }
                  Err(_) => {
                    println!("That's not a number! Enter the number of days to ban this user as your second argument.");
                  }
                }
              } else {
                println!("Please enter the number of days to ban this user as your second argument.");
              }
            } else {
              println!("Please enter the bannee's username.");
            }
          }
          // unban <user>
          "unban" => {
            if let Some(token_1) = tokens.get(1) {
              {
                let mut database = commandline_database.lock().unwrap();
                let player_data_result = database::get_player(&database, *token_1);
                match player_data_result {
                  Ok(mut player_data) => {
                    player_data.ban = 0;
                    let _ = database::create_player(&mut database, token_1, player_data);
                    println!("{} unbanned.", token_1);
                  }
                  Err(_) => {
                    println!("Database error");
                  }
                }
              }
            }
          }
          _ => {
            println!("Unrecognised command.");
          }
        }
      }
      // no command
      else {
        println!("Enter a command.");
      }
    }
  });
  info!("Server started.");
  loop {
    // MARK: Net init
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
    let local_server_setup = Arc::clone(&server_setup);
    let local_gamemode_rotation = Arc::clone(&gamemode_rotation);
    let mut logged_in: bool = false;
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

      let mut rate_limit_counter: u8 = 0;
      let mut last_packet_time = Instant::now();

      loop {
        // this thing is really cool and handles whichever branch is ready first
        tokio::select! {
          // wait until we recieve packet, and write it to buffer.
          socket_read = socket.read(&mut buffer) => {
            println!("packet recieved.");
            //std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
            let len: usize = match socket_read {
              Ok(0) => {
                println!("disconnect.");

                // disconnect
                // remove player from players.
                if logged_in {
                  {
                    let mut players = local_players.lock().unwrap();
                    let p_index = match from_user(&username, players.clone()) {
                      Ok(index) => {index}
                      Err(_err) => {
                        return;
                      }
                    };
                    players.remove(p_index);
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
            //{
            //  println!("==================================");
            //  let players = local_players.lock().unwrap();
            //  for player in players.clone() {
            //    println!("");
            //    println!("name:             {:?}", player.username);
            //    println!("queued:           {:?}", player.queued);
            //    println!("queued_with:      {:?}", player.queued_with);
            //    println!("queued_gamemodes: {:?}", player.queued_gamemodes);
            //    println!("is_pary_leader:   {:?}", player.is_party_leader);
            //    println!("in_game_with:     {:?}", player.in_game_with);
            //  }
            //}

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
                    println!("user wants to register");
                    username = recieved_username.clone();
                    let valid = filter::valid_username(&username) && !contains_profanity(&username, ProfanityLevel::SlursAndSwears);
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
                    let player_data_result: Result<PlayerData, redb::Error>;
                    let password_file_real: ServerRegistration<DefaultCipherSuite>;
                    let user_exists: Result<bool, redb::Error>;
                    let user_exists_real: bool;
                    {
                      let database = local_database.lock().unwrap();
                      user_exists = database::username_taken(&database, &username);
                      player_data_result = database::get_player(&database, &username);
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
                    match player_data_result {
                      Ok(player_data) => {
                        password_file_real = player_data.password_hash;
                        // check if this user is banned.
                        let ban_duration = player_data.ban - SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
                        if ban_duration > 0 {
                          let _ = socket.write_all(&network::tcp_encode(&ServerToClientPacket {
                            information: ServerToClient::InteractionRefused(RefusalReason::Ban(player_data.ban)),
                          }).expect("hi7")).await;
                          continue;
                        }
                      },
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
                        match from_user(&username, players.clone()) {
                          Ok(p_index) => {
                            channel_copy = Some(players[p_index].channel.clone());
                            players.remove(p_index);
                          },
                          Err(_err) => {
                            // player doesn't already exist so don't worry about it
                          }
                        };
                      }
                      if let Some(channel) = channel_copy {
                        match channel.send(PlayerMessage::ForceDisconnect).await{
                          Ok(_) => {},
                          Err(err) => {
                            println!("{:?}", err);
                          },
                        };
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
                            assigned_team: Team::Blue,
                            queued_with: Vec::new(),
                            is_party_leader: true, // pary leader by default. demoted if invited. promoted back if you leave the party.
                            invited_by: Vec::new(),
                            in_game_with: Vec::new(),
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
              // rate limiting calculations
              
              println!("pre decay: {:?}", rate_limit_counter);
              let rate_limit_decay = 3.0;
              let elapsed = last_packet_time.elapsed().as_secs_f32();
              let decay = (elapsed * rate_limit_decay) as u8;
              // if we are above threshold, make the decay sctricter.
              if rate_limit_counter > RATE_LIMIT_THRESHOLD {
                if elapsed > 3.0 {
                  if decay > rate_limit_counter {
                    rate_limit_counter = 0;
                  } else {
                    rate_limit_counter -= decay;
                  }
                }
              } else {
                // otherwise be nice to the user.
                if decay > rate_limit_counter {
                  rate_limit_counter = 0;
                } else {
                  rate_limit_counter -= decay;
                }
              }
              rate_limit_counter = rate_limit_counter.clamp(0, (RATE_LIMIT_THRESHOLD as f32 * 1.5) as u8);
              println!("decay: {:?}", decay);
              last_packet_time = Instant::now();
              println!("post decay: {:?}", rate_limit_counter);

              if rate_limit_counter > RATE_LIMIT_THRESHOLD {
                match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                  information: ServerToClient::InteractionRefused(RefusalReason::RateLimit),
                })).await{
                  Ok(_) => {},
                  Err(err) => {
                    println!("{:?}", err);
                    error!("{:?}", err);
                  },
                };
                continue;
              }
              
              
              let packets = network::tcp_decode_decrypt::<ClientToServerPacket>(buffer[..len].to_vec(), cipher_key.clone(), &mut last_nonce);
              let packets = match packets {
                Ok(packets) => {packets},
                Err(_err) => {
                  continue;
                }
              };
              for packet in packets {
                match packet.information {
                  // MARK: Match Request
                  ClientToServer::MatchRequest(data) => {

                    rate_limit_counter += 10;

                    let mut gamemode_rotation = Vec::new();
                    {
                      let gamemodes = local_gamemode_rotation.lock().unwrap();
                      for gamemode in gamemodes.clone() {
                        gamemode_rotation.push(gamemode);
                      }
                    }
                    if data.gamemodes.len() > gamemode_rotation.len()  {
                      // ignore invalid request
                      continue;
                    }
                    let mut gamemodes_valid = true;
                    {
                      let gamemodes = local_gamemode_rotation.lock().unwrap();
                      for gamemode in data.gamemodes.clone() {
                        if !gamemodes.contains(&gamemode) {
                          // trying to queue for a gamemode that is not in the current rotaion.
                          gamemodes_valid = false;
                        }
                      }
                    }
                    if !gamemodes_valid {
                      match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(RefusalReason::InvalidGameModeQueued),
                      })).await{
                        Ok(_) => {},
                        Err(err) => {
                          println!("{:?}", err);
                          error!("{:?}", err);
                        },
                      };
                    }
                    let mut players_to_inform: Vec<tokio::sync::mpsc::Sender<PlayerMessage>> = Vec::new();
                    let mut lobby_info: Vec<LobbyPlayerInfo> = Vec::new();

                    // perform 1 (one) matchmaking check
                    let players_copy: Vec<PlayerInfo>;
                    let mut players_to_match: Vec<usize> = Vec::new();
                    let mut match_gamemode = GameMode::Standard1V1;

                    {
                      // Find players to match.
                      let mut players = local_players.lock().unwrap();

                      let own_index = from_user(&username, players.clone()).expect("oops");
                      players[own_index].queued = true;
                      players[own_index].queued_gamemodes = data.gamemodes;
                      players[own_index].selected_character = data.character;

                      let mut queues: HashMap<GameMode, Vec<Vec<usize>>> = HashMap::new();
                      for gamemode in gamemode_rotation.clone() {
                        queues.insert(gamemode, Vec::new());
                      }

                      if !players[own_index].queued_with.is_empty() {
                        let mut party_leader_index = own_index;
                        if !players[own_index].is_party_leader {
                          match from_user(&players[own_index].queued_with[0], players.clone()) {
                            Ok(index) => {
                              party_leader_index = index;
                            }
                            Err(err) => {
                              // whatever
                              warn!("{:?}", err);
                            }
                          }
                        }
                        lobby_info.push(
                          LobbyPlayerInfo {
                            username: players[party_leader_index].username.clone(),
                            is_ready: players[party_leader_index].queued,
                          }
                        );
                        players_to_inform.push(players[party_leader_index].channel.clone());
                        for player in players[party_leader_index].queued_with.clone() {
                          match from_user(&player, players.clone()) {
                            Ok(index) => {
                              players_to_inform.push(players[index].channel.clone());
                              lobby_info.push(
                                LobbyPlayerInfo {
                                  username: players[index].username.clone(),
                                  is_ready: players[index].queued
                                }
                              );
                            }
                            Err(err) => {
                              warn!("{:?}", err);
                            }
                          }
                        }
                      }
                      
                      for player_index in 0..players.len() {
                        // add players to the queue
                        if players[player_index].queued {
                          
                          // if solo queueing
                          if players[player_index].queued_with.is_empty() {
                            for gamemode in players[player_index].queued_gamemodes.clone() {
                              queues.get_mut(&gamemode).unwrap().push(vec![player_index]);
                            }
                            
                          }
                          // if non-solo queueing (duo)
                          else {

                            if players[player_index].is_party_leader {
                              let lobby_owner_index;
                              lobby_owner_index = player_index;
                              
                              let mut lobby_players: Vec<usize> = Vec::new();
                              let mut all_ready: bool = true;
                              
                              for player_username in players[lobby_owner_index].queued_with.clone() {
                                match from_user(&player_username, players.clone()) {
                                  Ok(index) => {
                                    lobby_players.push(index);
                                    if !players[index].queued {
                                      all_ready = false;
                                      continue;
                                    }
                                  }
                                  Err(err) => {
                                    warn!("{:?}", err);
                                  }
                                }
                              }
                              if all_ready {
                                for queued_gamemode in players[lobby_owner_index].queued_gamemodes.clone() {
                                  let mut lobby = lobby_players.clone();
                                  lobby.push(lobby_owner_index);
                                  queues.get_mut(&queued_gamemode).unwrap().push(lobby);
                                }
                              }
                            }
                          }
                        }
                      }
                      
                      // match players
                      
                      println!("{:?}", queues);
                      
                      
                      // ALGORITHM
                      //for each element:
                      //  if size is team size:
                      //    we got a team
                      //  for each other element:
                      //    if can be added:
                      //      add it to a variable
                      //    if variable's stored size is team size:
                      //      we got a team

                      for queue in queues.clone() {
                        let queued_gamemode = queue.0;
                        let mut queued_players = queue.1.clone();
                        let mut matched_players: Vec<usize> = Vec::new();
                        
                        let gamemode_parameters = queued_gamemode.get_data();
                        let team_size = gamemode_parameters.team_size;
                        let team_count = gamemode_parameters.team_count;
                        
                        for _ in 0..team_count {
                          println!("{:?}: {:?}", queued_gamemode, queued_players);
                          for party_1_index in 0..queued_players.len() {
                            // if already team size
                            if queued_players[party_1_index].len() == team_size {
                              for player in queued_players[party_1_index].clone() {
                                matched_players.push(player);
                              }
                              queued_players.remove(party_1_index);
                              break
                            }
                            let mut current_team = queued_players[party_1_index].clone();
                            let mut current_queued_players = queued_players.clone();
                            for party_2_index in (0..queued_players.len()).rev() {
                              if party_1_index != party_2_index {
                                if current_team.len() + queued_players[party_2_index].len() <= team_size {
                                  for player in queued_players[party_2_index].clone() {
                                    current_team.push(player);
                                  }
                                  current_queued_players.remove(party_2_index);
                                }
                                if current_team.len() == team_size {
                                  current_queued_players.remove(party_1_index);
                                  queued_players = current_queued_players;
                                  break;
                                }
                              }
                            }
                            // team successfully created
                            if current_team.len() == team_size {
                              // add these to the matched players.
                              matched_players.extend(current_team.iter().cloned());
                              break;
                            }
                          }
                        }

                        if matched_players.len() == team_size * team_count {
                          // the match is on!
                          println!("Matched: {:?}", matched_players);
                          players_to_match = matched_players;
                          match_gamemode = queued_gamemode;
                          break;
                        }
                      }

                      let mut team_counter = 0;
                      for player_index in players_to_match.clone() {
                        players[player_index].queued = false;
                        if team_counter < (players_to_match.len() / 2) {
                          players[player_index].assigned_team = Team::Red;
                          team_counter += 1;
                        }
                      }
                      let mut other_players = Vec::new();
                      for player_index in players_to_match.clone() {
                        other_players.push(players[player_index].clone());
                      }
                      for player_index in players_to_match.clone() {
                        let mut other_players_without_self = other_players.clone();
                        other_players_without_self.retain(|element| element.username != players[player_index].username);
                        players[player_index].in_game_with = other_players_without_self;
                      }
                      players_copy = players.clone();
                    }
                    // lobby updates
                    for player in players_to_inform {
                      match player.send(
                        PlayerMessage::SendPacket(
                          ServerToClientPacket {
                            information: ServerToClient::LobbyUpdate(
                              lobby_info.clone()
                            )
                          }
                        )
                      ).await {
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                        },
                      };
                    }
                    let map = Map::Control1;
                    // Create a game
                    if !players_to_match.is_empty() {
                      let game_id: u128 = rand::thread_rng().gen_range(0..u128::MAX);
                      let port = get_random_port();
                      {
                        let mut fleet = local_fleet.lock().unwrap();
                        let mut player_info = Vec::new();
                        for player_index in 0..players_copy.len() {
                          if players_to_match.contains(&player_index) {
                            let player = players_copy[player_index].clone();
                            player_info.push(player);
                          }
                        }
                        // MARK: | Game Server
                        let thread_database = Arc::clone(&local_database);
                        let thread_players = Arc::clone(&local_players);
                        let match_gamemode_copy = match_gamemode.clone();
                        let match_map_copy = map.clone();
                        fleet.push(
                          std::thread::spawn(move || {
                            let player_info = player_info.clone();
                            println!("{:?}", match_gamemode_copy);
                            match std::panic::catch_unwind(|| {sylvan_row::gameserver::game_server(port, player_info.clone(), match_gamemode_copy, match_map_copy)}){
                              // game ended successfully.
                              Ok(mut match_result) => {
                                // update to the correct game_id since the gameserver isn't aware of it.
                                match_result.game_id = game_id;
                                {
                                  let mut database = thread_database.lock().unwrap();
                                  // assign victories.
                                  for player in player_info.clone() {
                                    if player.assigned_team.to_result() == match_result.winning_team {
                                      // put the victory in the database
                                      let mut player_data: PlayerData = match database::get_player(&database, &player.username) {
                                        Ok(data) => data,
                                        Err(err) => {
                                          warn!("{:?}", err);
                                          continue;
                                        }
                                      };
                                      player_data.wins += 1;
                                      match database::create_player(&mut database, &player.username, player_data) {
                                        Ok(_) => {},
                                        Err(err) => {
                                          error!("{:?}", err);
                                        },
                                      }
                                    }
                                  }
                                }
                                // reset "in game with" and inform players.
                                {
                                  let mut players = thread_players.lock().unwrap();
                                  for player in player_info.clone() {
                                    let server_player = from_user(&player.username, players.clone());
                                    match server_player {
                                      Ok(player) => {
                                        players[player].in_game_with.clear();
                                        players[player].assigned_team = Team::Blue;
                                        match players[player].channel.send(
                                          PlayerMessage::SendPacket(
                                            ServerToClientPacket {
                                              information: ServerToClient::MatchEnded(
                                                match_result.clone()
                                              )
                                            }
                                          )
                                        ).block_on() { // equivalent to .await but polls instead
                                          Ok(_) => {},
                                          Err(err) => {
                                            error!("{:?}", err);
                                          },
                                        };
                                      }
                                      Err(err) => {
                                        warn!("{:?}", err);
                                      }
                                    }
                                  }
                                }
                              },
                              Err(err) => {
                                println!("Game server crashed: {:?}", err);
                                error!("Game server: {:?}", err);
                                for player in player_info {
                                  warn!("Player {:?} from crashed game server was playing: {:?}", player.username, player.selected_character);
                                  let _ = player.channel.send(
                                    PlayerMessage::SendPacket(
                                      ServerToClientPacket {
                                        information: ServerToClient::GameServerCrashApology,
                                      }
                                    )
                                  ).block_on();
                                }
                              }
                            };
                          }
                        ));
                      }
                      for pm_index in players_to_match {
                        match players_copy[pm_index].channel.send(PlayerMessage::SendPacket(
                          ServerToClientPacket {
                            information: ServerToClient::MatchAssignment(
                              MatchAssignmentData {
                                port: port,
                                game_id,
                                gamemode: match_gamemode.clone(),
                                map: map.clone(),
                              }
                            )
                          }
                        )).await{
                          Ok(_) => {},
                          Err(err) => {
                            println!("{:?}", err);
                            error!("{:?}", err);
                          },
                        };
                      }
                    }
                  }
                  // MARK: Match Cancel
                  ClientToServer::MatchRequestCancel => {

                    //rate_limit_counter += 0;

                    let mut players_to_inform: Vec<tokio::sync::mpsc::Sender<PlayerMessage>> = Vec::new();
                    let mut lobby_info: Vec<LobbyPlayerInfo> = Vec::new();
                    {
                      let mut players = local_players.lock().unwrap();
                      // player's index
                      let own_index = match from_user(&username, players.clone()) {
                        Ok(index) => index,
                        Err(err) => {
                          // this has no reason to happen lowkey.
                          warn!("{:?}", err);
                          continue;
                        }
                      };
                      players[own_index].queued = false;
                      if !players[own_index].queued_with.is_empty() {
                        let mut party_leader_index = own_index;
                        if !players[own_index].is_party_leader {
                          match from_user(&players[own_index].queued_with[0], players.clone()) {
                            Ok(index) => {
                              party_leader_index = index;
                            }
                            Err(err) => {
                              warn!("{:?}", err);
                            }
                          }
                        }
                        lobby_info.push(
                          LobbyPlayerInfo {
                            username: players[party_leader_index].username.clone(),
                            is_ready: players[party_leader_index].queued,
                          }
                        );
                        players_to_inform.push(players[party_leader_index].channel.clone());
                        for player in players[party_leader_index].queued_with.clone() {
                          match from_user(&player, players.clone()) {
                            Ok(index) => {
                              players_to_inform.push(players[index].channel.clone());
                              lobby_info.push(
                                LobbyPlayerInfo {
                                  username: players[index].username.clone(),
                                  is_ready: players[index].queued
                                }
                              );
                            }
                            Err(err) => {
                              // whatever
                              warn!("{:?}", err);
                            }
                          }
                        }
                      }
                    }
                    for player in players_to_inform {
                      match player.send(
                        PlayerMessage::SendPacket(
                          ServerToClientPacket {
                            information: ServerToClient::LobbyUpdate(
                              lobby_info.clone()
                            )
                          }
                        )
                      ).await {
                        Ok(_) => {},
                        Err(err) => {
                          println!("{:?}", err);
                          error!("{:?}", err);
                        },
                      };
                    }
                  }
                  // MARK: Gamemode request
                  ClientToServer::GameModeDataRequest => {

                    rate_limit_counter += 20;

                    let gamemodes: Vec<GameMode>;
                    {
                      gamemodes = local_gamemode_rotation.lock().unwrap().clone();
                    }
                    match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                      information: ServerToClient::GameModeDataResponse(
                        gamemodes
                      ),
                    })).await{
                      Ok(_) => {},
                      Err(err) => {
                        error!("{:?}", err);
                        println!("{:?}", err);
                      },
                    };
                  }
                  // MARK: Data Request
                  ClientToServer::PlayerDataRequest => {
                    rate_limit_counter += 20;
                    // client wants to see their stats!!!
                    let player_stats: PlayerStatistics;
                    {
                      let database = local_database.lock().unwrap();
                      let player_data = match database::get_player(&database, &username) {
                        Ok(data) => data,
                        Err(err) => {
                          error!("{:?}", err);
                          continue;
                        }
                      };
                      player_stats = PlayerStatistics {
                        wins: player_data.wins as u16,
                      };
                    }
                    match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                      information: ServerToClient::PlayerDataResponse(player_stats),
                    })).await{
                      Ok(_) => {},
                      Err(err) => {
                        error!("{:?}", err);
                        println!("{:?}", err);
                      },
                    };
                  }
                  // MARK: Get Friend List
                  // expensive operation
                  ClientToServer::GetFriendList => {
                    rate_limit_counter += 10;

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
                        match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                          information: ServerToClient::FriendListResponse(final_friend_list),
                        })).await{
                          Ok(_) => {},
                          Err(err) => {
                            println!("{:?}", err);
                            error!("{:?}", err);
                          },
                        };
                      }
                      Err(err) => {
                        println!("{:?}", err);
                        error!("{:?}", err);
                        match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                        })).await{
                          Ok(_) => {},
                          Err(err) => {
                            println!("{:?}", err);
                            error!("{:?}", err);
                          },
                        };
                      }
                    }
                  }
                  // FR = Friend Request
                  // MARK: FR / FR Accept
                  ClientToServer::SendFriendRequest(other_user) => {
                    
                    rate_limit_counter += 5;

                    if username == other_user {
                      match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(RefusalReason::ThatsYouDummy),
                      })).await{
                        Ok(_) => {},
                        Err(err) => {
                          println!("{:?}", err);
                          error!("{:?}", err);
                        },
                      };
                      rate_limit_counter += 10;

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
                          match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                            information: ServerToClient::InteractionRefused(RefusalReason::UsernameInexistent),
                          })).await{
                            Ok(_) => {},
                            Err(err) => {
                              println!("{:?}", err);
                              error!("{:?}", err);
                            },
                          };
                          continue;
                        }
                      }
                      Err(err) => {
                        println!("1 Error: {:?}", err);
                        error!("{:?}", err);
                        // database error (internal server error)
                        match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                        })).await{
                          Ok(_) => {},
                          Err(err) => {
                            println!("{:?}", err);
                            error!("{:?}", err);
                          },
                        };
                        continue;
                      }
                    }
                    match current_status {
                      Ok(status) => {
                        match status {
                          FriendShipStatus::Friends => {
                            // if they're already friends, ignore
                            match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                              information: ServerToClient::InteractionRefused(RefusalReason::AlreadyFriends),
                            })).await{
                              Ok(_) => {},
                              Err(err) => {
                                error!("{:?}", err);
                                println!("{:?}", err);
                              },
                            };
                            rate_limit_counter += 10;
                            continue;
                          }
                          FriendShipStatus::PendingForA | FriendShipStatus::PendingForB => {
                            let predicted_pending_status: FriendShipStatus = database::get_friend_request_type(&username, &other_user);
                            if predicted_pending_status == status {
                              // this friend request was already made.
                              match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                                information: ServerToClient::InteractionRefused(RefusalReason::FriendRequestAlreadySent),
                              })).await{
                                Ok(_) => {},
                                Err(err) => {
                                  error!("{:?}", err);
                                  println!("{:?}", err);
                                },
                              };
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
                                    error!("{:?}", err);
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
                            match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                              information: ServerToClient::InteractionRefused(RefusalReason::UsersBlocked),
                            })).await{
                              Ok(_) => {},
                              Err(err) => {
                                error!("{:?}", err);
                                println!("{:?}", err);
                              },
                            };
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
                                    error!("{:?}", err);
                                    // friend request failed (internal error).
                                    request_successful = false;
                                  }
                                }
                              }
                            }
                          }
                          _ => {
                            println!("3 Error: {:?}", err);
                            error!("{:?}", err);
                            // some other error happened in the database.
                            match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                              information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                            })).await{
                              Ok(_) => {},
                              Err(err) => {
                                error!("{:?}", err);
                                println!("{:?}", err);
                              },
                            };
                            continue;
                          }
                        }
                      }
                    }
                    if request_successful {
                      if friendship_achieved {
                        // users are now friends.
                        match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                          information: ServerToClient::FriendshipSuccessful,
                        })).await{
                          Ok(_) => {},
                          Err(err) => {
                            error!("{:?}", err);
                            println!("{:?}", err);
                          },
                        };
                      } else {
                        // the friend request was successfully sent to the peer.
                        match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                          information: ServerToClient::FriendRequestSuccessful,
                        })).await{
                          Ok(_) => {},
                          Err(err) => {
                            error!("{:?}", err);
                            println!("{:?}", err);
                          },
                        };
                      }
                    }
                    else {
                      // request unsuccessful due to an internal error (db error).
                      match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(RefusalReason::InternalError),
                      })).await{
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                          println!("{:?}", err);
                        },
                      };
                    }
                  }
                  // MARK: Chat Message
                  ClientToServer::SendChatMessage(peer_username, message) => {
                    rate_limit_counter += 2;

                    println!("Chat | {} -> {} | {}", username, peer_username, message);
                    let mut peers_are_friends: bool = false;
                    let mut internal_error_occurred: bool = false;
                    let mut channel_invalid: bool = false;

                    // team chat and all chat
                    if peer_username == String::from("tc") || peer_username == String::from("ac") {
                      let mut message_type = ChatMessageType::All;
                      let mut players_to_inform: Vec<tokio::sync::mpsc::Sender<PlayerMessage>> = Vec::new();
                      {
                        let players = local_players.lock().unwrap();
                        let own_index = from_user(&username, players.clone()).expect("oops");

                        if players[own_index].in_game_with.is_empty() {
                          channel_invalid = true;
                        }
                        else {
                          for other_player in players[own_index].in_game_with.clone() {
                            // team chat, so only teammates
                            if peer_username == String::from("tc") {
                              if other_player.assigned_team == players[own_index].assigned_team {
                                message_type = ChatMessageType::Team;
                                players_to_inform.push(other_player.channel.clone());
                              }
                            }
                            // all chat, so inform everyone
                            else {
                              players_to_inform.push(other_player.channel.clone());
                            }
                          }
                        }
                      }
                      if channel_invalid {
                        match tx.send(
                          PlayerMessage::SendPacket(
                            ServerToClientPacket {
                              information: ServerToClient::InteractionRefused(
                                RefusalReason::InvalidChannel,
                              )
                            }
                          )
                        ).await{
                          Ok(_) => {},
                          Err(err) => {
                            error!("{:?}", err);
                            println!("{:?}", err);
                          },
                        };
                        continue;
                      }
                      for player in players_to_inform {
                        match player.send(
                          PlayerMessage::SendPacket(
                            ServerToClientPacket { information: 
                            ServerToClient::ChatMessage(username.clone(), message.clone(), message_type.clone()) }
                          )
                        ).await {
                          Ok(_) => {},
                          Err(err) => {
                            error!("{:?}", err);
                          },
                        };
                      }
                      continue;
                    }
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
                              error!("{:?}", err);
                              internal_error_occurred = true;
                            }
                          }
                        }
                      }
                    }
                    if internal_error_occurred {
                      match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(
                          RefusalReason::InternalError
                        ),
                      })).await{
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                          println!("{:?}", err);
                        },
                      };
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
                          Err(err) => {
                            warn!("{:?}", err);
                            peer_user_online = false;
                            0 // return a gibberish value who cares it won't be read.
                          }
                        };
                        peer_channel = players[index_of_peer].channel.clone();
                      }
                      if !peer_user_online {
                        match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                          information: ServerToClient::InteractionRefused(
                            RefusalReason::UserNotOnline
                          ),
                        })).await{
                          Ok(_) => {},
                          Err(err) => {
                            error!("{:?}", err);
                            println!("{:?}", err);
                          },
                        };
                        continue;
                      }
                      match peer_channel.send(
                        PlayerMessage::SendPacket(
                          ServerToClientPacket { information: 
                          ServerToClient::ChatMessage(username.clone(), message, ChatMessageType::Private) }
                        )
                      ).await{
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                          println!("{:?}", err);
                        },
                      };
                    }
                    // peers are not friends.
                    else {
                      match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(
                          RefusalReason::NotFriends
                        ),
                      })).await{
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                          println!("{:?}", err);
                        },
                      };
                    }
                  }
                  // MARK: Lobby invite
                  ClientToServer::LobbyInvite(other_player) => {
                    rate_limit_counter += 5;

                    println!("Lobby invite");
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
                      match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(
                          RefusalReason::UserNotOnline
                        ),
                      })).await{
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                          println!("{:?}", err);
                        },
                      };
                      continue;
                    }
                    if not_friends {
                      match tx.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::InteractionRefused(
                          RefusalReason::NotFriends
                        ),
                      })).await{
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                          println!("{:?}", err);
                        },
                      };
                      continue;
                    }
                    // since everything else went well, operation inform other client is go.
                    if let Some(channel) = other_player_channel {
                      match channel.send(PlayerMessage::SendPacket(ServerToClientPacket {
                        information: ServerToClient::LobbyInvite(username.clone())
                      })).await{
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                          println!("{:?}", err);
                        },
                      };
                    }
                  }
                  // MARK: Lobby accept
                  ClientToServer::LobbyInviteAccept(other_player) => {
                    rate_limit_counter += 5;

                    println!("Lobby accept");
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
                              players[own_index].invited_by.retain(|element| element != &other_username);

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
                                        is_ready: players[index].queued.clone(),
                                      }
                                    );
                                  }
                                  Err(err) => {
                                    warn!("{:?}", err);
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
                      match tx.send(
                        PlayerMessage::SendPacket(
                          ServerToClientPacket { 
                            information: ServerToClient::InteractionRefused(
                              RefusalReason::UserNotOnline
                            )
                          }
                        )
                      ).await{
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                          println!("{:?}", err);
                        },
                      };
                      continue;
                    }
                    if already_in_party {
                      match tx.send(
                        PlayerMessage::SendPacket(
                          ServerToClientPacket { 
                            information: ServerToClient::InteractionRefused(
                              RefusalReason::AlreadyInPary
                            )
                          }
                        )
                      ).await{
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                          println!("{:?}", err);
                        },
                      };
                      continue;
                    }
                    for user in users_to_inform {
                      match user.send(
                        PlayerMessage::SendPacket(
                          ServerToClientPacket {
                            information: ServerToClient::LobbyUpdate(
                              lobby_info_update.clone(),
                            )
                          }
                        )
                      ).await{
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                          println!("{:?}", err);
                        },
                      };
                    }
                  }
                  // MARK: Lobby leave
                  ClientToServer::LobbyLeave => {
                    rate_limit_counter += 5;

                    println!("Lobby leave");
                    let mut users_to_inform: Vec<tokio::sync::mpsc::Sender<PlayerMessage>> = Vec::new();
                    let mut lobby_info_update: Vec<LobbyPlayerInfo> = Vec::new();

                    {
                      let mut players = local_players.lock().unwrap();
                      let own_index = from_user(&username, players.clone()).expect("oops");
                      if !players[own_index].queued_with.is_empty() {
                        // if is party leader
                        if players[own_index].is_party_leader {

                          // if we're with only 1 other player
                          if players[own_index].queued_with.len() == 1 {
                            let other_player_name = players[own_index].queued_with[0].clone();
                            players[own_index].queued_with = Vec::new();
                            match from_user(&other_player_name, players.clone()) {
                              Ok(index) => {
                                players[index].queued_with = Vec::new();
                                players[index].is_party_leader = true;
                              }
                              Err(err) => {
                                warn!("{:?}", err);
                              }
                            }
                          }
                          else {
                            match from_user(&players[own_index].queued_with[0], players.clone()) {
                              Ok(new_owner_index) => {
                                players[new_owner_index].is_party_leader = true;
                                let mut queued_with = players[own_index].queued_with.clone();
                                queued_with.retain(|element| element != &players[new_owner_index].username);
                                players[new_owner_index].queued_with = queued_with;
                                players[own_index].queued_with = Vec::new();
                              users_to_inform.push(players[new_owner_index].channel.clone());
                              lobby_info_update.push(
                                LobbyPlayerInfo {
                                  username: players[new_owner_index].username.clone(),
                                  is_ready: players[new_owner_index].queued
                                }
                              );
                                for (p_index, player) in players[new_owner_index].queued_with.clone().iter().enumerate() {
                                  players[p_index].queued_with = vec![players[new_owner_index].username.clone()];
                                  match from_user(&player, players.clone()) {
                                    Ok(index) => {
                                      users_to_inform.push(players[index].channel.clone());
                                      lobby_info_update.push(
                                        LobbyPlayerInfo {
                                          username: players[index].username.clone(),
                                          is_ready: players[index].queued
                                        }
                                      );
                                    }
                                    Err(err) => {
                                      // whatever
                                      warn!("{:?}", err);
                                    }
                                  }
                                }
                              }
                              Err(err) => {
                                warn!("{:?}", err);
                              }
                            };
                          }
                        }
                        // not party leader
                        else {
                          let party_leader_index = match from_user(&players[own_index].queued_with[0], players.clone()) {
                            Ok(index) => {
                              index
                            }
                            Err(err) => {
                              // idk bro
                              warn!("{:?}", err);
                              continue;
                            }
                          };
                          players[party_leader_index].queued_with.retain(|element| element != &username);
                          users_to_inform.push(players[party_leader_index].channel.clone());
                          lobby_info_update.push(
                            LobbyPlayerInfo {
                              username: players[party_leader_index].username.clone(),
                              is_ready: players[party_leader_index].queued
                            }
                          );
                          for player in players[party_leader_index].queued_with.clone() {
                            match from_user(&player, players.clone()) {
                              Ok(index) => {
                                users_to_inform.push(players[index].channel.clone());
                                lobby_info_update.push(
                                  LobbyPlayerInfo {
                                    username: players[index].username.clone(),
                                    is_ready: players[index].queued
                                  }
                                );
                              }
                              Err(err) => {
                                warn!("{:?}", err);
                                // whatever
                              }
                            }
                          }
                          players[own_index].queued_with.clear();
                          players[own_index].is_party_leader = true;
                        }
                      }
                    }
                    for player in users_to_inform {
                      match player.send(
                        PlayerMessage::SendPacket(
                          ServerToClientPacket {
                            information: ServerToClient::LobbyUpdate(
                              lobby_info_update.clone(),
                            )
                          }
                        )
                      ).await {
                        Ok(_) => {},
                        Err(err) => {
                          error!("{:?}", err);
                          println!("{:?}", err);
                        },
                      };
                    }
                  }
                  // MARK: Match Leave
                  ClientToServer::MatchLeave => {
                    rate_limit_counter += 5;
                    {
                      let mut players = local_players.lock().unwrap();
                      let p_index = from_user(&username, players.clone()).expect("oops");
                      players[p_index].in_game_with.clear();
                    }
                  }
                  // packets that shouldn't arrive.
                  // MARK: ======
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
                  match socket.write_all(
                    &network::tcp_encode_encrypt(packet, cipher_key.clone(), nonce).expect("oops")
                  ).await{
                    Ok(_) => {},
                    Err(err) => {
                      error!("{:?}", err);
                      println!("{:?}", err);
                    },
                  };
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
