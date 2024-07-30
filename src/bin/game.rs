use top_down_shooter::common::*;
use macroquad::prelude::*;
use gilrs::*;
use std::collections::HashMap;
use std::{net::UdpSocket, sync::MutexGuard};
use std::time::{Instant, Duration};
use bincode;
use std::sync::{Arc, Mutex};
use device_query::{DeviceQuery, DeviceState, Keycode};
use strum::IntoEnumIterator;

/// In the future this function will host the game menu. As of now it just starts the game unconditoinally.
#[macroquad::main("Game")]
async fn main() {
  game().await;
}

/// In the future this function will be called by main once the user starts the game
/// through the menu.
async fn game(/* server_ip: &str */) {

  // hashmap (dictionary) that holds the texture for each game object.
  // later (when doing animations) find way to do this with rust_embed
  let mut game_object_tetures: HashMap<GameObjectType, Texture2D> = HashMap::new();
  for game_object_type in GameObjectType::iter() {
    game_object_tetures.insert(
      game_object_type,
      match game_object_type {
        GameObjectType::Wall             => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/wall.png"), None),
        GameObjectType::UnbreakableWall  => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/wall.png"), None),
        GameObjectType::SniperGirlBullet => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/wall.png"), None),
      }
    );
  }

  // since only the main thread is allowed to read mouse position using macroquad,
  // main thread will have to modify it, and input thread will read it.
  let mouse_position: Vec2 = Vec2::new(0.0, 0.0);
  let mouse_position: Arc<Mutex<Vec2>> = Arc::new(Mutex::new(mouse_position));

  // player in a mutex because many threads need to access and modify this information safely.
  let player: ClientPlayer = ClientPlayer::new();
  let player: Arc<Mutex<ClientPlayer>> = Arc::new(Mutex::new(player));

  // temporary
  let player_texture: Texture2D = Texture2D::from_file_with_format(include_bytes!("../../assets/player/player1.png"), None);

  // modified by network listener thread, accessed by input handler and game thread
  let game_objects: Vec<GameObject> = Vec::new();
  let game_objects: Arc<Mutex<Vec<GameObject>>> = Arc::new(Mutex::new(game_objects));
  // accessed by game thread, modified by network listener thread.
  let other_players: Vec<ClientPlayer> = Vec::new();
  let other_players: Arc<Mutex<Vec<ClientPlayer>>> = Arc::new(Mutex::new(other_players));

  // express 1% of cropped screen width and height respectively.
  let mut vw: f32 = 10.0; // init with random value
  let mut vh: f32 = 10.0;

  // start the input listener and network sender thread.
  // give it all necessary references to shared mutexes
  let input_thread_player = Arc::clone(&player);
  let input_thread_mouse_position = Arc::clone(&mouse_position);
  std::thread::spawn(move || {
    input_listener_network_sender(input_thread_player, input_thread_mouse_position);
  });

  // start the network listener thread.
  // give it all necessary references to shared mutexes
  let network_listener_player = Arc::clone(&player);
  let network_listener_game_objects = Arc::clone(&game_objects);
  let network_listener_other_players = Arc::clone(&other_players);
  std::thread::spawn(move || {
    network_listener(network_listener_player, network_listener_game_objects, network_listener_other_players);
  });

  // Main thread
  loop {

    // update vw and vh, used to correctly draw things scale to the screen.
    // one vh for example is 1% of screen height.
    // it's the same as in css.
    if screen_height() * (16.0/9.0) > screen_width() {
      vw = screen_width() / 100.0;
      vh = vw / (16.0/9.0);
    } else {
      vh = screen_height() / 100.0;
      vw = vh * (16.0/9.0);
    }

    // update mouse position
    let mouse_position: Arc<Mutex<Vec2>> = Arc::clone(&mouse_position);
    let mut mouse_position: MutexGuard<Vec2> = mouse_position.lock().unwrap();
    // update mouse position for the input thread to handle.
    // and translate it from screen coordinates to the same coordinates the player uses (world coordinates)
    // with the below calculation:
    //                        [-1;+1] range to [0;1] range          world      aspect      correct shenanigans related
    //                        conversion.                           coords     ratio       to cropping.
    //                     .------------------'-----------------.   ,-'-.   .----'---.  .---------------'--------------.
    mouse_position.x = ((((mouse_position_local().x + 1.0) / 2.0) * 100.0 * (16.0/9.0)) / (vw * 100.0)) * screen_width();
    mouse_position.y = ((((mouse_position_local().y + 1.0) / 2.0) * 100.0             ) / (vh * 100.0)) * screen_height();
    drop(mouse_position);

    // access and lock all necessary mutexes
    let player: Arc<Mutex<ClientPlayer>> = Arc::clone(&player);
    let player: MutexGuard<ClientPlayer> = player.lock().unwrap();
    let game_objects: Arc<Mutex<Vec<GameObject>>> = Arc::clone(&game_objects);
    let game_objects: MutexGuard<Vec<GameObject>> = game_objects.lock().unwrap();
    let other_players: Arc<Mutex<Vec<ClientPlayer>>> = Arc::clone(&other_players);
    let other_players: MutexGuard<Vec<ClientPlayer>> = other_players.lock().unwrap();

    let player_copy = player.clone();
    drop(player);

    let game_objects_copy = game_objects.clone();
    drop(game_objects);

    let other_players_copy = other_players.clone();
    drop(other_players);

    clear_background(BLACK);
    draw_rectangle(0.0, 0.0, 100.0 * vw, 100.0 * vh, WHITE);

    // draw player and crosshair (aim laser)
    player_copy.draw(&player_texture, vh);
    player_copy.draw_crosshair(vh);

    // draw all gameobjects
    for game_object in game_objects_copy {
      let texture = &game_object_tetures[&game_object.object_type];
      draw_image(texture, game_object.position.x, game_object.position.y, 10.0, 10.0, vh)
    }

    draw_text(format!("{} fps", get_fps()).as_str(), 20.0, 20.0, 20.0, DARKGRAY);
    next_frame().await;
  }
}

