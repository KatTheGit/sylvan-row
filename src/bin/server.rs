use top_down_shooter::common::*;

use std::net::UdpSocket;
use bincode;

use std::thread::*;

static mut PLAYERS: Vec<ServerPlayer> = Vec::new();

fn main() {
  
  // init
  println!("SERVER STARTING");

  let server_address = "0.0.0.0:25567";
  let socket = UdpSocket::bind(server_address).expect("Error creating UDP socket");
  let mut buffer = [0; 1024];

  println!("Listening on: {}", server_address);

  let red_team_player_count = 0;
  let blue_team_player_count = 0;

  // temporary
  let max_players = 2;

  // networking thread
  std::thread::spawn(move || {
    // networking thread
    loop {

      // recieve packet
      let (amt, src) = socket.recv_from(&mut buffer).expect(":(");
      let data = &buffer[..amt];
      let player_info: ClientPacket = bincode::deserialize(data).expect("awwww");
      println!("SERVER: Received from {}: {:?}", src, player_info);
      
      // update PLAYERS Vector with recieved information.
      unsafe {

        let mut player_found: bool = false;

        // iterate through players
        for (index, _player) in PLAYERS.clone().iter().enumerate() {
          // use IP as identifier, check if packet from srent player correlates to our player
          if PLAYERS[index].ip == src.ip().to_string() {
            // if yes, update player info
            PLAYERS[index].shooting = player_info.shooting;
            PLAYERS[index].shooting_secondary = player_info.shooting_secondary;

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
            team,
            health: 255,
            position: match team {
              Team::Blue => Vector2 { x: 0.0, y: 0.0 },
              Team::Red => Vector2 { x: 100.0, y: 100.0 },
            },
            aim_direction: Vector2 { x: 0.0, y: 0.0 },
            shooting: false,
            shooting_secondary: false,
          });
        }
      }
    }
  });
  
  // server loop.
  loop {
    unsafe {
      println!("{:?}", PLAYERS);
    }
  }
}