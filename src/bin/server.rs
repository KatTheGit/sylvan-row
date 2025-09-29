use sylvan_row::common::*;
use core::f32;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, MutexGuard};
use bincode;
use std::{thread, time::*};

const WALL_TYPES: [GameObjectType; 3] = [GameObjectType::Wall, GameObjectType::UnbreakableWall, GameObjectType::SniperWall];
const WALL_TYPES_ALL: [GameObjectType; 5] = [GameObjectType::Wall, GameObjectType::UnbreakableWall, GameObjectType::SniperWall, GameObjectType::Water1, GameObjectType::Water2];
static mut SPAWN_RED: Vector2 = Vector2 {x: 30.0 * TILE_SIZE, y: 11.0 * TILE_SIZE};
static mut SPAWN_BLUE: Vector2 = Vector2 {x: 2.0 * TILE_SIZE, y: 12.0 * TILE_SIZE};

fn main() {
  // set the gamemode (temporary)
  let selected_game_mode = GameMode::DeathMatchArena;

  // Load character properties
  let characters: HashMap<Character, CharacterProperties> = load_characters();
  println!("Loaded character properties.");

  let players: Vec<ServerPlayer> = Vec::new();
  let players = Arc::new(Mutex::new(players));
  let game_objects:Vec<GameObject> = load_map_from_file(include_str!("../../assets/maps/map_maker.map"));
  println!("Loaded game objects.");
  let game_objects = Arc::new(Mutex::new(game_objects));

  // initiate all networking sockets
  let server_address = format!("0.0.0.0:{}", SERVER_PORT);
  let socket = UdpSocket::bind(server_address.clone()).expect("Error creating listener UDP socket");
  let mut buffer = [0; 4096]; // The size of this buffer is lowkey kind of low, especially with how big the gameobject struct is.
  println!("Sockets bound.");
  println!("Listening on: {}", server_address.clone());

  let mut red_team_player_count = 0;
  let mut blue_team_player_count = 0;

  // temporary, to be dictated by gamemode
  let max_players = 100;

   // holds game information, to be displayed by client, and modified when shit happens.
  let general_gamemode_info: GameModeInfo = GameModeInfo {
    time: 0,
    rounds_won_red: 0,
    rounds_won_blue: 0,
    kills_red: 0,
    kills_blue: 0,
    death_timeout: 3.0,
  };
  let general_gamemode_info = Arc::new(Mutex::new(general_gamemode_info));
  
  // (vscode) MARK: Networking - Listen
  let listener_players = Arc::clone(&players);
  let listener_gamemode_info = Arc::clone(&general_gamemode_info);
  let listener_game_objects = Arc::clone(&game_objects);
  println!();
  std::thread::spawn(move || {
    loop {
      // recieve packet
      let (amt, src) = socket.recv_from(&mut buffer).expect(":(");
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
      let mut listener_game_objects = listener_game_objects.lock().unwrap();

      
      let mut player_found: bool = false;
      
      // iterate through players
      for player_index in 0..listener_players.len() {
        
        // THIS VALUE WILL THEN BE ASSIGNED BACK TO listener_players[player_index] !!!!
        let mut player = listener_players[player_index].clone();
        
        // use IP as identifier, check if packet from sent player correlates to our player
        // Later on we might have an account system and use that as ID. For now, IP will do
        if player.ip == src.ip().to_string() && player.port == recieved_player_info.port {
          // If this check passes, we're now running logic for the player that sent the packet.
          // This block of code handles recieving data, and then sends out a return packet.
          let time_since_last_packet = recieved_player_info.packet_interval as f64;
          // if time_since_last_packet < MAX_PACKET_INTERVAL &&
          // time_since_last_packet > MIN_PACKET_INTERVAL  {
          //   // ignore this packet since it's coming in too fast
          //   player_found = true;
          //   break;
          // }

          if recieved_player_info.character != player.character {
            player.character = recieved_player_info.character;
            player.kill(false, &GameModeInfo::new());
          }

          player.aim_direction = recieved_player_info.aim_direction.normalize();
          player.shooting = recieved_player_info.shooting_primary;
          player.shooting_secondary = recieved_player_info.shooting_secondary;
          
          let mut new_position = Vector2::new();
          let recieved_position = recieved_player_info.position;
          let movement_error_margin = 5.0;
          let mut movement_legal = true;
          let previous_position = player.position.clone();
          
          // (vscode) MARK: Dashing Legality
          // If player wants to dash and isn't dashing...
          if recieved_player_info.dashing && !player.is_dashing && !player.is_dead && recieved_player_info.movement.magnitude() != 0.0 {
            let player_dash_cooldown = characters[&player.character].dash_cooldown;
            // And we're past the cooldown...
            if player.last_dash_time.elapsed().as_secs_f32() > player_dash_cooldown {
              // reset the cooldown
              player.last_dash_time = Instant::now();
              // set dashing to true
              player.is_dashing = true;
              // set the dashing direction
              player.dash_direction = recieved_player_info.movement;

              // (vscode) MARK: Special dashes
              match player.character {
                Character::HealerGirl => {
                  player.next_shot_empowered = true;
                }
                Character::SniperWolf => {
                  // Place down a bomb
                  listener_game_objects.push(
                    GameObject {
                      object_type: GameObjectType::HernaniLandmine,
                      size: Vector2 { x: TILE_SIZE, y: TILE_SIZE },
                      position: player.position,
                      direction: Vector2::new(),
                      to_be_deleted: false,
                      owner_port: listener_players[player_index].port,
                      hitpoints: 1,
                      lifetime: characters[&Character::SniperWolf].dash_cooldown,
                      players: Vec::new(),
                      traveled_distance: 0.0,
                    }
                  );
                }
                Character::TimeQueen => {}
                Character::Dummy => {}
              }
            }
          }
          // (vscode) MARK: Dashing
          if player.is_dashing && !player.is_dead {
            (new_position, player.dashed_distance, player.is_dashing) = dashing_logic(
              player.is_dashing,
              player.dashed_distance, 
              player.dash_direction, 
              time_since_last_packet, 
              characters[&player.character].dash_speed, 
              characters[&player.character].dash_distance, 
              listener_game_objects.clone(), 
              player.position
            );
          }

          else {
            // (vscode) MARK: Movement Legality
            // Movement legality calculations
            let raw_movement = recieved_player_info.movement;
            let mut movement = Vector2::new();
            let player_movement_speed: f32 = characters[&player.character].speed;
            let mut extra_speed: f32 = 0.0;
            for buff in player.buffs.clone() {
              if buff.buff_type == BuffType::Speed {
                extra_speed += buff.value;
              }
            }

            movement.x = raw_movement.x * (player_movement_speed + extra_speed) * time_since_last_packet as f32;
            movement.y = raw_movement.y * (player_movement_speed + extra_speed) * time_since_last_packet as f32;

            // calculate current expected position based on input
            let (new_movement_raw, _): (Vector2, Vector2) = object_aware_movement(
              previous_position,
              raw_movement,
              movement,
              listener_game_objects.clone()
            );

            new_position.x = previous_position.x + new_movement_raw.x * (player_movement_speed + extra_speed) * time_since_last_packet as f32;
            new_position.y = previous_position.y + new_movement_raw.y * (player_movement_speed + extra_speed) * time_since_last_packet as f32;

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
            for (other_player_index, player) in listener_players.clone().iter().enumerate() {
              if other_player_index != player_index {
                other_players.push(OtherPlayer {
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
                  is_dead: false,
                  camera: Camera::new(),  
                  buffs: player.buffs.clone(),
                  previous_positions: match player.character {
                    Character::TimeQueen => player.previous_positions.clone(),
                    _ => Vec::new(),
                  },
                })
              }
            }

            // packet sent to player with info about themselves and other players
            let gamemode_info = listener_gamemode_info.lock().unwrap();
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
                  Character::TimeQueen => player.previous_positions.clone(),
                  _ => Vec::new(),
                },
                team: player.team,
                time_since_last_primary: player.last_shot_time.elapsed().as_secs_f32(),
                time_since_last_dash: player.last_dash_time.elapsed().as_secs_f32(),
              },
              players: other_players,
              game_objects: listener_game_objects.clone(),
              gamemode_info: gamemode_info.clone(),
              timestamp: recieved_player_info.timestamp, // pong!
            };
            drop(gamemode_info);
            listener_players[player_index].had_illegal_position = false;
            
            let mut player_ip = player.ip.clone();
            let split_player_ip: Vec<&str> = player_ip.split(":").collect();
            player_ip = split_player_ip[0].to_string();
            player_ip = format!("{}:{}", player_ip, player.true_port);
            // println!("PLAYER IP: {}", player_ip);
            // println!("PACKET: {:?}", server_packet);
            let serialized: Vec<u8> = bincode::serialize(&server_packet).expect("Failed to serialize message (this should never happen)");
            socket.send_to(&serialized, player_ip).expect("Failed to send packet to client.");
            // player.had_illegal_position = false; // reset since we corrected the error.
         }

          // exit loop, and inform rest of program not to proceed with appending a new player.
          player_found = true;
          // println!("{:?}", player.position.clone());
          listener_players[player_index] = player;
          break
        }
      }
      drop(listener_game_objects);

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
        unsafe {
          listener_players.push(ServerPlayer {
            ip: src.ip().to_string(),
            port: recieved_player_info.port,
            true_port: src.port(),
            team,
            health: 100,
            position: match team {
              Team::Blue => SPAWN_BLUE,
              Team::Red  => SPAWN_RED,
            },
            move_direction:       Vector2::new(),
            aim_direction:        Vector2::new(),
            shooting:             false,
            shooting_secondary:   false,
            secondary_cast_time:  Instant::now(),
            secondary_charge:     100,
            had_illegal_position: false,
            character:            recieved_player_info.character,
            last_shot_time:       Instant::now(),
            is_dashing:           false,
            last_dash_time:       Instant::now(),
            dashed_distance:      0.0,
            dash_direction:       Vector2::new(),
            previous_positions:   Vec::new(),
            is_dead:              false,
            death_timer_start:    Instant::now(),
            next_shot_empowered:  false,
            buffs:                Vec::new(),
            last_packet_time: Instant::now(),
          });
        }
        println!("Player connected: {}: {} - {}", src.ip().to_string(), src.port().to_string(), recieved_player_info.port);
      }

      // Occasionally check for goners
      for player_index in 0..listener_players.len() {
        if (listener_players[player_index].last_packet_time.elapsed().as_secs_f32() > 10.0)
        && (listener_players[player_index].character != Character::Dummy                  ) {
          println!("Player forecefully disconnected: {}", listener_players[player_index].ip);
          listener_players.remove(player_index);
          break;
        }
      }

      drop(listener_players);
    }
  });
  
  // (vscode) MARK: Server Loop Initiate

  let mut server_counter: Instant = Instant::now();
  let mut delta_time: f64 = server_counter.elapsed().as_secs_f64();
  // Server logic frequency in Hertz. Doesn't need to be much higher than 120.
  // Higher frequency = higher precission with computation trade-off
  let desired_delta_time: f64 = 1.0 / 1000.0; // This needs to be limited otherwise it hogs the mutexess

  let main_game_objects = Arc::clone(&game_objects);

  // for once-per-second operations, called ticks
  let mut tick_counter = Instant::now();

  // for once-per-decisecond operations.
  let mut tenth_tick_counter = Instant::now();
  
  let characters = load_characters();
  let main_loop_players = Arc::clone(&players);
  
  // Used for game time counter. Can be reset when going into new rounds, for example...
  let game_start_time: Instant = Instant::now();

  
  // part of dummy summoning
  // set to TRUE in release server, so dummy doesn't get spawned
  let mut dummy_summoned: bool = false;
  
  // (vscode) MARK: Server Loop
  let main_gamemode_info = Arc::clone(&general_gamemode_info);
  loop {
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

    // update gamemode info
    if tick {
      // update game clock
      let mut gamemode_info = main_gamemode_info.lock().unwrap();
      gamemode_info.time = game_start_time.elapsed().as_secs() as u16;
      drop(gamemode_info);
    }
  
    let mut main_loop_players = main_loop_players.lock().unwrap();

    // (vscode) MARK: Gamemode
    if tick {
      let mut gamemode_info = main_gamemode_info.lock().unwrap();
      match selected_game_mode {
        GameMode::DeathMatchArena => {
          let mut round_restart = false;
          if gamemode_info.kills_blue >= 5 {
            gamemode_info.rounds_won_blue += 1;
            round_restart = true;
          }
          if gamemode_info.kills_red >= 5 {
            gamemode_info.rounds_won_red += 1;
            round_restart = true;
          }
          if round_restart {
            for player_index in 0..main_loop_players.len() {
              *gamemode_info = main_loop_players[player_index].kill(false, &gamemode_info.clone());
              gamemode_info.kills_blue = 0;
              gamemode_info.kills_red  = 0;
            }
            let mut reset_game_objects = main_game_objects.lock().unwrap();
            *reset_game_objects = load_map_from_file(include_str!("../../assets/maps/map_maker.map"));
            drop(reset_game_objects);
          }
        }

        _ => {}
      }
      drop(gamemode_info);
    }

    // Summon a dummy for testing
    if !dummy_summoned {
      dummy_summoned = true;
      main_loop_players.push(
        ServerPlayer {
          ip: String::from("hello"),
          port: 12,
          true_port: 12,
          team: Team::Red,
          character: Character::Dummy,
          health: 0,
          position: Vector2 { x: 10.0, y: 10.0 },
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
          next_shot_empowered: false,
          buffs: Vec::new(),
          last_packet_time: Instant::now(),
        }
      );
    }
    
    // (vscode) MARK: Player logic
    for player_index in 0..main_loop_players.len() {

      let mut gamemode_info = main_gamemode_info.lock().unwrap();

      let shooting = main_loop_players[player_index].shooting;
      let shooting_secondary = main_loop_players[player_index].shooting_secondary;
      let last_shot_time = main_loop_players[player_index].last_shot_time;
      let secondary_charge = main_loop_players[player_index].secondary_charge;
      let player_info = main_loop_players[player_index].clone();
      let character: CharacterProperties = characters[&main_loop_players[player_index].character].clone();

      // println!("{:?}", main_loop_players[player_index].character);

      // MARK: Handle death

      // if this player is at health 0
      if main_loop_players[player_index].health == 0 {
        *gamemode_info = main_loop_players[player_index].kill(true, &gamemode_info.clone());
      }
      // If the death timer is over, unkill them
      if (main_loop_players[player_index].is_dead) && (main_loop_players[player_index].death_timer_start.elapsed().as_secs_f32() > gamemode_info.death_timeout) {
        main_loop_players[player_index].is_dead = false;
      }

      // IGNORE ANYTHING BELOW IF PLAYER HAS DIED
      if main_loop_players[player_index].is_dead {
        continue;
      }

      drop(gamemode_info);

      // (vscode) MARK: Passives & Other
      // Handling of passive abilities and anything else that may need to be run all the time.

      // Reduce buff durations according to time passed, and remove buffs that ended.
      let mut buffs_to_keep: Vec<Buff> = Vec::new();
      for buff_index in 0..main_loop_players[player_index].buffs.len() {
        main_loop_players[player_index].buffs[buff_index].duration -= true_delta_time as f32;
        if main_loop_players[player_index].buffs[buff_index].duration > 0.0 {
          buffs_to_keep.push(main_loop_players[player_index].buffs[buff_index].clone());
        }
      }
      main_loop_players[player_index].buffs = buffs_to_keep;

      // Handling of time queen flashsback ability - keep a buffer of positions for the flashback
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

      // increase secondary charge passively
      if tick {
        let charge_amount = characters[&main_loop_players[player_index].character].secondary_passive_charge;
        main_loop_players[player_index].add_charge(charge_amount);
      }

      // Get stuck player out of walls
      let unstucker_game_objects = main_game_objects.lock().unwrap();
      for game_object_index in 0..unstucker_game_objects.len() {
        
        if WALL_TYPES_ALL.contains(&unstucker_game_objects[game_object_index].object_type) {
          let difference: Vector2 = Vector2::difference(unstucker_game_objects[game_object_index].position, main_loop_players[player_index].position);
          if f32::abs(difference.x) < TILE_SIZE && f32::abs(difference.y) < TILE_SIZE {
            // push out the necessary amount
            main_loop_players[player_index].position.x += (TILE_SIZE + 0.1 )* difference.normalize().x;
            main_loop_players[player_index].position.y += (TILE_SIZE + 0.1 )* difference.normalize().y;
            main_loop_players[player_index].had_illegal_position = true;
            break;
          }
        }
      }
      drop(unstucker_game_objects);

      // (vscode) MARK: Primaries
      // If someone is shooting, spawn a bullet according to their character.
      let mut cooldown: f32 = character.primary_cooldown;
      if main_loop_players[player_index].character == Character::TimeQueen {
        cooldown -= cooldown * ((secondary_charge as f32 / 100.0) * 0.10)
      }

      for buff in main_loop_players[player_index].buffs.clone() {
        if buff.buff_type == BuffType::FireRate || buff.buff_type == BuffType::HealerFireRate {
          cooldown -= cooldown * buff.value;
        }
      }
      // not sure why this was here, nothing wrong with holding down both buttons
      //                      \/
      if shooting /*&& !shooting_secondary*/  && last_shot_time.elapsed().as_secs_f32() > cooldown && main_loop_players[player_index].aim_direction.magnitude() != 0.0 {
        // main_loop_players[player_index].buffs.push(Buff { value: 0.1, duration: 2.2, buff_type: BuffType::FireRate });
        // main_loop_players[player_index].buffs.push(Buff { value: 20.0, duration: 2.2, buff_type: BuffType::Speed });
        main_loop_players[player_index].last_shot_time = Instant::now();
        let mut game_objects = main_game_objects.lock().unwrap();
        // Do primary shooting logic
        match main_loop_players[player_index].character {
          Character::SniperWolf => {
            game_objects.push(GameObject {
              object_type: GameObjectType::SniperWolfBullet,
              size: Vector2 { x: TILE_SIZE * 1.0 * (10.0/4.0), y: TILE_SIZE * 1.0 },
              position: main_loop_players[player_index].position,
              direction: main_loop_players[player_index].aim_direction,
              to_be_deleted: false,
              hitpoints: 0,
              owner_port: main_loop_players[player_index].port,
              lifetime: character.primary_range / character.primary_shot_speed,
              players: Vec::new(),
              traveled_distance: 0.0,
            });
          }
          Character::HealerGirl => {
            game_objects.push(GameObject {
              object_type: match main_loop_players[player_index].next_shot_empowered {
                true  => {GameObjectType::HealerGirlBulletEmpowered},
                false => {GameObjectType::HealerGirlBullet},
              },
              size: Vector2 { x: TILE_SIZE*2.0, y: TILE_SIZE*2.0 },
              position: main_loop_players[player_index].position,
              direction: main_loop_players[player_index].aim_direction,
              to_be_deleted: false,
              hitpoints: 0,
              owner_port: main_loop_players[player_index].port,
              lifetime: character.primary_range / character.primary_shot_speed,
              players: Vec::new(),
              traveled_distance: 0.0,
            });
            if main_loop_players[player_index].next_shot_empowered {
              main_loop_players[player_index].next_shot_empowered = false;
            }
          }
          Character::TimeQueen => {
            game_objects.push(GameObject {
              object_type: GameObjectType::TimeQueenSword,
              size: Vector2 { x: TILE_SIZE*3.0, y: TILE_SIZE*3.0 },
              position: Vector2 {
                x: main_loop_players[player_index].position.x + main_loop_players[player_index].aim_direction.x * 5.0 ,
                y: main_loop_players[player_index].position.y + main_loop_players[player_index].aim_direction.y * 5.0 },
              direction: main_loop_players[player_index].aim_direction,
              to_be_deleted: false,
              hitpoints: 0,
              owner_port: main_loop_players[player_index].port,
              lifetime: character.primary_range / character.primary_shot_speed,
              players: Vec::new(),
              traveled_distance: 0.0,
            });
          }
          Character::Dummy => {
            game_objects.push(GameObject {
              object_type: GameObjectType::SniperWolfBullet,
              size: Vector2 { x: TILE_SIZE, y: TILE_SIZE },
              position: main_loop_players[player_index].position,
              direction: main_loop_players[player_index].aim_direction,
              to_be_deleted: false,
              hitpoints: 0,
              owner_port: main_loop_players[player_index].port,
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
              size: Vector2 { x: 60.0, y: 60.0 },
              position: player_info.position,
              direction: Vector2::new(),
              to_be_deleted: false,
              owner_port: main_loop_players[player_index].port,
              hitpoints: 0,
              lifetime: 5.0,
              players: vec![],
              traveled_distance: 0.0,
            });
            drop(game_objects);
            secondary_used_successfully = true;
          },
          // Place walls
          Character::SniperWolf => {
            // Place down a wall at a position rounded to TILE_SIZE, unless a wall is alredy there.
            let wall_place_distance = character.secondary_range;
            let mut desired_placement_position: Vector2 = player_info.position;
            // round to closest 10
            desired_placement_position.x = ((((desired_placement_position.x + player_info.aim_direction.x * wall_place_distance + TILE_SIZE/2.0) / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;
            desired_placement_position.y = ((((desired_placement_position.y + player_info.aim_direction.y * wall_place_distance + TILE_SIZE/2.0) / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;

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
                size: Vector2 { x: TILE_SIZE, y: TILE_SIZE*2.0 },
                position: desired_placement_position,
                direction: Vector2::new(),
                to_be_deleted: false,
                owner_port: main_loop_players[player_index].port,
                hitpoints: 20,
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
              main_loop_players[player_index].heal(10, characters.clone());
            }
          },

          Character::Dummy => {}
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

        // WOLF primary
        GameObjectType::SniperWolfBullet => {
          (main_loop_players, *game_objects, _) = apply_simple_bullet_logic(main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, false);
        }
        // WOLF dash special
        GameObjectType::HernaniLandmine => {
          // if the landmine has existed for long enough...
          if game_objects[game_object_index].lifetime < (characters[&Character::SniperWolf].dash_cooldown - 0.5) {
            for player_index in 0..main_loop_players.len() {
              // if not on same team
              if main_loop_players[player_index].team != main_loop_players[index_by_port(game_objects[game_object_index].owner_port,main_loop_players.clone())].team {
                // if within range
                if Vector2::distance(game_objects[game_object_index].position, main_loop_players[player_index].position)
                < (game_objects[game_object_index].size.x / 2.0) {
                  main_loop_players[player_index].damage(characters[&Character::SniperWolf].primary_damage_2, characters.clone());
                  game_objects[game_object_index].to_be_deleted = true;
                  break;
                }
              }
            }
          }
        }

        // HEALER GIRL primary
        GameObjectType::HealerGirlBullet => {
          let hit: bool;
          (main_loop_players, *game_objects, hit) = apply_simple_bullet_logic(main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, true);
          
          // Restore nearby ally health
          if hit {
            for player_index in 0..main_loop_players.len() {
              let range: f32 = characters[&Character::HealerGirl].primary_range;
              if Vector2::distance(
                main_loop_players[player_index].position,
                main_loop_players[index_by_port(game_objects[game_object_index].owner_port,main_loop_players.clone())].position
              ) < range &&
                main_loop_players[player_index].team == main_loop_players[index_by_port(game_objects[game_object_index].owner_port,main_loop_players.clone())].team {
                // Anyone within range
                if player_index == index_by_port(game_objects[game_object_index].owner_port,main_loop_players.clone()) {
                  // if self, heal less
                  let heal_self: u8 = characters[&Character::HealerGirl].primary_lifesteal;
                  main_loop_players[player_index].heal(heal_self, characters.clone());
                }
                else {
                  // otherwise, apply normal heal
                  let heal: u8 = characters[&Character::HealerGirl].primary_heal_2;
                  main_loop_players[player_index].heal(heal, characters.clone());
                }
                  // restore dash charge (0.2s)
                // main_loop_players[game_objects[game_object_index].owner_index].last_dash_time -= Duration::from_millis(200);
              }
            }
          }
        }
        // HEALER GIRL primary, EMPOWERED
        GameObjectType::HealerGirlBulletEmpowered => {
          let hit: bool;
          (main_loop_players, *game_objects, hit) = apply_simple_bullet_logic_extra(
            main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, true,
            characters[&Character::HealerGirl].primary_damage_2, 255);
          if hit {
            // restore dash charge (0.5s)
            let owner_index = index_by_port(game_objects[game_object_index].owner_port, main_loop_players.clone());
            main_loop_players[owner_index].last_dash_time -= Duration::from_millis(450);
          }
        }

        // HEALER GIRL secondary
        GameObjectType::HealerAura => {
          // game_objects[game_object_index].position = main_loop_players[game_objects[game_object_index].owner_index].position;
          // every second apply heal
          for player_index in 0..main_loop_players.len() {
            // if on same team
            if main_loop_players[player_index].team == main_loop_players[index_by_port(game_objects[game_object_index].owner_port,main_loop_players.clone())].team {
              // if within range
              if Vector2::distance(game_objects[game_object_index].position, main_loop_players[index_by_port(game_objects[game_object_index].owner_port,main_loop_players.clone())].position)
              < (game_objects[game_object_index].size.x / 2.0) {
                // heal up
                if tick {
                  let heal_amount = characters[&main_loop_players[player_index].character].secondary_heal;
                  main_loop_players[player_index].heal(heal_amount, characters.clone());
                }
                // provide fire rate buff, if not present
                let mut buff_found = false;
                for buff_index in 0..main_loop_players[player_index].buffs.len() {
                  if main_loop_players[player_index].buffs[buff_index].buff_type == BuffType::HealerFireRate {
                    buff_found = true;
                    break; // exit early
                  }
                }
                if !buff_found {                                 //        2  0%  <- find way to source this from properties file sometime idk
                  main_loop_players[player_index].buffs.push(Buff { value: 0.2, duration: 0.1, buff_type: BuffType::HealerFireRate });
                }
              } else {
                for buff_index in 0..main_loop_players[player_index].buffs.len() {
                  if main_loop_players[player_index].buffs[buff_index].buff_type == BuffType::HealerFireRate {
                    main_loop_players[player_index].buffs.remove(buff_index);
                    break; // exit early
                  }
                }
              }
            }
          }
        }

        // QUEEN primary
        GameObjectType::TimeQueenSword => {
          (main_loop_players, *game_objects, _) = apply_simple_bullet_logic(main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, true);
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
    drop(main_loop_players);
    // println!("Server Hz: {}", 1.0 / delta_time);
    delta_time = server_counter.elapsed().as_secs_f64();
    if delta_time < desired_delta_time {
      thread::sleep(Duration::from_secs_f64(desired_delta_time - delta_time));
    }
  }
}

// (vscode) MARK: Functions, Structs

/// Information held by server about players.
/// 
/// This struct can be as hefty as we want, it stays here, doesn't get sent through network.
#[derive(Debug, Clone)]
struct ServerPlayer {
  ip:                   String,
  /// also used as an identifier
  port:                 u16,
  true_port:            u16,
  team:                 Team,
  character:            Character,
  health:               u8,
  position:             Vector2,
  shooting:             bool,
  /// To calculate cooldowns
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
  /// bro forgor to live
  is_dead:              bool,
  death_timer_start:    Instant,
  /// Must be disabled after check.
  next_shot_empowered:  bool,
  /// list of buffs
  buffs:                Vec<Buff>,
  last_packet_time:     Instant,
}
impl ServerPlayer {
  fn damage(&mut self, mut dmg: u8, characters: HashMap<Character, CharacterProperties>) -> () {
    if self.is_dead {
      return;
    }
    // Special per-character handling
    match self.character {
      Character::HealerGirl => {
        self.buffs.push(
          Buff { value: 6.0, duration: 0.5, buff_type: BuffType::Speed }
        );
      }
      _ => {}
    }

    // apply dashing damage reduction or increase.
    if self.is_dashing {
      dmg = (dmg as f32 * characters[&self.character].dash_damage_multiplier) as u8;
    }

    if self.health < dmg {
      self.health = 0
    } else {
      self.health -= dmg;
    }
  }
  fn heal(&mut self, heal: u8, characters: HashMap<Character, CharacterProperties>) -> () {
    if self.is_dead {
      return;
    }

    // this edge case crashes the server
    if self.health as i16 + heal as i16 > characters[&self.character].health as i16 {
      self.health = characters[&self.character].health;
    } else {
      self.health += heal;
    }
  }
  fn add_charge(&mut self, charge: u8) -> () {
    if self.is_dead {
      return;
    }

    if self.secondary_charge + charge > 100 {
      self.secondary_charge = 100;
    } else {
      self.secondary_charge += charge;
    }
  }
  fn kill(&mut self, credit_other_team: bool, gamemode_info: &GameModeInfo) -> GameModeInfo{
    let mut updated_gamemode_info: GameModeInfo = gamemode_info.clone();
    // remove all previous positions
    self.previous_positions = Vec::new();
    self.secondary_charge = 0;
    // set them back to 100
    self.health = 100;
    // set dead flag for other handling
    self.is_dead = true;
    // mark when they died so we know when to respawn them
    self.death_timer_start = Instant::now();
    // send them to their respective spawn
    if self.team == Team::Blue {
      unsafe {
        self.position = SPAWN_BLUE;
        // println!("Sending {} to blue spawn", self.ip);
        // Give a kill to the red team
        if credit_other_team {
          updated_gamemode_info.kills_red += 1;
        }
      }
    }
      else {
        unsafe {
          self.position = SPAWN_RED;
          // println!("Sending {} to red team spawn", self.ip);
          // Give a kill to the blue team
          if credit_other_team {
            updated_gamemode_info.kills_blue += 1;
          }
        }
      }
    return updated_gamemode_info;
  }
}
/// Applies modifications to players and game objects as a result of
/// bullet behaviour this frame, for a specific bullet. This dumbed
/// down version is only to be used for character primaries.
/// 
/// This is a wrapper for `apply_simple_bullet_logic_extra`, with just
/// the simplest parameters.
fn apply_simple_bullet_logic(
  main_loop_players:     MutexGuard<Vec<ServerPlayer>>,
  characters:            HashMap<Character, CharacterProperties>,
  game_objects:          Vec<GameObject>,
  game_object_index:     usize,
  true_delta_time:       f64,
  pierceing_shot:        bool,
) -> (MutexGuard<Vec<ServerPlayer>>, Vec<GameObject>, bool) {
  return apply_simple_bullet_logic_extra(main_loop_players, characters, game_objects, game_object_index, true_delta_time, pierceing_shot, 255, 255);
}

/// Applies modifications to players and game objects as a result of
/// bullet behaviour this frame, for a specific bullet.
/// 
/// Set `special_samage` to `255` to use default character damage number.
/// Same with `special_healing`. Setting it to 0 will nullify it.
fn apply_simple_bullet_logic_extra(
  mut main_loop_players: MutexGuard<Vec<ServerPlayer>>,
  characters:            HashMap<Character, CharacterProperties>,
  mut game_objects:      Vec<GameObject>,
  game_object_index:     usize,
  true_delta_time:       f64,
  pierceing_shot:        bool,
  special_damage:        u8,
  special_healing:       u8,
) -> (MutexGuard<Vec<ServerPlayer>>, Vec<GameObject>, bool) {
  let game_object = game_objects[game_object_index].clone();
  let owner_port = game_object.owner_port;
  let player = main_loop_players[index_by_port(owner_port, main_loop_players.clone())].clone();
  let character = player.character;
  let character_properties = characters[&character].clone();
  let hit_radius: f32 = character_properties.primary_hit_radius;
  let wall_hit_radius: f32 = character_properties.primary_wall_hit_radius;
  let bullet_speed: f32 = character_properties.primary_shot_speed;
  
  // Set special values
  let damage: u8;
  if special_damage == 255 { damage = character_properties.primary_damage; }
  else                     { damage = special_damage; }
  
  let healing: u8;
  if special_damage == 255 { healing = character_properties.primary_heal; }
  else                     { healing = special_healing; }
  
  // Temporary. To be improved later.
  let wall_damage = (damage as f32 * character_properties.wall_damage_multiplier) as u8;
  
  // Calculate collisions with walls
  for victim_object_index in 0..game_objects.len() {
    // if it's a wall
    if WALL_TYPES.contains(&game_objects[victim_object_index].object_type) {
      // if it's colliding
      if Vector2::distance(game_object.position, game_objects[victim_object_index].position) < (5.0 + wall_hit_radius) {
        // delete the bullet
        game_objects[game_object_index].to_be_deleted = true;
        // damage the wall if it's not unbreakable
        if game_objects[victim_object_index].object_type != GameObjectType::UnbreakableWall {

          if game_objects[victim_object_index].hitpoints < wall_damage {
            game_objects[victim_object_index].to_be_deleted = true;
          } else {
            game_objects[victim_object_index].hitpoints -= wall_damage;
          }
        }
        return (main_loop_players, game_objects, false); // return early
      }
    }
  }

  // Calculate collisions with players
  let mut hit: bool = false; // whether we've hit a player
  for player_index in 0..main_loop_players.len() {
    if main_loop_players[player_index].is_dead {
      continue; // skip dead player
    }
    // If we hit a bloke
    if Vector2::distance(game_object.position, main_loop_players[player_index].position) < hit_radius &&
    owner_port != main_loop_players[player_index].port {
      // And if we didn't hit this bloke before
      if !(game_object.players.contains(&player_index)) {
        // Apply bullet damage
        if main_loop_players[player_index].team != player.team {
          // Confirmed hit.
          hit = true;
          main_loop_players[player_index].damage(damage, characters.clone());
          game_objects[game_object_index].players.push(player_index);
          // Destroy the bullet if it doesn't pierce.
          if !pierceing_shot {
            game_objects[game_object_index].to_be_deleted = true;
          }
        }
        // Apply bullet healing, only if in the same team
        if main_loop_players[player_index].team == player.team && healing > 0 {
          main_loop_players[player_index].heal(healing, characters.clone());
          game_objects[game_object_index].players.push(player_index);
          // Destroy the bullet if it doesn't pierce.
          if !pierceing_shot {
            game_objects[game_object_index].to_be_deleted = true;
          }
        }
        // Apply appropriate secondary charge
        let owner_index = index_by_port(owner_port, main_loop_players.clone());
        main_loop_players[owner_index].add_charge(character_properties.secondary_hit_charge);
      }
    }
  }
  game_objects[game_object_index].position.x += game_object.direction.x * true_delta_time as f32 * bullet_speed;
  game_objects[game_object_index].position.y += game_object.direction.y * true_delta_time as f32 * bullet_speed;
  return (main_loop_players, game_objects, hit);
}

fn index_by_port(port: u16, players: Vec<ServerPlayer>) -> usize{
  for player_index in 0..players.len() {
    if players[player_index].port == port {
      return player_index;
    }
  }
  panic!("index_by_port function error - data race condition, mayhaps?");
}