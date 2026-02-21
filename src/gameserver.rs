use crate::mothership_common::MatchEndResult;
use crate::{common::*, mothership_common::*};
use core::{f32, panic};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use bincode;
use std::{thread, time::*, vec};
use crate::maths::*;
use crate::const_params::*;
use crate::gamedata::*;
use chacha20poly1305::{
  aead::{Aead, KeyInit},
  ChaCha20Poly1305, Nonce
};
use opaque_ke::generic_array::GenericArray;


// these should probably be read by the map file
static SPAWN_RED: Vector2 = Vector2 {x: 31.0 * TILE_SIZE, y: 14.0 * TILE_SIZE};
static SPAWN_BLUE: Vector2 = Vector2 {x: 3.0 * TILE_SIZE, y: 14.0 * TILE_SIZE};

/// Gameplay server. Returns a winning team.
pub fn game_server(min_players: usize, port: u16, player_info: Vec<PlayerInfo>, is_practice: bool) -> MatchEndResult {
  // Load character properties
  let characters: HashMap<Character, CharacterProperties> = load_characters();
  println!("Loaded character properties.");

  let mut players: Vec<ServerPlayer> = Vec::new();
  // initiate the players
  for player in player_info {
    players.push(
      ServerPlayer {
        username: player.username,
        cipher_key: player.session_key,
        last_nonce: 0,
        ip: String::new(),
        port: 0,
        team: player.assigned_team,
        character: player.selected_character,
        health: 100,
        position: Vector2::new(),
        shooting: false,
        last_shot_time: Instant::now(),
        shooting_secondary: false,
        secondary_cast_time: Instant::now(),
        secondary_charge: 0,
        aim_direction: Vector2::new(),
        move_direction: Vector2::new(),
        had_illegal_position: false,
        is_dashing: false,
        dash_direction: Vector2::new(),
        dashed_distance: 0.0,
        last_dash_time: Instant::now(),
        previous_positions: Vec::new(),
        is_dead: false,
        death_timer_start: Instant::now(),
        stacks: 0,
        buffs: Vec::new(),
        last_packet_time: Instant::now(),
      }
    );
  }
  let mut game_object_id_counter: u16 = 0;
  let game_objects:Vec<GameObject> = load_map_from_file(include_str!("../assets/maps/map1.map"), &mut game_object_id_counter);
  let game_object_id_counter = Arc::new(Mutex::new(game_object_id_counter));
  let players = Arc::new(Mutex::new(players));
  println!("Loaded map game objects.");
  let game_objects = Arc::new(Mutex::new(game_objects));
  // holds game information, to be displayed by client, and modified when shit happens.
  let general_gamemode_info: GameModeInfo = GameModeInfo::new();
  let general_gamemode_info = Arc::new(Mutex::new(general_gamemode_info));
  

  // initiate all networking sockets
  let server_address = format!("0.0.0.0:{}", port);
  let socket = UdpSocket::bind(server_address.clone()).expect("Error creating listener UDP socket");
  let mut buffer = [0; 4096]; // The size of this buffer is lowkey kind of low, especially with how big the gameobject struct is.
  println!("Sockets bound.");
  println!("Listening on: {}", server_address.clone());

  //let max_players = min_players;

  // (vscode) MARK: Networking - Listen
  // and also return lol
  let listener_players = Arc::clone(&players);
  let listener_gamemode_info = Arc::clone(&general_gamemode_info);
  let listener_game_objects = Arc::clone(&game_objects);
  let listener_game_object_id_counter = Arc::clone(&game_object_id_counter);

  let mut nonce: u32 = 1;
  
  println!();
  std::thread::spawn(move || {
    loop {
      // recieve packet
      let (amt, src) = socket.recv_from(&mut buffer).expect(":(");
      let data = &buffer[..amt];
      
      // clean all recieved NaNs and infinites so the server doesnt explode
      //recieved_player_info.aim_direction.clean();
      //recieved_player_info.movement.clean();
      //recieved_player_info.position.clean();
      //recieved_player_info.packet_interval.clean();

      // claim all the mutexes
      let mut players = listener_players.lock().unwrap();
      let mut game_objects = listener_game_objects.lock().unwrap();
      let mut game_object_id_counter = listener_game_object_id_counter.lock().unwrap();
      let     gamemode_info = listener_gamemode_info.lock().unwrap();


      let mut player_found: bool = false;
      
      // iterate through players
      for p_index in 0..players.len() {
        // THIS VALUE WILL THEN BE ASSIGNED BACK TO players[p_index] !!!!
        let mut player = players[p_index].clone();
        
        // use IP as identifier, check if packet from sent player correlates to our player
        // Later on we might have an account system and use that as ID. For now, IP will do
        if player.ip == src.ip().to_string() && player.port == src.port() {

          // get nonce
          let recv_nonce = &buffer[..4];
          let recv_nonce = match bincode::deserialize::<u32>(&recv_nonce){
            Ok(nonce) => nonce,
            Err(_) => {
              continue;
            }
          };
          if recv_nonce <= player.last_nonce {
            continue;
          }
          let mut nonce_bytes = [0u8; 12];
          nonce_bytes[8..].copy_from_slice(&recv_nonce.to_be_bytes());
          let formatted_nonce = Nonce::from_slice(&nonce_bytes);
          
          let key = GenericArray::from_slice(&players[p_index].cipher_key.as_slice());
          let cipher = ChaCha20Poly1305::new(key);
          
          let deciphered = match cipher.decrypt(&formatted_nonce, data[4..].as_ref()) {
            Ok(decrypted) => {
              decrypted
            },
            Err(_err) => {
              continue; // this is an erroneous packet, ignore it.
            },
          };
          let recieved_player_info = match bincode::deserialize::<ClientPacket>(&deciphered) {
            Ok(packet) => packet,
            Err(_err) => {
              continue; // ignore invalid packet
            }
          };

          // If this check passes, we're now running logic for the player that sent the packet.
          // This block of code handles recieving data, and then sends out a return packet.
          let time_since_last_packet = recieved_player_info.packet_interval as f64;
          // if time_since_last_packet < MAX_PACKET_INTERVAL &&
          // time_since_last_packet > MIN_PACKET_INTERVAL  {
          //   // ignore this packet since it's coming in too fast
          //   player_found = true;
          //   break;
          // }

          player.aim_direction = recieved_player_info.aim_direction.normalize();
          player.shooting = recieved_player_info.shooting_primary;
          player.shooting_secondary = recieved_player_info.shooting_secondary;
          
          let mut new_position: Vector2;
          let recieved_position = recieved_player_info.position;
          let movement_error_margin = 3.0;
          let mut movement_legal = true;
          let previous_position = player.position.clone();

          // let wait_time = 0.02 * crappy_random();
          // std::thread::sleep(Duration::from_secs_f64(wait_time));
          
          // (vscode) MARK: Dashing Legality

          match player.character {
            Character::Temerity => {
              // INITIATE WALLRIDE
              let wallride_initiation_distance = characters[&player.character].dash_distance;
              if recieved_player_info.dashing && !player.is_dashing
              && player.last_dash_time.elapsed().as_secs_f32() > characters[&player.character].dash_cooldown {
                // inform the rest of the code we're wallriding.
                player.is_dashing = true;
                // we now want to determine in which direction we're dashing.
                // find the closest object
                let mut closest_pos: Vector2 = Vector2::new();
                let mut shortest_distance: f32 = f32::INFINITY;
                for game_object in game_objects.clone() {
                  let distance = Vector2::distance(game_object.position, player.position);
                  if distance < wallride_initiation_distance
                  && WALL_TYPES.contains(&game_object.object_type) {
                    if distance < shortest_distance {
                      closest_pos = game_object.position;
                      shortest_distance = distance;
                    }
                  }
                }
                // if we CAN wallride
                if shortest_distance != f32::INFINITY {
                  // "radius" vector
                  let difference = player.position - closest_pos;
                  // perpendicular vector (tangent vector)
                  let difference_perpendicular: Vector2 = Vector2 { x: difference.y, y: -difference.x };
                  let player_direction = player.move_direction;
                  // use the dot product to get the direction as a rotation
                  let dot_product: f32 = player_direction.x * difference_perpendicular.x + player_direction.y * difference_perpendicular.y;
                  // kinda lame that trigonometrical direction is the opposite of clockwise,
                  // always gotta put an "anti" in my sentence ykwhatimean
                  let clockwise = f32::signum(dot_product);
                  // using dash_direction.x to store our direction, since this variable is unused on this character.
                  player.dash_direction.x = clockwise;
                }
                // else, if we can't wallride (no nearby objects)
                else {
                  // just inform the code we're not wallriding.
                  player.is_dashing = false;
                }
              }
              // TERMINATE WALLRIDE
              if !recieved_player_info.dashing && player.is_dashing {
                // update variables to inform the code we're no longer wallriding
                player.is_dashing = false;
                player.last_dash_time = Instant::now();

                // clear all other impulses
                let mut cleaned_buffs: Vec<Buff> = Vec::new();
                for buff in player.buffs.clone() {
                  if buff.buff_type != BuffType::Impulse {
                    cleaned_buffs.push(buff);
                  }
                }
                player.buffs = cleaned_buffs;

                // apply an impulse
                player.buffs.push(
                  Buff {
                    value: 0.1 * TILE_SIZE,
                    duration: 1.0,
                    buff_type: BuffType::Impulse,
                    direction: player.move_direction,
                  }
                );
              }
            }
            // NORMAL DASHES
            _ => {
              // If player wants to dash and isn't dashing...
              if recieved_player_info.dashing && !player.is_dashing && !player.is_dead && recieved_player_info.movement.magnitude() != 0.0 {
                let player_dash_cooldown = characters[&player.character].dash_cooldown;
                // And we're past the cooldown...
                if player.last_dash_time.elapsed().as_secs_f32() >= player_dash_cooldown {
                  // reset the cooldown
                  player.last_dash_time = Instant::now();
                  // set dashing to true
                  player.is_dashing = true;
                  // set the dashing direction
                  player.dash_direction = recieved_player_info.movement;

                  // (vscode) MARK: Special dashes
                  match player.character {
                    Character::Raphaelle => {
                      player.stacks = 1;
                    }
                    Character::Hernani => {
                      // Place down a trap
                      game_objects.push(
                        GameObject {
                          object_type: GameObjectType::HernaniLandmine,
                          position: player.position,
                          to_be_deleted: false,
                          id: game_object_id_counter.increment(),
                          extra_data: ObjectData::BulletData(
                            BulletData {
                              direction: Vector2::new(),
                              owner_username: players[p_index].username.clone(),
                              hitpoints: 1,
                              lifetime: characters[&Character::Hernani].dash_cooldown,
                              hit_players: Vec::new(),
                              traveled_distance: 0.0,
                            }
                          )
                        }
                      );
                    }
                    Character::Cynewynn => {}
                    Character::Wiro => {
                      if player.stacks == 1 {
                        // Spawn the projectile that applies the mid-dash logic.
                        game_objects.push(
                          GameObject {
                            object_type: GameObjectType::WiroDashProjectile,
                            position: player.position,
                            to_be_deleted: false,
                            id: game_object_id_counter.increment(),
                            extra_data: ObjectData::BulletData(
                              BulletData {
                                direction: Vector2::new(),
                                owner_username: players[p_index].username.clone(),
                                hitpoints: 0,
                                lifetime: characters[&Character::Hernani].dash_distance / characters[&Character::Hernani].dash_speed + 0.25, // give it a "grace" period because I'm bored
                                hit_players: Vec::new(),
                                traveled_distance: 0.0,
                              }
                            )
                          }
                        );
                      }
                      player.stacks = 0;
                    }
                    Character::Elizabeth => {
                      // Change the type of all her current static projectiles to the type
                      // that follows her.
                      for index in 0..game_objects.len() {
                        if game_objects[index].object_type == GameObjectType::ElizabethProjectileGround
                        && game_objects[index].get_bullet_data().owner_username == player.username {
                          game_objects[index].to_be_deleted = true;
                          let object_clone = game_objects[index].clone();
                          game_objects.push(
                            GameObject {
                              object_type: GameObjectType::ElizabethProjectileGroundRecalled,
                              position: object_clone.position,
                              to_be_deleted: false,
                              id: game_object_id_counter.increment(),
                              extra_data: ObjectData::BulletData(
                                BulletData {
                                  direction: Vector2::new(),
                                  owner_username: object_clone.get_bullet_data().owner_username,
                                  hitpoints: 0,
                                  lifetime: 15.0,
                                  hit_players: Vec::new(),
                                  traveled_distance: 0.0,
                                }
                              )
                            }
                          );
                        }
                      }
                    }
                    Character::Temerity => {
                      // technically this is redundant. She should never show up here.
                    }
                    Character::Dummy => {}
                  }
                }
              }
            }
          }
          // (vscode) MARK: Dashing
          if player.is_dashing && !player.is_dead {
            match player.character {
              // DURING WALLRIDE
              Character::Temerity => {

                // Move around the nearest wall in the desired direction
                // update new_position

                // using dash_direction.x to store our direction, since this variable is unused on this character.
                let clockwise: f32 = player.dash_direction.x;

                // find the closest object
                let mut closest_pos: Vector2 = Vector2::new();
                let mut shortest_distance: f32 = f32::INFINITY;
                for game_object in game_objects.clone() {
                  let distance = Vector2::distance(game_object.position, player.position);
                  if WALL_TYPES.contains(&game_object.object_type) {
                    if distance < shortest_distance {
                      closest_pos = game_object.position;
                      shortest_distance = distance;
                    }
                  }
                }

                // now we pivot around closest_pos
                // find the radius vector to the nearest wall
                let difference = (player.position - closest_pos).normalize();
                // get the perpendicular tangent to the nearest wall circle thingy.
                // also make sure it points in the right direction
                let difference_perpendicular: Vector2 = Vector2 { x: difference.y * clockwise, y: -difference.x * clockwise };

                let speed = characters[&player.character].dash_speed;
                let wallride_distance = characters[&player.character].dash_distance;
                let pow = 8.0;
                let scale: f32 = 0.7;
                //let wallride_distance = scale * (TILE_SIZE/2.0) * f32::powf(f32::powf(difference.x, pow) + f32::powf(difference.y, pow), 1.0/pow);
                //let wallride_distance = TILE_SIZE * scale * f32::powf(f32::powf(f32::abs(difference.x), pow) + f32::powf(f32::abs(difference.y), pow), 1.0/pow);

                // now apply this movement as our new movement vector
                new_position = player.position + difference_perpendicular * speed * time_since_last_packet as f32;
                
                // lock our position at the right distance.
                let mut diff: Vector2 = (new_position - closest_pos).normalize();

                // we want the new vector (x2, y2) to satisfy the equation x^a + y^a = r^a
                // we have its direction, (x1, y1), so (x2, y2) = z * (x1, y1), and we want
                // to find the scalar z.
                // We want (x2, y2) to satisfy the equation so just chuck it in there:
                // x2^a + y2^a = r^a         <=>
                // (zx1^)a + (zy1)^a = r^a   <=>
                // (z^a)(x1^a + y1^a) = r^a  <=>
                // z^a = r^a / (x1^a + y1^a) <=>
                // z = (r^a / (x1^a + y1^a))^(1/a)
                // look at that we found x.
                diff = diff * (scale.powf(pow)/(diff.x.powf(pow) + diff.y.powf(pow))).powf(1.0/pow);

                new_position = closest_pos + diff * wallride_distance;

                // update player info. This will be used when giving the character an impulse
                player.move_direction = difference_perpendicular;
                // force out an override
                movement_legal = false
              }
              _ => {
                (new_position, player.dashed_distance, player.is_dashing) = dashing_logic(
                  player.is_dashing,
                  player.dashed_distance, 
                  player.dash_direction, 
                  time_since_last_packet, 
                  characters[&player.character].dash_speed, 
                  characters[&player.character].dash_distance, 
                  game_objects.clone(), 
                  player.position
                );
                movement_legal = false; // force a correction
              }
            }
          }
          else {
            // (vscode) MARK: Movement Legality
            // Movement legality calculations
            let mut raw_movement = recieved_player_info.movement;
            let player_movement_speed: f32 = characters[&player.character].speed;
            let mut extra_speed: f32 = 0.0;
            for b_index in 0..player.buffs.len() {
              if vec![BuffType::Speed, BuffType::WiroSpeed].contains(&player.buffs[b_index].buff_type) {
                extra_speed += player.buffs[b_index].value;
              }
              if player.buffs[b_index].buff_type == BuffType::Impulse {
                // yeet
                let direction = player.buffs[b_index].direction.normalize();
                // time left serves as impulse decay
                let time_left = player.buffs[b_index].duration;
                let strength = player.buffs[b_index].value;
                raw_movement += direction * f32::powi(time_left, 1) * strength;
                //movement_legal = false;
              }
            }

            let movement = raw_movement * (player_movement_speed + extra_speed) * time_since_last_packet as f32;

            // calculate current expected position based on input
            let (new_movement_raw, _): (Vector2, Vector2) = object_aware_movement(
              previous_position,
              raw_movement,
              movement,
              game_objects.clone()
            );

            new_position = previous_position + new_movement_raw * (player_movement_speed + extra_speed) * time_since_last_packet as f32;

            player.move_direction = new_movement_raw;

          }
          if !player.is_dead {
            if Vector2::distance(new_position, recieved_position) > movement_error_margin {
              movement_legal = false;
            }
            
            if movement_legal {
              // Since the client had correct movement, let's comply with theirs, to avoid desync.
              player.position = recieved_position;
              // inform the rest of the code we're all good.
              player.had_illegal_position = false;
            } else {
              // Inform the network sender it needs to send a correction packet (position override packet).
              player.had_illegal_position = true;
              // Also apply movement.
              player.position = new_position;
            }
          }

          // Send a return packet.
          // (vscode) MARK: Network Return
          // not to the dummy though.
          if player.character == Character::Dummy {
            player_found = true;
            break;
          }

          //// this stupid SHIT somehow fixes the bug where ping keeps increasing
          //if player.last_packet_time.elapsed().as_secs_f64() < MAX_PACKET_INTERVAL {
          //  // do nothing
          //} else
          {
            player.last_packet_time = Instant::now();


            // Gather info to send about other players
            let mut other_players: Vec<OtherPlayer> = Vec::new();
            for (other_player_index, player) in players.clone().iter().enumerate() {
              if other_player_index != p_index {
                other_players.push(OtherPlayer {
                  username: player.username.clone(),
                  health: player.health,
                  position: player.position,
                  secondary_charge: player.secondary_charge,
                  aim_direction: player.aim_direction,
                  movement_direction: player.move_direction,
                  shooting_primary: player.shooting,
                  shooting_secondary: player.shooting_secondary,
                  team: player.team,
                  character: player.character,
                  time_since_last_dash: player.last_dash_time.elapsed().as_secs_f32(),
                  is_dead: player.is_dead,
                  camera: Camera::new(),  
                  buffs: player.buffs.clone(),
                  previous_positions: match player.character {
                    Character::Cynewynn => player.previous_positions.clone(),
                    _ => Vec::new(),
                  },
                  stacks : player.stacks,
                })
              }
            }

            // packet sent to player with info about themselves and other players
            let server_packet: ServerPacket = ServerPacket {
              player_packet_is_sent_to: ServerRecievingPlayerPacket {
                health: player.health,
                override_position: player.had_illegal_position,
                position_override: player.position,
                shooting_primary: player.shooting,
                shooting_secondary: player.shooting_secondary,
                secondary_charge: player.secondary_charge,
                character: player.character,
                is_dead: player.is_dead,
                buffs: player.buffs.clone(),
                previous_positions: match player.character {
                  Character::Cynewynn => player.previous_positions.clone(),
                  _ => Vec::new(),
                },
                team: player.team,
                time_since_last_primary: player.last_shot_time.elapsed().as_secs_f32(),
                time_since_last_dash: player.last_dash_time.elapsed().as_secs_f32(),
                time_since_last_secondary: player.secondary_cast_time.elapsed().as_secs_f32(),
                stacks: player.stacks,
                is_dashing: player.is_dashing,
              },
              players: other_players,
              game_objects: game_objects.clone(),
              gamemode_info: gamemode_info.clone(),
              timestamp: recieved_player_info.timestamp, // pong!
            };
            players[p_index].had_illegal_position = false;
            
            let mut player_ip = player.ip.clone();
            let split_player_ip: Vec<&str> = player_ip.split(":").collect();
            player_ip = split_player_ip[0].to_string();
            player_ip = format!("{}:{}", player_ip, player.port);

            // send data to client
            let serialized_packet: Vec<u8> = bincode::serialize(&server_packet).expect("Failed to serialize message");
            let mut nonce_bytes = [0u8; 12];
            nonce_bytes[8..].copy_from_slice(&nonce.to_be_bytes());

            let formatted_nonce = Nonce::from_slice(&nonce_bytes);
            let cipher_key = player.cipher_key.clone();
            let key = GenericArray::from_slice(&cipher_key);
            let cipher = ChaCha20Poly1305::new(&key);
            let ciphered = cipher.encrypt(&formatted_nonce, serialized_packet.as_ref()).expect("shit");

            let serialized_nonce: Vec<u8> = bincode::serialize::<u32>(&nonce).expect("oops");
            let serialized = [&serialized_nonce[..], &ciphered[..]].concat();
            nonce += 1;
            socket.send_to(&serialized, player_ip).expect("Failed to send packet to client.");
          }

          // exit loop, and inform rest of program not to proceed with appending a new player.
          player_found = true;
          // println!("{:?}", player.position.clone());
          players[p_index] = player;
          break
        }
      }

      // (vscode) MARK: Instantiate Player

      if !player_found {

        for p_index in 0..players.len() {

          // get nonce
          let nonce = &buffer[..4];
          let nonce = match bincode::deserialize::<u32>(&nonce){
            Ok(nonce) => nonce,
            Err(_) => {
              continue;
            }
          };
          let mut nonce_bytes = [0u8; 12];
          nonce_bytes[8..].copy_from_slice(&nonce.to_be_bytes());
          let nonce = Nonce::from_slice(&nonce_bytes);
          
          let key = GenericArray::from_slice(&players[p_index].cipher_key.as_slice());
          let cipher = ChaCha20Poly1305::new(key);
          
          /*let deciphered =*/ match cipher.decrypt(&nonce, data[4..].as_ref()) {
            Ok(_decrypted) => {
              //if nonce_num <= last_nonce {
              //  continue; // this is a parroted packet, ignore it.
              //}
              // this is a valid packet, update last_nonce
              //last_nonce = nonce_num;

              // SUCCESSFULLY DECRYPTED. ASSIGN IP TO THIS PLAYER.
              players[p_index].ip = src.ip().to_string();
              players[p_index].port = src.port();
              break; // decrypted
            },
            Err(_err) => {
              continue; // this is an erroneous packet, ignore it.
            },
          };
        }
      }
    }
  });
  
  // (vscode) MARK: Server Loop Initiate

  // counter used to calculate delta_time
  let mut server_counter: Instant = Instant::now();
  let mut delta_time: f64 = server_counter.elapsed().as_secs_f64();
  // Server logic frequency in Hertz. Doesn't need to be much higher than 120.
  // Higher frequency = higher precission with computation trade-off
  let desired_delta_time: f64 = 1.0 / 120.0;

  let main_game_objects = Arc::clone(&game_objects);
  let main_game_object_id_counter = Arc::clone(&game_object_id_counter);

  // start the game with an orb
  let orb_spawn_interval: f32 = 20.0; //seconds
  let mut orb_timer: f32 = orb_spawn_interval;

  // for once-per-second operations, called ticks
  let mut tick_counter = Instant::now();

  // for once-per-decisecond operations.
  let mut tenth_tick_counter = Instant::now();
  
  let characters = load_characters();
  let main_loop_players = Arc::clone(&players);
  
  // Used for game time counter. Can be reset when going into new rounds, for example...
  let mut game_start_time: Instant = Instant::now();

  
  // part of dummy summoning
  // set to TRUE in release server, so dummy doesn't get spawned
  let mut dummy_summoned: bool = !SPAWN_DUMMY && !is_practice;

  // (vscode) MARK: Server Loop
  let main_gamemode_info = Arc::clone(&general_gamemode_info);
  loop {

    // claim all mutexes
    let mut players = main_loop_players.lock().unwrap();
    let mut game_objects = main_game_objects.lock().unwrap();
    let mut game_object_id_counter = main_game_object_id_counter.lock().unwrap();
    let mut gamemode_info = main_gamemode_info.lock().unwrap();


    let mut tick: bool = false;
    let mut tenth_tick: bool = false;
    server_counter = Instant::now();
    
    // Accurate time between two "frames" (server loops)
    let true_delta_time: f64; // does not need to be mutable, since in both branches the value is assigned.
    if delta_time > desired_delta_time {
      true_delta_time = delta_time;
    } else {
      true_delta_time = desired_delta_time;
    }
    
    // for once-per-second operations
    if tick_counter.elapsed().as_secs_f32() > 1.0 {
      tick = true;
      tick_counter = Instant::now();
    }
    // for once-per-decisecond operations
    if tenth_tick_counter.elapsed().as_secs_f32() > 0.1 {
      tenth_tick = true;
      tenth_tick_counter = Instant::now();
    }
    
    // (vscode) MARK: Gamemode Logic
    if tenth_tick {
      gamemode_info.time = game_start_time.elapsed().as_secs() as u16;
      let mut total_players: u8 = 0;
      let mut red_players: u8 = 0;
      let mut blue_players: u8 = 0;
      let mut red_alive: u8 = 0;
      let mut blue_alive: u8 = 0;
      for player in players.clone() {
        total_players += 1;
        if player.team == Team::Red {
          red_players += 1;
          if !player.is_dead {
            red_alive += 1;
          }
        } else {
          blue_players += 1;
          if !player.is_dead {
            blue_alive += 1;
          }
        }
      }
      gamemode_info.total_red = red_players;
      gamemode_info.total_blue = blue_players;
      gamemode_info.alive_red = red_alive;
      gamemode_info.alive_blue = blue_alive;

      // we can start the game
      if total_players >= min_players as u8 {

        let mut reset = false;
        if red_alive == 0 &&(!(MATCHMAKE_ALONE | is_practice) | SPAWN_DUMMY) { //debug
          gamemode_info.rounds_won_blue += 1;
          reset = true;
        }
        if blue_alive == 0 &&(!(MATCHMAKE_ALONE | is_practice) | SPAWN_DUMMY) {
          gamemode_info.rounds_won_red += 1;
          reset = true;
        }
        // set the game state as inactive, and update the timer
        if reset {
          gamemode_info.game_active = false;
          game_start_time = Instant::now();
        }

        // if the round is over, or the game started:
        if gamemode_info.game_active == false {
          // reset player aliveness
          for p_index in 0..players.len() {
            players[p_index].is_dead = false;
          }

          // at the start of this in-between phase...
          if game_start_time.elapsed().as_secs_f32() < 0.5 {
            // force the map with player cages
            *game_objects = load_map_from_file(include_str!("../assets/maps/map1-cages.map"), &mut game_object_id_counter);
            // reset player positions
            for p_index in 0..players.len() {
              players[p_index].is_dead = false;
              players[p_index].position = match players[p_index].team {
                Team::Blue => SPAWN_BLUE,
                Team::Red => SPAWN_RED,
              };
            }
          }

          // restart the round after the end of the in-between phase
          if game_start_time.elapsed().as_secs_f32() > 3.0 {
            // set the game as active for the rest of the code
            gamemode_info.game_active = true;
            game_start_time = Instant::now();
            // reset player data, but not positions. It's ok if players get a small headstart
            // in the palyer "cages".
            for p_index in 0..players.len() {
              players[p_index].is_dead = false;
              players[p_index].secondary_charge = 0;
              players[p_index].stacks = 0;
              players[p_index].health = 100;
              //players[p_index].last_dash_time = Instant::now();
              //players[p_index].last_shot_time = Instant::now();
              //players[p_index].secondary_cast_time = Instant::now();
            }
            // reset the game objects too
            *game_objects = load_map_from_file(include_str!("../assets/maps/map1.map"), &mut game_object_id_counter);
          }
          // MARK: Game End
          if gamemode_info.rounds_won_blue >= ROUNDS_TO_WIN {
            return MatchEndResult {
              winning_team: Team::Blue,
              is_draw: false,
              game_id: 0, // don't worry about this value, the main server handles it.
            };
          }
          if gamemode_info.rounds_won_red >= ROUNDS_TO_WIN {
            return MatchEndResult {
              winning_team: Team::Red,
              is_draw: false,
              game_id: 0, // don't worry about this value, the main server handles it.
            };
          }
        }

      // if we don't have enough players, kindly wait
      } else {
        // and make sure the start time doesn't update
        game_start_time = Instant::now();
      }
    }

    // Summon a dummy for testing
    if !dummy_summoned {
      dummy_summoned = true;

      players.push(
        ServerPlayer {
          username: String::from("Dummy McDummington"),
          cipher_key: Vec::new(),
          last_nonce: 0,
          ip: String::from("hello"),
          port: 12,
          team: Team::Red,
          character: Character::Dummy,
          health: 100,
          position: Vector2 { x: 100.0, y: 100.0 },
          shooting: true,
          last_dash_time: Instant::now(),
          last_shot_time: Instant::now(),
          shooting_secondary: false,
          secondary_cast_time: Instant::now(),
          secondary_charge: 0,
          aim_direction: Vector2 { x: -1.0, y: 0.0 },
          move_direction: Vector2::new(),
          had_illegal_position: false,
          is_dashing: false,
          dash_direction: Vector2::new(),
          dashed_distance: 0.0,
          previous_positions: vec![],
          is_dead: false,
          death_timer_start: Instant::now(),
          stacks: 0,
          buffs: Vec::new(),
          last_packet_time: Instant::now(),
        }
      );
    }
    
    // (vscode) MARK: Player logic
    for p_index in 0..players.len() {


      let shooting = players[p_index].shooting;
      let shooting_secondary = players[p_index].shooting_secondary;
      let last_shot_time = players[p_index].last_shot_time;
      let secondary_charge = players[p_index].secondary_charge;
      let player_info = players[p_index].clone();
      let character: CharacterProperties = characters[&players[p_index].character].clone();

      // println!("{:?}", players[p_index].character);

      // MARK: Handle death

      // if this player is at health 0
      if players[p_index].health == 0 && !players[p_index].is_dead {
        players[p_index].kill(SPAWN_RED, SPAWN_BLUE);
      }

      // IGNORE ANYTHING BELOW IF PLAYER HAS DIED
      if players[p_index].is_dead {
        if is_practice {
          if players[p_index].death_timer_start.elapsed().as_secs_f32() > 1.0 {
            players[p_index].is_dead = false;
          }
        }
        continue;
      }
      if players[p_index].last_packet_time.elapsed().as_secs_f32() > 5.0
      && players[p_index].character != Character::Dummy {
        let player_team_copy = players[p_index].team.clone();
        players.remove(p_index);
        if players.is_empty() {
          return MatchEndResult {
            winning_team: player_team_copy,
            is_draw: false,
            game_id: 0, // don't worry about this value, the main server handles it.
          };
        }
        break;
      }


      // (vscode) MARK: Passives & Other
      // Handling of passive abilities and anything else that may need to be run all the time.

      // Reduce buff durations according to time passed, and remove buffs that ended.
      let mut buffs_to_keep: Vec<Buff> = Vec::new();
      for buff_index in 0..players[p_index].buffs.len() {
        players[p_index].buffs[buff_index].duration -= true_delta_time as f32;
        if players[p_index].buffs[buff_index].duration > 0.0 {
          buffs_to_keep.push(players[p_index].buffs[buff_index].clone());
        }
      }
      players[p_index].buffs = buffs_to_keep;

      // Handling of time queen flashsback ability - keep a buffer of positions for the flashback
      if players[p_index].character == Character::Cynewynn {
        // Update once per decisecond
        if tenth_tick {
          // update buffer of positions when secondary isnt active
          let position: Vector2 = players[p_index].position.clone();
          players[p_index].previous_positions.push(position);
          // cut the buffer to remain the correct size
          let position_buffer_length: usize = (character.secondary_cooldown * 10.0) as usize;
          if players[p_index].previous_positions.len() > position_buffer_length {
            players[p_index].previous_positions.remove(0);
          }
        }
      }
      // TEMERITY - heal walls around her
      if tick {
        if players[p_index].character == Character::Temerity {
          for o_index in 0..game_objects.len() {
            if WALL_TYPES.contains(&game_objects[o_index].object_type) {
              let range = characters[&Character::Temerity].passive_range;
              if Vector2::distance(game_objects[o_index].position, players[p_index].position) < range {
                let heal_value = characters[&Character::Temerity].passive_value;
                let mut wall_data = game_objects[o_index].get_wall_data();
                if wall_data.hitpoints < WALL_HP - heal_value {
                  wall_data.hitpoints += heal_value;
                } else {
                  wall_data.hitpoints = WALL_HP;
                }
                game_objects[o_index].extra_data = ObjectData::WallData(wall_data);
              }
            } 
          }
        }
      }

      // increase secondary charge passively
      if tick {
        let charge_amount = characters[&players[p_index].character].secondary_passive_charge;
        players[p_index].add_charge(charge_amount);
      }

      // Get stuck player out of walls
      let player_size = TILE_SIZE/4.0;
      let tile_size: f32 = TILE_SIZE/2.0;
      let collision_size = tile_size + player_size;
      for o_index in 0..game_objects.len() {
        
        if WALL_TYPES_ALL.contains(&game_objects[o_index].object_type) {
          let difference: Vector2 = Vector2::difference(game_objects[o_index].position, players[p_index].position);
          if f32::abs(difference.x) < collision_size && f32::abs(difference.y) < collision_size {
            // push out the necessary amount
            players[p_index].position.x += (TILE_SIZE + 0.1 )* difference.normalize().x;
            players[p_index].position.y += (TILE_SIZE + 0.1 )* difference.normalize().y;
            players[p_index].had_illegal_position = true;
            break;
          }
        }
      }
      // Delete extra Elizabeth ground daggers
      if players[p_index].character == Character::Elizabeth {

        let mut objects_to_consider: Vec<usize> = Vec::new();
        for o_index in 0..game_objects.len() {
          if game_objects[o_index].object_type == GameObjectType::ElizabethProjectileGround {
            if game_objects[o_index].get_bullet_data().owner_username == players[p_index].username {
              objects_to_consider.push(o_index);
            }
          }
        }
        if objects_to_consider.len() > 2 {
          // selection sort is by far the best sorting algorithm
          let mut lowest_val: f32 = f32::MAX;
          let mut lowest_index: usize = 0;
          for object_to_consider in objects_to_consider {
            if game_objects[object_to_consider].get_bullet_data().lifetime < lowest_val {
              lowest_val = game_objects[object_to_consider].get_bullet_data().lifetime;
              lowest_index = object_to_consider;
            }
          }
          game_objects[lowest_index].to_be_deleted = true;
        }
      }

      // Apply wiro's speed boost
      if players[p_index].character == Character::Wiro {
        if !players[p_index].shooting_secondary
        || players[p_index].secondary_cast_time.elapsed().as_secs_f32() < character.secondary_cooldown
        || players[p_index].secondary_charge == 0 {
          for victim_index in 0..players.len() {
            if players[victim_index].team == players[p_index].team
            && Vector2::distance(players[victim_index].position, players[p_index].position) < character.passive_range{
              // provide speed buff, if not present
              let mut buff_found = false;
              for buff_index in 0..players[victim_index].buffs.len() {
                if players[victim_index].buffs[buff_index].buff_type == BuffType::WiroSpeed {
                  buff_found = true;
                  players[victim_index].buffs.remove(buff_index);
                  players[victim_index].buffs.push(Buff { value: 5.0, duration: 0.25, buff_type: BuffType::WiroSpeed, direction: Vector2::new() });
                  break; // exit early
                }
              }
              if !buff_found {
                players[victim_index].buffs.push(Buff { value: 5.0, duration: 0.25, buff_type: BuffType::WiroSpeed, direction: Vector2::new() });
              }
            }
          }
        }
      }


      // (vscode) MARK: Primaries
      // If someone is shooting, spawn a bullet according to their character.
      let mut cooldown: f32 = character.primary_cooldown;
      if players[p_index].character == Character::Cynewynn {
        cooldown -= cooldown * ((secondary_charge as f32 / 100.0) * characters[&Character::Cynewynn].primary_cooldown_2)
      }

      for buff in players[p_index].buffs.clone() {
        if buff.buff_type == BuffType::FireRate || buff.buff_type == BuffType::RaphaelleFireRate {
          cooldown -= cooldown * buff.value;
        }
      }
      // not sure why this was here, nothing wrong with holding down both buttons
      //                      \/
      if shooting /*&& !shooting_secondary*/  && last_shot_time.elapsed().as_secs_f32() > cooldown && players[p_index].aim_direction.magnitude() != 0.0 {
        // players[p_index].buffs.push(Buff { value: 0.1, duration: 2.2, buff_type: BuffType::FireRate });
        // players[p_index].buffs.push(Buff { value: 20.0, duration: 2.2, buff_type: BuffType::Speed });
        let mut shot_successful: bool = false;
        // Do primary shooting logic
        match players[p_index].character {
          Character::Hernani => {
            game_objects.push(GameObject {
              object_type: GameObjectType::HernaniBullet,
              position: players[p_index].position,
              to_be_deleted: false,
              id: game_object_id_counter.increment(),
              extra_data: ObjectData::BulletData(
                BulletData {
                  direction: players[p_index].aim_direction,
                  owner_username: players[p_index].username.clone(),
                  lifetime: character.primary_range / character.primary_shot_speed,
                  hit_players: Vec::new(),
                  traveled_distance: 0.0,
                  hitpoints: 0,
                }
              )
            });
            shot_successful = true;
          }
          Character::Raphaelle => {
            game_objects.push(GameObject {
              object_type: match players[p_index].stacks {
                1  => {GameObjectType::RaphaelleBulletEmpowered},
                0 => {GameObjectType::RaphaelleBullet},
                _ => panic!()
              },
              position: players[p_index].position,
              to_be_deleted: false,
              id: game_object_id_counter.increment(),
              extra_data: ObjectData::BulletData(
                BulletData {
                  direction: players[p_index].aim_direction,
                  hitpoints: 0,
                  owner_username: players[p_index].username.clone(),
                  lifetime: character.primary_range / character.primary_shot_speed,
                  hit_players: Vec::new(),
                  traveled_distance: 0.0,
                }
              ),
            });
            if players[p_index].stacks == 1 {
              players[p_index].stacks = 0;
            }
            shot_successful = true;
          }
          Character::Cynewynn => {
            game_objects.push(GameObject {
              object_type: GameObjectType::CynewynnSword,
              position: Vector2 {
                x: players[p_index].position.x + players[p_index].aim_direction.x * 5.0,
                y: players[p_index].position.y + players[p_index].aim_direction.y * 5.0
              },
              to_be_deleted: false,
              id: game_object_id_counter.increment(),
              extra_data: ObjectData::BulletData(
                BulletData {
                  direction: players[p_index].aim_direction,
                  hitpoints: 0,
                  owner_username: players[p_index].username.clone(),
                  lifetime: character.primary_range / character.primary_shot_speed,
                  hit_players: Vec::new(),
                  traveled_distance: 0.0,
                }
              ),
            });
            shot_successful = true;
          }
          Character::Elizabeth => {
            game_objects.push(GameObject {
              object_type: GameObjectType::ElizabethProjectileRicochet,
              position: players[p_index].position,
              to_be_deleted: false,
              id: game_object_id_counter.increment(),
              extra_data: ObjectData::BulletData(
                BulletData {
                  direction: players[p_index].aim_direction,
                  owner_username: players[p_index].username.clone(),
                  hitpoints: 1, // ricochet projectiles use hitpoints to keep track of wether they've already bounced
                  lifetime: character.primary_range / character.primary_shot_speed,
                  hit_players: vec![],
                  traveled_distance: 0.0,
                }
              )
            });
            shot_successful = true;
          }
          Character::Wiro => {
            if !players[p_index].shooting_secondary
            || players[p_index].secondary_cast_time.elapsed().as_secs_f32() < character.secondary_cooldown {
              game_objects.push(GameObject {
                object_type: GameObjectType::WiroGunShot,
                position: players[p_index].position,
                to_be_deleted: false,
                id: game_object_id_counter.increment(),
                extra_data: ObjectData::BulletData(
                  BulletData {
                    hitpoints: 0,
                    direction: players[p_index].aim_direction,
                    owner_username: players[p_index].username.clone(),
                    lifetime: character.primary_range / character.primary_shot_speed,
                    hit_players: Vec::new(),
                    traveled_distance: 0.0,
                  }
                )
              });
              shot_successful = true;
            }
          }
          Character::Temerity => {
            game_objects.push(GameObject {
              object_type: GameObjectType::TemerityRocket,
              position: players[p_index].position,
              to_be_deleted: false,
              id: game_object_id_counter.increment(),
              extra_data: ObjectData::BulletData(
                BulletData {
                  direction: players[p_index].aim_direction,
                  hitpoints: 0,
                  owner_username: players[p_index].username.clone(),
                  lifetime: match players[p_index].stacks {
                    0 => character.primary_range   / character.primary_shot_speed,
                    1 => character.primary_range_2 / character.primary_shot_speed,
                    2 => character.primary_range_3 / character.primary_shot_speed,
                    _ => panic!()
                  },
                  hit_players: Vec::new(),
                  traveled_distance: 0.0,
                }
              ),
            });
            shot_successful = true;
          }
          Character::Dummy => {
            game_objects.push(GameObject {
              object_type: GameObjectType::HernaniBullet,
              position: players[p_index].position,
              to_be_deleted: false,
              id: game_object_id_counter.increment(),
              extra_data: ObjectData::BulletData(
                BulletData {
                  direction: players[p_index].aim_direction,
                  hitpoints: 0,
                  owner_username: players[p_index].username.clone(),
                  lifetime: character.primary_range / character.primary_shot_speed,
                  hit_players: Vec::new(),
                  traveled_distance: 0.0,
                }
              ),
            });
            shot_successful = true;
          }
        }
        if shot_successful {
          players[p_index].last_shot_time = Instant::now();
        }
      }
      // (vscode) MARK: Secondaries
      // If a player is trying to use their secondary and they have enough charge to do so, apply custom logic.
      if shooting_secondary && secondary_charge >= character.secondary_charge_use {
        let mut secondary_used_successfully = false;
        
        match players[p_index].character {
          
          // Create a healing aura
          Character::Raphaelle => {
            // Create a bullet type and then define its actions in the next loop that handles bullets
            game_objects.push(GameObject {
              object_type: GameObjectType::RaphaelleAura,
              position: player_info.position,
              to_be_deleted: false,
              id: game_object_id_counter.increment(),
              extra_data: ObjectData::BulletData(
                BulletData {
                  direction: Vector2::new(),
                  owner_username: players[p_index].username.clone(),
                  hitpoints: 0,
                  lifetime: 5.0,
                  hit_players: vec![],
                  traveled_distance: 0.0,
                }
              ),
            });
            secondary_used_successfully = true;
          },
          // Place walls
          Character::Hernani => {
            // Place down a wall at a position rounded to TILE_SIZE, unless a wall is alredy there.
            let wall_place_distance = character.secondary_range;
            let mut desired_placement_position_center: Vector2 = player_info.position + player_info.aim_direction * wall_place_distance + Vector2{x: TILE_SIZE/2.0, y:TILE_SIZE/2.0};
            // round to closest 10
            let direction_perpendicular = Vector2 {x: player_info.aim_direction.y, y: -player_info.aim_direction.x};
            let side_offset = 1.0;
            let mut desired_placement_position_perpendicular_1 = desired_placement_position_center + direction_perpendicular *  side_offset * TILE_SIZE;
            let mut desired_placement_position_perpendicular_2 = desired_placement_position_center + direction_perpendicular * -side_offset * TILE_SIZE;
            desired_placement_position_perpendicular_1.x = (((desired_placement_position_perpendicular_1.x / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;
            desired_placement_position_perpendicular_1.y = (((desired_placement_position_perpendicular_1.y / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;
            desired_placement_position_perpendicular_2.x = (((desired_placement_position_perpendicular_2.x / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;
            desired_placement_position_perpendicular_2.y = (((desired_placement_position_perpendicular_2.y / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;
            desired_placement_position_center.x = ((((desired_placement_position_center.x) / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;
            desired_placement_position_center.y = ((((desired_placement_position_center.y) / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;

            let mut desired_placement_positions: Vec<Vector2> = Vec::new();
            desired_placement_positions.push(desired_placement_position_center);
            desired_placement_positions.push(desired_placement_position_perpendicular_1);
            desired_placement_positions.push(desired_placement_position_perpendicular_2);

            for (index, desired_placement_position) in desired_placement_positions.iter().enumerate() {
              let mut wall_can_be_placed = true;
              
              for game_object in game_objects.clone() {
                match game_object.object_type {
                  GameObjectType::HernaniWall | GameObjectType::UnbreakableWall | GameObjectType::Wall => {
                    if game_object.position.x == desired_placement_position.x && game_object.position.y == desired_placement_position.y {
                      if index == 0 {
                        // if the center wall can't be placed, give up.
                        break;
                      }
                      wall_can_be_placed = false;
                    }
                  },
                  _ => {}
                }
              }
              if wall_can_be_placed {
                game_objects.push(GameObject {
                  object_type: GameObjectType::HernaniWall,
                  position: *desired_placement_position,
                  to_be_deleted: false,
                  id: game_object_id_counter.increment(),
                  extra_data: ObjectData::WallData(
                    WallData {
                      hitpoints: 20,
                      lifetime: 5.0,
                    }
                  ),
                });
                secondary_used_successfully = true;
              }
            }
          },
          // position revert
          Character::Cynewynn => {
            let flashback_length = (character.secondary_cooldown * 10.0) as usize; // deciseconds
            if player_info.previous_positions.len() >= flashback_length
            && players[p_index].secondary_cast_time.elapsed().as_secs_f32() >= character.secondary_cooldown {
              secondary_used_successfully = true;
              // set position to beginning of buffer (where player was 3 seconds ago)
              players[p_index].position = players[p_index].previous_positions[0];
              players[p_index].previous_positions = Vec::new();
              players[p_index].heal(10, characters.clone());
            }
          },

          Character::Elizabeth => {
            // Spawn a prakata billar bug.
            // (but for copyright reasons it only looks like one and isn't one!!!!!!)

            // beforehand we need to check if there's already one in the game, and delete it.
            for o_index in 0..game_objects.len() {
              if game_objects[o_index].object_type == GameObjectType::ElizabethTurret
              && game_objects[o_index].get_bullet_data().owner_username == players[p_index].username {
                game_objects[o_index].to_be_deleted = true;
              }
            }


            // spawn the new one
            game_objects.push(GameObject {
              object_type: GameObjectType::ElizabethTurret,
              position: players[p_index].position,
              to_be_deleted: false,
              id: game_object_id_counter.increment(),
              extra_data: ObjectData::BulletData(
                BulletData {
                  direction: players[p_index].aim_direction,
                  owner_username: players[p_index].username.clone(),
                  hitpoints: 0,
                  lifetime: character.secondary_cooldown,
                  hit_players: vec![],
                  traveled_distance: 0.0,
                }
              )
            });
            secondary_used_successfully = true;
          }
          Character::Wiro => {
            if players[p_index].secondary_charge > 0 
            && players[p_index].secondary_cast_time.elapsed().as_secs_f32() > character.secondary_cooldown {

              // spawn a shield object, if one can't be found already.
              
              // look for a shield
              let position: Vector2 = Vector2 {
                x: players[p_index].position.x + players[p_index].aim_direction.x * TILE_SIZE,
                y: players[p_index].position.y + players[p_index].aim_direction.y * TILE_SIZE,
              };
              let mut shield_found = false;
              for o_index in 0..game_objects.len() {
                // if it's a shield, and it's ours
                if game_objects[o_index].object_type == GameObjectType::WiroShield
                && game_objects[o_index].get_bullet_data().owner_username == players[p_index].username {
                  let mut shield_data = game_objects[o_index].get_bullet_data();
                  shield_data.direction = players[p_index].aim_direction;
                  game_objects[o_index].extra_data = ObjectData::BulletData(shield_data);
                  game_objects[o_index].position = position;
                  shield_found = true;
                  break;
                }
              }
              if !shield_found {
                game_objects.push(GameObject {
                  object_type: GameObjectType::WiroShield,
                  //size: Vector2 { x: TILE_SIZE*0.5, y: characters[&Character::Wiro].secondary_range },
                  position: position,
                  to_be_deleted: false,
                  id: game_object_id_counter.increment(),
                  extra_data: ObjectData::BulletData(
                    BulletData {
                      direction: players[p_index].aim_direction,
                      owner_username: players[p_index].username.clone(),
                      hitpoints: 0,
                      lifetime: f32::INFINITY,
                      hit_players: vec![],
                      traveled_distance: 0.0,
                    }
                  )
                });
              }
            } else {
              // delete the shield, if it exists.
              for o_index in 0..game_objects.len() {
                // if it's a shield, and it's ours
                if game_objects[o_index].object_type == GameObjectType::WiroShield
                && game_objects[o_index].get_bullet_data().owner_username == players[p_index].username {
                  game_objects[o_index].to_be_deleted = true;
                  // if our secondary charge is 0, also set the cooldown
                  if players[p_index].secondary_charge == 0 {
                    players[p_index].secondary_cast_time = Instant::now();
                  }
                  break;
                }
              }
            }
          }
          // TEMERITY ROCKET JUMP
          Character::Temerity => {
            if players[p_index].secondary_cast_time.elapsed().as_secs_f32() > character.secondary_cooldown {
              // apply an impulse
              let direction = players[p_index].aim_direction;
              let yeet = 0.2 * TILE_SIZE;
              let lifetime = 0.2;
              players[p_index].buffs.push(
                Buff { value: yeet, duration: 1.0, buff_type: BuffType::Impulse, direction: direction * -1.0 }
              );
              game_objects.push(GameObject {
                object_type: GameObjectType::TemerityRocketSecondary,
                position: players[p_index].position + players[p_index].aim_direction * characters[&Character::Temerity].secondary_range,
                to_be_deleted: false,
                id: game_object_id_counter.increment(),
                extra_data: ObjectData::BulletData(
                  BulletData {
                    direction: players[p_index].aim_direction * -1.0,
                    owner_username: players[p_index].username.clone(),
                    hitpoints: 0,
                    lifetime,
                    hit_players: vec![],
                    traveled_distance: 0.0,
                  }
                ),
              });
              secondary_used_successfully = true;
            }
          }
          Character::Dummy => {}  
        }
        if secondary_used_successfully {
          players[p_index].secondary_charge -= character.secondary_charge_use;
          players[p_index].secondary_cast_time = Instant::now();
        }
      }
      // if the secondary button is released..
      else {
        match players[p_index].character {
          Character::Wiro => {
            for o_index in 0..game_objects.len() {
              // if it's a shield, and it's ours
              if game_objects[o_index].object_type == GameObjectType::WiroShield
              && game_objects[o_index].get_bullet_data().owner_username == players[p_index].username {
                game_objects[o_index].to_be_deleted = true;
                players[p_index].secondary_cast_time = Instant::now();
                break;
              }
            }
          }
          _ => {}
        }
      }
    }

    // println!("{:?}", game_objects);
    // println!("{}", 1.0 / delta_time);

    // (vscode) MARK: Object Handlin'
    // Do all logic related to game objects

    // contemplating my orb
    if tick {
      orb_timer += 1.0 as f32;
      let mut orb_found = false;
      for game_object in game_objects.clone() {
        if game_object.object_type == GameObjectType::CenterOrb {
          orb_found = true;
          orb_timer = 0.0;
          break;
        }
      }
      if orb_timer > orb_spawn_interval && !orb_found {
        let mut orb_position: Vector2 = Vector2::new();
        // check if there's already an orb
        // get the position of the spawner
        for game_object in game_objects.clone() {
          if game_object.object_type == GameObjectType::CenterOrbSpawnPoint {
            orb_position = game_object.position;
            break;
          }
        }
        game_objects.push(
          GameObject {
            object_type: GameObjectType::CenterOrb,
            position: orb_position,
            to_be_deleted: false,
            id: game_object_id_counter.increment(),
            extra_data: ObjectData::WallData(
              WallData {
                hitpoints: 60,
                lifetime: f32::INFINITY,
              }
            ),
          }
        );
      }
    }

    for o_index in 0..game_objects.len() {
      let game_object_type = game_objects[o_index].object_type.clone();
      match game_object_type {

        // WOLF primary
        GameObjectType::HernaniBullet => {
          (players, *game_objects, _) = apply_simple_bullet_logic(players, characters.clone(), game_objects.clone(), o_index, true_delta_time, false);
        }
        // WOLF dash special
        GameObjectType::HernaniLandmine => {
          // if the landmine has existed for long enough...
          //if game_objects[o_index].lifetime < (characters[&Character::Hernani].dash_cooldown - 0.5) {
          for p_index in 0..players.len() {
            // if not on same team
            if players[p_index].team != players[index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone())].team {
              // if within range
              let landmine_range = characters[&Character::Hernani].primary_range_2;
              if Vector2::distance(game_objects[o_index].position, players[p_index].position)
              < landmine_range {
                players[p_index].damage(characters[&Character::Hernani].primary_damage_2, characters.clone());
                game_objects[o_index].to_be_deleted = true;
                break;
              }
            }
          }
        }

        // HEALER GIRL primary
        GameObjectType::RaphaelleBullet => {
          let hit: bool;
          (players, *game_objects, hit) = apply_simple_bullet_logic(players, characters.clone(), game_objects.clone(), o_index, true_delta_time, true);
          
          // Restore nearby ally health
          if hit {
            for p_index in 0..players.len() {
              let range: f32 = characters[&Character::Raphaelle].primary_range;
              if Vector2::distance(
                players[p_index].position,
                players[index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone())].position
              ) < range &&
                players[p_index].team == players[index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone())].team {
                // Anyone within range
                if p_index == index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone()) {
                  // if self, heal less
                  let heal_self: u8 = characters[&Character::Raphaelle].primary_lifesteal;
                  players[p_index].heal(heal_self, characters.clone());
                }
                else {
                  // otherwise, apply normal heal
                  let heal: u8 = characters[&Character::Raphaelle].primary_heal_2;
                  players[p_index].heal(heal, characters.clone());
                }
                  // restore dash charge (0.2s)
                // players[game_objects[o_index].owner_index].last_dash_time -= Duration::from_millis(200);
              }
            }
          }
        }
        // RAPHAELLE primary, EMPOWERED
        GameObjectType::RaphaelleBulletEmpowered => {
          let hit: bool;
          (players, *game_objects, hit) = apply_simple_bullet_logic_extra(
            players, characters.clone(), game_objects.clone(), o_index, true_delta_time, true,
            characters[&Character::Raphaelle].primary_damage_2, 255, false, f32::INFINITY, f32::INFINITY);
          if hit {
            // restore dash charge
            let owner_index = index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone());
            players[owner_index].last_dash_time -= Duration::from_secs_f32(characters[&Character::Raphaelle].primary_cooldown_2);
          }
        }

        // RAPHAELLE secondary
        GameObjectType::RaphaelleAura => {
          // game_objects[o_index].position = players[game_objects[o_index].owner_index].position;
          // every second apply heal
          for p_index in 0..players.len() {
            // if on same team
            if players[p_index].team == players[index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone())].team {
              // if within range
              if Vector2::distance(game_objects[o_index].position, players[index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone())].position)
              < (characters[&Character::Raphaelle].secondary_range) {
                // heal up
                if tick {
                  let heal_amount = characters[&players[p_index].character].secondary_heal;
                  players[p_index].heal(heal_amount, characters.clone());
                }
                // provide fire rate buff, if not present
                let mut buff_found = false;
                for buff_index in 0..players[p_index].buffs.len() {
                  if players[p_index].buffs[buff_index].buff_type == BuffType::RaphaelleFireRate {
                    buff_found = true;
                    break; // exit early
                  }
                }
                if !buff_found {                                 //        2  0%  <- find way to source this from properties file sometime idk
                  players[p_index].buffs.push(Buff { value: 0.2, duration: 0.1, buff_type: BuffType::RaphaelleFireRate, direction: Vector2::new() });
                }
              }
              // not actually necessary
              //else {
              //  for buff_index in 0..players[p_index].buffs.len() {
              //    if players[p_index].buffs[buff_index].buff_type == BuffType::RaphaelleFireRate {
              //      players[p_index].buffs.remove(buff_index);
              //      break; // exit early
              //    }
              //  }
              //}
            }
          }
        }

        // QUEEN primary
        GameObjectType::CynewynnSword => {
          (players, *game_objects, _) = apply_simple_bullet_logic(players, characters.clone(), game_objects.clone(), o_index, true_delta_time, true);
        }
        // ELIZABETH primary
        GameObjectType::ElizabethProjectileRicochet => {
          (players, *game_objects, _) = apply_simple_bullet_logic_extra(players, characters.clone(), game_objects.clone(), o_index, true_delta_time, true, 
            255, 255, true, f32::INFINITY, f32::INFINITY);
        }
        // ELIZABETH primary but recalled
        GameObjectType::ElizabethProjectileGroundRecalled => {
          // needs to move towards owner
          let owner_username = game_objects[o_index].get_bullet_data().owner_username.clone();
          let owner_index = index_by_username(&owner_username, players.clone());
          let target_position: Vector2 = players[owner_index].position;
          let object_position: Vector2 = game_objects[o_index].position;
          let speed = characters[&players[owner_index].character].primary_shot_speed;
          let direction: Vector2 = Vector2::difference(object_position, target_position);
          // update position
          game_objects[o_index].position.x += direction.normalize().x * speed * true_delta_time as f32;
          game_objects[o_index].position.y += direction.normalize().y * speed * true_delta_time as f32;
          // If the projectiles are close enough to us, delete them, since their trip is over.
          if direction.magnitude() < TILE_SIZE /* arbitrary value */ {
            game_objects[o_index].to_be_deleted = true;
          }
          let hit_radius = characters[&players[owner_index].character].primary_hit_radius;
          let damage = characters[&players[owner_index].character].primary_damage_2;
          for p_index in 0..players.len() {
            let player_position = players[p_index].position;
            // if we hit a player
            if Vector2::distance(player_position, object_position) < hit_radius
            // and we haven't already
            && !game_objects[o_index].get_bullet_data().hit_players.contains(&p_index) {
              // damage them
              players[p_index].damage(damage, characters.clone());
              // and check if they were already hit by a projectile.
              let mut was_already_hit: bool = false;
              for o_index_2 in 0..game_objects.len() {
                match game_objects[o_index_2].get_bullet_data_safe() {
                  Ok(bullet_data) => {
                    if bullet_data.hit_players.contains(&p_index)
                    && o_index_2 != o_index {
                      was_already_hit = true;
                      break;
                    }
                  }
                  Err(()) => {

                  }
                }
              }
              if was_already_hit {
                // apply a debuff
                players[p_index].buffs.push(
                  Buff {
                    value: -2.5 ,
                    duration: 0.25,
                    buff_type: BuffType::Speed,
                    direction: Vector2::new(),
                  }
                );
              }
              // Finally, update the game object to know this player was already hit
              let mut bullet_data = game_objects[o_index].get_bullet_data();
              bullet_data.hit_players.push(p_index);
              game_objects[o_index].extra_data = ObjectData::BulletData(bullet_data);
            }
          }
        }
        // ELIZABETH'S TURRET
        GameObjectType::ElizabethTurret => {
          // PROJECTILES
          // shoot projectiles. use secondary_cast_time as cooldown counter.
          let owner = index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone());
          let owner_team = players[owner].team;
          let object_pos = game_objects[o_index].position;
          let range = characters[&Character::Elizabeth].secondary_range;
          let cooldown = characters[&Character::Elizabeth].primary_cooldown_2;
          let speed = characters[&Character::Elizabeth].primary_shot_speed_2;

          for player in players.clone() {
            if player.team != owner_team
            && Vector2::distance(object_pos, player.position) < range {
              if players[owner].secondary_cast_time.elapsed().as_secs_f32() > cooldown {
                // shoot
                game_objects.push(GameObject {
                  object_type: GameObjectType::ElizabethTurretProjectile,
                  position: object_pos,
                  to_be_deleted: false,
                  id: game_object_id_counter.increment(),
                  extra_data: ObjectData::BulletData(
                    BulletData {
                      direction: Vector2::difference(object_pos, player.position).normalize(),
                      owner_username: players[owner].username.clone(),
                      hitpoints: 0,
                      lifetime: range/speed,
                      hit_players: vec![],
                      traveled_distance: 0.0,
                    }
                  )
                });

                // reset CD
                players[owner].secondary_cast_time = Instant::now();
              }
            }
          }
          // MOVEMENT
          // flip
          let speed = characters[&Character::Elizabeth].primary_range_3;

          // check for collisions with walls
          let pos = game_objects[o_index].position;
          let direction = game_objects[o_index].get_bullet_data().direction;

          let check_distance = TILE_SIZE * 0.1;
          let buffer = 0.5 * 2.0;
          let check_position: Vector2 = Vector2 {
            x: pos.x + direction.x * check_distance ,
            y: pos.y + direction.y * check_distance ,
          };
          for game_object in game_objects.clone() {
            if WALL_TYPES_ALL.contains(&game_object.object_type)
            && Vector2::distance(game_object.position, check_position) < TILE_SIZE * buffer {
              // we have a collision. flip our direction.
              let distance: Vector2 = Vector2::difference(game_object.position, check_position);
              let obj_distance = Vector2::difference(game_object.position, pos);
              // if distance.x is greater, it means we need to flip horizonrally.
              // otherwise, flip vertically.
              if f32::abs(distance.x) > f32::abs(distance.y) {
                // flip horizontally
                // also check that we're going in opposing directions :)
                if direction.x * obj_distance.x < 0.0 {
                  let mut bullet_data = game_objects[o_index].get_bullet_data();
                  bullet_data.direction.x *= -1.0;
                  game_objects[o_index].extra_data = ObjectData::BulletData(bullet_data);
                }
              } else {
                // also check that we're going in opposing directions :)
                if direction.y * obj_distance.y < 0.0 {
                  let mut bullet_data = game_objects[o_index].get_bullet_data();
                  bullet_data.direction.y *= -1.0;
                  game_objects[o_index].extra_data = ObjectData::BulletData(bullet_data);
                }
              }
              break;
            }
          }
          // move
          game_objects[o_index].position.x += game_objects[o_index].get_bullet_data().direction.x * speed * true_delta_time as f32;
          game_objects[o_index].position.y += game_objects[o_index].get_bullet_data().direction.y * speed * true_delta_time as f32;

        }
        // ELIZABETH TURRET PROJECTILE
        GameObjectType::ElizabethTurretProjectile => {
          let damage = characters[&Character::Elizabeth].secondary_damage;
          let speed = characters[&Character::Elizabeth].primary_shot_speed_2;
          (players, *game_objects, _) = apply_simple_bullet_logic_extra(
            players, characters.clone(), game_objects.clone(), o_index, true_delta_time, false,
            damage, 255, false, speed, f32::INFINITY);
        }
        // WIRO'S SHIELD
        GameObjectType::WiroShield => {
          // delete any projectiles that have come into contact
          let countered_projectiles: Vec<GameObjectType> = vec![
            GameObjectType::HernaniBullet,                    // hernani
            GameObjectType::RaphaelleBullet,                  // raph
            GameObjectType::RaphaelleBulletEmpowered,
            GameObjectType::CynewynnSword,                    // cyne
            GameObjectType::ElizabethProjectileRicochet,      // elizabeth
            GameObjectType::ElizabethProjectileGroundRecalled,
            GameObjectType::ElizabethTurretProjectile,
            GameObjectType::WiroGunShot,                      // wiro
          ];
          for victim_object_index in 0..game_objects.len() {
            let object_type = game_objects[victim_object_index].object_type.clone();
            // if one of these objects is one we can counter...
            if countered_projectiles.contains(&object_type) {
              let obj1_owner_team = players[index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone())].team;
              let obj1_owner_index = index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone());
              let obj2_owner_team = players[index_by_username(&game_objects[victim_object_index].get_bullet_data().owner_username,players.clone())].team;
              let obj2_owner_character = players[index_by_username(&game_objects[victim_object_index].get_bullet_data().owner_username,players.clone())].character;
              if obj1_owner_team != obj2_owner_team {
                let hits_shield = hits_shield(
                  game_objects[o_index].position,
                  game_objects[o_index].get_bullet_data().direction,
                  game_objects[victim_object_index].position,
                  characters[&Character::Wiro].secondary_range,
                  5.0, // temporary
                );
                if hits_shield {
                  let damage_multiplier: f32 = 1.0;
                  let damage = (damage_multiplier * match game_objects[victim_object_index].object_type {
                    GameObjectType::HernaniBullet
                    | GameObjectType::CynewynnSword
                    | GameObjectType::ElizabethProjectileRicochet
                    | GameObjectType::RaphaelleBullet           => { characters[&obj2_owner_character].primary_damage }
                    GameObjectType::ElizabethProjectileGroundRecalled
                    | GameObjectType::RaphaelleBulletEmpowered  => { characters[&obj2_owner_character].primary_damage_2 }
                    _ => {panic!()}
                  } as f32) as u8;
                  if players[obj1_owner_index].secondary_charge > damage{
                    players[obj1_owner_index].secondary_charge -= damage;
                  } else {
                    players[obj1_owner_index].secondary_charge = 0;
                  }
                  game_objects[victim_object_index].to_be_deleted = true;
                }
              }
            }
          }
        }
        // WIRO'S PRIMARY FIRE
        GameObjectType::WiroGunShot => {
          let distance_traveled = game_objects[o_index].get_bullet_data().traveled_distance;
          let damage: u8;
          if distance_traveled > characters[&Character::Wiro].primary_range_2 {
            damage = characters[&Character::Wiro].primary_damage_2;
          } else {
            damage = characters[&Character::Wiro].primary_damage;
          }
          let hit: bool;
          (players, *game_objects, hit) = apply_simple_bullet_logic_extra(players, characters.clone(), game_objects.clone(), o_index, true_delta_time, true, 
            damage, 255, false, f32::INFINITY, f32::INFINITY);
          if hit {
            let owner_index = index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone());
            players[owner_index].stacks = 1;
          }
        }
        // WIRO'S DASH
        GameObjectType::WiroDashProjectile => {
          // lock it to wiro's position
          let owner_index = index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone());
          let range = characters[&Character::Wiro].primary_range_3;
          let heal = characters[&Character::Wiro].secondary_heal;
          let damage = characters[&Character::Wiro].secondary_damage;
          game_objects[o_index].position = players[index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone())].position;
          for victim_index in 0..players.len() {
            // if we get a hit, and we didn't already hit
            if Vector2::distance(players[victim_index].position, game_objects[o_index].position) < range
            && !game_objects[o_index].get_bullet_data().hit_players.contains(&victim_index) {
              if players[victim_index].team == players[owner_index].team {
                players[victim_index].heal(heal, characters.clone());
              } else {
                players[victim_index].damage(damage, characters.clone());
              }
              let mut bullet_data = game_objects[o_index].get_bullet_data();
              bullet_data.hit_players.push(victim_index);
              game_objects[o_index].extra_data = ObjectData::BulletData(bullet_data);
            }
          }
        }
        // TEMERITY ROCKET LAUNCHAAA
        GameObjectType::TemerityRocket => {
          let hit: bool;
          (players, *game_objects, hit) = apply_simple_bullet_logic(players, characters.clone(), game_objects.clone(), o_index, true_delta_time, false);
          let owner_index = index_by_username(&game_objects[o_index].get_bullet_data().owner_username,players.clone());
          if hit && players[owner_index].stacks < 2 {
            players[owner_index].stacks += 1;
          }
        }
        GameObjectType::TemerityRocketSecondary => {
          (players, *game_objects, _) = apply_simple_bullet_logic_extra(
            players, characters.clone(),
            game_objects.clone(),
            o_index,
            true_delta_time,
            true,
            characters[&Character::Temerity].secondary_damage,
            255,
            false,
            characters[&Character::Temerity].primary_shot_speed_2,
            characters[&Character::Temerity].secondary_range,
          );
        }
        _ => {}
      }
      // lifetimes
      match game_objects[o_index].get_bullet_data_safe() {
        Ok(mut bullet_data) => {
          bullet_data.lifetime -= true_delta_time as f32;
          game_objects[o_index].extra_data = ObjectData::BulletData(bullet_data);
          if game_objects[o_index].get_bullet_data().lifetime < 0.0 {
            game_objects[o_index].to_be_deleted = true;
          }
        },
        Err(()) => {
          match game_objects[o_index].get_wall_data_safe() {
            Ok(mut wall_data) => {
              wall_data.lifetime -= true_delta_time as f32;
              game_objects[o_index].extra_data = ObjectData::WallData(wall_data);
              if game_objects[o_index].get_wall_data().lifetime < 0.0 {
                game_objects[o_index].to_be_deleted = true;
              }
            }
            Err(()) => {

            }
          }
        }
      }
    }

    // (vscode) MARK: Object Deletion
    let mut cleansed_game_objects: Vec<GameObject> = Vec::new();
    for game_object in game_objects.clone() {
      if game_object.to_be_deleted == true {
        // EXTRA LOGIC
        match game_object.object_type.clone() {
          // Elizabeth's projectile needs to fall down on deletion,
          // if it hit somebody,
          GameObjectType::ElizabethProjectileRicochet => {
            cleansed_game_objects.push(
              GameObject {
                object_type: GameObjectType::ElizabethProjectileGround,
                position: game_object.position,
                to_be_deleted: false,
                id: game_object_id_counter.increment(),
                extra_data: ObjectData::BulletData(
                  BulletData {

                    direction: Vector2::new(),
                    owner_username: game_object.get_bullet_data().owner_username,
                    hitpoints: 0,
                    lifetime: 5.0,
                    hit_players: Vec::new(),
                    traveled_distance: 0.0,
                  }
                ),
              }
            );
          },
          // If we didn't hit anybody, take away all stacks, to bring the combo
          // back to its first projectile
          GameObjectType::TemerityRocket => {
            if game_object.get_bullet_data().hit_players.is_empty() {
              let owner_index = index_by_username(&game_object.get_bullet_data().owner_username,players.clone());
              players[owner_index].stacks = 0;
            }
          }
          _ => {},
        }
      } else {
        cleansed_game_objects.push(game_object);
      }
    }

    *game_objects = cleansed_game_objects;

    // free the mutexes BEFORE we start the sleep.
    drop(gamemode_info);
    drop(game_objects);
    drop(players);
    // println!("Server Hz: {}", 1.0 / delta_time);
    delta_time = server_counter.elapsed().as_secs_f64();
    if delta_time < desired_delta_time {
      thread::sleep(Duration::from_secs_f64(desired_delta_time - delta_time));
    }
  }
}