/// This thread:
/// - handles input and updates player info
/// - handles sending player info to the server
/// 
/// The goal is to have a non-fps limited way of giving the server as precise
/// as possible player info, recieveing inputs independently of potentially
/// slow monitors.
fn input_listener_network_sender(player: Arc<Mutex<ClientPlayer>>, mouse_position: Arc<Mutex<Vec2>>) -> ! {

  // temporary
  let server_ip: &str = "0.0.0.0";
  let server_ip: String = format!("{}:{}", server_ip, SERVER_LISTEN_PORT);
  // create the socket for sending info.
  let sending_ip: String = format!("0.0.0.0:{}", CLIENT_SEND_PORT);
  let sending_socket: UdpSocket = UdpSocket::bind(sending_ip)
    .expect("Could not bind client sender socket");

  // initiate gamepad stuff
  let mut gilrs = Gilrs::new().expect("Gilrs failed");
  let mut active_gamepad: Option<GamepadId> = None;
  // temporary
  let controller_deadzone: f32 = 0.3;

  let mut delta_time_counter: Instant = Instant::now();
  let mut delta_time: f32 = delta_time_counter.elapsed().as_secs_f32();
  let desired_delta_time: f32 = 1.0 / 300.0; // run this thread at 300Hz

  // Whether in keyboard or controller mode.
  // Ignore mouse pos in controller mode for example.
  let mut keyboard_mode: bool = true;

  loop {
    // println!("network sender Hz: {}", 1.0 / delta_time);

    // update active gamepad info
    while let Some(Event { id, event: _, time: _ }) = gilrs.next_event() {
      active_gamepad = Some(id);
    }

    let mut player: MutexGuard<ClientPlayer> = player.lock().unwrap();

    let mut movement_vector: Vector2 = Vector2::new();
    let mut shooting_primary: bool = false;
    let mut shooting_secondary: bool = false;

    // maybe? temporary
    let movement_speed: f32 = 100.0;

    // gamepad input handling
    if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {

      keyboard_mode = false;

      // Right stick (aim)
      match gamepad.axis_data(Axis::RightStickX)  {
        Some(axis_data) => {
          player.aim_direction.x = axis_data.value();
        } _ => {}
      }
      match gamepad.axis_data(Axis::RightStickY)  {
        Some(axis_data) => {
          player.aim_direction.y = -axis_data.value();
        } _ => {}
      }

      // left stick (movement)
      match gamepad.axis_data(Axis::LeftStickX)  {
        Some(axis_data) => {
          // crazy rounding shenanigans to round to closest multiple of 0.2
          movement_vector.x = ((axis_data.value() * 5.0).round() as i32) as f32 / 5.0;
        } _ => {}
      }
      match gamepad.axis_data(Axis::LeftStickY)  {
        Some(axis_data) => {
          movement_vector.y = ((-axis_data.value() * 5.0).round() as i32) as f32 / 5.0;
          // println!("{}", axis_data.value());
        } _ => {}
      }

      // triggers (shooting)
      match gamepad.button_data(Button::RightTrigger2) {
        Some(button_data) => {
          if button_data.value() > 0.2 {
            shooting_primary = true;
          } else {
            shooting_primary = false;
          }
        } _ => {}
      }
      match gamepad.button_data(Button::LeftTrigger2) {
        Some(button_data) => {
          if button_data.value() > 0.2 {
            shooting_secondary = true;
          } else {
            shooting_secondary = false;
          }
        } _ => {}
      }
    }

    // This solution is vile because it will take input even if the window
    // is not active. If you ask me that's funny as shit. It also allows
    // for input precision beyond framerate.
    let device_state: DeviceState = DeviceState::new();
    let keys: Vec<Keycode> = device_state.get_keys();
    if !keys.is_empty() {
      movement_vector = Vector2::new();
      keyboard_mode = true; // since we used the keyboard
    }
    for key in keys {
      match key {
        Keycode::W => movement_vector.y += -1.0,
        Keycode::A => movement_vector.x += -1.0,
        Keycode::S => movement_vector.y +=  1.0,
        Keycode::D => movement_vector.x +=  1.0,
        _ => {}
      }
    }

    if keyboard_mode { 
      let mouse_position = Arc::clone(&mouse_position);
      let mouse_position = mouse_position.lock().unwrap();
      let player_position = player.position;
      let mut aim_direction = Vector2::new();
      aim_direction.x = mouse_position.x - player_position.x;
      aim_direction.y = mouse_position.y - player_position.y;
      drop(mouse_position);
      aim_direction = aim_direction.normalize();
      player.aim_direction = aim_direction;
    }

    // janky but good enough to correct controllers that give weird inputs.
    // should not happen on normal controllers anyways.
    // also corrects keyboard input.
    if movement_vector.magnitude() > 1.0 {
      // println!("normalizing");
      movement_vector = movement_vector.normalize();
    }

    movement_vector.x *= movement_speed * delta_time;
    movement_vector.y *= movement_speed * delta_time;

    player.position.x += movement_vector.x;
    player.position.y += movement_vector.y;
    if player.aim_direction.magnitude() < controller_deadzone {
      player.aim_direction = Vector2::new();
    }

    // create the packet to be sent to server.
    let client_packet: ClientPacket = ClientPacket {
      position:      Vector2 {x: player.position.x, y: player.position.y },
      movement:      movement_vector,
      aim_direction: Vector2 { x: player.aim_direction.x, y: player.aim_direction.y },
      shooting_primary,
      shooting_secondary,
    };

    // drop mutexguard ASAP so other threads can use player ASAP.
    drop(player);
    
    // send data to server
    let serialized: Vec<u8> = bincode::serialize(&client_packet).expect("Failed to serialize message");
    sending_socket.send_to(&serialized, server_ip.clone()).expect("Failed to send packet to server.");

    
    // update delta_time and reset counter.
    let delta_time_difference: f32 = desired_delta_time - delta_time_counter.elapsed().as_secs_f32();
    if delta_time_difference > 0.0 {
      std::thread::sleep(Duration::from_secs_f32(delta_time_difference));
    }

    delta_time = delta_time_counter.elapsed().as_secs_f32();
    delta_time_counter = Instant::now();
  }
}

