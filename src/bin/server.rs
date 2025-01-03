use top_down_shooter::common::*;
use core::f32;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, MutexGuard};
use bincode;
use std::{thread, time::*};

const WALL_TYPES: [GameObjectType; 3] = [GameObjectType::Wall, GameObjectType::UnbreakableWall, GameObjectType::SniperWall];

fn main() {

  // Load character properties
  let characters: HashMap<Character, CharacterProperties> = load_characters();
  println!("Loaded character properties.");


  let players: Vec<ServerPlayer> = Vec::new();
  let players = Arc::new(Mutex::new(players));
  let game_objects:Vec<GameObject> = load_map_from_file(include_str!("../../assets/maps/map_maker.map"));
  println!("Loaded game objects.");
  let game_objects = Arc::new(Mutex::new(game_objects));

  // initiate all networking sockets
  let server_listen_address = format!("0.0.0.0:{}", SERVER_LISTEN_PORT);
  let server_send_address = format!("0.0.0.0:{}", SERVER_SEND_PORT);
  let listening_socket = UdpSocket::bind(server_listen_address.clone()).expect("Error creating listener UDP socket");
  let sending_socket = UdpSocket::bind(server_send_address).expect("Error creating sender UDP socket");
  let mut buffer = [0; 4096]; // The size of this buffer is lowkey kind of low, especially with how big the gameobject struct is.
  println!("Sockets bound.");
  println!("Listening on: {}", server_listen_address.clone());

  let mut red_team_player_count = 0;
  let mut blue_team_player_count = 0;

  // temporary, to be dictated by gamemode
  let mut character_queue: Vec<Character> = vec![Character::SniperGirl, Character::TimeQueen, Character::HealerGirl, Character::SniperGirl, Character::TimeQueen, Character::HealerGirl];
  let max_players = character_queue.len();
  
  // (vscode) MARK: Networking - Listen
  let listener_players = Arc::clone(&players);
  let listener_game_objects = Arc::clone(&game_objects);
  std::thread::spawn(move || {
    loop {
      // recieve packet
      let (amt, src) = listening_socket.recv_from(&mut buffer).expect(":(");
      let data = &buffer[..amt];
      let mut recieved_player_info: ClientPacket = bincode::deserialize(data).expect("Might need to find a solution to this");
      // println!("SERVER: Received from {}: {:?}", src, recieved_player_info);
      
      // clean all recieved NaNs and infinites so the server doesnt explode
      recieved_player_info.aim_direction.clean();
      recieved_player_info.movement.clean();
      recieved_player_info.position.clean();
      recieved_player_info.packet_interval.clean();
      
      // update PLAYERS Vector with recieved information.
      let mut listener_players = listener_players.lock().unwrap();
      let listener_game_objects = listener_game_objects.lock().unwrap();
      let readonly_game_objects = listener_game_objects.clone();
      drop(listener_game_objects);
      
      let mut player_found: bool = false;
      
      // iterate through players
      for player_index in 0..listener_players.len() {
        
        let mut player = listener_players[player_index].clone();
        
        // use IP as identifier, check if packet from srent player correlates to our player
        if player.ip == src.ip().to_string() {
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
          
          let mut new_position = Vector2::new();
          let recieved_position = recieved_player_info.position;
          let movement_error_margin = 5.0;
          let mut movement_legal = true;
          let previous_position = player.position.clone();
          
          // If player wants to dash and isn't dashing...
          if recieved_player_info.dashing && !player.is_dashing {
            let player_dash_cooldown = characters[&player.character].dash_cooldown;
            // And we're past the cooldown...
            if player.last_dash_time.elapsed().as_secs_f32() > player_dash_cooldown {
              // reset the cooldown
              player.last_dash_time = Instant::now();
              // set dashing to true
              player.is_dashing = true;
              // set the dashing direction
              if recieved_player_info.movement.magnitude() > 0.0 {
                player.dash_direction = recieved_player_info.movement;
              }
              else {
                player.is_dashing = false;
              }
            }
          }
          // (vscode) MARK: Dashing Legality
          if player.is_dashing {
            let player_dashing_speed: f32 = characters[&player.character].dash_speed;
            let player_max_dash_distance: f32 = characters[&player.character].dash_distance;
            
            let mut dash_movement = Vector2::new();
            dash_movement.x = player.dash_direction.x * player_dashing_speed * time_since_last_packet as f32;
            dash_movement.y = player.dash_direction.y * player_dashing_speed * time_since_last_packet as f32;
            
            // calculate current expected position based on input
            let (new_movement_raw, _): (Vector2, Vector2) = object_aware_movement(
              previous_position,
              player.dash_direction,
              dash_movement,
              readonly_game_objects
            );
            
            if player.dashed_distance < player_max_dash_distance {
              new_position.x = previous_position.x + new_movement_raw.x * player_dashing_speed * time_since_last_packet as f32;
              new_position.y = previous_position.y + new_movement_raw.y * player_dashing_speed * time_since_last_packet as f32;
              
              player.dashed_distance += Vector2::distance(new_position, previous_position);
            }
            else {
              player.dashed_distance = 0.0;
              player.is_dashing = false;
              // The final frame of the dash new_position isn't updated, if not
              // for this line the player would get sent back to 0.0-0.0
              new_position = player.position;
            }
          }

          else {
            // (vscode) MARK: Movement Legality
            // Movement legality calculations
            let raw_movement = recieved_player_info.movement;
            let mut movement = Vector2::new();
            let player_movement_speed: f32 = characters[&player.character].speed;

            movement.x = raw_movement.x * player_movement_speed * time_since_last_packet as f32;
            movement.y = raw_movement.y * player_movement_speed * time_since_last_packet as f32;

            // calculate current expected position based on input
            let (new_movement_raw, _): (Vector2, Vector2) = object_aware_movement(
              previous_position,
              raw_movement,
              movement,
              readonly_game_objects
            );

            new_position.x = previous_position.x + new_movement_raw.x * player_movement_speed * time_since_last_packet as f32;
            new_position.y = previous_position.y + new_movement_raw.y * player_movement_speed * time_since_last_packet as f32;

            player.move_direction = new_movement_raw;

          }

          if Vector2::distance(new_position, recieved_position) > movement_error_margin {
            movement_legal = false;
          }
          // println!("{:?}", new_movement_raw);
          
          if movement_legal {
            // Apply the movement that the server calculated, which should be similar to the client's.
            player.position = new_position;
          } else {
            // Inform the network sender it needs to send a correction packet (position override packet).
            player.had_illegal_position = true;
            // Also apply movement.
            player.position = new_position;
          }
          // exit loop, and inform rest of program not to proceed with appending a new player.
          player_found = true;
          listener_players[player_index] = player;
          break
        }
      }
      // (vscode) MARK: Instantiate Player
      // otherwise, add the player
      // NOTE: In the future this entire chunk of code will be gone, the matchmaker will populate
      // the list of players beforehand.
      if !player_found && (blue_team_player_count + red_team_player_count < max_players) {
        // decide the player's team (alternate for each player)
        let mut team: Team = Team::Blue;
        if blue_team_player_count > red_team_player_count {
          team = Team::Red;
          red_team_player_count += 1;
        } else {
          blue_team_player_count += 1;
        }
        // create server player data
        // this data is pretty irrelevant, we're just initialising the player.
        listener_players.push(ServerPlayer {
          ip: src.ip().to_string(),
          team,
          health: 100,
          position: match team {
            Team::Blue => Vector2 { x: 10.0, y: 10.0 },
            Team::Red  => Vector2 { x: 90.0, y: 90.0 },
          },
          move_direction:       Vector2::new(),
          aim_direction:        Vector2::new(),
          shooting:             false,
          shooting_secondary:   false,
          secondary_cast_time:  Instant::now(),
          secondary_charge:     100,
          had_illegal_position: false,
          character:            character_queue[0],
          last_shot_time:       Instant::now(),
          is_dashing:           false,
          last_dash_time:       Instant::now(),
          dashed_distance:      0.0,
          dash_direction:       Vector2::new(),
          previous_positions:   Vec::new(),
        });
        println!("Player connected: {}", src.ip().to_string());
        character_queue.remove(0);
      }
      drop(listener_players);
    }
  });
  
  let mut server_counter: Instant = Instant::now();
  let mut delta_time: f64 = server_counter.elapsed().as_secs_f64();
  // Server logic frequency in Hertz. Doesn't need to be much higher than 120.
  // Higher frequency = higher precission with computation trade-off
  let desired_delta_time: f64 = 1.0 / 400.0;
  let mut networking_counter: Instant = Instant::now();

  let main_game_objects = Arc::clone(&game_objects);

  // for once-per-second operations, called ticks
  let mut tick_counter = Instant::now();

  // for once-per-decisecond operations.
  let mut tenth_tick_counter = Instant::now();
  
  // (vscode) MARK: Server Loop
  let characters = load_characters();
  let main_loop_players = Arc::clone(&players);
  loop {
    let mut tick: bool = false;
    let mut tenth_tick: bool = false;
    server_counter = Instant::now();
    
    let mut true_delta_time: f64 = 0.0;
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
    
    let mut main_loop_players = main_loop_players.lock().unwrap();
    
    // do all logic related to players
    for player_index in 0..main_loop_players.len() {
      let shooting = main_loop_players[player_index].shooting;
      let shooting_secondary = main_loop_players[player_index].shooting_secondary;
      let last_shot_time = main_loop_players[player_index].last_shot_time;
      let secondary_charge = main_loop_players[player_index].secondary_charge;
      let player_info = main_loop_players[player_index].clone();
      let character: CharacterProperties = characters[&main_loop_players[player_index].character].clone();

      // if main_loop_players[player_index].health >= 0 {
      //   main_loop_players[player_index].health = 100;
      //   main_loop_players[player_index].position = Vector2::new();
      // }

      // (vscode) MARK: Passives & Other
      // Handling of passive abilities and anything else that may need to be run all the time.

      // Handling of time queen flashsback ability
      if main_loop_players[player_index].character == Character::TimeQueen {
        // Update once per decisecond
        if tenth_tick {
          // update buffer of positions when secondary isnt active
          let position: Vector2 = main_loop_players[player_index].position.clone();
          main_loop_players[player_index].previous_positions.push(position);
          // cut the buffer to remain the correct size
          let position_buffer_length: usize = (character.secondary_cooldown * 10.0) as usize;
          if main_loop_players[player_index].previous_positions.len() > position_buffer_length {
            main_loop_players[player_index].previous_positions.remove(0);
          }
        }
      }

      // Get stuck player out of walls
      let unstucker_game_objects = main_game_objects.lock().unwrap();
      for game_object_index in 0..unstucker_game_objects.len() {
        
        if WALL_TYPES.contains(&unstucker_game_objects[game_object_index].object_type) {
          let difference: Vector2 = Vector2::difference(unstucker_game_objects[game_object_index].position, main_loop_players[player_index].position);
          if f32::abs(difference.x) < TILE_SIZE && f32::abs(difference.y) < TILE_SIZE {
            // push out the necessary amount
            main_loop_players[player_index].position.x -= TILE_SIZE - difference.x;
            main_loop_players[player_index].position.y -= TILE_SIZE - difference.y;
          }
        }
      }
      drop(unstucker_game_objects);
      // (vscode) MARK: Primaries
      // If someone is shooting, spawn a bullet according to their character.s
      if shooting && !shooting_secondary && last_shot_time.elapsed().as_secs_f32() > character.primary_cooldown {
        main_loop_players[player_index].last_shot_time = Instant::now();
        let mut game_objects = main_game_objects.lock().unwrap();
        // Do primary shooting logic
        match main_loop_players[player_index].character {
          Character::SniperGirl => {
            game_objects.push(GameObject {
              object_type: GameObjectType::SniperGirlBullet,
              size: TILE_SIZE,
              position: main_loop_players[player_index].position,
              direction: main_loop_players[player_index].aim_direction,
              to_be_deleted: false,
              hitpoints: 0,
              owner_index: player_index,
              lifetime: character.primary_range / character.primary_shot_speed,
              players: Vec::new(),
              traveled_distance: 0.0,
            });
          }
          Character::HealerGirl => {
            game_objects.push(GameObject {
              object_type: GameObjectType::HealerGirlPunch,
              size: TILE_SIZE,
              position: main_loop_players[player_index].position,
              direction: main_loop_players[player_index].aim_direction,
              to_be_deleted: false,
              hitpoints: 0,
              owner_index: player_index,
              lifetime: character.primary_range / character.primary_shot_speed,
              players: Vec::new(),
              traveled_distance: 0.0,
            });
          }
          Character::TimeQueen => {
            game_objects.push(GameObject {
              object_type: GameObjectType::TimeQueenSword,
              size: TILE_SIZE,
              position: main_loop_players[player_index].position,
              direction: main_loop_players[player_index].aim_direction,
              to_be_deleted: false,
              hitpoints: 0,
              owner_index: player_index,
              lifetime: character.primary_range / character.primary_shot_speed,
              players: Vec::new(),
              traveled_distance: 0.0,
            });
          }
        }
        drop(game_objects);
      }
      // (vscode) MARK: Secondaries
      // If a player is trying to use their secondary and they have enough charge to do so, apply custom logic.
      if shooting_secondary && secondary_charge >= character.secondary_charge_use {
        main_loop_players[player_index].shooting_secondary = false;
        let mut secondary_used_successfully = false;
        
        match main_loop_players[player_index].character {
          
          // Create a healing aura
          Character::HealerGirl => {
            // Create a bullet type and then define its actions in the next loop that handles bullets
            let mut game_objects = main_game_objects.lock().unwrap();
            game_objects.push(GameObject {
              object_type: GameObjectType::HealerAura,
              size: 60.0,
              position: player_info.position,
              direction: Vector2::new(),
              to_be_deleted: false,
              owner_index: player_index,
              hitpoints: 0,
              lifetime: 5.0,
              players: vec![],
              traveled_distance: 0.0,
            });
            drop(game_objects);
            secondary_used_successfully = true;
          },
          // Place walls
          Character::SniperGirl => {
            // Place down a wall at a position rounded to TILE_SIZE, unless a wall is alredy there.
            let wall_place_distance = character.secondary_range;
            let mut desired_placement_position: Vector2 = player_info.position;
            // round to closest 10
            desired_placement_position.x = ((((desired_placement_position.x + player_info.aim_direction.x * wall_place_distance) / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;
            desired_placement_position.y = ((((desired_placement_position.y + player_info.aim_direction.y * wall_place_distance) / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;

            let mut wall_can_be_placed = true;
            let mut game_objects = main_game_objects.lock().unwrap();
            for game_object in game_objects.clone() {
              match game_object.object_type {
                GameObjectType::SniperWall | GameObjectType::UnbreakableWall | GameObjectType::Wall => {
                  if game_object.position.x == desired_placement_position.x && game_object.position.y == desired_placement_position.y {
                    wall_can_be_placed = false;
                  }
                },
                _ => {}
              }
            }
            if wall_can_be_placed {
              game_objects.push(GameObject {
                object_type: GameObjectType::SniperWall,
                size: TILE_SIZE,
                position: desired_placement_position,
                direction: Vector2::new(),
                to_be_deleted: false,
                owner_index: 0,
                hitpoints: 255,
                lifetime: 5.0,
                players: vec![],
                traveled_distance: 0.0,
              });
              secondary_used_successfully = true;
            }
            drop(game_objects);
          },
          // position revert
          Character::TimeQueen => {
            let flashback_length = (character.secondary_cooldown * 10.0) as usize; // deciseconds
            if player_info.previous_positions.len() >= flashback_length
            && main_loop_players[player_index].secondary_cast_time.elapsed().as_secs_f32() >= character.secondary_cooldown {
              main_loop_players[player_index].secondary_cast_time = Instant::now();
              secondary_used_successfully = true;
              main_loop_players[player_index].secondary_cast_time = Instant::now();
              // set position to beginning of buffer (where player was 3 seconds ago)
              main_loop_players[player_index].position = main_loop_players[player_index].previous_positions[0];
              main_loop_players[player_index].previous_positions = Vec::new();
            }
          },
        }
        if secondary_used_successfully {
          main_loop_players[player_index].secondary_charge -= character.secondary_charge_use;
        }
      }
      
    }
    
    // println!("{:?}", game_objects);
    // println!("{}", 1.0 / delta_time);

    // (vscode) MARK: Object Handling
    // Do all logic related to game objects
    let mut game_objects = main_game_objects.lock().unwrap();
    for game_object_index in 0..game_objects.len() {
      let game_object_type = game_objects[game_object_index].object_type;
      match game_object_type {
        GameObjectType::SniperGirlBullet => {
          (main_loop_players, *game_objects) = apply_simple_bullet_logic(main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, false);
        }
        GameObjectType::HealerGirlPunch => {
          (main_loop_players, *game_objects) = apply_simple_bullet_logic(main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, true);
        }
        GameObjectType::TimeQueenSword => {
          (main_loop_players, *game_objects) = apply_simple_bullet_logic(main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, true);
        }
        GameObjectType::HealerAura => {
          game_objects[game_object_index].position = main_loop_players[game_objects[game_object_index].owner_index].position;
           // every second apply heal
           if tick {
             for player_index in 0..main_loop_players.len() {
              if main_loop_players[player_index].team == main_loop_players[game_objects[game_object_index].owner_index].team {
                let heal_amount = characters[&main_loop_players[player_index].character].secondary_heal;
                main_loop_players[player_index].health.heal(heal_amount);
              }
             }
           }
        }
        _ => {}
      }
      game_objects[game_object_index].lifetime -= true_delta_time as f32;
      if game_objects[game_object_index].lifetime < 0.0 {
        game_objects[game_object_index].to_be_deleted = true;
      }
    }

    let mut cleansed_game_objects: Vec<GameObject> = Vec::new();
    for game_object in game_objects.clone() {
      if game_object.to_be_deleted == false {
        cleansed_game_objects.push(game_object);
      }
    }

    *game_objects = cleansed_game_objects;
    let game_objects_readonly = game_objects.clone();
    drop(game_objects);

    // (vscode) MARK: Networking - Send
    // Only do networking logic at some frequency
    if networking_counter.elapsed().as_secs_f64() > MAX_PACKET_INTERVAL {
      // reset the counter
      networking_counter = Instant::now();

      // Send a packet to each player
      for (index, player) in main_loop_players.clone().iter().enumerate() {

        // Gather info to send about other players
        let mut other_players: Vec<ClientPlayer> = Vec::new();
        for (other_player_index, player) in main_loop_players.clone().iter().enumerate() {
          if other_player_index != index {
            other_players.push(ClientPlayer {
              health: player.health,
              position: player.position,
              secondary_charge: player.secondary_charge,
              aim_direction: player.aim_direction,
              movement_direction: player.move_direction,
              shooting_primary: player.shooting,
              shooting_secondary: player.shooting_secondary,
              team: player.team,
              character: player.character,
              time_since_last_dash: player.last_dash_time.elapsed().as_secs_f32()
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
            last_dash_time: player.last_dash_time.elapsed().as_secs_f32(),
            character: player.character,
          },
          players: other_players,
          game_objects: game_objects_readonly.clone(),
        };
        main_loop_players[index].had_illegal_position = false;
        
        let mut player_ip = player.ip.clone();
        let split_player_ip: Vec<&str> = player_ip.split(":").collect();
        player_ip = split_player_ip[0].to_string();
        player_ip = format!("{}:{}", player_ip, CLIENT_LISTEN_PORT);
        // println!("PLAYER IP: {}", player_ip);
        // println!("PACKET: {:?}", server_packet);
        let serialized: Vec<u8> = bincode::serialize(&server_packet).expect("Failed to serialize message (this should never happen)");
        sending_socket.send_to(&serialized, player_ip).expect("Failed to send packet to client.");
        // player.had_illegal_position = false; // reset since we corrected the error.
      }
    }
    drop(main_loop_players);
    // println!("Server Hz: {}", 1.0 / delta_time);
    delta_time = server_counter.elapsed().as_secs_f64();
    if delta_time < desired_delta_time {
      thread::sleep(Duration::from_secs_f64(desired_delta_time - delta_time));
    }
  }
}

// (vscode) MARK: Functions, Structs

/// Loads any map from a properly formatted string: `<object> [posX] [posY]`
/// 
/// example:
/// ```rust
/// let game_objects: Vec<GameObject> = load_map_from_file(include_str!("../../assets/maps/map1.map"));
/// ```
/// map1.map:
/// ```
/// wall 10.0 10.0
/// wall 20.0 10.0
/// wall 30.0 10.0
/// wall 40.0 10.0
/// wall 50.0 10.0
/// ```
/// information held by server about players.
#[derive(Debug, Clone)]
struct ServerPlayer {
  ip:                   String,
  team:                 Team,
  character:            Character,
  health:               u8,
  position:             Vector2,
  shooting:             bool,
  last_shot_time:       Instant,
  shooting_secondary:   bool,
  secondary_cast_time:  Instant,
  secondary_charge:     u8,
  aim_direction:        Vector2,
  move_direction:       Vector2,
  had_illegal_position: bool,
  is_dashing:           bool,
  dash_direction:       Vector2,
  dashed_distance:      f32,
  last_dash_time:       Instant,
  previous_positions:   Vec<Vector2>,
}

/// Applies modifications to players and game objects as a result of
/// bullet behaviour this frame, for a specific bullet.
fn apply_simple_bullet_logic(
  mut main_loop_players: MutexGuard<Vec<ServerPlayer>>,
  characters: HashMap<Character, CharacterProperties>,
  mut game_objects: Vec<GameObject>,
  game_object_index: usize,
  true_delta_time: f64,
  pierceing_shot: bool,
) -> (MutexGuard<Vec<ServerPlayer>>, Vec<GameObject>) {
  let game_object = game_objects[game_object_index].clone();
  let player = main_loop_players[game_object.owner_index].clone();
  let character = player.character;
  let character_properties = characters[&character].clone();
  let owner_index = game_object.owner_index;
  let hit_radius: f32 = character_properties.primary_hit_radius;
  let bullet_speed: f32 = character_properties.primary_shot_speed;
  // Calculate collisions with walls
  for victim_object_index in 0..game_objects.len() {
    // if it's a wall
    if WALL_TYPES.contains(&game_objects[victim_object_index].object_type) {
      // if it's colliding
      if Vector2::distance(game_object.position, game_objects[victim_object_index].position) < (5.0 + hit_radius) {
        // delete the bullet
        game_objects[game_object_index].to_be_deleted = true;
        if game_objects[victim_object_index].object_type == GameObjectType::Wall {
          // damage the wall if it's not unbreakable
          if game_objects[victim_object_index].hitpoints < character_properties.primary_damage {
            game_objects[victim_object_index].to_be_deleted = true;
          } else {
            game_objects[victim_object_index].hitpoints -= character_properties.primary_damage;
          }
        }
        return (main_loop_players, game_objects); // return early
      }
    }
  }

  // Calculate collisions with players
  for player_index in 0..main_loop_players.len() {
    if Vector2::distance(game_object.position, main_loop_players[player_index].position) < hit_radius &&
    owner_index != player_index {

      if !(game_object.players.contains(&player_index)) {
        // Apply bullet damage
        if main_loop_players[player_index].team != player.team {
          if main_loop_players[player_index].health > character_properties.primary_damage {
            main_loop_players[player_index].health -= character_properties.primary_damage;
          } else {
            main_loop_players[player_index].health = 0;
          }
          game_objects[game_object_index].players.push(player_index);
        }
        // Apply bullet healing, only if in the same team
        if main_loop_players[player_index].team == player.team {
          if main_loop_players[player_index].health < character_properties.primary_heal {
            main_loop_players[player_index].health += character_properties.primary_heal;
          } else {
            main_loop_players[player_index].health = 100;
          }
          game_objects[game_object_index].players.push(player_index);
        }
        // Apply appropriate secondary charge
        if main_loop_players[owner_index].secondary_charge < 100 - character_properties.secondary_hit_charge {
          main_loop_players[owner_index].secondary_charge += character_properties.secondary_hit_charge;
        } else {
          main_loop_players[owner_index].secondary_charge = 100;
        }
        // Destroy the bullet if it doesn't pierce.
        if !pierceing_shot {
          game_objects[game_object_index].to_be_deleted = true;
        }
      }
    }
  }
  game_objects[game_object_index].position.x += game_object.direction.x * true_delta_time as f32 * bullet_speed;
  game_objects[game_object_index].position.y += game_object.direction.y * true_delta_time as f32 * bullet_speed;
  return (main_loop_players, game_objects);
}