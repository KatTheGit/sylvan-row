// Don't show console window on Windows
#![windows_subsystem = "windows"]

use miniquad::conf::Icon;
use miniquad::window::{set_mouse_cursor, set_window_size};
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

#[cfg(target_os = "macos")]
fn rmb_index() -> usize {
  return 2;
}
#[cfg(target_os = "linux")]
fn rmb_index() -> usize {
  return 3;
}
#[cfg(target_os = "windows")]
fn rmb_index() -> usize {
  return 3; // to be tested
}

fn window_conf() -> Conf {
  Conf {
    window_title: "Game".to_owned(),
    fullscreen: false,
    icon: Some(Icon {
      small:  Image::from_file_with_format(include_bytes!("../../assets/icon/icon-small.png"), None).expect("File not found").bytes.as_slice().try_into().expect("womp womp"),
      medium: Image::from_file_with_format(include_bytes!(concat!("../../assets/icon/icon-medium.png")), None).expect("File not found").bytes.as_slice().try_into().expect("womp womp"),
      big:    Image::from_file_with_format(include_bytes!(concat!("../../assets/icon/icon-big.png")), None).expect("File not found").bytes.as_slice().try_into().expect("womp womp"),
    }),
    ..Default::default()
  }
}
/// In the future this function will host the game menu. As of now it just starts the game unconditoinally.
#[macroquad::main(window_conf)]
async fn main() {
  set_window_size(800, 450);
  game().await;
}

