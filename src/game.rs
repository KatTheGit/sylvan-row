#![allow(unused_parens)]

use std::{collections::HashMap, io::{ErrorKind, Read, Write}, net::{TcpStream, UdpSocket}, sync::{Arc, Mutex, MutexGuard}, time::{Duration, Instant, SystemTime}};
use kira::{track::TrackBuilder, AudioManager, AudioManagerSettings, DefaultBackend};
use crate::{audio, common::*, const_params::*, database::{self, FriendShipStatus}, gamedata::*, graphics, maths::*, mothership_common::*, network, ui::{self, Settings}};
use miniquad::window::set_window_size;
use device_query::{DeviceQuery, DeviceState, Keycode};
use macroquad::{prelude::*, rand::rand};
use gilrs::*;
use bincode;
use opaque_ke::generic_array::GenericArray;
use chacha20poly1305::{
  aead::{Aead, KeyInit},
  ChaCha20Poly1305, Nonce
};

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

pub async fn game(server_ip: String, character: Character, client_port: u16, server_port: u16, cipher_key: Vec<u8>, username: String, mut settings: &mut Settings, server_interaction: &mut MainServerInteraction, main_nonce: &mut u32, mut main_last_nonce: &mut u32, fullscreen: &mut bool, game_id: u128, settings_tabs: &mut ui::Tabs) {
  //set_mouse_cursor(miniquad::CursorIcon::Crosshair);
  // hashmap (dictionary) that holds the texture for each game object.
  // later (when doing animations) find way to do this with rust_embed
  let game_object_tetures = graphics::load_game_object_textures();
  let kill_all_threads: bool = false;
  let kill_all_threads: Arc<Mutex<bool>> = Arc::new(Mutex::new(kill_all_threads));

  let mut packet_queue: Vec<ClientToServerPacket> = Vec::new();

  let mut chat_timer: Instant = Instant::now();

  let gamemode_info: GameModeInfo = GameModeInfo::new();
  let gamemode_info: Arc<Mutex<GameModeInfo>> = Arc::new(Mutex::new(gamemode_info));
  
  let input_halt: bool = false;
  let input_halt: Arc<Mutex<bool>> = Arc::new(Mutex::new(input_halt));

  let keyboard_mode: bool = true;
  let keyboard_mode: Arc<Mutex<bool>> = Arc::new(Mutex::new(keyboard_mode));

  let sender_fps: f32 = 0.0;
  let sender_fps: Arc<Mutex<f32>> = Arc::new(Mutex::new(sender_fps));

  // since only the main thread is allowed to read mouse position using macroquad,
  // main thread will have to modify it, and input thread will read it.
  let mut mouse_position: Vec2 = Vec2::new(0.0, 0.0);

  let mut menu_paused = false;
  let mut settings_open_flag = false;

  // player in a mutex because many threads need to access and modify this information safely.
  let mut player: ClientPlayer = ClientPlayer::new();
  // temporary: define character. In the future this will be given by the server and given to this function (game()) as an argument
  player.character = character;
  player.position = Vector2 { x: 10.0, y: 10.0 };
  player.username = username.clone();
  let player: Arc<Mutex<ClientPlayer>> = Arc::new(Mutex::new(player));
  

  let player_textures: HashMap<Character, Texture2D> = graphics::load_character_textures();

  // modified by network listener thread, accessed by input handler and game thread
  let game_objects: Vec<GameObject> = load_map_from_file(include_str!("../assets/maps/map1.map"), &mut 0);
  let game_objects: Arc<Mutex<Vec<GameObject>>> = Arc::new(Mutex::new(game_objects));

  // accessed by game thread, modified by network listener thread.
  let other_players: Vec<ClientPlayer> = Vec::new();
  let other_players: Arc<Mutex<Vec<ClientPlayer>>> = Arc::new(Mutex::new(other_players));

  // express 1% of cropped screen width and height respectively.
  let mut vw: f32;
  let mut vh: f32;

  // used to allow other thread to play sounds (which are managed by main thread)
  let sound_queue: Vec<(&[u8], AudioTrack)> = Vec::new();
  let sound_queue: Arc<Mutex<Vec<(&[u8], AudioTrack)>>> = Arc::new(Mutex::new(sound_queue));

  // start the input listener and network sender thread.
  // give it all necessary references to shared mutexes
  let input_thread_sound_queue = Arc::clone(&sound_queue);
  let input_thread_player = Arc::clone(&player);
  let input_thread_game_objects = Arc::clone(&game_objects);
  let input_thread_sender_fps = Arc::clone(&sender_fps);
  let input_thread_killall = Arc::clone(&kill_all_threads);
  let input_thread_keyboard_mode = Arc::clone(&keyboard_mode);
  let network_listener_other_players = Arc::clone(&other_players);
  let gamemode_info_listener= Arc::clone(&gamemode_info);
  let input_halt_listener= Arc::clone(&input_halt);
  let cipher_key_copy = cipher_key.clone();
  std::thread::spawn(move || {
    input_listener_network_sender(input_thread_player, input_thread_game_objects, input_thread_sender_fps, input_thread_killall, input_thread_keyboard_mode, client_port, network_listener_other_players, gamemode_info_listener, server_port, cipher_key_copy, input_halt_listener, server_ip, input_thread_sound_queue);
  });

  let character_properties: HashMap<Character, CharacterProperties> = load_characters();
  let character_descriptions = CharacterDescription::create_all_descriptions(character_properties.clone());

  // assets/fonts/Action_Man.ttf
  let health_bar_font = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Action_Man.ttf")).expect("");

  let background_tiles: Vec<BackGroundTile> = load_background_tiles(34, 24);

  let mut timer_for_text_update = Instant::now();
  let mut slow_sender_fps: f32 = 0.0;
  let mut slow_draw_fps = 0;
  let mut slow_ping = 0;

  let mut connected_to_server = false;

  // for menus. don't press escape again if it was already pressed. avoids
  // toggling the menu every frame
  let mut escape_already_pressed: bool = false;

  let mut prev_gamemode_info: GameModeInfo = GameModeInfo::new();

  let mut game_ended: bool = false;
  let mut server_crashed: bool = false;
  let mut game_ended_timer = Instant::now();
  let mut winning_team = Team::Blue;

  // set up audio tracks
  let mut audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).expect("oops");
  let mut sfx_self_track = audio_manager.add_sub_track(TrackBuilder::default()).expect("oops");
  let mut sfx_other_track = audio_manager.add_sub_track(TrackBuilder::default()).expect("oops");
  let mut music_track = audio_manager.add_sub_track(TrackBuilder::default()).expect("oops");
  let sfx_self_volume = settings.master_volume * settings.sfx_self_volume / 100.0;
  let sfx_other_volume = settings.master_volume * settings.sfx_other_volume / 100.0;
  let music_volume = settings.master_volume * settings.music_volume / 100.0;
  audio::set_volume(sfx_self_volume, &mut sfx_self_track);
  audio::set_volume(sfx_other_volume, &mut sfx_other_track);
  audio::set_volume(music_volume, &mut music_track);

  // Main thread
  loop {

    let delta_time: f32 = 1.0 / get_fps() as f32;

    // SUPER MEGA TEMPORARY
    let device_state: DeviceState = DeviceState::new();
    let keys: Vec<Keycode> = device_state.get_keys();
    if keys.contains(&Keycode::Escape) {
      //let mut killall: MutexGuard<bool> = kill_all_threads.lock().unwrap();
      //*killall = true;
      if !escape_already_pressed {
        menu_paused = !menu_paused;
        settings_open_flag = false;
      }
      escape_already_pressed = true;
      //return;
    } else {
      escape_already_pressed = false;
    }
    drop(device_state);
    drop(keys);
    let killall: MutexGuard<bool> = kill_all_threads.lock().unwrap();
    if *killall {
      packet_queue.push(
        ClientToServerPacket {
          information: ClientToServer::MatchLeave,
        }
      );
      return;
    }
    drop(killall);
    {
      let mut input_halt = input_halt.lock().unwrap();
      *input_halt = menu_paused | server_interaction.is_chatbox_open;
    }

    if get_keys_pressed().contains(&KeyCode::F11) {
      *fullscreen = !*fullscreen;
      set_fullscreen(*fullscreen);
    }

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
    let mut player: MutexGuard<ClientPlayer> = player.lock().unwrap();
    let game_objects: Arc<Mutex<Vec<GameObject>>> = Arc::clone(&game_objects);
    let mut game_objects: MutexGuard<Vec<GameObject>> = game_objects.lock().unwrap();
    let other_players: Arc<Mutex<Vec<ClientPlayer>>> = Arc::clone(&other_players);
    let mut other_players: MutexGuard<Vec<ClientPlayer>> = other_players.lock().unwrap();

    // (vscode) MARK: Extrapolation

    // for game objects
    for game_object in game_objects.iter_mut() {
      match game_object.object_type {
        GameObjectType::RaphaelleBullet | GameObjectType::CynewynnSword | GameObjectType::HernaniBullet | GameObjectType::RaphaelleBulletEmpowered
        | GameObjectType::ElizabethProjectileGroundRecalled | GameObjectType::ElizabethProjectileRicochet | GameObjectType::WiroGunShot
        | GameObjectType::TemerityRocket => {
          let speed: f32 = character_properties[&(match game_object.object_type {
            GameObjectType::RaphaelleBullet => Character::Raphaelle,
            GameObjectType::RaphaelleBulletEmpowered => Character::Raphaelle,
            GameObjectType::HernaniBullet => Character::Hernani,
            GameObjectType::CynewynnSword => Character::Cynewynn,
            GameObjectType::ElizabethProjectileRicochet => Character::Elizabeth,
            GameObjectType::ElizabethProjectileGroundRecalled => Character::Elizabeth,
            GameObjectType::WiroGunShot => Character::Wiro,
            GameObjectType::TemerityRocket => Character::Temerity,
            _ => panic!()
          })].primary_shot_speed;
          game_object.position.x += speed * game_object.get_bullet_data().direction.x * get_frame_time();
          game_object.position.y += speed * game_object.get_bullet_data().direction.y * get_frame_time();
        }
        _ => {},
      }
    }

    // MARK: Interpolation
    // for now this is just simple linear interpolation, no shenanigans yet.
    for player in other_players.iter_mut() {
      let distance = player.interpol_next - player.position;
      let cutoff = 7.0 * TILE_SIZE * get_frame_time();
      if distance.magnitude() > cutoff {
        let period = PACKET_INTERVAL;
        let speed = distance / period;
        //let speed = player.movement_direction * character_properties[&player.character].speed;
        player.position += distance.normalize() * (70.0 * speed.magnitude().powf(1.0/15.0) + 3.0 * speed.magnitude()) * get_frame_time() * 0.1;
      }
      //player.position += distance * PACKET_INTERVAL * get_frame_time() * 2.0;
      //player.position += distance * get_frame_time();
      //draw_line(player.position.x, player.position.y, player.interpol_next.x, player.interpol_next.y, 1.0*vh, PURPLE);
      // I can't get the interpolation to work, so temporarily I'll swap it with this very simple
      // extrapolation method.
      //player.position += player.movement_direction * character_properties[&player.character].speed * get_frame_time();
    }

    // MARK: Audio

    // Primary fire cast.
    if player.last_shot_time > character_properties[&player.character].primary_cooldown / 2.0 {
      player.used_primary = false;
    }
    else {
      if !player.used_primary {
        let sound: &[u8] = match player.character {
          Character::Hernani =>    include_bytes!("../assets/audio/gunshot.mp3"),
          Character::Cynewynn =>   include_bytes!("../assets/audio/whoosh.mp3"),
          Character::Raphaelle =>  {
            if player.stacks == 0 {
              include_bytes!("../assets/audio/whoosh.mp3")
            } else {
              include_bytes!("../assets/audio/whoosh.mp3")
            }
          },
          Character::Elizabeth =>  include_bytes!("../assets/audio/whoosh.mp3"),
          Character::Wiro =>       include_bytes!("../assets/audio/whoosh.mp3"),
          Character::Temerity =>   include_bytes!("../assets/audio/rpgshot.mp3"),
          Character::Dummy =>      include_bytes!("../assets/audio/whoosh.mp3"),
        };
        audio::play_sound(sound, &mut sfx_self_track);
        player.used_primary = true;
      }
    }

    {
      let mut sound_queue = sound_queue.lock().unwrap();
      for (i, sound) in sound_queue.clone().iter().enumerate() {
        audio::play_sound(sound.0, match sound.1 {
          AudioTrack::Music => &mut music_track,
          AudioTrack::SoundEffectSelf => &mut sfx_self_track,
          AudioTrack::SoundEffectOther => &mut sfx_other_track,
        });
        // don't play too many sounds.
        if i > 10 {
          break;
        }
      }
      sound_queue.clear();
    }

    let mut game_objects_copy = game_objects.clone();
    drop(game_objects);
    
    let other_players_copy = other_players.clone();
    drop(other_players);

    // readonly
    let player_copy = player.clone();
    
    // Do camera logic
    //camera_offset = Vector2::difference( player_copy.camera.position, player_copy.position);
    if !player_copy.is_dead {
      match settings.camera_smoothing {
        true => {
          // if delta_time is too long, the camera behaves very weirdly, so let's arficially assume
          // framerate never goes below 20fps.
          let safe_delta_time = f32::min(delta_time, 1.0/20.0);
          let camera_distance: Vector2 = Vector2::difference(player_copy.camera.position, player_copy.position);
          let camera_distance_mag = camera_distance.magnitude();
          let camera_smoothing: f32 = 1.5; // higher = less smoothing
          let safe_quadratic = f32::min(camera_distance_mag*camera_smoothing*10.0, (camera_distance_mag).powf(2.0)*camera_smoothing*5.0);
          let camera_movement_speed = safe_quadratic;

          player.camera.position += camera_distance.normalize() * safe_delta_time * camera_movement_speed;
        }
        false => {
          player.camera.position = player.position;
        }
      }
    }
    // (vscode) MARK: update mouse
    // update mouse position for the input thread to handle.
    // This hot garbage WILL be removed once camera is implemented correctly. Mayhaps.
    // But what this does is turn the mouse's screen coordinates into game coordinates,
    // the same type of coordinates the player uses
    //                        [-1;+1] range to [0;1] range          world      aspect      correct shenanigans related         center
    //                        conversion.                           coords     ratio       to cropping.
    //                     .------------------'-----------------.   ,-'-.   .----'---.  .---------------'--------------.   ,-------'----------,
    mouse_position.x =((((mouse_position_local().x + 1.0) / 2.0) * 100.0 * (16.0/9.0)) / (vw * 100.0)) * screen_width() - 50.0 * (16.0 / 9.0) + player_copy.camera.position.x; 
    mouse_position.y =((((mouse_position_local().y + 1.0) / 2.0) * 100.0             ) / (vh * 100.0)) * screen_height()- 50.0                + player_copy.camera.position.y;
    let keyboard_mode: MutexGuard<bool> = keyboard_mode.lock().unwrap();
    let mut aim_direction: Vector2 = Vector2::difference(player_copy.position, Vector2::from(mouse_position.clone()));
    if !*keyboard_mode {
      aim_direction = player_copy.aim_direction;
    }
    if *keyboard_mode {
      player.aim_direction = aim_direction;
    }
    {
      let input_halt = input_halt.lock().unwrap();
      if *input_halt {
        player.aim_direction = Vector2::new();
        aim_direction = Vector2::new();
      }
    }
    drop(player);
    drop(keyboard_mode);

    // (vscode) MARK: Draw

    // Draw the backgrounds
    clear_background(SKYBLUE);
    // TEMPORARY
    draw_rectangle(0.0, 0.0, 100.0 * vw, 100.0 * vh, Color { r: 0.55, g: 0.75, b: 0.5, a: 1.0 });
    for background_tile in background_tiles.clone() {
      let texture = &game_object_tetures[&background_tile.object_type];
      let size: Vector2 = Vector2 { x: TILE_SIZE, y: TILE_SIZE };
      graphics::draw_image_relative(texture, background_tile.position.x - size.x/2.0, background_tile.position.y - size.y/2.0, size.x, size.y, vh, player_copy.camera.position, Vector2::new(), WHITE);
    }

    // adjust certain positions.
    // adjust the location of Wiro's shield.
    for game_object_index in 0..game_objects_copy.len() {
      if game_objects_copy[game_object_index].object_type == GameObjectType::WiroShield {
        // if it's ours...
        if game_objects_copy[game_object_index].get_bullet_data().owner_username == username {
          let position: Vector2 = Vector2 {
            x: player_copy.position.x + player_copy.aim_direction.normalize().x * TILE_SIZE,
            y: player_copy.position.y + player_copy.aim_direction.normalize().y * TILE_SIZE,
          };

          game_objects_copy[game_object_index].position = position;
          let mut shield_data = game_objects_copy[game_object_index].get_bullet_data();
          shield_data.direction = player_copy.aim_direction.normalize();
          game_objects_copy[game_object_index].extra_data = ObjectData::BulletData(shield_data);
        }
      }
    }
    
    // draw all gameobjects
    game_objects_copy = sort_by_depth(game_objects_copy);
    for game_object in game_objects_copy.clone() {
      let texture = &game_object_tetures[&game_object.object_type];
      let size = match game_object.object_type {
        GameObjectType::Wall => Vector2 {x: 1.0 * TILE_SIZE, y: 2.0* TILE_SIZE},
        GameObjectType::UnbreakableWall => Vector2 {x: 1.0 * TILE_SIZE, y: 2.0* TILE_SIZE},
        GameObjectType::HernaniWall => Vector2 {x: 1.0 * TILE_SIZE, y: 2.0* TILE_SIZE},
        GameObjectType::HernaniBullet => Vector2 { x: TILE_SIZE * 1.0 * (10.0/4.0), y: TILE_SIZE * 1.0 },
        GameObjectType::RaphaelleBullet => Vector2 { x: TILE_SIZE*2.0, y: TILE_SIZE*2.0 },
        GameObjectType::RaphaelleBulletEmpowered => Vector2 { x: TILE_SIZE*2.0, y: TILE_SIZE*2.0 },
        GameObjectType::RaphaelleAura => Vector2 {x: character_properties[&Character::Raphaelle].secondary_range*2.0, y: character_properties[&Character::Raphaelle].secondary_range*2.0,},
        GameObjectType::WiroShield => Vector2 { x: TILE_SIZE*0.5, y: character_properties[&Character::Wiro].secondary_range },
        GameObjectType::TemerityRocketSecondary => Vector2 { x: TILE_SIZE*2.0, y: TILE_SIZE*2.0 },
        GameObjectType::CenterOrb => Vector2 { x: TILE_SIZE*2.0, y: TILE_SIZE*2.0 },
        GameObjectType::CynewynnSword => Vector2 { x: TILE_SIZE*3.0, y: TILE_SIZE*3.0 },
        _ => Vector2 {x: 1.0 * TILE_SIZE, y: 1.0* TILE_SIZE},
      };
      let shadow_offset: f32 = 5.0;

      // Draw shadows on certain objects
      let shaded_objects = vec![GameObjectType::RaphaelleBullet,
                                                     GameObjectType::RaphaelleBulletEmpowered,
                                                     GameObjectType::HernaniBullet,
                                                     GameObjectType::CynewynnSword,
                                                     GameObjectType::CenterOrb,
                                                     GameObjectType::ElizabethProjectileRicochet,
                                                    ];
      let rotation: Vector2 = match game_object.get_bullet_data_safe() {
        Ok(data) => {
          data.direction
        }
        Err(()) => {
          Vector2::new()
        }
      };
      if shaded_objects.contains(&game_object.object_type) {
        graphics::draw_image_relative(
          texture,
          game_object.position.x - size.x/2.0,
          game_object.position.y - size.y/2.0 + shadow_offset,
          size.x,
          size.y,
          vh, player_copy.camera.position,
          rotation,
          Color { r: 0.05, g: 0.0, b: 0.1, a: 0.15 });
      }
      graphics::draw_image_relative(texture, game_object.position.x - size.x/2.0, game_object.position.y - size.y/2.0, size.x, size.y, vh, player_copy.camera.position, rotation, WHITE);
    }



    // draw player and aim laser
    let mut range = character_properties[&player_copy.character].primary_range;
    if player_copy.character == Character::Temerity {
      if player_copy.stacks == 0 {
        range = character_properties[&player_copy.character].primary_range
      }
      if player_copy.stacks == 1 {
        range = character_properties[&player_copy.character].primary_range_2
      }
      if player_copy.stacks == 2 {
        range = character_properties[&player_copy.character].primary_range_3
      }
    }
    let relative_position_x = 50.0 * (16.0/9.0) + player_copy.position.x - player_copy.camera.position.x; //+ ((vh * (16.0/9.0)) * 100.0 )/ 2.0;
    let relative_position_y = 50.0              + player_copy.position.y - player_copy.camera.position.y; //+ (vh * 100.0) / 2.0;
    // test
    //let relative_position_x = main_camera.position.x;
    //let relative_position_y = main_camera.position.y;
    if !player_copy.is_dead {
      let mut range_limited: f32 = Vector2::distance(player_copy.position, Vector2::from(mouse_position.clone()));
      if range_limited > range {
        range_limited = range;
      }
      let low_limit = 10.0;
      if range_limited < low_limit {
        range_limited = low_limit;
      }
      // full line
      draw_line(
        (aim_direction.normalize().x * low_limit * vh) + relative_position_x * vh,
        (aim_direction.normalize().y * low_limit * vh) + relative_position_y * vh,
        (aim_direction.normalize().x * range * vh) + (relative_position_x * vh),
        (aim_direction.normalize().y * range * vh) + (relative_position_y * vh),
        0.6 * vh, Color { r: 1.0, g: 0.2, b: 0.0, a: 0.2 }
      );
      // shorter, matte line
      draw_line(
        (aim_direction.normalize().x * low_limit * vh) + relative_position_x * vh,
        (aim_direction.normalize().y * low_limit * vh) + relative_position_y * vh,
        (aim_direction.normalize().x * range_limited * vh) + (relative_position_x * vh),
        (aim_direction.normalize().y * range_limited * vh) + (relative_position_y * vh),
        0.4 * vh, Color { r: 1.0, g: 0.2, b: 0.0, a: 1.0 }
      );
      if player_copy.character == Character::Hernani {
        let range: f32 = character_properties[&Character::Hernani].secondary_range;
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

    // (vscode) MARK: Draw Players

    // draw players and optionally their trails
    let trail_y_offset: f32 = 4.5;
    for player in other_players_copy.clone() {
      if player.character == Character::Cynewynn && !player.is_dead {
        graphics::draw_lines(player.previous_positions.clone(), player_copy.camera.position, vh, player.team, trail_y_offset-0.0, 1.0);
        graphics::draw_lines(player.previous_positions.clone(), player_copy.camera.position, vh, player.team, trail_y_offset-0.3, 0.5);
        graphics::draw_lines(player.previous_positions,         player_copy.camera.position, vh, player.team, trail_y_offset-0.6, 0.25);
      }
    }
    if player_copy.character == Character::Cynewynn && !player_copy.is_dead {
      graphics::draw_lines(player_copy.previous_positions.clone(), player_copy.camera.position, vh, player_copy.team, trail_y_offset-0.0, 0.6);
      graphics::draw_lines(player_copy.previous_positions.clone(), player_copy.camera.position, vh, player_copy.team, trail_y_offset-0.3, 0.4);
      graphics::draw_lines(player_copy.previous_positions.clone(),         player_copy.camera.position, vh, player_copy.team, trail_y_offset-0.6, 0.2);
    }

    // Draw raphaelle's tethering.
    let mut all_players_copy: Vec<ClientPlayer> = other_players_copy.clone();
    all_players_copy.push(player_copy.clone());
    for player in all_players_copy.clone() {
      if player.character == Character::Raphaelle {
        for player_2 in all_players_copy.clone() {
          if Vector2::distance(player.position, player_2.position) < character_properties[&Character::Raphaelle].primary_range
          && player.team == player_2.team
          && (player.is_dead & player_2.is_dead) == false {
            // if on same team, green. If on enemy team, orange.
            let color = match player.team == player_copy.team {
              true => GREEN,
              false => ORANGE,
            };
            graphics::draw_line_relative(player.position.x, player.position.y, player_2.position.x, player_2.position.y, 0.5, color, player_copy.camera.position, vh);
          }
        }
      }
    }

    // MARK: UI
    // temporary ofc
    if !player_copy.is_dead {
      player_copy.draw(&player_textures[&player_copy.character], vh, player_copy.camera.position, &health_bar_font, character_properties[&player_copy.character].clone(), settings.clone());
    }
    for player in other_players_copy.clone() {
      if !player.is_dead {
        player.draw(&player_textures[&player.character], vh, player_copy.camera.position, &health_bar_font, character_properties[&player.character].clone(), settings.clone());
      }
    }
    if player_copy.is_dead {
      draw_text("You dead rip", 20.0*vh, 50.0*vh, 20.0*vh, RED);
    }
    // time, kills, rounds
    let gamemode_info_main = gamemode_info.lock().unwrap();

    if gamemode_info_main.time < 3 {
      if gamemode_info_main.rounds_won_blue > prev_gamemode_info.rounds_won_blue
      && gamemode_info_main.rounds_won_red > prev_gamemode_info.rounds_won_red {
        draw_text("It's a draw!", 20.0*vh, 50.0*vh, 17.0*vh, BLUE);
      }
      if gamemode_info_main.rounds_won_blue > prev_gamemode_info.rounds_won_blue {
        draw_text("Blue wins this round!", 20.0*vh, 50.0*vh, 15.0*vh, BLUE);
      }
      if gamemode_info_main.rounds_won_red > prev_gamemode_info.rounds_won_red {
        draw_text("Red wins this round!", 20.0*vh, 50.0*vh, 15.0*vh, RED);
      }
    } else if gamemode_info_main.time < 5 {
      prev_gamemode_info = gamemode_info_main.clone();
    }
    //if gamemode_info_main.rounds_won_red >= ROUNDS_TO_WIN {
    //  draw_text("Red wins the game!", 20.0*vh, 50.0*vh, 15.0*vh, RED);
    //}
    //if gamemode_info_main.rounds_won_blue >= ROUNDS_TO_WIN {
    //  draw_text("Blue wins the game!", 20.0*vh, 50.0*vh, 15.0*vh, BLUE);
    //}

    // let timer_width: f32 = 5.0;
    draw_rectangle((50.0-20.0)*vw, 0.0, 40.0 * vw, 10.0*vh, Color { r: 1.0, g: 1.0, b: 1.0, a: 0.5 });
    graphics::draw_text_relative(format!("Time: {}", gamemode_info_main.time.to_string().as_str()).as_str(), -7.0, 6.0, &health_bar_font, 4, vh, Vector2 { x: 0.0, y: 50.0 }, BLACK);
    graphics::draw_text_relative(format!("Remaining: {}", gamemode_info_main.alive_blue.to_string().as_str()).as_str(), 10.0, 4.0, &health_bar_font, 4, vh, Vector2 { x: 0.0, y: 50.0 }, BLUE);
    graphics::draw_text_relative(format!("Rounds won: {}", gamemode_info_main.rounds_won_blue.to_string().as_str()).as_str(), 10.0, 8.0, &health_bar_font, 4, vh, Vector2 { x: 0.0, y: 50.0 }, BLUE);
    graphics::draw_text_relative(format!("Remaining: {}", gamemode_info_main.alive_red.to_string().as_str()).as_str(), -33.0, 4.0, &health_bar_font, 4, vh, Vector2 { x: 0.0, y: 50.0 }, RED);
    graphics::draw_text_relative(format!("Rounds won: {}", gamemode_info_main.rounds_won_red.to_string().as_str()).as_str(), -33.0, 8.0, &health_bar_font, 4, vh, Vector2 { x: 0.0, y: 50.0 }, RED);
    // let bar_offsets = 5.0;
    // draw_line_relative(bar_offsets+10.0, 100.0 -bar_offsets, bar_offsets + (player_copy.health-50) as f32 , 100.0 - bar_offsets, 3.0, GREEN, Vector2 { x: 100.0, y: 50.0 }, vh);
    drop(gamemode_info_main);

    // Ability icons
    let ability_info_box: ui::DivBox =ui:: DivBox { position: Vector2 { x: 5.0, y: 83.0 }, nested: Vec::new() };
    let primary_cooldown: f32 = if player_copy.last_shot_time < character_properties[&player_copy.character].primary_cooldown {
      player_copy.last_shot_time / character_properties[&player_copy.character].primary_cooldown
    } else {
      1.0
    };
    let mut secondary_cooldown: f32 = player_copy.secondary_charge as f32 / 100.0;
    if character == Character::Wiro {
      if player_copy.last_secondary_time < character_properties[&Character::Wiro].secondary_cooldown {
        secondary_cooldown = player_copy.last_secondary_time / character_properties[&Character::Wiro].secondary_cooldown;
      }
    }

    let dash_cooldown: f32 = if player_copy.time_since_last_dash < character_properties[&player_copy.character].dash_cooldown {
      player_copy.time_since_last_dash / character_properties[&player_copy.character].dash_cooldown
    } else {
      1.0
    };
    ui::draw_ability_icon(ability_info_box.rel_pos(Vector2 { x: 37.5, y: 0.0 }), Vector2 { x: 10.0, y: 10.0 }, 2, player_copy.shooting_secondary, secondary_cooldown , vh, vw, &health_bar_font, character_descriptions.clone(), character);
    ui::draw_ability_icon(ability_info_box.rel_pos(Vector2 { x: 25.0, y: 0.0 }), Vector2 { x: 10.0, y: 10.0 }, 3, player_copy.dashing, dash_cooldown , vh, vw, &health_bar_font, character_descriptions.clone(), character);
    ui::draw_ability_icon(ability_info_box.rel_pos(Vector2 { x: 12.5,  y: 0.0 }), Vector2 { x: 10.0, y: 10.0 }, 1, player_copy.shooting_primary, primary_cooldown , vh, vw, &health_bar_font, character_descriptions.clone(), character);
    ui::draw_ability_icon(ability_info_box.rel_pos(Vector2 { x:  0.0, y: 0.0 }), Vector2 { x: 10.0, y: 10.0 }, 4, false, 1.0 , vh, vw, &health_bar_font, character_descriptions.clone(), character);

    let mut red_team_players: u8  = 0;
    let mut blue_team_players :u8 = 0;
    let height: f32 = 7.5;
    let red_team_box: ui::DivBox = ui::DivBox { position: Vector2 { x: (85.0 / vh) * vw, y: height }, nested: Vec::new() };
    let blue_team_box: ui::DivBox = ui::DivBox { position: Vector2 { x: 5.0, y: height }, nested: Vec::new() };
    let mut all_players = other_players_copy.clone();
    all_players.push(player_copy.clone());
    for player in all_players {

      match player.team {
        Team::Blue => {
          blue_team_players += 1;
          ui::draw_player_info(blue_team_box.rel_pos(Vector2 { x: 0.0, y: 10.0 * blue_team_players as f32 }), 10.0, player, &health_bar_font, vh, settings.clone());
        },
        Team::Red => {
          red_team_players += 1;
          ui::draw_player_info(red_team_box.rel_pos(Vector2 { x: 0.0, y: 10.0 * red_team_players as f32 }), 10.0, player, &health_bar_font, vh, settings.clone());
        }
      }
    }

    if player_copy.ping != 0 {
      connected_to_server = true;
    }
    if !connected_to_server {
      draw_text(format!("Not connected to server").as_str(), 20.0, 80.0, 40.0, RED);
    }

    // chat box
    let chatbox_position = Vector2{x: 5.0 * vw, y: 20.0 * vh};
    let chatbox_size = Vector2{x: 30.0 * vw, y: 70.0 * vh};
    ui::chatbox(chatbox_position, chatbox_size, server_interaction.friend_list.clone(), &mut server_interaction.is_chatbox_open, &mut server_interaction.selected_friend, &mut server_interaction.recv_messages_buffer, &mut server_interaction.chat_input_buffer, &mut server_interaction.chat_selected, vh, player_copy.username.clone(), &mut packet_queue, &mut server_interaction.chat_scroll_index, true, &mut chat_timer);


    // Draw pause menu
    if menu_paused {
      let mut kill_all_threads = kill_all_threads.lock().unwrap();
      (menu_paused, *kill_all_threads) = ui::draw_pause_menu(vh, vw, &mut settings, &mut settings_open_flag, settings_tabs, (&mut sfx_self_track, &mut sfx_other_track, &mut music_track));
      drop(kill_all_threads);
      // Draw fps, etc
      if timer_for_text_update.elapsed().as_secs_f32() > 0.5 {
        timer_for_text_update = Instant::now();

        let sender_fps: Arc<Mutex<f32>> = Arc::clone(&sender_fps);
        let sender_fps: MutexGuard<f32> = sender_fps.lock().unwrap();
        slow_sender_fps = *sender_fps;
        drop(sender_fps);

        slow_draw_fps = get_fps();

        slow_ping = player_copy.ping;
      }
      draw_text(format!("{} draw fps", slow_draw_fps).as_str(), 20.0, 20.0, 32.0, BLACK);
      draw_text(format!("{} input fps", slow_sender_fps).as_str(), 20.0, 45.0, 32.0, BLACK);
      draw_text(format!("{} ms ping", slow_ping).as_str(), 20.0, 70.0, 32.0, BLACK);
    }


    // MARK: MMS Listen
    if let Some(ref mut server_stream) = server_interaction.server_stream {
      let mut buffer: [u8; 2048] = [0; 2048];
      match server_stream.read(&mut buffer) {
        Ok(len) => {
          let packets = network::tcp_decode_decrypt::<ServerToClientPacket>(buffer[..len].to_vec(), cipher_key.clone(), &mut main_last_nonce);
          let packets = match packets {
            Ok(packets) => packets,
            Err(_) => {
              continue;
            }
          };
          for packet in packets {
            match packet.information {
              ServerToClient::ChatMessage(sender, message, message_type) => {
                // update friend list
                for f_index in 0..server_interaction.friend_list.len() {
                  if sender == database::get_friend_name(&username, &server_interaction.friend_list[f_index].0) {
                    server_interaction.friend_list[f_index].2 = true;
                  }
                }
                server_interaction.recv_messages_buffer.push((sender, message, message_type));
                chat_timer = Instant::now();
              }
              ServerToClient::MatchEnded(data) => {
                // make sure that this game end packet is indeed from this game, and not another one.
                if data.game_id == game_id {
                  game_ended = true;
                  // wait a bit to give the player time to process what happened.
                  game_ended_timer = Instant::now();
                  winning_team = data.winning_team;
                }
              }
              ServerToClient::GameServerCrashApology => {
                game_ended_timer = Instant::now();
                server_crashed = true;
              }
              ServerToClient::MatchAssignment(_match_assignment_data) => {
                // fix this later.
              }
              _ => {}
            }
          }
        },
        Err(error) => {
          match error.kind() {
            ErrorKind::WouldBlock => {
              
            }
            _ => {
              //println!("{:?}", error);
            }
          }
        }
      }
      for packet in packet_queue.clone() {
        server_stream.write_all(
          &network::tcp_encode_encrypt(packet, cipher_key.clone(), *main_nonce).expect("oops")
        ).expect("idk 1");
        *main_nonce += 1;
      }
      packet_queue = Vec::new();
    }
    if game_ended {
      if winning_team == player_copy.team {
        draw_text("Victory", 40.0*vw, 50.0*vh, 15.0*vh, BLUE);
      }
      else {
        draw_text("Defeat", 40.0*vw, 50.0*vh, 15.0*vh, RED);
      }
    }
    if server_crashed {
      draw_text("Server crashed. Sorry.", 10.0*vw, 50.0*vh, 15.0*vh, BLUE);
    }
    if (game_ended || server_crashed) && game_ended_timer.elapsed().as_secs_f32() > 4.0 {
      {
        // close the game and go back to the menu.
        // kill_all_threads is like "return" but gracefully stops all other threads.
        let mut kill_all_threads = kill_all_threads.lock().unwrap();
        *kill_all_threads = true;
      }
    }
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
fn input_listener_network_sender(player: Arc<Mutex<ClientPlayer>>, game_objects: Arc<Mutex<Vec<GameObject>>>, sender_fps: Arc<Mutex<f32>>, kill: Arc<Mutex<bool>>, global_keyboard_mode: Arc<Mutex<bool>>, client_port: u16, other_players: Arc<Mutex<Vec<ClientPlayer>>>, gamemode_info: Arc<Mutex<GameModeInfo>>, server_port: u16, cipher_key: Vec<u8>, input_halt: Arc<Mutex<bool>>, server_ip: String, sound_queue: Arc<Mutex<Vec<(&[u8], AudioTrack)>>>) -> () {

  let server_ip: Vec<&str> = server_ip.split(":").collect();
  let server_ip: String = format!("{}:{}", server_ip[0], server_port);


  // let server_ip: String = format!("{}", server_ip);
  // create the socket for sending info.
  let sending_ip: String = format!("0.0.0.0:{}", client_port);
  println!("{:?}, {:?}", client_port, server_port);
  let socket: UdpSocket = UdpSocket::bind(sending_ip)
    .expect("Could not bind client sender socket");

  socket.set_nonblocking(true).expect("idk");
  socket.set_read_timeout(Some(Duration::from_millis(100))).expect("Failed to set timeout ig...");
  // if we get another Io(Kind(UnexpectedEof)) then this buffer is too small
  const MUL: usize = 40;
  let mut buffer: [u8; 4096*MUL] = [0; 4096*MUL];

  let character_properties: HashMap<Character, CharacterProperties> = load_characters();

  // initiate gamepad stuff
  let mut gilrs = Gilrs::new().expect("Gilrs failed");
  let mut active_gamepad: Option<GamepadId> = None;
  // temporary
  let controller_deadzone: f32 = 0.3;


  let mut frame_counter:   Instant = Instant::now();
  let mut network_counter: Instant = Instant::now();

  // whether to enforce the frame time limit
  let frame_time_locked: bool = true;
  // applies to movement and network listen
  let desired_frame_time: f32 = 1.0 / 500.0;
  // only applies to network sending rate.
  let desired_network_time: f32 = PACKET_INTERVAL;

  // Whether in keyboard or controller mode.
  // Ignore mouse pos in controller mode for example.
  let mut keyboard_mode: bool = true;

  let mut toggle_time: Instant = Instant::now();

  let interpolate = true;

  let mut nonce: u32 = 1;
  let mut last_nonce: u32 = 0;

  loop {

    let delta_time = frame_counter.elapsed().as_secs_f32();
    frame_counter = Instant::now();

    let kill_this_thread: MutexGuard<bool> = kill.lock().unwrap();
    if *kill_this_thread {
      drop(socket);
      return;
    }
    drop(kill_this_thread);


    // MARK: recieve packet
    let recieved_server_info: ServerPacket;
    match socket.recv_from(&mut buffer) {
      Ok(data) => {
        let (amt, _): (usize, std::net::SocketAddr) = data;
        let data: &[u8] = &buffer[..amt];


        // get nonce
        let recv_nonce = &buffer[..4];
        let recv_nonce = match bincode::deserialize::<u32>(&recv_nonce){
          Ok(nonce) => nonce,
          Err(_) => {
            continue;
          }
        };
        if recv_nonce <= last_nonce {
          continue;
        }
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[8..].copy_from_slice(&recv_nonce.to_be_bytes());
        let formatted_nonce = Nonce::from_slice(&nonce_bytes);
        
        let key = cipher_key.clone();
        let key = GenericArray::from_slice(&key.as_slice());
        let cipher = ChaCha20Poly1305::new(key);
        
        let deciphered = match cipher.decrypt(&formatted_nonce, data[4..].as_ref()) {
          Ok(decrypted) => {
            decrypted
          },
          Err(_err) => {
            continue; // this is an erroneous packet, ignore it.
          },
        };
        recieved_server_info = match bincode::deserialize::<ServerPacket>(&deciphered) {
          Ok(packet) => packet,
          Err(_err) => {
            continue; // ignore invalid packet
          }
        };
        last_nonce = recv_nonce;

        //recieved_server_info = bincode::deserialize(data).expect("Could not deserialise server packet.");
        // println!("CLIENT: Received from {}: {:?}", src, recieved_server_info);
        let mut player: MutexGuard<ClientPlayer> = player.lock().unwrap();

        // If we're interpolating dashes, then the server should tell us when we're dashing.
        if interpolate {
          player.is_dashing = recieved_server_info.player_packet_is_sent_to.is_dashing;
        }

        // if we sent an illegal position, and server does a position override:
        if recieved_server_info.player_packet_is_sent_to.override_position {
          // If we're interpolating dashes *and* we're dashing, update interpolation info.
          if interpolate && player.is_dashing {
            // But if we're dashing (interpolating is set to true), then prepare to smoothly translate to that position.
            player.interpol_next = recieved_server_info.player_packet_is_sent_to.position_override;
            player.interpol_prev = player.position; // current position
          }
          // but under standard behaviour just teleport the player there.
          else {
            player.position = recieved_server_info.player_packet_is_sent_to.position_override;
          }
          println!("Recieved position override.");
        }

        let ping = match recieved_server_info.timestamp.elapsed() {
          Ok(val) => val.as_millis(),
          Err(_) => 0,
        };
        player.ping = ping as u16;
        player.health = recieved_server_info.player_packet_is_sent_to.health;
        player.secondary_charge = recieved_server_info.player_packet_is_sent_to.secondary_charge;
        player.character = recieved_server_info.player_packet_is_sent_to.character;
        player.is_dead = recieved_server_info.player_packet_is_sent_to.is_dead;
        player.buffs = recieved_server_info.player_packet_is_sent_to.buffs;
        player.previous_positions = recieved_server_info.player_packet_is_sent_to.previous_positions;
        player.team = recieved_server_info.player_packet_is_sent_to.team;
        player.last_shot_time = recieved_server_info.player_packet_is_sent_to.time_since_last_primary;
        player.time_since_last_dash = recieved_server_info.player_packet_is_sent_to.time_since_last_dash;
        player.last_secondary_time = recieved_server_info.player_packet_is_sent_to.time_since_last_secondary;
        player.stacks = recieved_server_info.player_packet_is_sent_to.stacks;


        let mut game_objects = game_objects.lock().unwrap();
        let mut sound_queue = sound_queue.lock().unwrap();
        // update game objects and calculate sound.
        // MARK: Sound
        let mut highest_id = 0;
        for prev_object in game_objects.clone() {
          if prev_object.id > highest_id {
            highest_id = prev_object.id;
          }
          let mut new_object_found: bool = false;
          for new_object in recieved_server_info.game_objects.clone() {
            // if this object was UPDATED...
            if new_object.id == prev_object.id {
              new_object_found = true;
              println!("1");
              match new_object.get_bullet_data_safe() {
                Ok(new_bullet_data) => {
                println!("2");

                  // this bullet hit a player.
                  if new_bullet_data.hit_players.len() > prev_object.get_bullet_data().hit_players.len() {
                    println!("3");

                    let track: AudioTrack;
                    // if this is our own bullet
                    if new_bullet_data.owner_username == player.username {
                      track = AudioTrack::SoundEffectSelf;
                    } else {
                      track = AudioTrack::SoundEffectOther;
                    }
                    sound_queue.push(match new_object.object_type {
                      GameObjectType::HernaniBullet =>               (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                      GameObjectType::CynewynnSword =>               (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                      GameObjectType::ElizabethProjectileRicochet => (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                      GameObjectType::ElizabethTurretProjectile =>   (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                      GameObjectType::RaphaelleBullet =>             (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                      GameObjectType::RaphaelleBulletEmpowered =>    (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                      GameObjectType::TemerityRocket =>              (include_bytes!("../assets/audio/explosion.mp3"), track),
                      GameObjectType::TemerityRocketSecondary =>     (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                      GameObjectType::WiroGunShot =>                 (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                      _ => {continue;}
                    });
                  }
                }
                Err(()) => {

                }
              }
              match prev_object.object_type {
                _ => {}
              }
            }
          }
          // if this object was DELETED...
          if !new_object_found {
            match prev_object.object_type {
              _ => {}
            }
          }
        }
        for (new_object) in recieved_server_info.game_objects.clone() {
          // if this object is NEW...
          if new_object.id > highest_id {
            // check if it's ours. If it is, skip it.
            let mut track: AudioTrack = AudioTrack::SoundEffectOther;
            let mut is_own: bool = false;
            match new_object.get_bullet_data_safe() {
              Ok(data) => {

                if data.owner_username == player.username {
                  track = AudioTrack::SoundEffectSelf;
                  is_own = true;
                }
              }
              Err(()) => { }
            }
            if !is_own {
              sound_queue.push(match new_object.object_type {
                GameObjectType::HernaniBullet =>               {(include_bytes!("../assets/audio/gunshot.mp3"), track)}
                GameObjectType::CynewynnSword =>               {(include_bytes!("../assets/audio/whoosh.mp3"), track)}
                GameObjectType::ElizabethProjectileRicochet => {(include_bytes!("../assets/audio/whoosh.mp3"), track)}
                GameObjectType::ElizabethTurretProjectile =>   {(include_bytes!("../assets/audio/whoosh.mp3"), track)}
                GameObjectType::RaphaelleBullet =>             {(include_bytes!("../assets/audio/whoosh.mp3"), track)}
                GameObjectType::RaphaelleBulletEmpowered =>    {(include_bytes!("../assets/audio/whoosh.mp3"), track)}
                GameObjectType::WiroGunShot =>                 {(include_bytes!("../assets/audio/whoosh.mp3"), track)}
                GameObjectType::TemerityRocket =>              {(include_bytes!("../assets/audio/rpgshot.mp3"), track)}
                GameObjectType::TemerityRocketSecondary =>     {(include_bytes!("../assets/audio/rpgshot.mp3"), track)}
                _ => {
                  continue;
                }
              });
            }
            // if this object spawned with a player already in hit_players, remove it,
            // so next frame the object update can register it as a hit.
            match new_object.get_bullet_data_safe() {
              Ok(data) => {
                if !data.hit_players.is_empty() {
                  sound_queue.push(match new_object.object_type {
                    GameObjectType::HernaniBullet =>               (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                    GameObjectType::CynewynnSword =>               (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                    GameObjectType::ElizabethProjectileRicochet => (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                    GameObjectType::ElizabethTurretProjectile =>   (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                    GameObjectType::RaphaelleBullet =>             (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                    GameObjectType::RaphaelleBulletEmpowered =>    (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                    GameObjectType::TemerityRocket =>              (include_bytes!("../assets/audio/explosion.mp3"), track),
                    GameObjectType::TemerityRocketSecondary =>     (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                    GameObjectType::WiroGunShot =>                 (include_bytes!("../assets/audio/sword-hit.mp3"), track),
                    _ => {continue;}
                  });
                }
              }
              Err(()) => {}
            }
          }
        }
        *game_objects = recieved_server_info.game_objects;
        drop(player); // free mutex guard ASAP for other thread to access player.
        drop(sound_queue);
        drop(game_objects);

        let mut other_players = other_players.lock().unwrap();
        let mut recieved_players: Vec<ClientPlayer> = Vec::new();
        for player in recieved_server_info.players {
          recieved_players.push(ClientPlayer::from_otherplayer(player));
        }
        // if a player left the game, recieved players has one less players, and other_players needs to
        // be adjusted since we index over other_players.
        other_players.retain(|element| {
          for player in recieved_players.clone() {
            if player.username == element.username {
              return true;
            }
          }
          return false;
        });

        // if a new player joins, skip this part, update directly.
        if other_players.len() == recieved_players.len() {
          for player_index in 0..recieved_players.len() {
            // new position
            recieved_players[player_index].interpol_prev = other_players[player_index].interpol_next;
            recieved_players[player_index].interpol_next = recieved_players[player_index].position;
            recieved_players[player_index].position = other_players[player_index].position;

            recieved_players[player_index].used_primary = other_players[player_index].used_primary;
            recieved_players[player_index].used_secondary = other_players[player_index].used_secondary;
            recieved_players[player_index].used_dash = other_players[player_index].used_dash;
            // previous position
            // if not moving, force a position
            //recieved_players[player_index].position = Vector2 { x: 0.0, y: 0.0 }; //other_players[player_index].position;
            //recieved_players[player_index].interpol_prev = other_players[player_index].position;
          }
        }
        *other_players = recieved_players;
        drop(other_players);

        let mut gamemode_info_listener = gamemode_info.lock().unwrap();
        *gamemode_info_listener = recieved_server_info.gamemode_info;
        drop (gamemode_info_listener)
      }
      Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
        // do nothing lol idc
      }
      Err(_) => {
        println!("error while recieving data.");
      }
    }


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
        Keycode::F10 => {
          // Dirty solution but works.
          if toggle_time.elapsed().as_secs_f32() > 0.05 {
            set_window_size(800, 450);
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
  

    // MARK: Movement calc
    // calc = slang for calculator

    // janky but good enough to correct controllers that give weird inputs.
    // should not happen on normal controllers anyways.
    // also corrects keyboard input.
    if movement_vector.magnitude() > 1.0 {
      // println!("normalizing");
      movement_vector = movement_vector.normalize();
    }

    let input_halt = input_halt.lock().unwrap();

    if *input_halt {
      movement_vector = Vector2::new();
      dashing = false;
      shooting_primary = false;
      shooting_secondary = false;
    }
    drop(input_halt);

    // expresses the player's movement without the multiplication
    // by delta time and speed. Sent to the server.
    let mut movement_vector_raw: Vector2 = movement_vector;

    // the server tells us if we're dashing or not when we're in interpolation mode.
    if interpolate && player.is_dashing {
      // do the interpolate
      let distance = player.interpol_next - player.interpol_prev;
      let speed: Vector2;
      if distance.magnitude() == 0.0 {
        // this is only true on the first "frame".
        // this measure helps reduce the percieved lag from the character standing still
        // before it obtains its second interpolation position.
        speed = player.movement_direction * (character_properties[&player.character].dash_speed / 2.0) * delta_time;
      } else {
        // this runs the rest of the time
        let period = PACKET_INTERVAL;
        speed = distance / period;
      }
      player.position += speed * delta_time;
    }
    else {
      if dashing && !player.is_dashing && !player.is_dead && movement_vector_raw.magnitude() != 0.0 {
        if player.time_since_last_dash > character_properties[&player.character].dash_cooldown {
          match player.character {
            Character::Temerity => {
            }
            _ => {
              player.is_dashing = true;
            }
          }
        }
      }
    
      if player.is_dashing {
        (player.position, player.dashed_distance, player.is_dashing) = dashing_logic(
          player.is_dashing,
          player.dashed_distance,
          movement_vector_raw,
          delta_time as f64,
          character_properties[&player.character].dash_speed,
          character_properties[&player.character].dash_distance,
          game_objects.clone(),
          player.position,
        );
      }
    }

    // Apply standard movement (non-dashing)
    if !player.is_dashing {
      let mut extra_speed: f32 = 0.0;
      for buff in player.buffs.clone() {
        if vec![BuffType::Speed, BuffType::WiroSpeed].contains(&buff.buff_type) {
          extra_speed += buff.value;
        }
        if buff.buff_type == BuffType::Impulse {
          // yeet
          let direction = buff.direction.normalize();
          // time left serves as impulse decay
          let time_left = buff.duration;
          let strength = buff.value;
          movement_vector += direction * f32::powi(time_left, 1) * strength;
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
    }

    // println!("{:?}", player.position);
    // println!("{:?}", movement_vector);
    // println!("{:?}", movement_vector_raw);
    // println!("{:?}", keyboard_mode);

    // update player info
    player.shooting_primary = shooting_primary;
    player.dashing = dashing;
    player.shooting_secondary = shooting_secondary;


    // (vscode) MARK: send packet
    let network_elapsed = network_counter.elapsed().as_secs_f32();
    if network_elapsed > desired_network_time {
      // reset counter
      network_counter = Instant::now();

      // create the packet to be sent to server.
      let client_packet: ClientPacket = ClientPacket {
        position:      player.position,
        movement:      movement_vector_raw,
        aim_direction: player.aim_direction,
        shooting_primary,
        shooting_secondary,
        packet_interval: network_elapsed,
        dashing,
        character: player.character,
        timestamp: SystemTime::now(), // ping!
      };

      // send data to server
      let serialized_packet: Vec<u8> = bincode::serialize(&client_packet).expect("Failed to serialize message");
      let mut nonce_bytes = [0u8; 12];
      nonce_bytes[8..].copy_from_slice(&nonce.to_be_bytes());
      
      let formatted_nonce = Nonce::from_slice(&nonce_bytes);
      let cipher_key = cipher_key.clone();
      let key = GenericArray::from_slice(&cipher_key);
      let cipher = ChaCha20Poly1305::new(&key);
      let ciphered = cipher.encrypt(&formatted_nonce, serialized_packet.as_ref()).expect("shit");
      
      let serialized_nonce: Vec<u8> = bincode::serialize::<u32>(&nonce).expect("oops");
      let serialized = [&serialized_nonce[..], &ciphered[..]].concat();
      socket.send_to(&serialized, server_ip.clone()).expect("Failed to send packet to server. Is your IP address correct?");
      nonce += 1;
    }
    // drop mutexguard ASAP so other threads can use player ASAP.
    drop(player);

    let mut update_keyboard_mode: MutexGuard<bool> = global_keyboard_mode.lock().unwrap();
    *update_keyboard_mode = keyboard_mode;
    drop(update_keyboard_mode);
    
    // update delta_time and reset counter.
    let delta_time_difference: f32 = desired_frame_time - delta_time;
    if delta_time_difference > 0.0 && frame_time_locked /* only if input fps is limited */ {
      std::thread::sleep(Duration::from_secs_f32(delta_time_difference));
    }

    let mut sender_fps: MutexGuard<f32> = sender_fps.lock().unwrap();
    *sender_fps = (1.0 / delta_time).round();
    drop(sender_fps);

  }
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
  let extra_offset_x: u16 = 9;
  let extra_offset_y: u16 = 5;
  for x in 0..map_size_x + (extra_offset_x*2) {
    for y in 0..map_size_y + (extra_offset_y*2) {
      let random_num_raw = rand();
      let mut random_num_f = (random_num_raw as f64) / u32::MAX as f64;
      random_num_f *= 6.0;
      let random_num = random_num_f.round() as usize;
      let pos_x: i16 = x.try_into().unwrap();
      let pos_x: f32 = (pos_x - extra_offset_x as i16) as f32 * TILE_SIZE;
      let pos_y: i16 = y.try_into().unwrap();
      let pos_y: f32 = (pos_y - extra_offset_y as i16) as f32 * TILE_SIZE + TILE_SIZE*0.5;
      if (x + y) % 2 == 1 {
        tiles.push(BackGroundTile { position: Vector2 { x: pos_x, y: pos_y }, object_type: bright_tiles[random_num].clone() });
      } else {
        tiles.push(BackGroundTile { position: Vector2 { x: pos_x, y: pos_y }, object_type: dark_tiles[random_num].clone() });
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
  // "lowest" layer
  let mut sorted_objects_layer_1: Vec<GameObject> = Vec::new();
  // "highest" layer
  let mut sorted_objects_layer_2: Vec<GameObject> = Vec::new();

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
    match unsorted_objects[current_lowest_index].object_type {
      GameObjectType::RaphaelleAura   => { sorted_objects_layer_1.push(unsorted_objects[current_lowest_index].clone()); }
      GameObjectType::Water1          => { sorted_objects_layer_1.push(unsorted_objects[current_lowest_index].clone()); }
      GameObjectType::Water2          => { sorted_objects_layer_1.push(unsorted_objects[current_lowest_index].clone()); }
      GameObjectType::HernaniLandmine => { sorted_objects_layer_2.push(unsorted_objects[current_lowest_index].clone()); }
      _                               => { sorted_objects_layer_2.push(unsorted_objects[current_lowest_index].clone()); }
    }
    unsorted_objects.remove(current_lowest_index);
  }
  if sorted_objects_layer_1.is_empty() {
    return sorted_objects_layer_2;
  }
  if sorted_objects_layer_2.is_empty() {
    return sorted_objects_layer_1;
  }
  return [&sorted_objects_layer_1[0..sorted_objects_layer_1.len()], &sorted_objects_layer_2[0..sorted_objects_layer_2.len()]].concat();

}

pub struct MainServerInteraction {
  // chat fields
  pub server_stream: Option<TcpStream>,
  pub is_chatbox_open: bool,
  pub selected_friend: usize,
  pub recv_messages_buffer: Vec<(String, String, ChatMessageType)>,
  pub chat_input_buffer: String,
  pub chat_selected: bool,
  pub chat_scroll_index: usize,
  pub friend_list: Vec<(String, FriendShipStatus, bool)>,
  pub lobby_invites: Vec<String>,
  pub lobby: Vec<LobbyPlayerInfo>,
}

#[derive(Clone, Debug, Copy)]
pub enum AudioTrack {
  Music,
  SoundEffectSelf,
  SoundEffectOther,
}