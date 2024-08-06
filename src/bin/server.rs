use top_down_shooter::common::*;

use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use bincode;
use std::{thread, time::*};

fn main() {

  let characters = load_characters();


  let players: Vec<ServerPlayer> = Vec::new();
  let players = Arc::new(Mutex::new(players));

  // init
  let server_listen_address = format!("0.0.0.0:{}", SERVER_LISTEN_PORT);
  let server_send_address = format!("0.0.0.0:{}", SERVER_SEND_PORT);
  let listening_socket = UdpSocket::bind(server_listen_address.clone()).expect("Error creating listener UDP socket");
  let sending_socket = UdpSocket::bind(server_send_address).expect("Error creating sender UDP socket");
  let mut buffer = [0; 1024];

  println!("Listening on: {}", server_listen_address.clone());

  let mut red_team_player_count = 0;
  let mut blue_team_player_count = 0;

  // temporary
  let max_players = 4;
  
  // networking thread
  let listener_players = Arc::clone(&players);
  std::thread::spawn(move || {
    loop {
      // recieve packet
      let (amt, src) = listening_socket.recv_from(&mut buffer).expect(":(");
      let data = &buffer[..amt];
      let recieved_player_info: ClientPacket = bincode::deserialize(data).expect("awwww");
      // println!("SERVER: Received from {}: {:?}", src, recieved_player_info);
      // temporary
      
      // update PLAYERS Vector with recieved information.
      let mut listener_players = listener_players.lock().unwrap();
      
      let mut player_found: bool = false;
      
      // iterate through players
      for player_index in 0..listener_players.len() {
        
        let mut player = listener_players[player_index].clone();
        
        // use IP as identifier, check if packet from srent player correlates to our player
        if player.ip == src.ip().to_string() {
          let time_since_last_packet = recieved_player_info.packet_interval as f64;
          if time_since_last_packet < MAX_PACKET_INTERVAL &&
          time_since_last_packet > MIN_PACKET_INTERVAL  {
            // ignore this packet since it's coming in too fast
            player_found = true;
            break;
          }

          player.aim_direction = recieved_player_info.aim_direction.normalize();
          
          let player_movement_speed: f32 = characters[&player.character].speed;
          // if yes, update player info
          player.shooting = recieved_player_info.shooting_primary;
          player.shooting_secondary = recieved_player_info.shooting_secondary;

          // check if movement is legal
          let movement_error_margin = 5.0;
          let mut movement_legal = true;

          let recieved_position = recieved_player_info.position;
          let movement = recieved_player_info.movement;
          let previous_position = player.position;

          // calculate current expected position based on input
          let mut new_position = Vector2::new();
          new_position.x = previous_position.x + movement.x * player_movement_speed * time_since_last_packet as f32;
          new_position.y = previous_position.y + movement.y * player_movement_speed * time_since_last_packet as f32;

          if Vector2::distance(new_position, recieved_position) > movement_error_margin {
            movement_legal = false;
          }

          if movement_legal {
            // do movement.
            player.position = new_position;
            println!("✅");
          } else {
            player.had_illegal_position = true;
            println!("❌, {}", Vector2::distance(new_position, recieved_position));
          }
          // exit loop, and inform rest of program not to proceed with appending a new player.
          player_found = true;
          listener_players[player_index] = player;
          break
        }
      }

      // otherwise, add the player
      // NOTE: In the future this entire lump of code will be gone, the matchmaker will populate
      // the list of players beforehand.
      if !player_found && (blue_team_player_count + red_team_player_count < max_players) {
        // decide the player's team
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
          health: 255,
          position: match team {
            Team::Blue => Vector2 { x: 10.0, y: 10.0 },
            Team::Red  => Vector2 { x: 90.0, y: 90.0 },
          },
          move_direction: Vector2 { x: 0.0, y: 0.0 },
          aim_direction: Vector2 { x: 0.0, y: 0.0 },
          shooting: false,
          shooting_secondary: false,
          secondary_charge: 0,
          had_illegal_position: false,
          character: Character::SniperGirl,
          last_shot_time: Instant::now(),
        });
      }
      drop(listener_players);
    }
  });
  
  let mut server_counter: Instant = Instant::now();
  let mut delta_time: f64 = server_counter.elapsed().as_secs_f64();
  let desired_delta_time: f64 = 1.0 / 2000.0; // Hz
  let mut networking_counter: Instant = Instant::now();
  
  let mut game_objects: Vec<GameObject> = load_map_from_file(include_str!("../../assets/maps/map1.map"));
  println!("Loaded game objects.");
  

  // server loop.
  let characters = load_characters();
  let main_loop_players = Arc::clone(&players);
  loop {
    server_counter = Instant::now();

    let mut true_delta_time: f64 = 0.0;
    if delta_time > desired_delta_time {
      true_delta_time = delta_time;
    } else {
      true_delta_time = desired_delta_time;
    }

    let mut main_loop_players = main_loop_players.lock().unwrap();

    // do all logic related to players
    for player_index in 0..main_loop_players.len() {
      let shooting = main_loop_players[player_index].shooting;
      let shooting_secondary = main_loop_players[player_index].shooting_secondary;
      let last_shot_time = main_loop_players[player_index].last_shot_time;

      let character: CharacterProperties = characters[&main_loop_players[player_index].character].clone();

      if shooting && !shooting_secondary && last_shot_time.elapsed().as_secs_f32() > character.primary_cooldown {
        main_loop_players[player_index].last_shot_time = Instant::now();
        // Do primary shooting logic
        match main_loop_players[player_index].character {
          Character::SniperGirl => {
            game_objects.push(GameObject {
              object_type: GameObjectType::SniperGirlBullet(player_index),
              position: main_loop_players[player_index].position,
              direction: main_loop_players[player_index].aim_direction,
              to_be_deleted: false,
            }); 
          }
          Character::HealerGirl => {
            
          }
          Character::ThrowerGuy => {

          }
        }
      }
    }

    // println!("{:?}", game_objects);
    println!("{}", 1.0 / delta_time);

    // Do all logic related to game objects
    for game_object_index in 0..game_objects.len() {
      let game_object = game_objects[game_object_index];
      let game_object_type = game_objects[game_object_index].object_type;
      match game_object_type {
        GameObjectType::SniperGirlBullet(owner_index) => {
          let hit_radius: f32 = 2.0;
          let bullet_speed: f32 = 100.0;
          for player_index in 0..main_loop_players.len() {
            if Vector2::distance(game_object.position, main_loop_players[player_index].position) < hit_radius &&
               owner_index != player_index 
            {
              if main_loop_players[player_index].health > characters[&Character::SniperGirl].primary_damage {
                main_loop_players[player_index].health -= characters[&Character::SniperGirl].primary_damage;
              } else {
                main_loop_players[player_index].health = 0;
              }
            }
          }
          game_objects[game_object_index].position.x += game_object.direction.x * true_delta_time as f32 * bullet_speed;
          game_objects[game_object_index].position.y += game_object.direction.y * true_delta_time as f32 * bullet_speed;

          main_loop_players[owner_index].secondary_charge += characters[&Character::SniperGirl].secondary_hit_charge;
        }
        _ => {}
      }
    }
    // Only do networking logic at some frequency
    if networking_counter.elapsed().as_secs_f64() > MAX_PACKET_INTERVAL {
      // reset the counter
      networking_counter = Instant::now();

      for (index, player) in main_loop_players.clone().iter().enumerate() {

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
            })
          }
        }
        
        // packet sent to players
        let server_packet: ServerPacket = ServerPacket {
          player_packet_is_sent_to: ServerRecievingPlayerPacket {
            health: player.health,
            override_position: player.had_illegal_position,
            position_override: player.position,
            shooting_primary: player.shooting,
            shooting_secondary: player.shooting_secondary,
          },
          players: other_players,
          game_objects: game_objects.clone(),
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
    //println!("{:?}", main_loop_players);
    drop(main_loop_players);
    // println!("Server Hz: {}", 1.0 / delta_time);
    delta_time = server_counter.elapsed().as_secs_f64();
    if delta_time < desired_delta_time {
      thread::sleep(Duration::from_secs_f64(desired_delta_time - delta_time));
    }
  }
}

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
fn load_map_from_file(map: &str) -> Vec<GameObject> {
  let mut map_to_return: Vec<GameObject> = Vec::new();
  for line in map.lines() {
    let opcodes: Vec<&str> = line.split(" ").collect();
    let gameobject_type = opcodes[0].to_lowercase();
    let gameobject_type = gameobject_type.as_str();
    let pos_x: f32 = opcodes[1].parse().unwrap();
    let pos_y: f32 = opcodes[2].parse().unwrap();

    map_to_return.push(GameObject {
      object_type: match gameobject_type {
        "wall"            => {GameObjectType::Wall(255)},
        "unbreakablewall" => {GameObjectType::UnbreakableWall},
        _                 => {panic!("Unexpected ojbect in map file.")},
      },
      position: Vector2 { x: pos_x, y: pos_y },
      direction: Vector2::new(),
      to_be_deleted: false,
    });
  }
  return map_to_return;
}

/// information held by server about players.
#[derive(Debug, Clone)]
pub struct ServerPlayer {
  pub ip:                            String,
  pub team:                          Team,
  pub character:                     Character,
  pub health:                        u8,
  pub position:                      Vector2,
  pub shooting:                      bool,
  pub last_shot_time:                Instant,
  pub shooting_secondary:            bool,
  pub secondary_charge:              u8,
  pub aim_direction:                 Vector2,
  pub move_direction:                Vector2,
  pub had_illegal_position:          bool,
}