/// In the future this function will be called by main once the user starts the game
/// through the menu.a
async fn game(/* server_ip: &str */) {
  set_mouse_cursor(miniquad::CursorIcon::Crosshair);
  // hashmap (dictionary) that holds the texture for each game object.
  // later (when doing animations) find way to do this with rust_embed
  let mut game_object_tetures: HashMap<GameObjectType, Texture2D> = HashMap::new();
  for game_object_type in GameObjectType::iter() {
    game_object_tetures.insert(
      game_object_type,
      match game_object_type {
        GameObjectType::Wall             => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/wall.png"), None),
        GameObjectType::SniperWall       => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/wall.png"), None),
        GameObjectType::HealerAura       => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/healer_girl/textures/secondary.png"), None),
        GameObjectType::UnbreakableWall  => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/wall.png"), None),
        GameObjectType::SniperGirlBullet => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/wall.png"), None),
        GameObjectType::HealerGirlPunch  => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/wall.png"), None),
        GameObjectType::TimeQueenSword   => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/wall.png"), None),
      }
    );
  }

  // since only the main thread is allowed to read mouse position using macroquad,
  // main thread will have to modify it, and input thread will read it.
  let mouse_position: Vec2 = Vec2::new(0.0, 0.0);
  let mouse_position: Arc<Mutex<Vec2>> = Arc::new(Mutex::new(mouse_position));

  // player in a mutex because many threads need to access and modify this information safely.
  let mut player: ClientPlayer = ClientPlayer::new();
  // temporary: define character. In the future this will be given by the server and given to this function (game()) as an argument
  player.character = Character::HealerGirl;
  let player: Arc<Mutex<ClientPlayer>> = Arc::new(Mutex::new(player));

  // temporary
  let player_texture: Texture2D = Texture2D::from_file_with_format(include_bytes!("../../assets/player/player1.png"), None);

  // modified by network listener thread, accessed by input handler and game thread
  let game_objects: Vec<GameObject> = load_map_from_file(include_str!("../../assets/maps/map1.map"));
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
  let input_thread_game_objects = Arc::clone(&game_objects);
  std::thread::spawn(move || {
    input_listener_network_sender(input_thread_player, input_thread_mouse_position, input_thread_game_objects);
  });

  // start the network listener thread.
  // give it all necessary references to shared mutexes
  let network_listener_player = Arc::clone(&player);
  let network_listener_game_objects = Arc::clone(&game_objects);
  let network_listener_other_players = Arc::clone(&other_players);
  std::thread::spawn(move || {
    network_listener(network_listener_player, network_listener_game_objects, network_listener_other_players);
  });

  let character_properties: HashMap<Character, CharacterProperties> = load_characters();

  // assets/fonts/Action_Man.ttf
  let health_bar_font = load_ttf_font_from_bytes(include_bytes!("./../../assets/fonts/Action_Man.ttf")).expect("");

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

    // access and lock all necessary mutexes
    let player: Arc<Mutex<ClientPlayer>> = Arc::clone(&player);
    let player: MutexGuard<ClientPlayer> = player.lock().unwrap();
    let game_objects: Arc<Mutex<Vec<GameObject>>> = Arc::clone(&game_objects);
    let mut game_objects: MutexGuard<Vec<GameObject>> = game_objects.lock().unwrap();
    let other_players: Arc<Mutex<Vec<ClientPlayer>>> = Arc::clone(&other_players);
    let mut other_players: MutexGuard<Vec<ClientPlayer>> = other_players.lock().unwrap();

    // (vscode) MARK: Extrapolation

    // for game objects
    for game_object in game_objects.iter_mut() {
      match game_object.object_type {
        GameObjectType::HealerGirlPunch | GameObjectType::TimeQueenSword | GameObjectType::SniperGirlBullet => {
          let speed: f32 = character_properties[&(match game_object.object_type {
            GameObjectType::HealerGirlPunch => Character::HealerGirl,
            GameObjectType::SniperGirlBullet => Character::SniperGirl,
            GameObjectType::TimeQueenSword => Character::TimeQueen,
            _ => panic!()
          })].primary_shot_speed;
          game_object.position.x += speed * game_object.direction.x * get_frame_time();
          game_object.position.y += speed * game_object.direction.y * get_frame_time();
        }
        _ => {},
      }
    }
    // for players
    for player in other_players.iter_mut() {
      player.position.x += character_properties[&player.character].speed * player.movement_direction.x * get_frame_time() / 2.0;
      player.position.y += character_properties[&player.character].speed * player.movement_direction.y * get_frame_time() / 2.0;
    }

    let player_copy = player.clone();
    drop(player);

    let game_objects_copy = game_objects.clone();
    drop(game_objects);

    let other_players_copy = other_players.clone();
    drop(other_players);

    // (vscode) MARK: update mouse pos
    let mouse_position: Arc<Mutex<Vec2>> = Arc::clone(&mouse_position);
    let mut mouse_position: MutexGuard<Vec2> = mouse_position.lock().unwrap();
    // update mouse position for the input thread to handle.
    // This hot garbage WILL be removed once camera is implemented correctly. Mayhaps.
    // But what this does is turn the mouse's screen coordinates into game coordinates,
    // the same type of coordinates the player uses
    //                        [-1;+1] range to [0;1] range          world      aspect      correct shenanigans related         center
    //                        conversion.                           coords     ratio       to cropping.
    //                     .------------------'-----------------.   ,-'-.   .----'---.  .---------------'--------------.   ,-------'----------,
    mouse_position.x =((((mouse_position_local().x + 1.0) / 2.0) * 100.0 * (16.0/9.0)) / (vw * 100.0)) * screen_width()  - 50.0 * (16.0 / 9.0);
    mouse_position.y =((((mouse_position_local().y + 1.0) / 2.0) * 100.0             ) / (vh * 100.0)) * screen_height() - 50.0;
    let aim_direction: Vector2 = Vector2::difference(Vector2::new(), Vector2::from(mouse_position.clone()));
    drop(mouse_position);

    // (vscode) MARK: Draw
    // Draw the backgrounds
    clear_background(BLACK);
    draw_rectangle(0.0, 0.0, 100.0 * vw, 100.0 * vh, GRAY);

    // draw all gameobjects
    for game_object in game_objects_copy {
      let texture = &game_object_tetures[&game_object.object_type];
      let size = game_object.size;
      draw_image_relative(texture, game_object.position.x - size/2.0, game_object.position.y - size/2.0, size, size, vh, player_copy.position);
    }

    // draw player and crosshair (aim laser)
    let range = character_properties[&player_copy.character].primary_range;
    // player_copy.draw_crosshair(vh, player_copy.position, range);
    let relative_position_x = 50.0 * (16.0/9.0); //+ ((vh * (16.0/9.0)) * 100.0 )/ 2.0;
    let relative_position_y = 50.0; //+ (vh * 100.0) / 2.0;
    draw_line(
      (aim_direction.normalize().x * 5.0 * vh) + relative_position_x * vh,
      (aim_direction.normalize().y * 5.0 * vh) + relative_position_y * vh,
      (aim_direction.normalize().x * range * vh) + (relative_position_x * vh),
      (aim_direction.normalize().y * range * vh) + (relative_position_y * vh),
      2.0, Color { r: 1.0, g: 0.5, b: 0.0, a: 1.0 }
    );
    player_copy.draw(&player_texture, vh, player_copy.position, &health_bar_font);
    
    for player in other_players_copy {
      player.draw(&player_texture /* <-- temporary */, vh, player_copy.position, &health_bar_font);
    }

    draw_text(format!("{} fps", get_fps()).as_str(), 20.0, 20.0, 20.0, DARKGRAY);
    next_frame().await;
  }
}