fn network_listener(
  player: Arc<Mutex<ClientPlayer>>,
  game_objects: Arc<Mutex<Vec<GameObject>>>,
  other_players: Arc<Mutex<Vec<ClientPlayer>>>) -> ! {

  let listening_ip: String = format!("0.0.0.0:{}", CLIENT_LISTEN_PORT);
  let listening_socket: UdpSocket = UdpSocket::bind(listening_ip)
    .expect("Could not bind client listener socket");
  let mut buffer: [u8; 2048] = [0; 2048];
  loop {
    // recieve packet
    let (amt, _src): (usize, std::net::SocketAddr) = listening_socket.recv_from(&mut buffer)
      .expect("Listening socket failed to recieve.");
    let data: &[u8] = &buffer[..amt];
    let recieved_server_info: ServerPacket = bincode::deserialize(data).expect("Could not deserialise server packet.");
    // println!("CLIENT: Received from {}: {:?}", src, recieved_server_info);

    // if we sent an illegal position, and server does a position override:
    if recieved_server_info.player_packet_is_sent_to.override_position {
      // gain access to the player mutex
      let mut player: MutexGuard<ClientPlayer> = player.lock().unwrap();
      player.position = recieved_server_info.player_packet_is_sent_to.position_override;
      drop(player); // free mutex guard ASAP for others to access player.
    }
    
    let mut game_objects = game_objects.lock().unwrap();
    *game_objects = recieved_server_info.game_objects;
    drop(game_objects);

    let mut other_players = other_players.lock().unwrap();
    for player in recieved_server_info.players {
      other_players.push(player);
    }
    drop(other_players);
  }
}