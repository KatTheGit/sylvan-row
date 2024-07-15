use top_down_shooter::common::*;

use std::net::UdpSocket;
use bincode;
use std::time::*;

static mut PLAYERS: Vec<ServerPlayer> = Vec::new();

fn main() {
  
  // init
  println!("SERVER STARTING");

  let server_listen_address = format!("0.0.0.0:{}", SERVER_LISTEN_PORT);
  let server_send_address = format!("0.0.0.0:{}", SERVER_SEND_PORT);
  let listening_socket = UdpSocket::bind(server_listen_address.clone()).expect("Error creating listener UDP socket");
  let sending_socket = UdpSocket::bind(server_send_address).expect("Error creating sender UDP socket");
  let mut buffer = [0; 1024];

  println!("Listening on: {}", server_listen_address.clone());

  let red_team_player_count = 0;
  let blue_team_player_count = 0;

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
      println!("SERVER: Received from {}: {:?}", src, recieved_player_info);
      
      // update PLAYERS Vector with recieved information.
      unsafe {

        let mut player_found: bool = false;

        // iterate through players
        for (index, _player) in PLAYERS.clone().iter().enumerate() {
          // use IP as identifier, check if packet from srent player correlates to our player
          if PLAYERS[index].ip == src.ip().to_string() {

            if PLAYERS[index].time_since_last_packet < MAX_PACKET_INTERVAL /* + PACKET_INTERVAL_ERROR_MARGIN */ {
              // ignore this packet since it's coming in too fast
              player_found = true;
              break;
            }

            // if yes, update player info
            PLAYERS[index].shooting = recieved_player_info.shooting;
            PLAYERS[index].shooting_secondary = recieved_player_info.shooting_secondary;

            // check if movement is legal
            let movement_error_margin: f32 = 1.0; // later find a way to make this equal to the server's deltatime
            let previous_position: Vector2 = PLAYERS[index].position;
            let current_position: Vector2 = recieved_player_info.position;
            let highest_legal_distance: f64 = (player_movement_speed + movement_error_margin) as f64 * PLAYERS[index].time_since_last_packet;
            // check if traveled distance is higher than theoretically maximal traveled distance
            if vector_distance(previous_position, current_position) <= highest_legal_distance as f32 {
              // if it is, apply movement
              if true /* apply extra logic here later */{
                PLAYERS[index].position = recieved_player_info.position;
              }
            } else {
              // movement is illegal
              PLAYERS[index].had_illegal_position = true;
            }

            PLAYERS[index].time_since_last_packet = 0.0;
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
          }
          // create server player data
          // this data is pretty irrelevant, we're just initialising the player.
          PLAYERS.push(ServerPlayer {
            ip: src.ip().to_string(),
            time_since_last_packet: 0.0,
            team,
            health: 255,
            position: match team {
              Team::Blue => Vector2 { x: 0.0, y: 0.0 },
              Team::Red => Vector2 { x: 100.0, y: 100.0 },
            },
            move_direction: Vector2 { x: 0.0, y: 0.0 },
            aim_direction: Vector2 { x: 0.0, y: 0.0 },
            shooting: false,
            shooting_secondary: false,
            had_illegal_position: false,
            secondary_charge: 0,
          });
        }
      }
    }
  });
  
  let mut server_counter: Instant = Instant::now();
  let mut delta_time: f64 = server_counter.elapsed().as_secs_f64();

  let mut networking_counter: Instant = Instant::now();
  let game_objects: Vec<GameObject> = vec![GameObject { object_type: GameObjectType::Wall, position: Vector2 { x: 10.0, y: 10.0 }}];

  // MARK: server loop.
  loop {
    server_counter = Instant::now();

    unsafe {
      for (index, _player) in PLAYERS.iter().enumerate() {
        PLAYERS[index].time_since_last_packet += delta_time;
      }
    }


    // unsafe {
    //   //println!("{:?}", PLAYERS);
    //   print!("SERVER:");
    //   for player in PLAYERS.clone() {
    //     print!(" ip: {}", player.ip)
    //   }
    //   println!("");
    // }

    // Only do networking logic at 20Hz
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
              })
            }
          }

          // packet sent to players
          let server_packet: ServerPacket = ServerPacket {
            player_packet_is_sent_to: ServerRecievingPlayerPacket {
              health: PLAYERS[index].health,
              override_position: false,
              position_override: Vector2 { x: 0.0, y: 0.0 } },
            players: other_players,
            game_objects: game_objects.clone(),
          };
          let mut player_ip = PLAYERS[index].ip.clone();
          let split_player_ip: Vec<&str> = player_ip.split(":").collect();
          player_ip = split_player_ip[0].to_string();
          player_ip = format!("{}:{}", player_ip, CLIENT_LISTEN_PORT);
          println!("PLAYER IP: {}", player_ip);
          let serialized: Vec<u8> = bincode::serialize(&server_packet).expect("Failed to serialize message (this should never happen)");
          sending_socket.send_to(&serialized, player_ip).expect("Failed to send packet to client.");
        }
      }
    }

    // println!("Server Hz: {}", 1.0 / delta_time);
    delta_time = server_counter.elapsed().as_secs_f64();
  }
}