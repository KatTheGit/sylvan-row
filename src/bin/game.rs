// Don't show console window on Windows
#![windows_subsystem = "windows"]

use macroquad::rand::rand;
use miniquad::window::{set_mouse_cursor, set_window_size};
use device_query::{DeviceQuery, DeviceState, Keycode};
use top_down_shooter::common::*;
use top_down_shooter::ui;
use strum::IntoEnumIterator;
use macroquad::prelude::*;
use miniquad::conf::Icon;
use gilrs::*;
use bincode;
use std::{net::UdpSocket, sync::MutexGuard};
use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use std::fs::File;

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
  return 2;
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

  let mut timer: Instant = Instant::now();
  loop {
    clear_background(WHITE);
    
    if timer.elapsed().as_secs_f32() > 0.5 {
      let healer = ui::button(Vector2 { x: 30.0, y: 30.0 }, Vector2 { x: 200.0, y: 70.0 }, "Raphaelle");
      let queen = ui::button(Vector2 { x: 30.0, y: 130.0 }, Vector2 { x: 200.0, y: 70.0 }, "Cynewynn");
      let wolf: bool = ui::button(Vector2 { x: 30.0, y: 230.0 }, Vector2 { x: 200.0, y: 70.0 },  "Hernani");
      // println!("{:?}", healer);
      
      if healer { game(Character::HealerGirl).await; timer = Instant::now() }
      if queen  { game(Character::TimeQueen).await;  timer = Instant::now() }
      if wolf   { game(Character::SniperWolf).await; timer = Instant::now() }
    } else {
      draw_text("Stopping other threads...", 30.0, 100.0, 30.0, DARKGRAY);
    }

    next_frame().await;
  }

  //game(Character::HealerGirl).await;
}
// (vscode) MARK: main()
/// In the future this function will be called by main once the user starts the game
/// through the menu.a
async fn game(/* server_ip: &str */ character: Character) {

  set_mouse_cursor(miniquad::CursorIcon::Crosshair);
  // hashmap (dictionary) that holds the texture for each game object.
  // later (when doing animations) find way to do this with rust_embed
  let mut game_object_tetures: HashMap<GameObjectType, Texture2D> = HashMap::new();
  for game_object_type in GameObjectType::iter() {
    game_object_tetures.insert(
      game_object_type,
      match game_object_type {
        GameObjectType::Wall                      => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/wall.png"), None),
        GameObjectType::SniperWall                => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/sniper_girl/textures/wall.png"), None),
        GameObjectType::HealerAura                => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/healer_girl/textures/secondary.png"), None),
        GameObjectType::UnbreakableWall           => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/unbreakable_wall.png"), None),
        GameObjectType::SniperWolfBullet          => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/sniper_girl/textures/bullet.png"), None),
        GameObjectType::HealerGirlBullet          => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/healer_girl/textures/bullet.png"), None),
        GameObjectType::HealerGirlBulletEmpowered => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/healer_girl/textures/bullet.png"), None),
        GameObjectType::TimeQueenSword            => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/time_queen/textures/bullet.png"), None),
        GameObjectType::HernaniLandmine           => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/sniper_girl/textures/trap.png"), None),
        GameObjectType::Grass1                    => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-1.png"), None),
        GameObjectType::Grass2                    => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-2.png"), None),
        GameObjectType::Grass3                    => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-3.png"), None),
        GameObjectType::Grass4                    => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-4.png"), None),
        GameObjectType::Grass5                    => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-5.png"), None),
        GameObjectType::Grass6                    => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-6.png"), None),
        GameObjectType::Grass7                    => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-7.png"), None),
        GameObjectType::Grass1Bright              => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-1-b.png"), None),
        GameObjectType::Grass2Bright              => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-2-b.png"), None),
        GameObjectType::Grass3Bright              => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-3-b.png"), None),
        GameObjectType::Grass4Bright              => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-4-b.png"), None),
        GameObjectType::Grass5Bright              => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-5-b.png"), None),
        GameObjectType::Grass6Bright              => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-6-b.png"), None),
        GameObjectType::Grass7Bright              => Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/grass-7-b.png"), None),
      }
    );
  }

  let kill_all_threads: bool = false;
  let kill_all_threads: Arc<Mutex<bool>> = Arc::new(Mutex::new(kill_all_threads));

  let gamemode_info: GameModeInfo = GameModeInfo::new();
  let gamemode_info: Arc<Mutex<GameModeInfo>> = Arc::new(Mutex::new(gamemode_info));

  let keyboard_mode: bool = true;
  let keyboard_mode: Arc<Mutex<bool>> = Arc::new(Mutex::new(keyboard_mode));

  let sender_fps: f32 = 0.0;
  let sender_fps: Arc<Mutex<f32>> = Arc::new(Mutex::new(sender_fps));

  // since only the main thread is allowed to read mouse position using macroquad,
  // main thread will have to modify it, and input thread will read it.
  let mouse_position: Vec2 = Vec2::new(0.0, 0.0);
  let mouse_position: Arc<Mutex<Vec2>> = Arc::new(Mutex::new(mouse_position));

  // player in a mutex because many threads need to access and modify this information safely.
  let mut player: ClientPlayer = ClientPlayer::new();
  // temporary: define character. In the future this will be given by the server and given to this function (game()) as an argument
  player.character = character;
  let player: Arc<Mutex<ClientPlayer>> = Arc::new(Mutex::new(player));

  let mut player_textures: HashMap<Character, Texture2D> = HashMap::new();
  for character in Character::iter() {
    player_textures.insert(
      character,
      match character {
        Character::TimeQueen  => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/time_queen/textures/main.png"), None),
        Character::HealerGirl => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/healer_girl/textures/main.png"), None),
        Character::SniperWolf => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/sniper_girl/textures/main.png"), None),
        Character::Dummy      => Texture2D::from_file_with_format(include_bytes!("../../assets/characters/sniper_girl/textures/main.png"), None),
      }
    );
  }

  // modified by network listener thread, accessed by input handler and game thread
  let game_objects: Vec<GameObject> = load_map_from_file(include_str!("../../assets/maps/map_maker.map"));
  let game_objects: Arc<Mutex<Vec<GameObject>>> = Arc::new(Mutex::new(game_objects));

  // accessed by game thread, modified by network listener thread.
  let other_players: Vec<ClientPlayer> = Vec::new();
  let other_players: Arc<Mutex<Vec<ClientPlayer>>> = Arc::new(Mutex::new(other_players));

  // express 1% of cropped screen width and height respectively.
  let mut vw: f32;
  let mut vh: f32;

  // start the input listener and network sender thread.
  // give it all necessary references to shared mutexes
  let input_thread_player = Arc::clone(&player);
  let input_thread_mouse_position = Arc::clone(&mouse_position);
  let input_thread_game_objects = Arc::clone(&game_objects);
  let input_thread_sender_fps = Arc::clone(&sender_fps);
  let input_thread_killall = Arc::clone(&kill_all_threads);
  let input_thread_keyboard_mode = Arc::clone(&keyboard_mode);
  std::thread::spawn(move || {
    input_listener_network_sender(input_thread_player, input_thread_mouse_position, input_thread_game_objects, input_thread_sender_fps, input_thread_killall, input_thread_keyboard_mode);
  });

  // start the network listener thread.
  // give it all necessary references to shared mutexes
  let network_listener_player = Arc::clone(&player);
  let network_listener_game_objects = Arc::clone(&game_objects);
  let network_listener_other_players = Arc::clone(&other_players);
  let gamemode_info_listener= Arc::clone(&gamemode_info);
  let killall_listener = Arc::clone(&kill_all_threads);
  std::thread::spawn(move || {
    network_listener(network_listener_player, network_listener_game_objects, network_listener_other_players, gamemode_info_listener, killall_listener);
  });

  let character_properties: HashMap<Character, CharacterProperties> = load_characters();

  // assets/fonts/Action_Man.ttf
  let health_bar_font = load_ttf_font_from_bytes(include_bytes!("./../../assets/fonts/Action_Man.ttf")).expect("");

  let background_tiles: Vec<BackGroundTile> = load_background_tiles(34, 24);

  // Main thread
  loop {

    // SUPER MEGA TEMPORARY
    let device_state: DeviceState = DeviceState::new();
    let keys: Vec<Keycode> = device_state.get_keys();
    if keys.contains(&Keycode::Escape) {
      // return; // Exit the game, go back to character picker menu // THIS WONT KILL ALL THREADS
      let mut killall: MutexGuard<bool> = kill_all_threads.lock().unwrap();
      *killall = true;
      return;
    }
    drop(device_state);
    drop(keys);

    // update vw and vh, used to correctly draw things scale to the screen.
    // one vh for example is 1% of screen height.
    // it's the same as in css.
    // TEMPORARY - In the future, don't restrict to 16/9
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
        GameObjectType::HealerGirlBullet | GameObjectType::TimeQueenSword | GameObjectType::SniperWolfBullet | GameObjectType::HealerGirlBulletEmpowered => {
          let speed: f32 = character_properties[&(match game_object.object_type {
            GameObjectType::HealerGirlBullet => Character::HealerGirl,
            GameObjectType::HealerGirlBulletEmpowered => Character::HealerGirl,
            GameObjectType::SniperWolfBullet => Character::SniperWolf,
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

    // readonly
    let mut player_copy = player.clone();
    drop(player);

    let mut game_objects_copy = game_objects.clone();
    drop(game_objects);

    let other_players_copy = other_players.clone();
    drop(other_players);
    
    let mut camera_offset: Vector2 = Vector2::new();
    // Set camera offset (lock to player, freecam if dead)
    if !player_copy.is_dead {
      camera_offset = Vector2 { x: 0.0, y: 0.0 };
      player_copy.camera.position.x = player_copy.position.x + camera_offset.x;
      player_copy.camera.position.y = player_copy.position.y + camera_offset.y;
    }

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
    mouse_position.x =((((mouse_position_local().x + 1.0) / 2.0) * 100.0 * (16.0/9.0)) / (vw * 100.0)) * screen_width()  - 50.0 * (16.0 / 9.0) + camera_offset.x; 
    mouse_position.y =((((mouse_position_local().y + 1.0) / 2.0) * 100.0             ) / (vh * 100.0)) * screen_height() - 50.0                + camera_offset.y;
    let keyboard_mode: MutexGuard<bool> = keyboard_mode.lock().unwrap();
    let mut aim_direction: Vector2 = Vector2::difference(Vector2::new(), Vector2::from(mouse_position.clone()));
    if !*keyboard_mode {
      aim_direction = player_copy.aim_direction;
    }
    drop(keyboard_mode);
    drop(mouse_position);

    // (vscode) MARK: Draw

    // Draw the backgrounds
    clear_background(SKYBLUE);
    // TEMPORARY
    draw_rectangle(0.0, 0.0, 100.0 * vw, 100.0 * vh, Color { r: 0.55, g: 0.75, b: 0.5, a: 1.0 });
    for background_tile in background_tiles.clone() {
      let texture = &game_object_tetures[&background_tile.object_type];
      let size: Vector2 = Vector2 { x: TILE_SIZE, y: TILE_SIZE };
      draw_image_relative(texture, background_tile.position.x - size.x/2.0, background_tile.position.y - size.y/2.0, size.x, size.y, vh, player_copy.camera.position, Vector2::new(), WHITE);
    }

    // draw all gameobjects
    game_objects_copy = sort_by_depth(game_objects_copy);
    for game_object in game_objects_copy {
      let texture = &game_object_tetures[&game_object.object_type];
      let size = game_object.size;
      let shadow_offset: f32 = 5.0;

      // Draw shadows on certain objects
      let shaded_objects = vec![GameObjectType::HealerGirlBullet,
                                                     GameObjectType::HealerGirlBulletEmpowered,
                                                     GameObjectType::SniperWolfBullet,
                                                     GameObjectType::TimeQueenSword,
                                                    ];
      if shaded_objects.contains(&game_object.object_type) {
        draw_image_relative(
          texture,
          game_object.position.x - size.x/2.0,
          game_object.position.y - size.y/2.0 + shadow_offset,
          size.x,
          size.y,
          vh, player_copy.camera.position,
          game_object.direction,
          Color { r: 0.05, g: 0.0, b: 0.1, a: 0.15 });
      }
      draw_image_relative(texture, game_object.position.x - size.x/2.0, game_object.position.y - size.y/2.0, size.x, size.y, vh, player_copy.camera.position, game_object.direction, WHITE);
    }



    // draw player and crosshair (aim laser)
    let range = character_properties[&player_copy.character].primary_range;
    let relative_position_x = 50.0 * (16.0/9.0) - camera_offset.x; //+ ((vh * (16.0/9.0)) * 100.0 )/ 2.0;
    let relative_position_y = 50.0 - camera_offset.y; //+ (vh * 100.0) / 2.0;
    // test
    //let relative_position_x = main_camera.position.x;
    //let relative_position_y = main_camera.position.y;
    if !player_copy.is_dead {
      draw_line(
        (aim_direction.normalize().x * 10.0 * vh) + relative_position_x * vh,
        (aim_direction.normalize().y * 10.0 * vh) + relative_position_y * vh,
        (aim_direction.normalize().x * range * vh) + (relative_position_x * vh),
        (aim_direction.normalize().y * range * vh) + (relative_position_y * vh),
        0.4 * vh, Color { r: 1.0, g: 0.5, b: 0.0, a: 1.0 }
      );
      if player_copy.character == Character::SniperWolf {
        let range: f32 = character_properties[&Character::SniperWolf].secondary_range - TILE_SIZE/2.0;
        let aim_dir = aim_direction.normalize();
        // perpendicular direction 1
        let aim_dir_alpha = Vector2 {x:   aim_dir.y, y: - aim_dir.x};
        // perpendicular direction 2
        let aim_dir_gamma = Vector2 {x: - aim_dir.y, y:   aim_dir.x};

        let width = 2.0;
        draw_line(
        (aim_dir.x * range + aim_dir_alpha.x * width) * vh + relative_position_x * vh,
        (aim_dir.y * range + aim_dir_alpha.y * width) * vh + relative_position_y * vh,
        (aim_dir.x * range + aim_dir_gamma.x * width) * vh + relative_position_x * vh,
        (aim_dir.y * range + aim_dir_gamma.y * width) * vh + relative_position_y * vh,
        0.4 * vh, Color { r: 1.0, g: 0.5, b: 0.0, a: 1.0 }
      );
      }
    }
    
    // draw players and optionally their trails
    let trail_y_offset: f32 = 4.5;
    for player in other_players_copy.clone() {
      if player.character == Character::TimeQueen && !player.is_dead {
        draw_lines(player.previous_positions.clone(), player_copy.camera.position, vh, player.team, trail_y_offset-0.0, 1.0);
        draw_lines(player.previous_positions.clone(), player_copy.camera.position, vh, player.team, trail_y_offset-0.3, 0.5);
        draw_lines(player.previous_positions,         player_copy.camera.position, vh, player.team, trail_y_offset-0.6, 0.25);
      }
    }
    if player_copy.character == Character::TimeQueen && !player_copy.is_dead {
      draw_lines(player_copy.previous_positions.clone(), player_copy.camera.position, vh, player_copy.team, trail_y_offset-0.0, 0.6);
      draw_lines(player_copy.previous_positions.clone(), player_copy.camera.position, vh, player_copy.team, trail_y_offset-0.3, 0.4);
      draw_lines(player_copy.previous_positions.clone(),         player_copy.camera.position, vh, player_copy.team, trail_y_offset-0.6, 0.2);
    }

    // Draw raphaelle's tethering.
    let mut all_players_copy: Vec<ClientPlayer> = other_players_copy.clone();
    all_players_copy.push(player_copy.clone());
    for player in all_players_copy.clone() {
      if player.character == Character::HealerGirl {
        for player_2 in all_players_copy.clone() {
          if Vector2::distance(player.position, player_2.position) < character_properties[&Character::HealerGirl].primary_range
          && player.team == player_2.team {
            draw_line_relative(player.position.x, player.position.y, player_2.position.x, player_2.position.y, 0.5, GREEN, player_copy.camera.position, vh);
          }
        }
      }
    }

    // temporary ofc
    if !player_copy.is_dead {
      player_copy.draw(&player_textures[&player_copy.character], vh, player_copy.camera.position, &health_bar_font, character_properties[&player_copy.character].clone());
    }
    for player in other_players_copy {
      player.draw(&player_textures[&player.character], vh, player_copy.camera.position, &health_bar_font, character_properties[&player.character].clone());
    }
    if player_copy.is_dead {
      draw_text("You dead rip", 20.0*vh, 50.0*vh, 20.0*vh, RED);
    }
    // MARK: UI
    // time, kills, rounds
    let gamemode_info_main = gamemode_info.lock().unwrap();
    // let timer_width: f32 = 5.0;
    draw_rectangle((50.0-20.0)*vw, 0.0, 40.0 * vw, 10.0*vh, Color { r: 1.0, g: 1.0, b: 1.0, a: 0.5 });
    draw_text_relative(format!("Time: {}", gamemode_info_main.time.to_string().as_str()).as_str(), -7.0, 6.0, &health_bar_font, 4, vh, Vector2 { x: 0.0, y: 50.0 }, BLACK);
    draw_text_relative(format!("Blue Kills: {}", gamemode_info_main.kills_blue.to_string().as_str()).as_str(), 10.0, 4.0, &health_bar_font, 4, vh, Vector2 { x: 0.0, y: 50.0 }, BLUE);
    draw_text_relative(format!("Blue Wins : {}", gamemode_info_main.rounds_won_blue.to_string().as_str()).as_str(), 10.0, 8.0, &health_bar_font, 4, vh, Vector2 { x: 0.0, y: 50.0 }, BLUE);
    draw_text_relative(format!("Red Kills : {}", gamemode_info_main.kills_red.to_string().as_str()).as_str(), -33.0, 4.0, &health_bar_font, 4, vh, Vector2 { x: 0.0, y: 50.0 }, RED);
    draw_text_relative(format!("Red Wins  : {}", gamemode_info_main.rounds_won_red.to_string().as_str()).as_str(), -33.0, 8.0, &health_bar_font, 4, vh, Vector2 { x: 0.0, y: 50.0 }, RED);
    // let bar_offsets = 5.0;
    // draw_line_relative(bar_offsets+10.0, 100.0 -bar_offsets, bar_offsets + (player_copy.health-50) as f32 , 100.0 - bar_offsets, 3.0, GREEN, Vector2 { x: 100.0, y: 50.0 }, vh);
    drop(gamemode_info_main);

    draw_text(format!("{} draw fps", get_fps()).as_str(), 20.0, 20.0, 20.0, DARKGRAY);
    let sender_fps: Arc<Mutex<f32>> = Arc::clone(&sender_fps);
    let sender_fps: MutexGuard<f32> = sender_fps.lock().unwrap();
    draw_text(format!("{} input fps", sender_fps).as_str(), 20.0, 40.0, 20.0, DARKGRAY);
    draw_text(format!("{} ms server to client", player_copy.owl).as_str(), 20.0, 60.0, 20.0, DARKGRAY);
    drop(sender_fps);
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
fn input_listener_network_sender(player: Arc<Mutex<ClientPlayer>>, mouse_position: Arc<Mutex<Vec2>>, game_objects: Arc<Mutex<Vec<GameObject>>>, sender_fps: Arc<Mutex<f32>>, kill: Arc<Mutex<bool>>, global_keyboard_mode: Arc<Mutex<bool>>) -> () {

  let mut server_ip: String; // immutable binding. cool.
  let ip_file_name = "moba_ip.txt";
  let ip_file = File::open(ip_file_name);
  match ip_file {
    // file exists
    Ok(mut file) => {
      let mut data = vec![];
      match file.read_to_end(&mut data) {
        // could read file
        Ok(_) => {
          server_ip = String::from_utf8(data).expect("Couldn't read IP.");
          server_ip.retain(|c| !c.is_whitespace());
        }
        // couldnt read file
        Err(_) => {
          println!("Couldn't read IP. defaulting to 0.0.0.0.");
          server_ip = String::from("0.0.0.0");
        }
      }
    }
    // file doesn't exist
    Err(error) => {
      println!("Config file not found, attempting to creating one... Error: {}.", error);
      match File::create(ip_file_name) {
        // Could create file
        Ok(mut file) => {
          let _ = file.write_all(b"0.0.0.0");
          println!("Config file created with default ip 0.0.0.0.");
          server_ip = String::from("0.0.0.0");
        }
        // Couldn't create file
        Err(error) => {
          println!("Could not create config file. Defaulting to 0.0.0.0.\nReason:\n{}", error);
          server_ip = String::from("0.0.0.0");
        }
      }
    }
  }

  let server_ip: String = format!("{}:{}", server_ip, SERVER_LISTEN_PORT);
  // create the socket for sending info.
  let sending_ip: String = format!("0.0.0.0:{}", CLIENT_SEND_PORT);
  let sending_socket: UdpSocket = UdpSocket::bind(sending_ip)
    .expect("Could not bind client sender socket");
  println!("Socket bound to IP: {}", server_ip);

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
    //println!("Shit");
    let kill_this_thread: MutexGuard<bool> = kill.lock().unwrap();
    if *kill_this_thread {
      drop(sending_socket);
      return;
    }
    drop(kill_this_thread);

    // update active gamepad info
    while let Some(Event { id, event: _, time: _ }) = gilrs.next_event() {
      active_gamepad = Some(id);
    }

    let mut player: MutexGuard<ClientPlayer> = player.lock().unwrap();
    let real_game_objects: MutexGuard<Vec<GameObject>> = game_objects.lock().unwrap();
    let game_objects = real_game_objects.clone();
    drop(real_game_objects);

    let mut movement_vector: Vector2 = Vector2::new();
    let mut aim_vector: Vector2 = Vector2::new();
    let mut shooting_primary: bool = false;
    let mut shooting_secondary: bool = false;
    let mut dashing: bool = false;

    // maybe? temporary
    let movement_speed: f32 = character_properties[&player.character].speed;

    // println!("sender Hz: {}", 1.0 / delta_time);

    // gamepad input handling
    if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {

      // keyboard_mode = false;

      // Right stick (aim)
      match gamepad.axis_data(Axis::RightStickX)  {
        Some(axis_data) => {
          aim_vector.x = axis_data.value();
        } _ => {}
      }
      match gamepad.axis_data(Axis::RightStickY)  {
        Some(axis_data) => {
          aim_vector.y = -axis_data.value();
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
            keyboard_mode = false;
          } else {
            shooting_primary = false;
          }
        } _ => {}
      }
      match gamepad.button_data(Button::LeftTrigger2) {
        Some(button_data) => {
          if button_data.value() > 0.2 {
            shooting_secondary = true;
            keyboard_mode = false;
          } else {
            shooting_secondary = false;
          }
        } _ => {}
      }
      match gamepad.button_data(Button::South) {
        Some(button_data) => {
          if button_data.value() > 0.0 {
            dashing = true;
            keyboard_mode = false;
          }
        } _ => {}
      }
      match gamepad.button_data(Button::LeftTrigger) {
        Some(button_data) => {
          if button_data.value() > 0.0 {
            dashing = true;
            keyboard_mode = false;
          }
        } _ => {}
      }
    }

    if movement_vector.magnitude() > controller_deadzone {
      keyboard_mode = false;
    } else {
      movement_vector = Vector2::new();
    }
    if aim_vector.magnitude() > controller_deadzone {
      keyboard_mode = false;
      player.aim_direction = aim_vector;
    } else {
      if !keyboard_mode {
        player.aim_direction = Vector2 { x: 0.0, y: 0.0 };
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
      keyboard_mode = true;
    }
    //  RMB
    // 3 anywhere, 2 on macos
    if mouse[rmb_index()] == true {
      shooting_secondary = true;
      keyboard_mode = true;
    }
    
    // println!("{}", dashing);
    //println!("{} {}", shooting_primary, shooting_secondary);

    // MARK: Idk figure shit out

    if keyboard_mode { 
      let mouse_position = Arc::clone(&mouse_position);
      let mouse_position = mouse_position.lock().unwrap();
      let aim_direction = Vector2::from(*mouse_position);
      drop(mouse_position);
      player.aim_direction = aim_direction;
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

    let mut extra_speed: f32 = 0.0;
    for buff in player.buffs.clone() {
      if buff.buff_type == BuffType::Speed {
        extra_speed += buff.value;
      }
    }
    
    movement_vector.x *= (movement_speed + extra_speed) * delta_time;
    movement_vector.y *= (movement_speed + extra_speed) * delta_time;

    if player.is_dead == false {  
      (movement_vector_raw, movement_vector) = object_aware_movement(player.position, movement_vector_raw, movement_vector, game_objects.clone());
      player.position.x += movement_vector.x;
      player.position.y += movement_vector.y;
    } if player.is_dead {
      player.camera.position.x += movement_vector.x;
      player.camera.position.y += movement_vector.y;
    }

    // println!("{:?}", player.position);
    // println!("{:?}", movement_vector);
    // println!("{:?}", movement_vector_raw);
    // println!("{:?}", keyboard_mode);

    // create the packet to be sent to server.
    let client_packet: ClientPacket = ClientPacket {
      position:      player.position,
      movement:      movement_vector_raw,
      aim_direction: player.aim_direction,
      shooting_primary,
      shooting_secondary,
      packet_interval: delta_time,
      dashing,
      character: player.character,
    };

    // drop mutexguard ASAP so other threads can use player ASAP.
    drop(player);
    
    // send data to server
    let serialized: Vec<u8> = bincode::serialize(&client_packet).expect("Failed to serialize message");
    sending_socket.send_to(&serialized, server_ip.clone()).expect("Failed to send packet to server. Is your IP address correct?");

    let mut update_keyboard_mode: MutexGuard<bool> = global_keyboard_mode.lock().unwrap();
    *update_keyboard_mode = keyboard_mode;
    drop(update_keyboard_mode);
    
    // update delta_time and reset counter.
    let delta_time_difference: f32 = desired_delta_time - delta_time_counter.elapsed().as_secs_f32();
    if delta_time_difference > 0.0 {
      std::thread::sleep(Duration::from_secs_f32(delta_time_difference));
    }

    delta_time = delta_time_counter.elapsed().as_secs_f32();
    delta_time_counter = Instant::now();

    let mut sender_fps: MutexGuard<f32> = sender_fps.lock().unwrap();
    *sender_fps = (1.0 / delta_time).round();
    drop(sender_fps);

  }
}

// (vscode) MARK: Network Listen
fn network_listener(
  player: Arc<Mutex<ClientPlayer>>,
  game_objects: Arc<Mutex<Vec<GameObject>>>,
  other_players: Arc<Mutex<Vec<ClientPlayer>>>,
  gamemode_info: Arc<Mutex<GameModeInfo>>,
  kill: Arc<Mutex<bool>>, ) -> () {

  let listening_ip: String = format!("0.0.0.0:{}", CLIENT_LISTEN_PORT);
  let listening_socket: UdpSocket = UdpSocket::bind(listening_ip)
  .expect("Could not bind client listener socket");
  listening_socket.set_read_timeout(Some(Duration::from_millis(100))).expect("Failed to set timeout ig...");
  // if we get another Io(Kind(UnexpectedEof)) then this buffer is too small
  let mut buffer: [u8; 4096*4] = [0; 4096*4];
  loop {

    let kill_this_thread: MutexGuard<bool> = kill.lock().unwrap();
    if *kill_this_thread {
      //drop(listening_socket);
      return;
    }
    drop(kill_this_thread);

    // recieve packet
    let recieved_server_info: ServerPacket;
    match listening_socket.recv_from(&mut buffer) {
      Ok(data) => {
        let (amt, _): (usize, std::net::SocketAddr) = data;
        let data: &[u8] = &buffer[..amt];
        recieved_server_info = bincode::deserialize(data).expect("Could not deserialise server packet.");
      }
      Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
        continue;
      }
      Err(_) => {
        println!("error while recieving data.");
        break;
      }
    }
    // println!("CLIENT: Received from {}: {:?}", src, recieved_server_info);

    let mut player: MutexGuard<ClientPlayer> = player.lock().unwrap();
    // if we sent an illegal position, and server does a position override:
    if recieved_server_info.player_packet_is_sent_to.override_position {
      // gain access to the player mutex
      player.position = recieved_server_info.player_packet_is_sent_to.position_override;
    }

    // handle camera position upon death
    if !player.is_dead && recieved_server_info.player_packet_is_sent_to.is_dead {
      // we just died rn, so set the camera pos (which is now a freecam) to current position
      // no clue why i have to do this, but for some reason upon death the camera moves "randomly"
      //player.camera.position = player.position;
      // IDK SEEMS TO WORK WITHOUT
    }

    let one_way_ping = match recieved_server_info.timestamp.elapsed() {
      Ok(val) => val.as_millis(),
      Err(_) => 0,
    };
    // println!("Server to client latency: {:?}ms", one_way_ping);
    player.owl = one_way_ping as u16;
    player.health = recieved_server_info.player_packet_is_sent_to.health;
    player.secondary_charge = recieved_server_info.player_packet_is_sent_to.secondary_charge;
    player.time_since_last_dash = recieved_server_info.player_packet_is_sent_to.last_dash_time;
    player.character = recieved_server_info.player_packet_is_sent_to.character;
    player.is_dead = recieved_server_info.player_packet_is_sent_to.is_dead;
    player.buffs = recieved_server_info.player_packet_is_sent_to.buffs;
    player.previous_positions = recieved_server_info.player_packet_is_sent_to.previous_positions;
    drop(player); // free mutex guard ASAP for other thread to access player.
    

    let mut game_objects = game_objects.lock().unwrap();
    *game_objects = recieved_server_info.game_objects;
    drop(game_objects);

    let mut other_players = other_players.lock().unwrap();
    *other_players = recieved_server_info.players;
    drop(other_players);

    let mut gamemode_info_listener = gamemode_info.lock().unwrap();
    *gamemode_info_listener = recieved_server_info.gamemode_info;
    drop (gamemode_info_listener)
  }
}

fn draw_lines(positions: Vec<Vector2>, camera: Vector2, vh: f32, team: Team, y_offset: f32, alpha: f32) -> () {
  if positions.len() < 2 { return; }
  for position_index in 0..positions.len()-1 {
    // if position_index > positions.len() / 3  {
    //   draw_line_relative(positions[position_index].x, positions[position_index].y, positions[position_index+1].x, positions[position_index+1].y, 0.4, match team {Team::Blue => BLUE, Team::Red => RED}, camera, vh);
    // } else {
    //   draw_line_relative(positions[position_index].x, positions[position_index].y, positions[position_index+1].x, positions[position_index+1].y, 0.4, match team {Team::Blue => SKYBLUE, Team::Red => ORANGE}, camera, vh);
    // }
    draw_line_relative(positions[position_index].x, positions[position_index].y + y_offset, positions[position_index+1].x, positions[position_index+1].y + y_offset, 0.4, match team {Team::Blue => Color { r: 0.2, g: 1.0-(position_index as f32 / positions.len() as f32), b: 0.8, a: alpha }, Team::Red => Color { r: 0.8, g: 0.7-0.3*(position_index as f32 / positions.len() as f32), b: 0.2, a: alpha }}, camera, vh);
  }
  // let texture = Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/tq-flashback.png"), None  );
  // draw_image_relative(&texture, positions[0].x - TILE_SIZE/2.0, positions[0].y - (TILE_SIZE*1.5)/2.0, TILE_SIZE, TILE_SIZE * 1.5, vh, camera);
}

#[derive(Debug, Clone)]
struct BackGroundTile {
  position: Vector2,
  object_type: GameObjectType,
}

fn load_background_tiles(map_size_x: u16, map_size_y: u16) -> Vec<BackGroundTile> {
  let mut tiles: Vec<BackGroundTile> = Vec::new();
  let bright_tiles = vec![GameObjectType::Grass1Bright,
                                               GameObjectType::Grass2Bright,
                                               GameObjectType::Grass3Bright,
                                               GameObjectType::Grass4Bright,
                                               GameObjectType::Grass5Bright,
                                               GameObjectType::Grass6Bright,
                                               GameObjectType::Grass7Bright, ];
  let dark_tiles = vec![GameObjectType::Grass1,
                                               GameObjectType::Grass2,
                                               GameObjectType::Grass3,
                                               GameObjectType::Grass4,
                                               GameObjectType::Grass5,
                                               GameObjectType::Grass6,
                                               GameObjectType::Grass7, ];

  for x in 0..map_size_x {
    for y in 0..map_size_y {
      let random_num_raw = rand();
      let mut random_num_f = (random_num_raw as f64) / u32::MAX as f64;
      random_num_f *= 6.0;
      let random_num = random_num_f.round() as usize;
      if (x + y) % 2 == 1 {
        tiles.push(BackGroundTile { position: Vector2 { x: x as f32 * TILE_SIZE, y: y as f32 * TILE_SIZE + TILE_SIZE*0.5 }, object_type: bright_tiles[random_num] });
      } else {
        tiles.push(BackGroundTile { position: Vector2 { x: x as f32 * TILE_SIZE, y: y as f32 * TILE_SIZE + TILE_SIZE*0.5 }, object_type: dark_tiles[random_num] });
      }
    }
  }
  return tiles;
}

/// Incredibly inefficient algorithm to sort gameobjects by height, used
/// to draw them in order without weird overlaps.
fn sort_by_depth(objects: Vec<GameObject>) -> Vec<GameObject> {
  // drawn last = drawn "higher" on "z-axis"
  // things at the bottom (high y axis) should be higher in "z-axis", ergo drawn last
  let mut unsorted_objects = objects;
  let mut sorted_objects: Vec<GameObject> = Vec::new();

  for _ in 0..unsorted_objects.len() {
    let mut current_lowest_index: usize = 0;
    let mut current_lowest_height: f32 = f32::MAX;
    for index in 0..unsorted_objects.len() {
      let pos = unsorted_objects[index].position.y;
      if pos < current_lowest_height {
        current_lowest_index = index;
        current_lowest_height = pos;
      }
    }
    sorted_objects.push(unsorted_objects[current_lowest_index].clone());
    unsorted_objects.remove(current_lowest_index);
  }
  return sorted_objects
}