// (vscode) MARK: Network and Input
/// This thread:
/// - handles input and updates player info
/// - handles sending player info to the server
/// 
/// The goal is to have a non-fps limited way of giving the server as precise
/// as possible player info, recieveing inputs independently of potentially
/// slow monitors.
fn input_listener_network_sender(player: Arc<Mutex<ClientPlayer>>, mouse_position: Arc<Mutex<Vec2>>, game_objects: Arc<Mutex<Vec<GameObject>>>) -> ! {

  // temporary
  let server_ip: &str = "0.0.0.0";
  let server_ip: String = format!("{}:{}", server_ip, SERVER_LISTEN_PORT);
  // create the socket for sending info.
  let sending_ip: String = format!("0.0.0.0:{}", CLIENT_SEND_PORT);
  let sending_socket: UdpSocket = UdpSocket::bind(sending_ip)
    .expect("Could not bind client sender socket");

  let character_properties: HashMap<Character, CharacterProperties> = load_characters();

  // initiate gamepad stuff
  let mut gilrs = Gilrs::new().expect("Gilrs failed");
  let mut active_gamepad: Option<GamepadId> = None;
  // temporary
  let controller_deadzone: f32 = 0.3;

  let mut delta_time_counter: Instant = Instant::now();
  let mut delta_time: f32 = delta_time_counter.elapsed().as_secs_f32();
  let desired_delta_time: f32 = 1.0 / 60.0; // Hz

  // Whether in keyboard or controller mode.
  // Ignore mouse pos in controller mode for example.
  let mut keyboard_mode: bool = true;

  let mut fullscreen = false;
  let mut toggle_time: Instant = Instant::now();

  loop {
    // println!("network sender Hz: {}", 1.0 / delta_time);

    // update active gamepad info
    while let Some(Event { id, event: _, time: _ }) = gilrs.next_event() {
      active_gamepad = Some(id);
    }

    let mut player: MutexGuard<ClientPlayer> = player.lock().unwrap();
    let real_game_objects: MutexGuard<Vec<GameObject>> = game_objects.lock().unwrap();
    let game_objects = real_game_objects.clone();
    drop(real_game_objects);

    let mut movement_vector: Vector2 = Vector2::new();
    let mut shooting_primary: bool = false;
    let mut shooting_secondary: bool = false;
    let mut dashing: bool = false;

    // maybe? temporary
    let movement_speed: f32 = character_properties[&player.character].speed;

    // println!("sender Hz: {}", 1.0 / delta_time);

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
      match gamepad.button_data(Button::South) {
        Some(button_data) => {
          if button_data.value() > 0.0 {
            dashing = true;
          }
        } _ => {}
      }
    }

    // This solution is vile because it will take input even if the window
    // is not active. If you ask me that's funny as shit. It also allows
    // for input precision beyond framerate.
    let device_state: DeviceState = DeviceState::new();
    let keys: Vec<Keycode> = device_state.get_keys();
    let mouse: Vec<bool> = device_state.get_mouse().button_pressed;
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
        Keycode::Space => dashing = true,
        Keycode::F11 => {
          // Dirty solution but works.
          if toggle_time.elapsed().as_secs_f32() > 0.05 {
            // can't unset fullscreen on Linux because of macroquad issue.
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
          }
          toggle_time = Instant::now();
        },
        _ => {}
      }
    }
    //  LMB
    if mouse[1] == true {
      shooting_primary = true;
    }
    //  RMB
    // 3 anywhere, 2 on macos
    if mouse[rmb_index()] == true {
      shooting_secondary = true;
    }
    
    // println!("{}", dashing);
    // println!("{} {}", shooting_primary, shooting_secondary);

    if keyboard_mode { 
      let mouse_position = Arc::clone(&mouse_position);
      let mouse_position = mouse_position.lock().unwrap();
      let aim_direction = Vector2::from(*mouse_position);
      drop(mouse_position);
      player.aim_direction = aim_direction;
    }
    
    if player.aim_direction.magnitude() < controller_deadzone {
      player.aim_direction = Vector2::new();
    }

    // janky but good enough to correct controllers that give weird inputs.
    // should not happen on normal controllers anyways.
    // also corrects keyboard input.
    if movement_vector.magnitude() > 1.0 {
      // println!("normalizing");
      movement_vector = movement_vector.normalize();
    }

    // expresses the player's movement without the multiplication
    // by delta time and speed. Sent to the server.
    let mut movement_vector_raw: Vector2 = movement_vector;
    
    movement_vector.x *= movement_speed * delta_time;
    movement_vector.y *= movement_speed * delta_time;

    (movement_vector_raw, movement_vector) = object_aware_movement(player.position, movement_vector_raw, movement_vector, game_objects.clone());
    player.position.x += movement_vector.x;
    player.position.y += movement_vector.y;

    // println!("{:?}", player.position);

    // println!("{:?}", movement_vector);
    // println!("{:?}", movement_vector_raw);

    // create the packet to be sent to server.
    let client_packet: ClientPacket = ClientPacket {
      position:      player.position,
      movement:      movement_vector_raw,
      aim_direction: player.aim_direction,
      shooting_primary,
      shooting_secondary,
      packet_interval: delta_time,
      dashing,
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
  let mut buffer: [u8; 4096] = [0; 4096];
  loop {
    // recieve packet
    let (amt, _src): (usize, std::net::SocketAddr) = listening_socket.recv_from(&mut buffer)
      .expect("Listening socket failed to recieve.");
    let data: &[u8] = &buffer[..amt];
    let recieved_server_info: ServerPacket = bincode::deserialize(data).expect("Could not deserialise server packet.");
    // println!("CLIENT: Received from {}: {:?}", src, recieved_server_info);

    let mut player: MutexGuard<ClientPlayer> = player.lock().unwrap();
    // if we sent an illegal position, and server does a position override:
    if recieved_server_info.player_packet_is_sent_to.override_position {
      // gain access to the player mutex
      player.position = recieved_server_info.player_packet_is_sent_to.position_override;
    }
    player.health = recieved_server_info.player_packet_is_sent_to.health;
    player.secondary_charge = recieved_server_info.player_packet_is_sent_to.secondary_charge;
    drop(player); // free mutex guard ASAP for others to access player.
    

    let mut game_objects = game_objects.lock().unwrap();
    *game_objects = recieved_server_info.game_objects;
    drop(game_objects);

    let mut other_players = other_players.lock().unwrap();
    *other_players = recieved_server_info.players;
    drop(other_players);
  }
}