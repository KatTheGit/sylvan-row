use std::{any::Any, collections::HashMap, net::UdpSocket, sync::{Arc, Mutex}, thread, time::*, vec};
use crate::{common::*, const_params::*, gamedata::*, maths::*, mothership_common::*};
use core::f32;
use bincode;
use chacha20poly1305::{
  aead::{Aead, KeyInit},
  ChaCha20Poly1305, Nonce
};
use opaque_ke::generic_array::GenericArray;


pub fn game_server(min_players: usize, port: u16, player_info: Vec<PlayerInfo>) {

  println!("{:?}", player_info);
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
        ip: String::new(),
        port: 0,
        true_port: 0,
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
  let players = Arc::new(Mutex::new(players));
  let game_objects:Vec<GameObject> = load_map_from_file(include_str!("../assets/maps/map1.map"));
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

  let max_players = min_players;

  // (vscode) MARK: Network Listen
  // and also return lol
  let listener_players = Arc::clone(&players);
  let listener_gamemode_info = Arc::clone(&general_gamemode_info);
  let listener_game_objects = Arc::clone(&game_objects);

  // variable used to swap between which team we assign players
  let mut team_flag: bool = true;

  println!();
  std::thread::spawn(move || {
    loop {
      // recieve packet
      let (amt, src) = socket.recv_from(&mut buffer).expect(":(");
      let data = &buffer[..amt];
      println!("packet recieved");
      // claim all the mutexes
      let mut players = listener_players.lock().unwrap();
      let mut game_objects = listener_game_objects.lock().unwrap();
      let     gamemode_info = listener_gamemode_info.lock().unwrap();
      
      let mut player_found = false;
      // find out who owns the packet
      // iterate through players
      for p_index in 0..players.len() {
        // THIS VALUE WILL THEN BE ASSIGNED BACK TO players[p_index] !!!!
        let mut player = players[p_index].clone();

        // if we have the IP
        if player.ip == src.ip().to_string()
        && player.port == src.port() {
          println!("player exists");

          player_found = true;
          // get nonce
          let nonce = &buffer[..4];
          let nonce = match bincode::deserialize::<u32>(&nonce){
            Ok(nonce) => nonce,
            Err(_) => {
              continue;
            }
          };
          //if nonce <= last_nonce {
          //  continue;
          //}
          let nonce_num = nonce;
          let mut nonce_bytes = [0u8; 12];
          nonce_bytes[8..].copy_from_slice(&nonce.to_be_bytes());
          let nonce = Nonce::from_slice(&nonce_bytes);
          
          let key = GenericArray::from_slice(&players[p_index].cipher_key.as_slice());
          let cipher = ChaCha20Poly1305::new(key);
          
          let deciphered = match cipher.decrypt(&nonce, data[4..].as_ref()) {
            Ok(decrypted) => {
              //if nonce_num <= last_nonce {
              //  continue; // this is a parroted packet, ignore it.
              //}
              // this is a valid packet, update last_nonce
              //last_nonce = nonce_num;

              // SUCCESSFULLY DECRYPTED. ASSIGN IP TO THIS PLAYER.
              players[p_index].ip = src.ip().to_string();
              players[p_index].port = src.port();
              decrypted
            },
            Err(_err) => {
              continue; // this is an erroneous packet, ignore it.
            },
          };
          let packet = match bincode::deserialize::<ClientPacket>(&deciphered) {
            Ok(packet) => packet,
            Err(_err) => {
              continue; // ignore invalid packet
            }
          };
          println!("{:?}", packet);
        }
      }
      // player not found, initialize this fella!
      if ! player_found {
        println!("player not found");
        for p_index in 0..players.len() {
          println!("trying this bloke...");

          // get nonce
          let nonce = &buffer[..4];
          let nonce = match bincode::deserialize::<u32>(&nonce){
            Ok(nonce) => nonce,
            Err(_) => {
              continue;
            }
          };
          println!("{:?}", nonce);
          //if nonce <= last_nonce {
          //  continue;
          //}
          let nonce_num = nonce;
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
              println!("successfully found a bloke.");

              break; // decrypted
            },
            Err(err) => {
              continue; // this is an erroneous packet, ignore it.
            },
          };
          //let packet = match bincode::deserialize::<ClientPacket>(&deciphered) {
          //  Ok(packet) => packet,
          //  Err(_err) => {
          //    continue; // ignore invalid packet
          //  }
          //};
        }
      }
    }
  });
}