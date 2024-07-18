use top_down_shooter::common::*;

use std::net::UdpSocket;
use bincode;
use std::time::*;

static mut PLAYERS: Vec<ServerPlayer> = Vec::new();

fn main() {
  
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
  let max_players = 2;
  let player_movement_speed: f32 = 100.0;

  // MARK: networking thread
  std::thread::spawn(move || {
    // networking thread
    loop {

      // recieve packet
      let (amt, src) = listening_socket.recv_from(&mut buffer).expect(":(");
      let data = &buffer[..amt];
      let recieved_player_info: ClientPacket = bincode::deserialize(data).expect("awwww");
      // println!("SERVER: Received from {}: {:?}", src, recieved_player_info);
      
      // update PLAYERS Vector with recieved information.
      unsafe {

        let mut player_found: bool = false;

        // iterate through players
        for (index, _player) in PLAYERS.clone().iter().enumerate() {
          // use IP as identifier, check if packet from srent player correlates to our player
          if PLAYERS[index].ip == src.ip().to_string() {
            let time_since_last_packet = PLAYERS[index].last_update_time.elapsed().as_secs_f64();
            if time_since_last_packet < MAX_PACKET_INTERVAL &&
               time_since_last_packet > MIN_PACKET_INTERVAL  {
              // ignore this packet since it's coming in too fast
              player_found = true;
              break;
            }

            // if yes, update player info
            PLAYERS[index].shooting = recieved_player_info.shooting;
            PLAYERS[index].shooting_secondary = recieved_player_info.shooting_secondary;

            // check if movement is legal
            let movement_error_margin = 10.0;
            let mut movement_legal = true;

            let counter = PLAYERS[index].packet_average_counter;

            if counter > PACKET_AVERAGE_SAMPLES {
              PLAYERS[index].packet_average_counter = 0; // counter = 0;
              
              let movement_delta_time = PLAYERS[index].time_at_beginning_of_average.elapsed().as_secs_f32();
              
              let traveled_distance = PLAYERS[index].traveled_distance;
              let highest_plausible_distance: f32 = (player_movement_speed + movement_error_margin) * movement_delta_time;
              if traveled_distance >= highest_plausible_distance {
                movement_legal = false;
                println!("❌ | {} | {}", traveled_distance, highest_plausible_distance);
              } else {
                println!("✅ | {} | {}", traveled_distance, highest_plausible_distance);
              }
            }
            else {
              if PLAYERS[index].packet_average_counter == 0 {
                PLAYERS[index].time_at_beginning_of_average = Instant::now();
                PLAYERS[index].position_before_checks = PLAYERS[index].position;
                PLAYERS[index].traveled_distance = 0.0;
              }
              PLAYERS[index].packet_average_counter += 1; // counter += 1;
              let previous_position: Vector2 = PLAYERS[index].position;
              let current_position: Vector2 = recieved_player_info.position;
              PLAYERS[index].traveled_distance += vector_distance(previous_position, current_position);
              let movement_direction = vector_difference(previous_position, current_position);
              movement_direction.normalize();
              PLAYERS[index].move_direction = movement_direction;
            }

            if movement_legal && !PLAYERS[index].had_illegal_position {
              // do movement
              PLAYERS[index].position = recieved_player_info.position;
            } else {
              // reset position
              PLAYERS[index].had_illegal_position = true;
              PLAYERS[index].position = PLAYERS[index].position_before_checks;

            }

            //     // OLD LOGIC (fails with inconsistent packet times)
            //     let movement_error_margin: f32 = 5.0; // later find a way to make this equal to the server's deltatime
            //     let previous_position: Vector2 = PLAYERS[index].position;
            //     let current_position: Vector2 = recieved_player_info.position;
            //     let highest_legal_distance: f64 = (player_movement_speed + movement_error_margin) as f64 * PLAYERS[index].last_update_time.elapsed().as_secs_f64();
            //     // println!("{}", PLAYERS[index].time_since_last_packet);
            //     // check if traveled distance is higher than theoretically maximal traveled distance,
            //     // or if position is already illegal.
            //     if vector_distance(previous_position, current_position) <= highest_legal_distance as f32
            //     && PLAYERS[index].had_illegal_position == false {
            //       // if it is, apply movement
            //       println!("✅ {} | {}", highest_legal_distance, vector_distance(previous_position, current_position));
            //       PLAYERS[index].position = recieved_player_info.position;
            //     } else {
            //       // movement is illegal
            //       println!("❌ {} | {}", highest_legal_distance, vector_distance(previous_position, current_position));
            //       PLAYERS[index].had_illegal_position = true;
            //     }
            
            PLAYERS[index].last_update_time = Instant::now();
            // exit loop, and inform rest of program not to proceed with appending a new player.
            player_found = true;
            break
          }
        }

        // otherwise, add the player
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
          PLAYERS.push(ServerPlayer {
            ip: src.ip().to_string(),
            last_update_time: Instant::now(),
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
            packet_average_counter: 0,
            traveled_distance: 0.0,
            time_at_beginning_of_average: Instant::now(),
            position_before_checks: match team {
              Team::Blue => Vector2 { x: 10.0, y: 10.0 },
              Team::Red  => Vector2 { x: 90.0, y: 90.0 },
            },
          });
        }
      }
    }
  });
  
  let mut server_counter: Instant = Instant::now();
  let mut delta_time: f64 = server_counter.elapsed().as_secs_f64();

  let mut networking_counter: Instant = Instant::now();
  let game_objects: Vec<GameObject> = load_map_from_file(include_str!("../../assets/maps/map1.map"));

  // MARK: server loop.
  loop {
    server_counter = Instant::now();

    // unsafe {
    //   for (index, _player) in PLAYERS.iter().enumerate() {
    //     PLAYERS[index].time_since_last_packet += delta_time;
    //   }
    // }


    // unsafe {
    //   //println!("{:?}", PLAYERS);
    //   print!("SERVER:");
    //   for player in PLAYERS.clone() {
    //     print!(" ip: {}", player.ip)
    //   }
    //   println!("");
    // }

    // Only do networking logic at 100Hz
    if networking_counter.elapsed().as_secs_f64() > MAX_PACKET_INTERVAL {
      // reset the counter
      networking_counter = Instant::now();

      unsafe {
        for (index, _player) in PLAYERS.clone().iter().enumerate() {

          let mut other_players: Vec<ServerPlayerPacket> = Vec::new();
          for (other_player_index, _player) in PLAYERS.clone().iter().enumerate() {
            if  other_player_index != index {
              other_players.push(ServerPlayerPacket {
                health: PLAYERS[other_player_index].health,
                position: PLAYERS[other_player_index].position,
                secondary_charge: PLAYERS[other_player_index].secondary_charge,
                aim_direction: PLAYERS[other_player_index].aim_direction,
                movement_direction: PLAYERS[other_player_index].move_direction,
                shooting: PLAYERS[index].shooting,
                shooting_secondary: PLAYERS[index].shooting_secondary,
              })
            }
          }
          
          // packet sent to players
          let server_packet: ServerPacket = ServerPacket {
            player_packet_is_sent_to: ServerRecievingPlayerPacket {
              health: PLAYERS[index].health,
              override_position: PLAYERS[index].had_illegal_position,
              position_override: PLAYERS[index].position,
            },
            players: other_players,
            game_objects: game_objects.clone(),
          };
          PLAYERS[index].had_illegal_position = false; // reset since we corrected the error.

          let mut player_ip = PLAYERS[index].ip.clone();
          let split_player_ip: Vec<&str> = player_ip.split(":").collect();
          player_ip = split_player_ip[0].to_string();
          player_ip = format!("{}:{}", player_ip, CLIENT_LISTEN_PORT);
          // println!("PLAYER IP: {}", player_ip);
          let serialized: Vec<u8> = bincode::serialize(&server_packet).expect("Failed to serialize message (this should never happen)");
          sending_socket.send_to(&serialized, player_ip).expect("Failed to send packet to client.");
        }
      }
    }

    // println!("Server Hz: {}", 1.0 / delta_time);
    delta_time = server_counter.elapsed().as_secs_f64();
  }
}

fn load_map_from_file(map: &str) -> Vec<GameObject> {
  let mut map_to_return: Vec<GameObject> = Vec::new();
  for line in map.lines() {
    let opcodes: Vec<&str> = line.split(" ").collect();
    println!("{:?}", opcodes);
    let gameobject_type = opcodes[0];
    let pos_x: f32 = opcodes[1].parse().unwrap();
    let pos_y: f32 = opcodes[2].parse().unwrap();

    map_to_return.push(GameObject {
      object_type: match gameobject_type {
        "wall" => {GameObjectType::Wall},
        "unbreakablewall" => {GameObjectType::UnbreakableWall},
        _ => { panic!("Unexpected ojbect in map file.")}
      },
      position: Vector2 { x: pos_x, y: pos_y }
    });
  }
  return map_to_return;
}