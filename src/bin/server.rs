use gilrs::ff::DistanceModel;
use sylvan_row::common::*;
use core::f32;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, MutexGuard};
use bincode;
use std::{thread, time::*, vec};


static mut SPAWN_RED: Vector2 = Vector2 {x: 31.0 * TILE_SIZE, y: 14.0 * TILE_SIZE};
static mut SPAWN_BLUE: Vector2 = Vector2 {x: 3.0 * TILE_SIZE, y: 14.0 * TILE_SIZE};

fn main() {
  // not exactly sure if `max_players` is really needed, but whatevs
  game_server_instance(400, GameMode::DeathMatch);
}
fn game_server_instance(max_players: usize, selected_gamemode: GameMode) -> () {
  // set the gamemode (temporary)

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

  // temporary, to be dictated by gamemode
  let max_players = max_players;

  // holds game information, to be displayed by client, and modified when shit happens.
  let mut general_gamemode_info: GameModeInfo = GameModeInfo::new();
  general_gamemode_info.death_timeout = 1.0;
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
            //player.kill(false, &GameModeInfo::new());
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
                      lifetime: characters[&Character::Hernani].dash_cooldown,
                      players: Vec::new(),
                      traveled_distance: 0.0,
                    }
                  );
                }
                Character::Cynewynn => {}
                Character::Wiro => {
                  if player.stacks == 1{
                    // Spawn the projectile that applies the mid-dash logic.
                    listener_game_objects.push(
                      GameObject {
                        object_type: GameObjectType::WiroDashProjectile,
                        size: Vector2 { x: TILE_SIZE, y: TILE_SIZE },
                        position: player.position,
                        direction: Vector2::new(),
                        to_be_deleted: false,
                        owner_port: listener_players[player_index].port,
                        hitpoints: 0,
                        lifetime: characters[&Character::Hernani].dash_distance / characters[&Character::Hernani].dash_speed + 0.25, // give it a "grace" period because I'm bored
                        players: Vec::new(),
                        traveled_distance: 0.0,
                      }
                    );
                  }
                  player.stacks = 0;
                }
                Character::Elizabeth => {
                  // Change the type of all her current static projectiles to the type
                  // that follows her.
                  for index in 0..listener_game_objects.len() {
                    if listener_game_objects[index].object_type == GameObjectType::ElizabethProjectileGround
                    && listener_game_objects[index].owner_port == player.port {
                      listener_game_objects[index].to_be_deleted = true;
                      let object_clone = listener_game_objects[index].clone();
                      listener_game_objects.push(
                        GameObject {
                          object_type: GameObjectType::ElizabethProjectileGroundRecalled,
                          size: Vector2 { x: TILE_SIZE, y: TILE_SIZE },
                          position: object_clone.position,
                          direction: Vector2::new(),
                          to_be_deleted: false,
                          owner_port: object_clone.owner_port,
                          hitpoints: 0,
                          lifetime: 15.0,
                          players: Vec::new(),
                          traveled_distance: 0.0,
                        }
                      );
                    }
                  }
                }
                Character::Temerity => {
                  player.is_dashing = false;
                }
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
            movement_legal = false; // force a correction
          }

          else {
            // (vscode) MARK: Movement Legality
            // Movement legality calculations
            let raw_movement = recieved_player_info.movement;
            let mut movement = Vector2::new();
            let player_movement_speed: f32 = characters[&player.character].speed;
            let mut extra_speed: f32 = 0.0;
            for buff in player.buffs.clone() {
              if vec![BuffType::Speed, BuffType::WiroSpeed].contains(&buff.buff_type) {
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
                    Character::Cynewynn => player.previous_positions.clone(),
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
                  Character::Cynewynn => player.previous_positions.clone(),
                  _ => Vec::new(),
                },
                team: player.team,
                time_since_last_primary: player.last_shot_time.elapsed().as_secs_f32(),
                time_since_last_dash: player.last_dash_time.elapsed().as_secs_f32(),
                time_since_last_secondary: player.secondary_cast_time.elapsed().as_secs_f32(),
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
      let mut gamemode_info = listener_gamemode_info.lock().unwrap();
      if !player_found && (gamemode_info.total_blue + gamemode_info.total_red < max_players as u8) {
        // decide the player's team (alternate for each player)
        let mut team: Team = Team::Blue;

        if gamemode_info.total_blue > gamemode_info.total_red {
          team = Team::Red;
          gamemode_info.total_red += 1;
          gamemode_info.alive_red += 1;
        } else {
          gamemode_info.total_blue += 1;
          gamemode_info.alive_blue += 1;
        }
        drop(gamemode_info);
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
            stacks:               0,
            buffs:                Vec::new(),
            last_packet_time:     Instant::now(),
            is_wallriding:        false,
          });
        }
        println!("Player connected: {}: {} - {}", src.ip().to_string(), src.port().to_string(), recieved_player_info.port);
      }

      drop(listener_players);
    }
  });
  
  // (vscode) MARK: Server Loop Initiate

  let mut server_counter: Instant = Instant::now();
  let mut delta_time: f64 = server_counter.elapsed().as_secs_f64();
  // Server logic frequency in Hertz. Doesn't need to be much higher than 120.
  // Higher frequency = higher precission with computation trade-off
  let desired_delta_time: f64 = 1.0 / 120.0;

  let main_game_objects = Arc::clone(&game_objects);

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
  let game_start_time: Instant = Instant::now();

  
  // part of dummy summoning
  // set to TRUE in release server, so dummy doesn't get spawned
  let mut dummy_summoned: bool = !DEBUG;

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
      // println!("{}", 1.0/delta_time);
    }

    let mut main_loop_players = main_loop_players.lock().unwrap();

    // (vscode) MARK: Gamemode
    if tick {
      let mut gamemode_info = main_gamemode_info.lock().unwrap();
      // println!("{:?}", gamemode_info);
      match selected_gamemode {
        GameMode::Arena => {
          let mut round_restart = false;
          if gamemode_info.alive_blue == 0
          && (gamemode_info.total_blue > 1 && gamemode_info.total_red > 1) {
            gamemode_info.rounds_won_red += 1;
            round_restart = true;
          }
          if gamemode_info.alive_red == 0
          && (gamemode_info.total_blue > 1 && gamemode_info.total_red > 1) {
            gamemode_info.rounds_won_blue += 1;
            round_restart = true;
          }
          if round_restart {
            for player_index in 0..main_loop_players.len() {
              *gamemode_info = main_loop_players[player_index].kill(false, &gamemode_info.clone());
              gamemode_info.alive_blue = gamemode_info.total_blue;
              gamemode_info.alive_red  = gamemode_info.total_red;
              main_loop_players[player_index].is_dead = false;
            }
            let mut reset_game_objects = main_game_objects.lock().unwrap();
            *reset_game_objects = load_map_from_file(include_str!("../../assets/maps/map_maker.map"));
            drop(reset_game_objects);
          }
        }
        GameMode::DeathMatch => {
          // don't care
          if gamemode_info.alive_blue == 0 {
            gamemode_info.alive_blue = 200;
          }
          if gamemode_info.alive_red == 0 {
            gamemode_info.alive_red = 200;
          }
        }
      }
      // Occasionally check for goners
      for player_index in 0..main_loop_players.len() {
        if (main_loop_players[player_index].last_packet_time.elapsed().as_secs_f32() > 3.0)
        && (main_loop_players[player_index].character != Character::Dummy                  ) {
          println!("Player forcefully disconnected: {}", main_loop_players[player_index].ip);
          if main_loop_players[player_index].team == Team::Red {
            gamemode_info.total_red -=1;
            if !main_loop_players[player_index].is_dead {
              gamemode_info.alive_red -=1;
            }
          } else {
            gamemode_info.total_blue -=1;
            if !main_loop_players[player_index].is_dead {
              gamemode_info.alive_blue -=1;
            }
          }
          main_loop_players.remove(player_index);
          break;
        }
      }
      drop(gamemode_info);
    }

    // Summon a dummy for testing
    if !dummy_summoned {
      dummy_summoned = true;
      let mut gamemode_info = main_gamemode_info.lock().unwrap();
      gamemode_info.alive_red += 1;
      gamemode_info.total_red += 1;
      drop(gamemode_info);

      main_loop_players.push(
        ServerPlayer {
          ip: String::from("hello"),
          port: 12,
          true_port: 12,
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
          is_wallriding: false,
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
      if main_loop_players[player_index].health == 0 && !main_loop_players[player_index].is_dead {
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
      if main_loop_players[player_index].character == Character::Cynewynn {
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
      let mut unstucker_game_objects = main_game_objects.lock().unwrap();
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
      // Delete extra Elizabeth ground daggers
      if main_loop_players[player_index].character == Character::Elizabeth {

        let mut objects_to_consider: Vec<usize> = Vec::new();
        for game_object_index in 0..unstucker_game_objects.len() {
          if unstucker_game_objects[game_object_index].object_type == GameObjectType::ElizabethProjectileGround {
            if index_by_port(unstucker_game_objects[game_object_index].owner_port, main_loop_players.clone())
            == player_index {
              objects_to_consider.push(game_object_index);
            }
          }
        }
        if objects_to_consider.len() > 2 {
          // selection sort is by far the best sorting algorithm
          let mut lowest_val: f32 = f32::MAX;
          let mut lowest_index: usize = 0;
          for object_to_consider in objects_to_consider {
            if unstucker_game_objects[object_to_consider].lifetime < lowest_val {
              lowest_val = unstucker_game_objects[object_to_consider].lifetime;
              lowest_index = object_to_consider;
            }
          }
          unstucker_game_objects[lowest_index].to_be_deleted = true;
        }
      }

      // Apply wiro's speed boost
      if main_loop_players[player_index].character == Character::Wiro {
        if !main_loop_players[player_index].shooting_secondary
        || main_loop_players[player_index].secondary_cast_time.elapsed().as_secs_f32() < character.secondary_cooldown {
          for victim_index in 0..main_loop_players.len() {
            if main_loop_players[victim_index].team == main_loop_players[player_index].team
            && Vector2::distance(main_loop_players[victim_index].position, main_loop_players[player_index].position) < character.passive_range{
              // provide speed buff, if not present
              let mut buff_found = false;
              for buff_index in 0..main_loop_players[victim_index].buffs.len() {
                if main_loop_players[victim_index].buffs[buff_index].buff_type == BuffType::WiroSpeed {
                  buff_found = true;
                  main_loop_players[victim_index].buffs.remove(buff_index);
                  main_loop_players[victim_index].buffs.push(Buff { value: 5.0, duration: 0.25, buff_type: BuffType::WiroSpeed });
                  break; // exit early
                }
              }
              if !buff_found {
                main_loop_players[victim_index].buffs.push(Buff { value: 5.0, duration: 0.25, buff_type: BuffType::WiroSpeed });
              }
            }
          }
        }
      }

      drop(unstucker_game_objects);


      // (vscode) MARK: Primaries
      // If someone is shooting, spawn a bullet according to their character.
      let mut cooldown: f32 = character.primary_cooldown;
      if main_loop_players[player_index].character == Character::Cynewynn {
        cooldown -= cooldown * ((secondary_charge as f32 / 100.0) * 0.10)
      }

      for buff in main_loop_players[player_index].buffs.clone() {
        if buff.buff_type == BuffType::FireRate || buff.buff_type == BuffType::RaphaelleFireRate {
          cooldown -= cooldown * buff.value;
        }
      }
      // not sure why this was here, nothing wrong with holding down both buttons
      //                      \/
      if shooting /*&& !shooting_secondary*/  && last_shot_time.elapsed().as_secs_f32() > cooldown && main_loop_players[player_index].aim_direction.magnitude() != 0.0 {
        // main_loop_players[player_index].buffs.push(Buff { value: 0.1, duration: 2.2, buff_type: BuffType::FireRate });
        // main_loop_players[player_index].buffs.push(Buff { value: 20.0, duration: 2.2, buff_type: BuffType::Speed });
        let mut shot_successful: bool = false;
        let mut game_objects = main_game_objects.lock().unwrap();
        // Do primary shooting logic
        match main_loop_players[player_index].character {
          Character::Hernani => {
            game_objects.push(GameObject {
              object_type: GameObjectType::HernaniBullet,
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
            shot_successful = true;
          }
          Character::Raphaelle => {
            game_objects.push(GameObject {
              object_type: match main_loop_players[player_index].stacks {
                1  => {GameObjectType::RaphaelleBulletEmpowered},
                0 => {GameObjectType::RaphaelleBullet},
                _ => panic!()
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
            if main_loop_players[player_index].stacks == 1 {
              main_loop_players[player_index].stacks = 0;
            }
            shot_successful = true;
          }
          Character::Cynewynn => {
            game_objects.push(GameObject {
              object_type: GameObjectType::CynewynnSword,
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
            shot_successful = true;
          }
          Character::Elizabeth => {
            game_objects.push(GameObject {
              object_type: GameObjectType::ElizabethProjectileRicochet,
              size: Vector2 { x: TILE_SIZE, y: TILE_SIZE },
              position: main_loop_players[player_index].position,
              direction: main_loop_players[player_index].aim_direction,
              to_be_deleted: false,
              owner_port: main_loop_players[player_index].port,
              hitpoints: 1, // ricochet projectiles use hitpoints to keep track of wether they've already bounced
              lifetime: character.primary_range / character.primary_shot_speed,
              players: vec![],
              traveled_distance: 0.0,
            });
            shot_successful = true;
          }
          Character::Wiro => {
            if !main_loop_players[player_index].shooting_secondary
            || main_loop_players[player_index].secondary_cast_time.elapsed().as_secs_f32() < character.secondary_cooldown {
              game_objects.push(GameObject {
                object_type: GameObjectType::WiroGunShot,
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
              shot_successful = true;
            }
          }
          Character::Temerity => {

          }
          Character::Dummy => {
            game_objects.push(GameObject {
              object_type: GameObjectType::HernaniBullet,
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
            shot_successful = true;
          }
        }
        if shot_successful {
          main_loop_players[player_index].last_shot_time = Instant::now();
        }
        drop(game_objects);
      }
      // (vscode) MARK: Secondaries
      // If a player is trying to use their secondary and they have enough charge to do so, apply custom logic.
      let mut game_objects = main_game_objects.lock().unwrap();
      if shooting_secondary && secondary_charge >= character.secondary_charge_use {
        let mut secondary_used_successfully = false;
        
        match main_loop_players[player_index].character {
          
          // Create a healing aura
          Character::Raphaelle => {
            // Create a bullet type and then define its actions in the next loop that handles bullets
            game_objects.push(GameObject {
              object_type: GameObjectType::RaphaelleAura,
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
            secondary_used_successfully = true;
          },
          // Place walls
          Character::Hernani => {
            // Place down a wall at a position rounded to TILE_SIZE, unless a wall is alredy there.
            let wall_place_distance = character.secondary_range;
            let mut desired_placement_position: Vector2 = player_info.position;
            // round to closest 10
            desired_placement_position.x = ((((desired_placement_position.x + player_info.aim_direction.x * wall_place_distance + TILE_SIZE/2.0) / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;
            desired_placement_position.y = ((((desired_placement_position.y + player_info.aim_direction.y * wall_place_distance + TILE_SIZE/2.0) / TILE_SIZE) as i32) * TILE_SIZE as i32) as f32;

            let mut wall_can_be_placed = true;
            for game_object in game_objects.clone() {
              match game_object.object_type {
                GameObjectType::HernaniWall | GameObjectType::UnbreakableWall | GameObjectType::Wall => {
                  if game_object.position.x == desired_placement_position.x && game_object.position.y == desired_placement_position.y {
                    wall_can_be_placed = false;
                  }
                },
                _ => {}
              }
            }
            if wall_can_be_placed {
              game_objects.push(GameObject {
                object_type: GameObjectType::HernaniWall,
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
          },
          // position revert
          Character::Cynewynn => {
            let flashback_length = (character.secondary_cooldown * 10.0) as usize; // deciseconds
            if player_info.previous_positions.len() >= flashback_length
            && main_loop_players[player_index].secondary_cast_time.elapsed().as_secs_f32() >= character.secondary_cooldown {
              secondary_used_successfully = true;
              // set position to beginning of buffer (where player was 3 seconds ago)
              main_loop_players[player_index].position = main_loop_players[player_index].previous_positions[0];
              main_loop_players[player_index].previous_positions = Vec::new();
              main_loop_players[player_index].heal(10, characters.clone());
            }
          },

          Character::Elizabeth => {
            // Spawn a prakata billar bug.
            // (but for copyright reasons it only looks like one and isn't one!!!!!!)

            // beforehand we need to check if there's already one in the game, and delete it.
            for index in 0..game_objects.len() {
              if game_objects[index].owner_port == main_loop_players[player_index].port {
                game_objects[index].to_be_deleted = true;
              }
            }


            // spawn the new one
            game_objects.push(GameObject {
              object_type: GameObjectType::ElizabethTurret,
              size: Vector2 { x: TILE_SIZE, y: TILE_SIZE },
              position: main_loop_players[player_index].position,
              direction: main_loop_players[player_index].aim_direction,
              to_be_deleted: false,
              owner_port: main_loop_players[player_index].port,
              hitpoints: 0,
              lifetime: character.secondary_cooldown,
              players: vec![],
              traveled_distance: 0.0,
            });
            secondary_used_successfully = true;
          }
          Character::Wiro => {
            if main_loop_players[player_index].secondary_charge > 0 
            && main_loop_players[player_index].secondary_cast_time.elapsed().as_secs_f32() > character.secondary_cooldown {

                // spawn a shield object, if one can't be found already.
                
                // look for a shield
                let position: Vector2 = Vector2 {
                  x: main_loop_players[player_index].position.x + main_loop_players[player_index].aim_direction.x * TILE_SIZE,
                  y: main_loop_players[player_index].position.y + main_loop_players[player_index].aim_direction.y * TILE_SIZE,
                };
                let mut shield_found = false;
                for object_index in 0..game_objects.len() {
                  // if it's a shield, and it's ours
                  if game_objects[object_index].object_type == GameObjectType::WiroShield
                  && index_by_port(game_objects[object_index].owner_port, main_loop_players.clone()) == player_index {
                    
                    game_objects[object_index].direction = main_loop_players[player_index].aim_direction;
                    game_objects[object_index].position = position;
                    shield_found = true;
                    break;
                  }
                }
                if !shield_found {
                  game_objects.push(GameObject {
                    object_type: GameObjectType::WiroShield,
                    size: Vector2 { x: TILE_SIZE*0.5, y: characters[&Character::Wiro].secondary_range },
                    position: position,
                    direction: main_loop_players[player_index].aim_direction,
                    to_be_deleted: false,
                    owner_port: main_loop_players[player_index].port,
                    hitpoints: 0,
                    lifetime: f32::INFINITY,
                    players: vec![],
                    traveled_distance: 0.0,
                  });
                
              }
            } else {
              // delete the shield, if it exists.
              for object_index in 0..game_objects.len() {
                // if it's a shield, and it's ours
                if game_objects[object_index].object_type == GameObjectType::WiroShield
                && index_by_port(game_objects[object_index].owner_port, main_loop_players.clone()) == player_index {
                  game_objects[object_index].to_be_deleted = true;
                  // if our secondary charge is 0, also set the cooldown
                  if main_loop_players[player_index].secondary_charge == 0 {
                    main_loop_players[player_index].secondary_cast_time = Instant::now();
                  }
                  break;
                }
              }
            }
          }
          Character::Temerity => {

          }
          Character::Dummy => {}  
        }
        if secondary_used_successfully {
          main_loop_players[player_index].secondary_charge -= character.secondary_charge_use;
          main_loop_players[player_index].secondary_cast_time = Instant::now();
        }
      }
      // if the secondary button is released..
      else {
        match main_loop_players[player_index].character {
          Character::Wiro => {
            for object_index in 0..game_objects.len() {
              // if it's a shield, and it's ours
              if game_objects[object_index].object_type == GameObjectType::WiroShield
              && index_by_port(game_objects[object_index].owner_port, main_loop_players.clone()) == player_index {
                game_objects[object_index].to_be_deleted = true;
                main_loop_players[player_index].secondary_cast_time = Instant::now();
                break;
              }
            }
          }
          _ => {}
        }
      }
      drop(game_objects);
    }

    // println!("{:?}", game_objects);
    // println!("{}", 1.0 / delta_time);

    // (vscode) MARK: Object Handlin'
    // Do all logic related to game objects
    let mut game_objects = main_game_objects.lock().unwrap();

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
            size: Vector2 { x: TILE_SIZE*2.0, y: TILE_SIZE*2.0 },
            position: orb_position,
            direction: Vector2::new(),
            to_be_deleted: false,
            owner_port: 0,
            hitpoints: 60,
            lifetime: f32::INFINITY,
            players: Vec::new(),
            traveled_distance: 0.0
          }
        );
      }
    }

    for game_object_index in 0..game_objects.len() {
      let game_object_type = game_objects[game_object_index].object_type;
      match game_object_type {

        // WOLF primary
        GameObjectType::HernaniBullet => {
          (main_loop_players, *game_objects, _) = apply_simple_bullet_logic(main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, false);
        }
        // WOLF dash special
        GameObjectType::HernaniLandmine => {
          // if the landmine has existed for long enough...
          //if game_objects[game_object_index].lifetime < (characters[&Character::Hernani].dash_cooldown - 0.5) {
            for player_index in 0..main_loop_players.len() {
              // if not on same team
              if main_loop_players[player_index].team != main_loop_players[index_by_port(game_objects[game_object_index].owner_port,main_loop_players.clone())].team {
                // if within range
                let landmine_range = characters[&Character::Hernani].primary_range_2;
                if Vector2::distance(game_objects[game_object_index].position, main_loop_players[player_index].position)
                < landmine_range {
                  main_loop_players[player_index].damage(characters[&Character::Hernani].primary_damage_2, characters.clone());
                  game_objects[game_object_index].to_be_deleted = true;
                  break;
                }
              }
            }
          //}
        }

        // HEALER GIRL primary
        GameObjectType::RaphaelleBullet => {
          let hit: bool;
          (main_loop_players, *game_objects, hit) = apply_simple_bullet_logic(main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, true);
          
          // Restore nearby ally health
          if hit {
            for player_index in 0..main_loop_players.len() {
              let range: f32 = characters[&Character::Raphaelle].primary_range;
              if Vector2::distance(
                main_loop_players[player_index].position,
                main_loop_players[index_by_port(game_objects[game_object_index].owner_port,main_loop_players.clone())].position
              ) < range &&
                main_loop_players[player_index].team == main_loop_players[index_by_port(game_objects[game_object_index].owner_port,main_loop_players.clone())].team {
                // Anyone within range
                if player_index == index_by_port(game_objects[game_object_index].owner_port,main_loop_players.clone()) {
                  // if self, heal less
                  let heal_self: u8 = characters[&Character::Raphaelle].primary_lifesteal;
                  main_loop_players[player_index].heal(heal_self, characters.clone());
                }
                else {
                  // otherwise, apply normal heal
                  let heal: u8 = characters[&Character::Raphaelle].primary_heal_2;
                  main_loop_players[player_index].heal(heal, characters.clone());
                }
                  // restore dash charge (0.2s)
                // main_loop_players[game_objects[game_object_index].owner_index].last_dash_time -= Duration::from_millis(200);
              }
            }
          }
        }
        // RAPHAELLE primary, EMPOWERED
        GameObjectType::RaphaelleBulletEmpowered => {
          let hit: bool;
          (main_loop_players, *game_objects, hit) = apply_simple_bullet_logic_extra(
            main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, true,
            characters[&Character::Raphaelle].primary_damage_2, 255, false, f32::INFINITY);
          if hit {
            // restore dash charge (0.5s)
            let owner_index = index_by_port(game_objects[game_object_index].owner_port, main_loop_players.clone());
            main_loop_players[owner_index].last_dash_time -= Duration::from_millis(450);
          }
        }

        // RAPHAELLE secondary
        GameObjectType::RaphaelleAura => {
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
                  if main_loop_players[player_index].buffs[buff_index].buff_type == BuffType::RaphaelleFireRate {
                    buff_found = true;
                    break; // exit early
                  }
                }
                if !buff_found {                                 //        2  0%  <- find way to source this from properties file sometime idk
                  main_loop_players[player_index].buffs.push(Buff { value: 0.2, duration: 0.1, buff_type: BuffType::RaphaelleFireRate });
                }
              }
              // not actually necessary
              //else {
              //  for buff_index in 0..main_loop_players[player_index].buffs.len() {
              //    if main_loop_players[player_index].buffs[buff_index].buff_type == BuffType::RaphaelleFireRate {
              //      main_loop_players[player_index].buffs.remove(buff_index);
              //      break; // exit early
              //    }
              //  }
              //}
            }
          }
        }

        // QUEEN primary
        GameObjectType::CynewynnSword => {
          (main_loop_players, *game_objects, _) = apply_simple_bullet_logic(main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, true);
        }
        // ELIZABETH primary
        GameObjectType::ElizabethProjectileRicochet => {
          (main_loop_players, *game_objects, _) = apply_simple_bullet_logic_extra(main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, true, 
            255, 255, true, f32::INFINITY);
        }
        // ELIZABETH primary but recalled
        GameObjectType::ElizabethProjectileGroundRecalled => {
          // needs to move towards owner
          let owner_port = game_objects[game_object_index].owner_port;
          let owner_index = index_by_port(owner_port, main_loop_players.clone());
          let target_position: Vector2 = main_loop_players[owner_index].position;
          let object_position: Vector2 = game_objects[game_object_index].position;
          let speed = characters[&main_loop_players[owner_index].character].primary_shot_speed;
          // You are a promise, abolish hatred,
          // Child of the Sanctum, you are beloved,
          // You know to fathom, how I'm suffering,
          // Yourself to release, out in this clearing,
          let direction: Vector2 = Vector2::difference(object_position, target_position);
          // update position
          game_objects[game_object_index].position.x += direction.normalize().x * speed * true_delta_time as f32;
          game_objects[game_object_index].position.y += direction.normalize().y * speed * true_delta_time as f32;
          // If the projectiles are close enough to us, delete them, since their trip is over.
          if direction.magnitude() < TILE_SIZE /* arbitrary value */ {
            game_objects[game_object_index].to_be_deleted = true;
          }
          let hit_radius = characters[&main_loop_players[owner_index].character].primary_hit_radius;
          let damage = characters[&main_loop_players[owner_index].character].primary_damage_2;
          for player_index in 0..main_loop_players.len() {
            let player_position = main_loop_players[player_index].position;
            // if we hit a player
            if Vector2::distance(player_position, object_position) < hit_radius
            // and we haven't already
            && !game_objects[game_object_index].players.contains(&player_index) {
              // damage them
              main_loop_players[player_index].damage(damage, characters.clone());
              // and check if they were already hit by a projectile.
              let mut was_already_hit: bool = false;
              for game_object_index_2 in 0..game_objects.len() {
                if game_objects[game_object_index_2].players.contains(&player_index)
                && game_object_index_2 != game_object_index {
                  was_already_hit = true;
                  break;
                }
              }
              if was_already_hit {
                // apply a debuff
                main_loop_players[player_index].buffs.push(
                  Buff {
                    value: -2.5 ,
                    duration: 0.25,
                    buff_type: BuffType::Speed,
                  }
                );
              }
              // Finally, update the game object to know this player was already hit
              game_objects[game_object_index].players.push(player_index);
            }
          }
        }
        // ELIZABETH'S TURRET
        GameObjectType::ElizabethTurret => {
          // PROJECTILES
          // shoot projectiles. use secondary_cast_time as cooldown counter.
          let owner = index_by_port(game_objects[game_object_index].owner_port, main_loop_players.clone());
          let owner_team = main_loop_players[owner].team;
          let object_pos = game_objects[game_object_index].position;
          let range = characters[&Character::Elizabeth].secondary_range;
          let cooldown = characters[&Character::Elizabeth].primary_cooldown_2;
          let speed = characters[&Character::Elizabeth].primary_shot_speed_2;

          for player in main_loop_players.clone() {
            if player.team != owner_team
            && Vector2::distance(object_pos, player.position) < range {
              if main_loop_players[owner].secondary_cast_time.elapsed().as_secs_f32() > cooldown {
                // shoot
                game_objects.push(GameObject {
                  object_type: GameObjectType::ElizabethTurretProjectile,
                  size: Vector2 { x: TILE_SIZE, y: TILE_SIZE },
                  position: object_pos,
                  direction: Vector2::difference(object_pos, player.position).normalize(),
                  to_be_deleted: false,
                  owner_port: main_loop_players[owner].port,
                  hitpoints: 0,
                  lifetime: range/speed,
                  players: vec![],
                  traveled_distance: 0.0,
                });

                // reset CD
                main_loop_players[owner].secondary_cast_time = Instant::now();
              }
            }
          }
          // MOVEMENT
          // flip
          let speed = characters[&Character::Elizabeth].primary_range_3;

          // check for collisions with walls
          let pos = game_objects[game_object_index].position;
          let direction = game_objects[game_object_index].direction;

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
              // if distance.x is greater, it means we need to flip horizonrally.
              // otherwise, flip vertically.
              if f32::abs(distance.x) > f32::abs(distance.y) {
                // flip horizontally
                game_objects[game_object_index].direction.x *= -1.0;
              } else {
                game_objects[game_object_index].direction.y *= -1.0;
              }
              break;
            }
          }
          // move
          game_objects[game_object_index].position.x += game_objects[game_object_index].direction.x * speed * true_delta_time as f32;
          game_objects[game_object_index].position.y += game_objects[game_object_index].direction.y * speed * true_delta_time as f32;

        }
        // ELIZABETH TURRET PROJECTILE
        GameObjectType::ElizabethTurretProjectile => {
          let damage = characters[&Character::Elizabeth].secondary_damage;
          let speed = characters[&Character::Elizabeth].primary_shot_speed_2;
          (main_loop_players, *game_objects, _) = apply_simple_bullet_logic_extra(
            main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, false,
            damage, 255, false, speed);
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
            let object_type = game_objects[victim_object_index].object_type;
            // if one of these objects is one we can counter...
            if countered_projectiles.contains(&object_type) {
              let obj1_owner_team = main_loop_players[index_by_port(game_objects[game_object_index  ].owner_port, main_loop_players.clone())].team;
              let obj1_owner_index = index_by_port(game_objects[game_object_index  ].owner_port, main_loop_players.clone());
              let obj2_owner_team = main_loop_players[index_by_port(game_objects[victim_object_index].owner_port, main_loop_players.clone())].team;
              let obj2_owner_character = main_loop_players[index_by_port(game_objects[victim_object_index].owner_port, main_loop_players.clone())].character;
              if obj1_owner_team != obj2_owner_team {
                let hits_shield = hits_shield(
                  game_objects[game_object_index].position,
                  game_objects[game_object_index].direction,
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
                  if main_loop_players[obj1_owner_index].secondary_charge > damage{
                    main_loop_players[obj1_owner_index].secondary_charge -= damage;
                  } else {
                    main_loop_players[obj1_owner_index].secondary_charge = 0;
                  }
                  game_objects[victim_object_index].to_be_deleted = true;
                }
              }
            }
          }
        }
        // WIRO'S PRIMARY FIRE
        GameObjectType::WiroGunShot => {
          let distance_traveled = game_objects[game_object_index].traveled_distance;
          let damage: u8;
          if distance_traveled > characters[&Character::Wiro].primary_range_2 {
            damage = characters[&Character::Wiro].primary_damage_2;
          } else {
            damage = characters[&Character::Wiro].primary_damage;
          }
          let hit: bool;
          (main_loop_players, *game_objects, hit) = apply_simple_bullet_logic_extra(main_loop_players, characters.clone(), game_objects.clone(), game_object_index, true_delta_time, true, 
            damage, 255, false, f32::INFINITY);
          if hit {
            let owner_index = index_by_port(game_objects[game_object_index].owner_port, main_loop_players.clone());
            main_loop_players[owner_index].stacks = 1;
          }
        }
        // WIRO'S DASH
        GameObjectType::WiroDashProjectile => {
          // lock it to wiro's position
          let owner_index = index_by_port(game_objects[game_object_index].owner_port, main_loop_players.clone());
          let range = characters[&Character::Wiro].primary_range_3;
          let heal = characters[&Character::Wiro].secondary_heal;
          let damage = characters[&Character::Wiro].secondary_damage;
          game_objects[game_object_index].position = main_loop_players[index_by_port(game_objects[game_object_index].owner_port, main_loop_players.clone())].position;
          for victim_index in 0..main_loop_players.len() {
            // if we get a hit, and we didn't already hit
            if Vector2::distance(main_loop_players[victim_index].position, game_objects[game_object_index].position) < range
            && !game_objects[game_object_index].players.contains(&victim_index) {
              if main_loop_players[victim_index].team == main_loop_players[owner_index].team {
                main_loop_players[victim_index].heal(heal, characters.clone());
              } else {
                main_loop_players[victim_index].damage(damage, characters.clone());
              }
              game_objects[game_object_index].players.push(victim_index);
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

    // (vscode) MARK: Object Deletion
    let mut cleansed_game_objects: Vec<GameObject> = Vec::new();
    for game_object in game_objects.clone() {
      if game_object.to_be_deleted == true {
        // EXTRA LOGIC
        match game_object.object_type.clone() {
          // Elizabeth's projectile needs to drop on deletion,
          // if it hit somebody,
          GameObjectType::ElizabethProjectileRicochet => {
            cleansed_game_objects.push(
              GameObject {
                object_type: GameObjectType::ElizabethProjectileGround,
                size: Vector2 { x: TILE_SIZE, y: TILE_SIZE },
                position: game_object.position,
                direction: Vector2::new(),
                to_be_deleted: false,
                owner_port: game_object.owner_port,
                hitpoints: 0,
                lifetime: 5.0,
                players: Vec::new(),
                traveled_distance: 0.0,
              }
            );
          },
          _ => {},
        }
      } else {
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
  /// Remember to apply appropriate logic after check.
  /// 
  /// General counter to keep track of ability stacks. Helps determine things
  /// like whether the next shot is empowered, or how powerful an ability
  /// should be after being charged up.
  stacks:  u8,
  /// list of buffs
  buffs:                Vec<Buff>,
  last_packet_time:     Instant,
  /// As of now only used by Temerity because standard dashing logic can't coexist with this.
  is_wallriding:        bool,
}
impl ServerPlayer {
  fn damage(&mut self, mut dmg: u8, characters: HashMap<Character, CharacterProperties>) -> () {
    if self.is_dead {
      return;
    }
    // Special per-character handling
    match self.character {
      Character::Raphaelle => {
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
    //println!("killing");
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
          updated_gamemode_info.alive_blue -= 1;
        }
      }
    } 
    else {
      unsafe {
        self.position = SPAWN_RED;
        // println!("Sending {} to red team spawn", self.ip);
        // Give a kill to the blue team
        if credit_other_team {
          updated_gamemode_info.alive_red -= 1;
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
  return apply_simple_bullet_logic_extra(main_loop_players, characters, game_objects, game_object_index, true_delta_time, pierceing_shot, 255, 255, false, f32::INFINITY);
}

/// Applies modifications to players and game objects as a result of
/// bullet behaviour this frame, for a specific bullet.
/// 
/// Set `special_samage` to `255` to use default character damage number.
/// Same with `special_healing`. Setting it to 0 will nullify it.
/// Set `special_speed` to f32::INFINITY to use default.
fn apply_simple_bullet_logic_extra(
  mut main_loop_players: MutexGuard<Vec<ServerPlayer>>,
  characters:            HashMap<Character, CharacterProperties>,
  mut game_objects:      Vec<GameObject>,
  game_object_index:     usize,
  true_delta_time:       f64,
  pierceing_shot:        bool,
  special_damage:        u8,
  special_healing:       u8,
  ricochet:              bool,
  special_speed:         f32,
) -> (MutexGuard<Vec<ServerPlayer>>, Vec<GameObject>, bool) {
  let game_object = game_objects[game_object_index].clone();
  let owner_port = game_object.owner_port;
  let player = main_loop_players[index_by_port(owner_port, main_loop_players.clone())].clone();
  let character = player.character;
  let character_properties = characters[&character].clone();
  let hit_radius: f32 = character_properties.primary_hit_radius;
  let wall_hit_radius: f32 = character_properties.primary_wall_hit_radius;


  let bullet_speed: f32;
  if special_speed == f32::INFINITY { bullet_speed  = character_properties.primary_shot_speed}
  else                    {bullet_speed = special_speed}
  
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
      if !ricochet || (ricochet && game_object.hitpoints == 0){
        let distance = Vector2::distance(game_object.position, game_objects[victim_object_index].position);
        let buffer = 0.5 * 1.2;
        if distance < (TILE_SIZE*buffer + wall_hit_radius) {
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

      if ricochet && game_object.hitpoints > 0 {
        let pos = game_object.position;
        let direction = game_object.direction;
        let check_distance = TILE_SIZE * 0.1;
        let buffer = 0.5 * 1.6;
        let check_position: Vector2 = Vector2 {
          x: pos.x + direction.x * check_distance ,
          y: pos.y + direction.y * check_distance ,
        };
        let distance = Vector2::distance(check_position, game_objects[victim_object_index].position);
        if distance < (TILE_SIZE*buffer + wall_hit_radius) {
          
          if game_objects[game_object_index].hitpoints == 0 {
            return (main_loop_players, game_objects, false);
          }
          if game_objects[game_object_index].hitpoints != 0 {
            game_objects[game_object_index].hitpoints = 0;
            // we need to flip x direction or flip y direction
            // |distance.x| - |distance.y|
            // negative => flip horizontal
            let distance = Vector2::difference(game_object.position, game_objects[victim_object_index].position);
            if f32::abs(distance.x) - f32::abs(distance.y) > 0.0 {
              game_objects[game_object_index].direction.x *= -1.0;
            } else {
              game_objects[game_object_index].direction.y *= -1.0;
            }
            //game_objects[game_object_index].lifetime = characters[&character].primary_range / characters[&character].primary_shot_speed;
            game_objects[game_object_index].position.x += game_objects[game_object_index].direction.x * (TILE_SIZE*0.3); // this might break at very low freq like 26Hz
            game_objects[game_object_index].position.y += game_objects[game_object_index].direction.y * (TILE_SIZE*0.3);

            // reset the lifetime (which in turn resets its range)
            let distance = characters[&character].primary_range;
            let speed = bullet_speed;
            game_objects[game_object_index].lifetime = distance / speed;
            game_objects[game_object_index].to_be_deleted = false;
          }
        }
      }
    }
  }
  // orb
  for victim_object_index in 0..game_objects.len() {
    if game_objects[victim_object_index].object_type == GameObjectType::CenterOrb {
      // if it's colliding
      let distance = Vector2::distance(game_object.position, game_objects[victim_object_index].position);
      if distance < (0.5 * TILE_SIZE + wall_hit_radius) {
        if game_objects[game_object_index].players.contains(&548) {
          continue;
        }
        let mut direction: Vector2 = game_object.direction;
        direction.x *= damage as f32 / 2.0;
        direction.y *= damage as f32 / 2.0;

        if game_objects[victim_object_index].hitpoints > damage {
          // hurt the orb :(
          game_objects[victim_object_index].hitpoints -= damage
        } else {
          // KILL THE ORB
          game_objects[victim_object_index].hitpoints = 0;
          game_objects[victim_object_index].to_be_deleted = true;
        }
        // apply knockback to the orb
        game_objects[victim_object_index].position.y += direction.y;
        game_objects[victim_object_index].position.x += direction.x;
        // apply orb healing
        if game_objects[victim_object_index].hitpoints == 0 {
          let team = player.team;
          for player_index in 0..main_loop_players.len() {
            if main_loop_players[player_index].team == team {
              main_loop_players[player_index].health += ORB_HEALING;
              if main_loop_players[player_index].health > 100 {
                main_loop_players[player_index].health = 100;
              }
            }
          }
        }
        // 548 IS THE NUMBER OF THE ORB
        game_objects[game_object_index].players.push(548);
        if !pierceing_shot {
          game_objects[game_object_index].to_be_deleted = true;
        }
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
  game_objects[game_object_index].position.x += game_objects[game_object_index].direction.x * true_delta_time as f32 * bullet_speed;
  game_objects[game_object_index].position.y += game_objects[game_object_index].direction.y * true_delta_time as f32 * bullet_speed;
  game_objects[game_object_index].traveled_distance += true_delta_time as f32 * bullet_speed;
  return (main_loop_players, game_objects, hit);
}

fn index_by_port(port: u16, players: Vec<ServerPlayer>) -> usize{
  for player_index in 0..players.len() {
    if players[player_index].port == port {
      return player_index;
    }
  }
  println!("index_by_port function error - data race condition, mayhaps?\nAlternatively, there's just no players at all");
  return 0;
}

/// ## Checks whether a projectile hits a shield
/// ### Math:
///- We have the direction the shield is facing as a vector (x, y),
///  i.e. (1, 2), a vector of slope 2/1
///  - To express it as a line in cartesian space, we can have ay = bx, where
///    in our example, a=1 and b=2, to have 2y = 1x, or 2y - 1x = 0
///    - Therefore, a vector (a, b) becomes the equation (by - ax = 0). 
///      - This is an equation of the form Ax + By + C = 0,
///        - A = -a
///        - B = b
///        - C = 0
///    - This line will always cross the origin (0, 0)
///- To get the distance of a point (c, d) from the line, we get its position
///  relative to the line's origin (and not the world origin),
///  and apply the equation:
///  - $dist = \frac{|Ac + Bd|}{\sqrt{b^2 + a^2}}
///          = \frac{|bd - ac|}{\sqrt{b^2 + a^2}}$
///  - **if $dist < hit\_radius$, we've got a hit on the shield.**
///- HOWEVER all of this logic assumes the shield is an infinite line. To apply
///  this to a finite line, we can add an additional distance check between the
///  origin of the shield and the projectile, once we've detected a collision:
///  - if dist(projectile_pos, shield_pos) < shield_width/2
///    - NOW we've got a hit.
///      - The only issue with this, is that, at the edge of the shield, it will
///        look like only the center of the projectile has a collision with the
///        shield, *which is good enough*.
/// - The direction vector is perpendicular to the shield's representative line.
///   - To obtain this perpendicular line, we can simply swap the coordinates.
fn hits_shield(shield_position: Vector2, shield_direction: Vector2, projectile_position: Vector2, shield_width: f32, projectile_hit_radius: f32) -> bool {
  // get the position of the projectile relative to the shield's origin
  let relative_projectile_pos: Vector2 = Vector2::difference(shield_position, projectile_position);
  // check that the relative distance is shorter than the shield's width/2.
  // if it is, we're in range to check for hits
  if relative_projectile_pos.magnitude() < shield_width/2.0 {
    // get the perpendicular line that represents our shield.
    // this was originally (x, y) = (-y, x), but that didn't work, for some reason.
    // This seems to work however. Why? Don't know, don't care.
    let shield_line: Vector2 = Vector2 { x: -shield_direction.x, y: shield_direction.y }.normalize();
    // calculate the perpendicular distance between the shield line and the projectile
    let perpendicular_distance =
      (f32::abs(shield_line.y * relative_projectile_pos.y - shield_line.x * relative_projectile_pos.x)) /
      (f32::sqrt(f32::powi(shield_line.x, 2) + f32::powi(shield_line.y, 2)));

    if perpendicular_distance < projectile_hit_radius {
      return true
    }
  }
  return false;
}