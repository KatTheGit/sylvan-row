/// Functions and structs related to any form of maths
/// or logic, like `Vector2` or movement logic functions.
pub mod maths;
/// Constant parameters, like , DEFAULT_IP_ADDRESS, etc...
pub mod const_params;
/// Gameobjects, Character Properties, any data that expresses anything regarding
/// the game, used by both client and server.
pub mod gamedata;
///// Functions and structs related to drawing things to the screen
//pub mod graphics;
/// Game server. Called by mothership when creating an instance for players.
pub mod gameserver;
/// Structs that need to be shared between the client and the mothership server.
pub mod mothership_common;
/// Interface for the server's database.
pub mod database;
/// Filters for the chat, user registration, etc...
pub mod filter;
/// Netcode
pub mod network;
///// The actual game, once it's connected to the server.
//pub mod game;
/// Immediate mode rendering wrapper for Bevy.
pub mod bevy_immediate;
/// Higher level wrapper for any graphics.
pub mod bevy_graphics;
/// Audio wrapper for bevy.
pub mod bevy_audio;

use std::{collections::HashMap, io::{ErrorKind, Read, Write}, net::{TcpStream, UdpSocket}, time::{Duration, Instant, SystemTime}};
use bevy::{color::palettes::css::*, input::{keyboard::KeyboardInput, mouse::MouseWheel}, prelude::*, tasks::futures_lite::io::Sink, ui, window::WindowResolution, winit::{UpdateMode, WinitSettings}};
use bevy_immediate::*;
use bevy_graphics::*;
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce};
use maths::*;
use opaque_ke::{generic_array::GenericArray, ClientLogin, ClientLoginFinishParameters, ClientLoginStartResult, ClientRegistration, ClientRegistrationFinishParameters, ClientRegistrationStartResult};
use rand::rngs::OsRng;
use ring::hkdf;
use crate::{bevy_graphics::Button, const_params::*, database::{get_friend_request_type, FriendShipStatus}, filter::{valid_password, valid_username}, gamedata::*, gameserver::game_server, mothership_common::{ChatMessageType, ClientToServer, ClientToServerPacket, GameMode, LobbyPlayerInfo, MatchRequestData, PlayerInfo, PlayerMessage, PlayerStatistics, RefusalReason, ServerToClient, ServerToClientPacket}, network::get_ip};
use device_query::{DeviceQuery, DeviceState, Keycode};

const CURRENT_SERVER_IP: &str = "13.38.240.14:25569";

#[bevy_main]
pub fn main() {
  App::new()
    .add_systems(Startup, setup)
    .add_systems(PreUpdate, sprite_clearer)
    .add_systems(PostUpdate, exit_catcher)
    .add_systems(Update, main_thread)
    .add_plugins(
      DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
          title: "Sylvan Row".into(),
          name: Some("sylvan.row".into()),
          //resolution: WindowResolution::new(720, 480),
          //#[cfg(target_os="android")]
          mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
          present_mode: bevy::window::PresentMode::Immediate, // vsync fucking sucks
          ..default()
        }),
        ..default()
      })
    )
    .run();
}

#[derive(Resource)]
pub struct GameData {
  pub startup: bool,
  pub current_menu: MenuScreen,
  pub tabs_login: Tabs,
  pub settings: Settings,
  pub username_input: TextInput,
  pub password_input: TextInput,
  pub server_stream: Option<TcpStream>,
  pub notifications: Vec<Notification>,
  pub opake_data: OpakeData,
  pub cipher_key: Vec<u8>,
  
  /// Our nonce
  pub nonce: u32,
  /// Last seen nonce from the server.
  pub last_nonce: u32,
  pub player_stats: PlayerStatistics,
  pub friend_list: Vec<(String, FriendShipStatus, bool)>,
  pub username: String,
  pub friend_request_input: TextInput,
  pub recv_messages_buffer: Vec<(String, String, ChatMessageType)>,
  pub lobby_invites: Vec<String>,
  pub lobby: Vec<LobbyPlayerInfo>,
  pub packet_queue: Vec<ClientToServer>,
  pub settings_tabs: Tabs,
  pub main_tabs: Tabs,
  pub heroes_tabs: Tabs,
  pub settings_timer: Instant,
  pub settings_open: bool,
  pub paused: bool,
  pub queued: bool,
  pub checkbox_1v1: bool,
  pub checkbox_2v2: bool,
  pub fullscreen_pressed: bool,
  pub chat_input: TextInput,
  pub chat_timer: Instant,
  pub chat_pressed: bool,
  pub chat_open: bool,
  pub chat_scroll: f32,
  pub selected_friend: usize,

  pub server_ip: String,
  pub game_server_ip: String,
  pub game_server_port: u16,
  pub game_id: u128,
  pub game_socket: Option<UdpSocket>,
  pub game_last_nonce: u32,
  pub character_properties: HashMap<Character, CharacterProperties>,
  pub character_animations: HashMap<Character, Vec<AnimationState>>,
  pub packet_timer: Instant,
  pub position:           Vector2,
  /// Raw movement vector
  pub movement:           Vector2,
  pub aim_direction:      Vector2,
  pub shooting_primary:   bool,
  pub shooting_secondary: bool,
  pub dashing:            bool,
  pub player:             ClientPlayer,
  pub players:            Vec<ClientPlayer>,
  pub game_objects:       Vec<GameObject>,
  pub gamemode_info:      GameModeInfo,
  pub game_object_animations: HashMap<GameObjectType, AnimationState>,
  pub background_tiles: Vec<BackGroundTile>,
}
impl Default for GameData {
  fn default() -> Self {
    return GameData {
      startup: true,
      current_menu: MenuScreen::Login(0),
      tabs_login: Tabs::new(vec!["Login".to_string(), "Register".to_string()]),
      settings: Settings::load(),
      username_input: TextInput {
        selected: false,
        buffer: String::new(),
        hideable: false,
        show_password: false,
      },
      password_input: TextInput {
        selected: false,
        buffer: String::new(),
        hideable: true,
        show_password: false,
      },
      server_stream: None,
      notifications: Vec::new(),
      opake_data: OpakeData { 
        timeout: Instant::now(),
        client_registration_start_result: None,
        client_login_start_result: None,
      },
      cipher_key: Vec::new(),
      last_nonce: 0,
      nonce: 1,
      player_stats: PlayerStatistics { wins: 0 },
      friend_list:Vec::new(), //vec![
      //  (String::from("joe1"), FriendShipStatus::Friends, false),
      //  (String::from("jane2"), FriendShipStatus::Friends, true),
      //  (String::from("john3"), FriendShipStatus::PendingForA, false),
      //],
      username: "Player".to_string(),
      friend_request_input: TextInput {
        selected: false,
        buffer: String::new(),
        hideable: false,
        show_password: false,
      },
      recv_messages_buffer: Vec::new(),
      lobby_invites: Vec::new(),
      lobby: Vec::new(),
      packet_queue: Vec::new(),
      settings_tabs: Tabs::new(vec!["Gameplay".to_string(), "Video".to_string(), "Audio".to_string(), "Controls".to_string(), "Other".to_string()]),
      main_tabs: Tabs::new(vec!["Play".to_string(), "Heroes".to_string(), "Tutorial".to_string(), "Stats".to_string(), "Friends".to_string()]),
      heroes_tabs: Tabs::new(CHARACTER_LIST.iter().map(|x| x.name()).collect()),
      settings_timer: Instant::now(),
      settings_open: false,
      paused: false,
      queued: false,
      checkbox_1v1: true,
      checkbox_2v2: true,
      fullscreen_pressed: false,
      chat_input: TextInput {
        selected: false,
        buffer: String::new(),
        hideable: false,
        show_password: false,
      },
      chat_timer: Instant::now() - Duration::from_secs_f32(2.0),
      chat_pressed: false,
      chat_open: false,
      chat_scroll: 0.0,
      selected_friend: 0,

      server_ip: String::from(CURRENT_SERVER_IP),
      game_server_ip: String::from("13.38.240.14"),
      game_server_port: 0,
      game_id: 0,
      game_socket: None,
      game_last_nonce: 0,
      character_properties: load_characters(),
      character_animations: HashMap::new(),
      packet_timer: Instant::now(),
      movement: Vector2::new(),
      position: Vector2::new(),
      aim_direction: Vector2::new(),
      shooting_primary: false,
      shooting_secondary: false,
      dashing: false,
      player: ClientPlayer::new(),
      players: Vec::new(),
      game_objects: Vec::new(),
      gamemode_info: GameModeInfo::new(),
      game_object_animations: HashMap::new(),
      background_tiles: Vec::new(),
    }
  }
}

fn main_thread(
  mut com: Commands,
  data: Option<ResMut<GameData>>,
  asset_server: Res<AssetServer>,
  time: Res<Time>,
  mut window: Query<&mut Window>,
  mut cam: Query<&mut Camera2d>,
  k: Res<ButtonInput<KeyCode>>,
  m: Res<ButtonInput<MouseButton>>,
  mut ki: MessageReader<KeyboardInput>,
  mut mw: MessageReader<MouseWheel>,
  t: Res<Touches>,
  mut exit: MessageWriter<AppExit>,
  mut settings_sync: ResMut<Settings>,
) {
  if let Some(mut data) = data {
    // MAIN LOOP
    let mut win = window.single_mut().expect("oops");
    let vw = win.width() / 100.0;
    let vh = win.height() / 100.0;

    let delta_time = time.delta().as_secs_f32();
    
    if data.startup {
      data.startup = false;
      data.game_object_animations = load_game_object_animations(asset_server.clone());
      data.character_animations = load_character_animations(asset_server.clone());
      if data.settings.store_credentials {
        data.username_input.buffer = data.settings.saved_username.clone();
        data.password_input.buffer = load_password(&data.settings.saved_username);
      }
      set_fullscreen(data.settings.fullscreen, &mut win);
    }

    // synchronise settings with the exit thread
    *settings_sync = data.settings.clone();

    // calculate UI scale
    let size_min = f32::min(vw, vh);
    let uiscale = if size_min < 5.0 {2.5} else if size_min < 10.0 {5.0} else {10.0};
    let tl_anchor = Vector2 {x: 0.0, y: 0.0};
    let tr_anchor = Vector2 {x: 100.0*vw, y: 0.0};
    let bl_anchor = Vector2 {x: 0.0, y: 100.0*vh};
    let br_anchor = Vector2 {x: 100.0*vw, y: 100.0*vh};

    let font: Handle<Font> = asset_server.load("fonts/Roboto-Black.ttf");
    let mono_font: Handle<Font> = asset_server.load("fonts/Roboto-Mono.ttf");
    let mouse_pos = get_mouse_pos(&win);

    let ui_clickable = !(data.paused || data.chat_open);

    let username = data.username.clone();
    match data.current_menu {
      // MARK: Main
      MenuScreen::Main(mode) => {
        let character_descriptions = CharacterDescription::create_all_descriptions(data.character_properties.clone());

        // menu
        if mode != 2 {
          clear_background(WHITE, &win, &mut com);
          data.main_tabs.update_size(tl_anchor + Vector2 { x: 5.0 * vw, y: 5.0 * uiscale}, Vector2 { x: 90.0*vw, y: 8.0*uiscale }, 6.0*uiscale);
          data.main_tabs.draw_and_process(uiscale, ui_clickable, MENU_Z, &font, &win, &mut com, &m);
          
          // play
          if data.main_tabs.selected_tab() == 0 {
            let selected_char = data.heroes_tabs.selected_tab();
            if !data.queued {
              checkbox(br_anchor - Vector2 {x: 30.0*uiscale, y: 21.0*uiscale }, 4.0*uiscale, "1v1", 4.0*uiscale, uiscale, &mut data.checkbox_1v1, MENU_Z, &font, &win, &mut com, &m);
              checkbox(br_anchor - Vector2 {x: 17.5*uiscale, y: 21.0*uiscale }, 4.0*uiscale, "2v2", 4.0*uiscale, uiscale, &mut data.checkbox_2v2, MENU_Z, &font, &win, &mut com, &m);
            } if data.queued {
              checkbox(br_anchor - Vector2 {x: 30.0*uiscale, y: 21.0*uiscale }, 4.0*uiscale, "1v1", 4.0*uiscale, uiscale, &mut data.checkbox_1v1.clone(), MENU_Z, &font, &win, &mut com, &m); // clone to disable writes
              checkbox(br_anchor - Vector2 {x: 17.5*uiscale, y: 21.0*uiscale }, 4.0*uiscale, "2v2", 4.0*uiscale, uiscale, &mut data.checkbox_2v2.clone(), MENU_Z, &font, &win, &mut com, &m);
            }

            let mut play_button = Button::new(br_anchor - Vector2 { x: 30.0*uiscale, y: 15.0*uiscale }, Vector2 { x: 25.0*uiscale, y: 13.0*uiscale }, "Play", 8.0*uiscale);
            play_button.draw(uiscale, ui_clickable, MENU_Z, &font, &win, &mut com);
            if data.queued {
              draw_text(&font, "In queue...", br_anchor + Vector2 {x: - 30.0*uiscale, y: - 24.0*uiscale}, Vector2 { x: 100.0*uiscale, y: 100.0*uiscale }, BLACK, 5.0*uiscale, MENU_Z, Justify::Left, &win, &mut com);
            }
            if play_button.was_pressed(&win, &m) {
              data.queued = !data.queued;
              if data.queued {
                let mut selected_gamemodes: Vec<GameMode> = Vec::new();
                if data.checkbox_1v1 {selected_gamemodes.push(GameMode::Standard1V1)}
                if data.checkbox_2v2 {selected_gamemodes.push(GameMode::Standard2V2)}
                if selected_gamemodes.is_empty() {
                  data.notifications.push(Notification::new("Pick a gamemode!", 1.0));
                  data.queued = false;
                  return;
                }
                // Send a match request packet
                data.packet_queue.push(
                  ClientToServer::MatchRequest(MatchRequestData {
                    gamemodes: selected_gamemodes,
                    character: CHARACTER_LIST[selected_char],
                  }),
                );
              } else {
                // Send a match cancel packet
                data.packet_queue.push(
                  ClientToServer::MatchRequestCancel,
                )
              }
            }
            // draw lobby
            let mut lobby = data.lobby.clone();
            // insert self
            lobby.insert(
              0, 
              LobbyPlayerInfo {
                username: username.clone(),
                is_ready: data.queued,
              }
            );

            let lobby_position: Vector2 = Vector2 { x: 5.0*uiscale, y: 19.0*uiscale };
            let lobby_size: Vector2 = Vector2 { x: 30.0*uiscale, y: 7.0*uiscale };
            let y_offset = lobby_size.y;
            let inner_shrink: f32 = 1.0 * uiscale;
            draw_text(&font, "Lobby", Vector2 {x: lobby_position.x, y: lobby_position.y-3.0*uiscale}, Vector2 {x: 100.0*vh, y: 100.0*vh}, BLACK, 3.0*uiscale, MENU_Z, Justify::Left, &win, &mut com);
            for (i, player) in lobby.iter().enumerate() {
              draw_rect(Color::Srgba(BLUE), lobby_position + Vector2 {x: 0.0, y: (i as f32)*y_offset}, lobby_size, MENU_Z, &win, &mut com );
              draw_rect(Color::Srgba(SKY_BLUE), lobby_position + Vector2{x: inner_shrink, y:inner_shrink} + Vector2 {x: 0.0, y: (i as f32)*y_offset}, lobby_size - Vector2{x: inner_shrink*2.0, y:inner_shrink*2.0}, MENU_Z, &win, &mut com);
              let is_ready_color = if player.is_ready {LIME} else {RED};
              let is_ready_text = if player.is_ready {"Ready"} else {"Not Ready"};
              draw_text(&font, &format!("{}", player.username), Vector2 {x: lobby_position.x + 2.0*vh, y: lobby_position.y + (i as f32)*y_offset}, Vector2{x: 100.0*vh, y: 100.0*vh}, BLACK, 3.0*uiscale, MENU_Z, Justify::Left, &win, &mut com);
              draw_text(&font, &format!("{}", is_ready_text), Vector2 {x: lobby_position.x + lobby_size.x * 0.67, y: lobby_position.y + (i as f32)*y_offset}, Vector2{x: 100.0*vh, y: 100.0*vh}, is_ready_color, 3.0*uiscale, MENU_Z, Justify::Left, &win, &mut com);
            }
            // lobby leave button
            if lobby.len() > 1 {
              let mut leave_button = Button::new(lobby_position + Vector2 {x: 0.0, y: y_offset * (lobby.len() as f32) + inner_shrink}, Vector2 { x: lobby_size.x/2.0, y: lobby_size.y - inner_shrink }, "Leave", 5.0*vh);
              leave_button.draw(vh, ui_clickable, MENU_Z, &font, &win, &mut com);
              if leave_button.was_pressed(&win, &m) {
                data.packet_queue.push(
                  ClientToServer::LobbyLeave,
                );
                data.lobby = Vec::new();
                data.notifications.push(
                  Notification::new("Left the party.", 1.0)
                )
              }
            }
          }

          // heroes
          if data.main_tabs.selected_tab() == 1 {
            data.heroes_tabs.update_size(bl_anchor + Vector2 { x: 5.0 * vw, y: - 20.0 * uiscale}, Vector2 { x: 90.0*vw, y: 15.0*uiscale }, 5.0*uiscale);
            data.heroes_tabs.draw_and_process(uiscale, ui_clickable, MENU_Z, &font, &win, &mut com, &m);

            let selected = data.heroes_tabs.selected_tab();
            let selected_character = CHARACTER_LIST[selected];
            for i in 0..4 {
              let texture = asset_server.load(format!("ui/temp_ability_{}.png", i+1));
              let size = Vector2 { x: 10.0*uiscale, y: 10.0*uiscale };
              draw_ability_icon(tl_anchor + Vector2 { x: 10.0*uiscale + (size.x + 4.0*uiscale) * i as f32, y: 67.5*uiscale }, size, i, false, 1.0, vh, vw, uiscale, &font, character_descriptions.clone(), selected_character, MENU_Z+1, &texture, &win, &mut com, data.settings.clone());
            }
            let profile_texture = asset_server.load(format!("characters/{}/textures/mini-profile.png", selected_character.name().to_lowercase() ));
            let profile_texture = Texture {
              image: profile_texture,
              size: Vec2 { x: 900.0, y: 1000.0 }
            };
            draw_sprite(&profile_texture, tl_anchor + Vector2 {x: 10.0*uiscale, y: 15.0*uiscale}, Vector2 { x: 55.0*uiscale, y: 55.0*uiscale*profile_texture.aspect_ratio() }, MENU_Z, &win, &mut com);

            // practice range
            let mut button = Button::new(tr_anchor + Vector2 { x: -30.0*uiscale, y: 50.0*uiscale }, Vector2 { x: 20.0*uiscale, y: 10.0*uiscale }, "Practice", 4.0*uiscale);
            button.draw(uiscale, ui_clickable, MENU_Z, &font, &win, &mut com);
            if button.was_released(&win, &m) {
              data.cipher_key = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
              let game_port = get_random_port();
              let practice_game_port = game_port.clone();
              let practice_username = username.clone();
              let session_key = data.cipher_key.clone();
              let practice_character = selected_character.clone();
              let (tx, mut _rx): (tokio::sync::mpsc::Sender<PlayerMessage>, tokio::sync::mpsc::Receiver<PlayerMessage>)
                = tokio::sync::mpsc::channel(32);
              let _game_server = std::thread::spawn(move || {
                game_server(1, practice_game_port, vec![
                  PlayerInfo {
                    // a lot of dummy data.
                    username: practice_username,
                    session_key: session_key,
                    channel: tx,
                    queued: true,
                    is_party_leader: true,
                    queued_with: Vec::new(),
                    invited_by: Vec::new(),
                    queued_gamemodes: Vec::new(),
                    selected_character: practice_character,
                    assigned_team: Team::Blue,
                    in_game_with: Vec::new(),
                  }
                ],
                true)
              });
              data.player.character = practice_character;
              data.game_server_port = practice_game_port;
              data.game_server_ip = String::from("127.0.0.1");
              data.game_id = 0;
              data.queued = false;
              data.game_last_nonce = 0;
              data.background_tiles = load_background_tiles(32, 24);

              // bind to udp socket for gameserver.
              let port = get_random_port();
              let ip = format!("0.0.0.0:{}", port);
              let udp_socket = UdpSocket::bind(ip);
              match udp_socket {
                Ok(socket) => {
                  socket.set_nonblocking(true).expect("oops");
                  data.game_socket = Some(socket);
                  // set game screen.
                  data.current_menu = MenuScreen::Main(2);
                }
                Err(err) => {
                  data.current_menu = MenuScreen::Main(0);
                  data.notifications.push(
                    Notification::new(&format!("Connection error: {:?}", err), 5.0),
                  );
                  return
                }
              }
            }
          }
          
          // tutorial
          if data.main_tabs.selected_tab() == 2 {
            draw_text(&font, "Use WASD to move.\nLMB: Primary fire\nSPACE: Dash (cooldown ability)\nRMB: Secondary/Ultimate (charge it up by dealing damage)", tl_anchor + Vector2 {x: 10.0 * uiscale, y: 30.0 * uiscale}, Vector2 { x: 80.0*uiscale, y: 100.0*uiscale }, BLACK, 5.0*uiscale, MENU_Z, Justify::Left, &win, &mut com);
          }
          
          // stats
          if data.main_tabs.selected_tab() == 3 {
            // refresh button
            let mut refresh_button = Button::new(tl_anchor + Vector2 {x: 10.0 * uiscale, y: 20.0*uiscale}, Vector2 {x: 20.0 * uiscale, y: 7.0*uiscale}, "Refresh", 5.0*uiscale);
            refresh_button.draw(uiscale, ui_clickable, MENU_Z, &font, &win, &mut com);
            if refresh_button.was_released(&win, &m) {
              data.packet_queue.push(
                ClientToServer::PlayerDataRequest
              )
            }
            // draw the stats
            draw_text(&font, format!("{}'s stats", data.username).as_str(), tl_anchor + Vector2 {x: 10.0 * uiscale, y: 30.0 * uiscale}, Vector2 { x: 100.0*uiscale, y: 100.0*uiscale }, BLACK, 5.0*uiscale, MENU_Z, Justify::Left, &win, &mut com);
            
            let wins = data.player_stats.wins;
            draw_text(&font, format!("wins: {}", wins).as_str(), tl_anchor + Vector2 {x: 10.0 * uiscale, y: 35.0 * uiscale}, Vector2 { x: 100.0*uiscale, y: 100.0*uiscale }, BLACK, 4.0*uiscale, MENU_Z, Justify::Left, &win, &mut com);
          }
          
          // friends tab
          if data.main_tabs.selected_tab() == 4 {
            // refresh button
            let mut refresh_button = Button::new(tl_anchor + Vector2 {x: 10.0 * uiscale, y: 20.0*uiscale}, Vector2 {x: 20.0 * uiscale, y: 7.0*uiscale}, "Refresh", 5.0*uiscale);
            refresh_button.draw(uiscale, ui_clickable, MENU_Z, &font, &win, &mut com);
            if refresh_button.was_released(&win, &m) {
              data.packet_queue.push(
                ClientToServer::GetFriendList,
              )
            }

            
            // FRIEND LIST
            let y_offset = 6.0 * uiscale;
            let y_start = 30.0*uiscale;
            for f_index in 0..data.friend_list.len() {
              let friend = data.friend_list[f_index].clone();
              let current_offset = y_offset * f_index as f32;
              let peer_username;
              let split: Vec<&str> = friend.0.split(":").collect();
              if *split[0] == username {
                peer_username = split[1];
              } else {
                peer_username = split[0];
              }

              draw_text(&font, peer_username, Vector2 { x: 10.0*uiscale, y: y_start + current_offset }, Vector2 { x: 100.0*uiscale, y: 100.0*uiscale }, BLACK, 4.0*uiscale, MENU_Z, Justify::Left, &win, &mut com);
              
              let status: &str;
              match friend.1 {
                FriendShipStatus::PendingForA | FriendShipStatus::PendingForB => {
                  let pending_for_you_status = database::get_friend_request_type(&username, &peer_username);
                  if pending_for_you_status != friend.1 {
                    // This user is requesting to be friends.
                    status = "Awaiting your response.";
                    let accept_button = Button::new(Vector2 { x: 70.0*uiscale, y: y_start + current_offset }, Vector2 { x: 15.0*uiscale, y: 6.0*uiscale }, "Accept", 4.0*uiscale);
                    if accept_button.was_pressed(&win, &m) {
                      // Accept the friend request by sending a friend request to this user, which the
                      // server processes as an accept.
                      data.packet_queue.push(
                        ClientToServer::SendFriendRequest(String::from(peer_username)),
                      );
                      data.packet_queue.push(
                        ClientToServer::GetFriendList,
                      );
                      //friend_request_input = peer_username.to_string();
                    }

                  } else {
                    // We sent a request to this user,
                    status = "Friend request sent..."
                  }
                }
                FriendShipStatus::Blocked => {
                  status = "Blocked"
                }
                FriendShipStatus::Friends => {
                  status = "Friends";
                  let online = friend.2;
                  draw_text(&font, match online {true => "Online", false => "Offline"}, Vector2 { x: 70.0*uiscale, y: 30.0*uiscale + current_offset }, Vector2 { x: 100.0*uiscale, y: 100.0*uiscale }, BLACK, 4.0*uiscale, MENU_Z, Justify::Left, &win, &mut com);

                  // if we were invited by this user, show accept button
                  if data.lobby_invites.contains(&String::from(peer_username)) {
                    let mut accept_button = Button::new(
                      Vector2 { x: 70.0*uiscale, y: y_start + current_offset }, Vector2 { x: 15.0*uiscale, y: 6.0*uiscale }, "Join", 4.0*uiscale
                    );
                    accept_button.draw(uiscale, ui_clickable, MENU_Z, &font, &win, &mut com);
                    if accept_button.was_pressed(&win, &m) {
                      data.packet_queue.push(
                        ClientToServer::LobbyInviteAccept(String::from(peer_username)),
                      );
                      data.lobby_invites.retain(|element| element != peer_username);  
                    }
                  }
                  // invite user button
                  else {
                    if online {
                      let mut invite_button = Button::new(
                        Vector2 { x: 90.0*uiscale, y: y_start + current_offset }, Vector2 { x: 15.0*uiscale, y: 6.0*uiscale }, "Invite", 4.0*uiscale
                      );
                      invite_button.draw(uiscale, ui_clickable, MENU_Z, &font, &win, &mut com);
                      if invite_button.was_pressed(&win, &m) {
                        data.packet_queue.push(
                          ClientToServer::LobbyInvite(String::from(peer_username)),
                        );
                        data.notifications.push(
                          Notification::new(&format!("Invited {} to lobby.", peer_username), 1.5)
                        );
                      }
                    }
                  }
                }
              }
              draw_text(&font, status, Vector2 { x: 40.0*uiscale, y: 30.0*uiscale + current_offset }, Vector2 { x: 100.0*uiscale, y: 100.0*uiscale }, BLACK, 4.0*uiscale, MENU_Z, Justify::Left, &win, &mut com);
            }

          }
        }
        // MARK: Game
        if mode == 1 || mode == 2 {

          // MARK: | Interpolation
          // for now this is just simple linear interpolation, no shenanigans yet.
          for player in data.players.iter_mut() {
            let distance = player.interpol_next - player.position;
            let cutoff = 7.0 * delta_time;
            if distance.magnitude() > cutoff {
              let period = PACKET_INTERVAL;
              let speed = distance / period;
              //let speed = player.movement_direction * character_properties[&player.character].speed;
              player.position += distance.normalize() * (70.0 * speed.magnitude().powf(1.0/15.0) + 3.0 * speed.magnitude()) * delta_time * 0.1;
            }
            //player.position += distance * PACKET_INTERVAL * get_frame_time() * 2.0;
            //player.position += distance * get_frame_time();
            //draw_line(player.position.x, player.position.y, player.interpol_next.x, player.interpol_next.y, 1.0*vh, PURPLE);
            // I can't get the interpolation to work, so temporarily I'll swap it with this very simple
            // extrapolation method.
            //player.position += player.movement_direction * character_properties[&player.character].speed * get_frame_time();
          }

          // MARK: | game graphics





          // MARK: | | Animation handl.

          // set idle animations
          for player_index in 0..data.players.len() {
            if data.players[player_index].current_animation.is_finished() {
              let character = data.players[player_index].character;
              data.players[player_index].current_animation = data.character_animations[&character][0].from_start();
            }
          }
          if data.player.current_animation.is_finished() {
            let character = data.player.character;
            data.player.current_animation = data.character_animations[&character][0].from_start();
          }

          for background_tile in data.background_tiles.clone() {
            if let Ok(texture) = data.game_object_animations[&background_tile.object_type].current_frame() {
              let size: Vector2 = Vector2 { x: 1.0, y: 1.0 };
              draw_image_relative(&texture, background_tile.position.x - size.x/2.0, background_tile.position.y - size.y/2.0, size.x, size.y, vh, vw, data.player.camera.clone(), GAME_BG_Z, &win, &mut com);
            }
          }

          // adjust certain positions.
          // adjust the location of Wiro's shield.
          for game_object_index in 0..data.game_objects.len() {
            if data.game_objects[game_object_index].object_type == GameObjectType::WiroShield {
              // if it's ours...
              if data.game_objects[game_object_index].get_bullet_data().owner_username == username {
                let position: Vector2 = Vector2 {
                  x: data.player.position.x + data.aim_direction.normalize().x * 1.0,
                  y: data.player.position.y + data.aim_direction.normalize().y * 1.0,
                };

                data.game_objects[game_object_index].position = position;
                let mut shield_data = data.game_objects[game_object_index].get_bullet_data();
                shield_data.direction = data.aim_direction.normalize();
                data.game_objects[game_object_index].extra_data = ObjectData::BulletData(shield_data);
              }
            }
          }
          
          // MARK: | | gameobjects

          // Extrapolation
          for game_object_index in 0..data.game_objects.len() {
            if let Ok(bullet_data) = data.game_objects[game_object_index].get_bullet_data_safe() {

              let speed: f32 = match data.game_objects[game_object_index].object_type {
                GameObjectType::RaphaelleBullet                   => data.character_properties[&Character::Raphaelle].primary_shot_speed,
                GameObjectType::RaphaelleBulletEmpowered          => data.character_properties[&Character::Raphaelle].primary_shot_speed,
                GameObjectType::HernaniBullet                     => data.character_properties[&Character::Hernani].primary_shot_speed,
                GameObjectType::CynewynnSword                     => data.character_properties[&Character::Cynewynn].primary_shot_speed,
                GameObjectType::FedyaProjectileRicochet           => data.character_properties[&Character::Fedya].primary_shot_speed,
                GameObjectType::FedyaProjectileGroundRecalled     => data.character_properties[&Character::Fedya].primary_shot_speed,
                GameObjectType::WiroGunShot                       => data.character_properties[&Character::Wiro].primary_shot_speed,
                GameObjectType::TemerityRocket                    => data.character_properties[&Character::Temerity].primary_shot_speed,
                GameObjectType::KoldoCannonBall                   => data.character_properties[&Character::Koldo].primary_shot_speed,
                GameObjectType::KoldoCannonBallEmpowered          => data.character_properties[&Character::Koldo].primary_shot_speed,
                GameObjectType::KoldoCannonBallEmpoweredUltimate  => data.character_properties[&Character::Koldo].primary_shot_speed,
                _ => 0.0
              };
              data.game_objects[game_object_index].position += bullet_data.direction * speed * delta_time;
            }
          }

          //data.game_objects = sort_by_depth(data.game_objects);
          for game_object in data.game_objects.clone() {
            if let Ok(texture) = data.game_object_animations[&game_object.object_type].current_frame() {

              let size = match game_object.object_type {
                GameObjectType::Wall => Vector2 {x: 1.0, y: 2.0 },
                GameObjectType::UnbreakableWall => Vector2 {x: 1.0, y: 2.0},
                GameObjectType::HernaniWall => Vector2 {x: 1.0, y: 2.0},
                GameObjectType::HernaniBullet => Vector2 { x: 1.0 * (10.0/4.0), y: 1.0 },
                GameObjectType::RaphaelleBullet => Vector2 { x: 2.0, y: 2.0 },
                GameObjectType::RaphaelleBulletEmpowered => Vector2 { x: 2.0, y: 2.0 },
                GameObjectType::RaphaelleAura => Vector2 {x: data.character_properties[&Character::Raphaelle].secondary_range*2.0, y: data.character_properties[&Character::Raphaelle].secondary_range*2.0,},
                GameObjectType::WiroShield => Vector2 { x: 0.5, y: data.character_properties[&Character::Wiro].secondary_range },
                GameObjectType::TemerityRocketSecondary => Vector2 { x: 2.0, y: 2.0 },
                GameObjectType::CenterOrb => Vector2 { x: 2.0, y: 2.0 },
                GameObjectType::CynewynnSword => Vector2 { x: 3.0, y: 3.0 },
                GameObjectType::KoldoCannonBall => Vector2 { x: 2.0, y: 2.0 },
                GameObjectType::KoldoCannonBallEmpowered => Vector2 { x: 2.0, y: 2.0 },
                GameObjectType::KoldoCannonBallEmpoweredUltimate => Vector2 { x: 2.0, y: 2.0 },
                _ => Vector2 {x: 1.0, y: 1.0},
              };
              let shadow_offset: f32 = 5.0;
              
              // Draw shadows on certain objects
              let shaded_objects = vec![GameObjectType::RaphaelleBullet,
                GameObjectType::RaphaelleBulletEmpowered,
                GameObjectType::HernaniBullet,
                GameObjectType::CynewynnSword,
                GameObjectType::CenterOrb,
                GameObjectType::FedyaProjectileRicochet,
              ];
              let rotation: Vector2 = match game_object.get_bullet_data_safe() {
                Ok(data) => {
                  data.direction
                }
                Err(()) => {
                  Vector2::new()
                }
              };
              //if shaded_objects.contains(&game_object.object_type) {
              //  draw_image_relative(
              //    texture,
              //    game_object.position.x - size.x/2.0,
              //    game_object.position.y - size.y/2.0 + shadow_offset,
              //    size.x,
              //    size.y,
              //    vh, data.player.camera.clone(),
              //    rotation,
              //    Color { r: 0.05, g: 0.0, b: 0.1, a: 0.15 }
              //  );
              //}
              draw_image_relative_ex(&texture, game_object.position.x - size.x/2.0, game_object.position.y - size.y/2.0, size.x, size.y, rotation, vh, vw, data.player.camera.clone(), GAME_OBJ_Z, &win, &mut com);
            }
          }
          // MARK: | |  Game UI
          let primary_cooldown: f32 = if data.player.last_shot_time < data.character_properties[&data.player.character].primary_cooldown {
            data.player.last_shot_time / data.character_properties[&data.player.character].primary_cooldown
          } else {
            1.0
          };
          let mut secondary_cooldown: f32 = data.player.secondary_charge as f32 / 100.0;
          if data.player.character == Character::Wiro {
            if data.player.last_secondary_time < data.character_properties[&Character::Wiro].secondary_cooldown {
              secondary_cooldown = data.player.last_secondary_time / data.character_properties[&Character::Wiro].secondary_cooldown;
            }
          }

          let dash_cooldown: f32 = if data.player.time_since_last_dash < data.character_properties[&data.player.character].dash_cooldown {
            data.player.time_since_last_dash / data.character_properties[&data.player.character].dash_cooldown
          } else {
            1.0
          };
          
          draw_ability_icon(bl_anchor + Vector2 { x: 2.0 * uiscale, y: -20.0 * uiscale }, Vector2 { x: 10.0 * uiscale, y: 10.0 * uiscale }, 0, false, 1.0 , vh, vw, uiscale, &font, character_descriptions.clone(), data.player.character, GAME_UI_Z, &asset_server.load("ui/temp_ability_4.png"), &win, &mut com, data.settings.clone());
          draw_ability_icon(bl_anchor + Vector2 { x: 14.5 * uiscale, y: -20.0 * uiscale }, Vector2 { x: 10.0 * uiscale, y: 10.0 * uiscale }, 1, data.player.shooting_primary, primary_cooldown , vh, vw, uiscale, &font, character_descriptions.clone(), data.player.character, GAME_UI_Z, &asset_server.load("ui/temp_ability_1.png"), &win, &mut com, data.settings.clone());
          draw_ability_icon(bl_anchor + Vector2 { x: 27.0 * uiscale, y: -20.0 * uiscale }, Vector2 { x: 10.0 * uiscale, y: 10.0 * uiscale }, 3, data.player.dashing, dash_cooldown , vh, vw, uiscale, &font, character_descriptions.clone(), data.player.character, GAME_UI_Z, &asset_server.load("ui/temp_ability_3.png"), &win, &mut com, data.settings.clone());
          draw_ability_icon(bl_anchor + Vector2 { x: 39.5 * uiscale,  y: -20.0 * uiscale }, Vector2 { x: 10.0 * uiscale, y: 10.0 * uiscale }, 2, data.player.shooting_secondary, secondary_cooldown , vh, vw, uiscale, &font, character_descriptions.clone(), data.player.character, GAME_UI_Z, &asset_server.load("ui/temp_ability_2.png"), &win, &mut com, data.settings.clone());

          // screen-space to world space conversion
          //                                          no idea what's up with "/vh" but it gotta be there
          let mouse_world_position = screen_to_world(mouse_pos/vh, data.player.camera.clone(), vh, vw);
      
          data.aim_direction = (mouse_world_position - data.player.position).normalize();
          
          // draw player and aim laser
          let mut range = data.character_properties[&data.player.character].primary_range * data.player.camera.zoom;
          if data.player.character == Character::Temerity {
            if data.player.stacks == 1 {
              range = data.character_properties[&Character::Temerity].primary_range_2 * data.player.camera.zoom
            }
            if data.player.stacks == 2 {
              range = data.character_properties[&Character::Temerity].primary_range_3 * data.player.camera.zoom
            }
          }
          if data.player.character == Character::Koldo {
            if data.player.passive_elapsed > data.character_properties[&Character::Koldo].passive_cooldown
            || data.player.stacks > 0 {
              range = data.character_properties[&Character::Koldo].primary_range_2 * data.player.camera.zoom;
            }
          }

          let relative_position = world_to_screen(data.player.position, data.player.camera.clone(), vh, vw);

          if !data.player.is_dead {
            let mut range_limited: f32 = Vector2::distance(data.player.position, mouse_world_position.clone()) * data.player.camera.zoom;
            if range_limited > range {
              range_limited = range;
            }
            let low_limit = 1.2 * data.player.camera.zoom;
            if range_limited < low_limit {
              range_limited = low_limit;
            }
            // full line
            draw_line(
              Vector2{
                x: (data.aim_direction.normalize().x * low_limit * vh) + relative_position.x * vh,
                y: (data.aim_direction.normalize().y * low_limit * vh) + relative_position.y * vh
              },
              Vector2 {
                x: (data.aim_direction.normalize().x * range * vh) + (relative_position.x * vh),
                y: (data.aim_direction.normalize().y * range * vh) + (relative_position.y * vh)
              },
              0.6 * vh, Srgba { red: 1.0, green: 0.2, blue: 0.0, alpha: 0.2 }, GAME_UI_Z-2, &win, &mut com
            );
            // shorter, matte line
            draw_line(
              Vector2{
                x: (data.aim_direction.normalize().x * low_limit * vh) + relative_position.x * vh,
                y: (data.aim_direction.normalize().y * low_limit * vh) + relative_position.y * vh
              },
              Vector2{
                x: (data.aim_direction.normalize().x * range_limited * vh) + (relative_position.x * vh),
                y: (data.aim_direction.normalize().y * range_limited * vh) + (relative_position.y * vh)
              },
              0.4 * vh, Srgba { red: 1.0, green: 0.2, blue: 0.0, alpha: 1.0 }, GAME_UI_Z-1, &win, &mut com
            );
            if data.player.character == Character::Hernani {
              let range: f32 = data.character_properties[&Character::Hernani].secondary_range * data.player.camera.zoom;
              let aim_dir = data.aim_direction.normalize();
              // perpendicular direction 1
              let aim_dir_alpha = Vector2 {x:   aim_dir.y, y: - aim_dir.x};
              // perpendicular direction 2
              let aim_dir_gamma = Vector2 {x: - aim_dir.y, y:   aim_dir.x};

              let width = 2.0;
              draw_line(
                Vector2{
                  x: (aim_dir.x * range + aim_dir_alpha.x * width) * vh + relative_position.x * vh,
                  y: (aim_dir.y * range + aim_dir_alpha.y * width) * vh + relative_position.y * vh
                },
                Vector2{
                  x: (aim_dir.x * range + aim_dir_gamma.x * width) * vh + relative_position.x * vh,
                  y: (aim_dir.y * range + aim_dir_gamma.y * width) * vh + relative_position.y * vh
                },
                0.4 * vh, Srgba { red: 1.0, green: 0.5, blue: 0.0, alpha: 1.0 }, GAME_UI_Z-1, &win, &mut com
              );
            }
          }

          
          // MARK: | | Draw Players
          if !data.player.is_dead {
            data.player.draw(vh, vw, uiscale, data.player.camera.clone(), &font, data.character_properties[&data.player.character].clone(), data.settings.clone(), GAME_PLAYER_Z+1, &mut com, &win);
          }
          for player in data.players.clone() {
            if !player.is_dead {
              player.draw(vh, vw, uiscale, data.player.camera.clone(), &font, data.character_properties[&player.character].clone(), data.settings.clone(), GAME_PLAYER_Z+1, &mut com, &win);
            }
          }

          // draw players and optionally their trails
          let trail_y_offset: f32 = 0.6;
          for player in data.players.clone() {
            if player.character == Character::Cynewynn && !player.is_dead {
              draw_lines(player.previous_positions.clone(), data.player.camera.clone(), vh, vw, player.team, trail_y_offset-0.0, 1.0, GAME_PLAYER_Z, &win, &mut com);
              draw_lines(player.previous_positions.clone(), data.player.camera.clone(), vh, vw, player.team, trail_y_offset-0.1, 0.5, GAME_PLAYER_Z, &win, &mut com);
              draw_lines(player.previous_positions,         data.player.camera.clone(), vh, vw, player.team, trail_y_offset-0.2, 0.25, GAME_PLAYER_Z, &win, &mut com);
            }
          }
          if data.player.character == Character::Cynewynn && !data.player.is_dead {
            draw_lines(data.player.previous_positions.clone(), data.player.camera.clone(), vh, vw, data.player.team, trail_y_offset-0.0, 0.6, GAME_PLAYER_Z, &win, &mut com);
            draw_lines(data.player.previous_positions.clone(), data.player.camera.clone(), vh, vw, data.player.team, trail_y_offset-0.1, 0.4, GAME_PLAYER_Z, &win, &mut com);
            draw_lines(data.player.previous_positions.clone(), data.player.camera.clone(), vh, vw, data.player.team, trail_y_offset-0.2, 0.2, GAME_PLAYER_Z, &win, &mut com);
          }

          // Draw raphaelle's tethering.
          let mut all_players_copy: Vec<ClientPlayer> = data.players.clone();
          all_players_copy.push(data.player.clone());
          for player in all_players_copy.clone() {
            if player.character == Character::Raphaelle {
              for player_2 in all_players_copy.clone() {
                if Vector2::distance(player.position, player_2.position) < data.character_properties[&Character::Raphaelle].primary_range
                && player.team == player_2.team
                && (player.is_dead & player_2.is_dead) == false {
                  // if on same team, green. If on enemy team, orange.
                  let color = match player.team == data.player.team {
                    true => GREEN,
                    false => ORANGE,
                  };
                  draw_line_relative(player.position.x, player.position.y, player_2.position.x, player_2.position.y, 0.5, color, data.player.camera.clone(), vh, vw, GAME_PLAYER_Z, &win, &mut com);
                }
              }
            }
          }

          // CAMERA MOVEMENT
          if !data.player.is_dead {
            match data.settings.camera_smoothing {
              true => {
                // if delta_time is too long, the camera behaves very weirdly, so let's arficially assume
                // framerate never goes below 20fps.
                let safe_delta_time = f32::min(delta_time, 1.0/20.0);
                let camera_distance: Vector2 = Vector2::difference(data.player.camera.position, data.player.position);
                let camera_distance_mag = camera_distance.magnitude();
                let camera_smoothing: f32 = 1.5; // higher = less smoothing
                let safe_quadratic = f32::min(camera_distance_mag*camera_smoothing*10.0, (camera_distance_mag).powf(2.0)*camera_smoothing*5.0);
                let camera_movement_speed = safe_quadratic;

                data.player.camera.position += camera_distance.normalize() * safe_delta_time * camera_movement_speed;
              }
              false => {
                data.player.camera.position = data.player.position;
              }
            }
          }

          // MARK: | game input
          let mut position = data.player.position;
          let mut movement = Vector2::new();
          //let mut aim_direction = Vector2::new();
          data.player.dashing = false;
          data.player.shooting_primary = false;
          data.player.shooting_secondary = false;

          // only register input if the window is active and the pause menu isn't open
          if is_window_focused(&win) && ui_clickable {

            #[cfg(not(target_os="android"))]
            {
              let device_state: DeviceState = DeviceState::new();
              let keys: Vec<Keycode> = device_state.get_keys();
              let mouse: Vec<MouseButton> = get_mouse_down(&m);
              if !keys.is_empty() {
                movement = Vector2::new();
                //keyboard_mode = true; // since we used the keyboard
              }
              
              // key binds
              for key in keys {
                let key = key as u16;
                // move
                if key == data.settings.keybinds.walk_up.0    || key == data.settings.keybinds.walk_up.1    { movement.y += -1.0 }
                if key == data.settings.keybinds.walk_down.0  || key == data.settings.keybinds.walk_down.1  { movement.y +=  1.0 }
                if key == data.settings.keybinds.walk_left.0  || key == data.settings.keybinds.walk_left.1  { movement.x += -1.0 }
                if key == data.settings.keybinds.walk_right.0 || key == data.settings.keybinds.walk_right.1 { movement.x +=  1.0 }
                // primary
                if key == data.settings.keybinds.primary.0    || key == data.settings.keybinds.primary.1    { data.player.shooting_primary = true; /*keyboard_mode = true*/ }
                // secondary
                if key == data.settings.keybinds.secondary.0  || key == data.settings.keybinds.secondary.1  { data.player.shooting_secondary = true; /*keyboard_mode = true*/ }
                // dash
                if key == data.settings.keybinds.dash.0       || key == data.settings.keybinds.dash.1       { data.player.dashing = true; /*keyboard_mode = true*/ }
              }
              
              // mouse button binds
              for button in mouse {
                let button = mb_to_num(button);
                // move
                if button == data.settings.keybinds.walk_up.2    || button == data.settings.keybinds.walk_up.3    { movement.y += -1.0 }
                if button == data.settings.keybinds.walk_down.2  || button == data.settings.keybinds.walk_down.3  { movement.y +=  1.0 }
                if button == data.settings.keybinds.walk_left.2  || button == data.settings.keybinds.walk_left.3  { movement.x += -1.0 }
                if button == data.settings.keybinds.walk_right.2 || button == data.settings.keybinds.walk_right.3 { movement.x +=  1.0 }
                // primary
                if button == data.settings.keybinds.primary.2    || button == data.settings.keybinds.primary.3    { data.player.shooting_primary = true; /*keyboard_mode = true*/ }
                // secondary
                if button == data.settings.keybinds.secondary.2  || button == data.settings.keybinds.secondary.3  { data.player.shooting_secondary = true; /*keyboard_mode = true*/ }
                // dash
                if button == data.settings.keybinds.dash.2       || button == data.settings.keybinds.dash.3       { data.player.dashing = true; /*keyboard_mode = true*/ }
              }
            }
          }

          if movement.magnitude() > 1.0 {
            // println!("normalizing");
            movement = movement.normalize();
          }

          let mut movement_raw: Vector2 = movement;

          // the server tells us if we're dashing or not
          if data.player.is_dashing {
            // do the interpolate
            let distance = data.player.interpol_next - data.player.interpol_prev;
            let speed: Vector2;
            if distance.magnitude() == 0.0 {
              // this is only true on the first "frame".
              // this measure helps reduce the percieved lag from the character standing still
              // before it obtains its second interpolation position.
              speed = data.player.movement_direction * (data.character_properties[&data.player.character].dash_speed / 2.0) * delta_time;
            } else {
              // this runs the rest of the time
              let period = PACKET_INTERVAL;
              speed = distance / period;
            }
            data.player.position += speed * delta_time;
          }
          else {
            if data.player.dashing && !data.player.is_dashing && !data.player.is_dead && movement_raw.magnitude() != 0.0 {
              if data.player.time_since_last_dash > data.character_properties[&data.player.character].dash_cooldown {
                match data.player.character {
                  Character::Temerity => {
                  }
                  _ => {
                    data.player.is_dashing = true;
                  }
                }
              }
            }
          
            if data.player.is_dashing {
              (data.player.position, data.player.dashed_distance, data.player.is_dashing) = dashing_logic(
                data.player.is_dashing,
                data.player.dashed_distance,
                movement_raw,
                delta_time as f64,
                data.character_properties[&data.player.character].dash_speed,
                data.character_properties[&data.player.character].dash_distance,
                data.game_objects.clone(),
                data.player.position,
              );
            }
          }

          // Apply standard movement (non-dashing)
          if !data.player.is_dashing {
            let mut extra_speed: f32 = 0.0;
            for buff in data.player.buffs.clone() {
              if vec![BuffType::Speed, BuffType::WiroSpeed].contains(&buff.buff_type) {
                extra_speed += buff.value;
              }
              if buff.buff_type == BuffType::Impulse {
                // yeet
                let direction = buff.direction.normalize();
                // time left serves as impulse decay
                let time_left = buff.duration;
                let strength = buff.value;
                movement += direction * f32::powi(time_left, 1) * strength;
              }
            }
  
            let movement_speed: f32 = data.character_properties[&data.player.character].speed;

            movement.x *= (movement_speed + extra_speed) * delta_time;
            movement.y *= (movement_speed + extra_speed) * delta_time;
            if data.player.is_dead == false {  
              (movement_raw, movement) = object_aware_movement(data.player.position, movement_raw, movement, data.game_objects.clone());
              data.player.position.x += movement.x;
              data.player.position.y += movement.y;
            } if data.player.is_dead {
              data.player.camera.position.x += movement.x;
              data.player.camera.position.y += movement.y;
            }
          }

          // CAMERA ZOOM
          if ui_clickable {
            let scrollwheel = get_mouse_wheel(&mut mw);
            data.player.camera.zoom = (data.player.camera.zoom + data.player.camera.zoom * scrollwheel.y * 4.0 * delta_time).clamp(4.0, 16.0);
          }

          // MARK: | game net
          let mut buffer: [u8; 16384] = [0; 16384];
          let mut len = 0;
          if let Some(ref mut game_socket) = data.game_socket {
            match game_socket.recv_from(&mut buffer) {
              Ok((length, _addr)) => {
                len = length;
              }
              Err(err) => {
                if err.kind() == std::io::ErrorKind::WouldBlock {
                  
                } else {
                }
              }
            }
          }
          if len > 0 {

            let recv_data = &buffer[..len];

            // get nonce
            let recv_nonce = &buffer[..4];
            let recv_nonce = match bincode::deserialize::<u32>(&recv_nonce){
              Ok(nonce) => nonce,
              Err(_) => {
                return;
              }
            };
            if recv_nonce <= data.game_last_nonce {
              return;
            }
            let mut nonce_bytes = [0u8; 12];
            nonce_bytes[8..].copy_from_slice(&recv_nonce.to_be_bytes());
            let formatted_nonce = Nonce::from_slice(&nonce_bytes);
            
            let key = data.cipher_key.clone();
            let key = GenericArray::from_slice(&key.as_slice());
            let cipher = ChaCha20Poly1305::new(key);
            
            let deciphered = match cipher.decrypt(&formatted_nonce, recv_data[4..].as_ref()) {
              Ok(decrypted) => {
                decrypted
              },
              Err(_err) => {
                return; // this is an erroneous packet, ignore it.
              },
            };
            data.game_last_nonce = recv_nonce;
            let recieved_server_info = match bincode::deserialize::<ServerPacket>(&deciphered) {
              Ok(packet) => packet,
              Err(_err) => {
                return; // ignore invalid packet
              }
            };
            //println!("server info: {:?}", recieved_server_info);
            
            // update data.
            data.player.is_dashing = recieved_server_info.player_packet_is_sent_to.is_dashing;
            
            // if server requests a position override:
            if recieved_server_info.player_packet_is_sent_to.override_position {
              println!("position override");
              // If we're dashing, update interpolation info.
              if data.player.is_dashing {
                // But if we're dashing (interpolating is set to true), then prepare to smoothly translate to that position.
                data.player.interpol_next = recieved_server_info.player_packet_is_sent_to.position_override;
                data.player.interpol_prev = data.player.position; // current position
              }
              // but under standard behaviour just update position.
              else {
                data.player.position = recieved_server_info.player_packet_is_sent_to.position_override;
              }
            }
            let ping = match recieved_server_info.timestamp.elapsed() {
              Ok(val) => val.as_millis(),
              Err(_) => 0,
            };

            // update the rest of the data.
            data.player.ping = ping as u16;
            data.player.health = recieved_server_info.player_packet_is_sent_to.health;
            data.player.secondary_charge = recieved_server_info.player_packet_is_sent_to.secondary_charge;
            data.player.character = recieved_server_info.player_packet_is_sent_to.character;
            data.player.is_dead = recieved_server_info.player_packet_is_sent_to.is_dead;
            data.player.buffs = recieved_server_info.player_packet_is_sent_to.buffs;
            data.player.previous_positions = recieved_server_info.player_packet_is_sent_to.previous_positions;
            data.player.team = recieved_server_info.player_packet_is_sent_to.team;
            data.player.last_shot_time = recieved_server_info.player_packet_is_sent_to.time_since_last_primary;
            data.player.time_since_last_dash = recieved_server_info.player_packet_is_sent_to.time_since_last_dash;
            data.player.last_secondary_time = recieved_server_info.player_packet_is_sent_to.time_since_last_secondary;
            data.player.stacks = recieved_server_info.player_packet_is_sent_to.stacks;
            data.player.passive_elapsed = recieved_server_info.player_packet_is_sent_to.passive_elapsed;
            
            data.game_objects = recieved_server_info.game_objects;
            data.gamemode_info = recieved_server_info.gamemode_info;

            // UPDATE OTHER PLAYERS
            let mut recieved_players: Vec<ClientPlayer> = Vec::new();
            for player in recieved_server_info.players {
              recieved_players.push(ClientPlayer::from_otherplayer(player));
            }
            // if a player left the game, recieved players has one less players, and other_players needs to
            // be adjusted since we index over other_players.
            data.players.retain(|element| {
              for player in recieved_players.clone() {
                if player.username == element.username {
                  return true;
                }
              }
              return false;
            });

            // if a new player joins, skip this part, update directly.
            if data.players.len() == recieved_players.len() {
              for player_index in 0..recieved_players.len() {
                // new position
                recieved_players[player_index].interpol_prev = data.players[player_index].interpol_next;
                recieved_players[player_index].interpol_next = recieved_players[player_index].position;
                recieved_players[player_index].position = data.players[player_index].position;

                recieved_players[player_index].used_primary = data.players[player_index].used_primary;
                recieved_players[player_index].used_secondary = data.players[player_index].used_secondary;
                recieved_players[player_index].used_dash = data.players[player_index].used_dash;
                // previous position
                // if not moving, force a position
                //recieved_players[player_index].position = Vector2 { x: 0.0, y: 0.0 }; //other_players[player_index].position;
                //recieved_players[player_index].interpol_prev = other_players[player_index].position;
              }
            }
            data.players = recieved_players;

            // MARK: | | Sound
            let mut sound_queue: Vec<(&str, AudioTrack, f32)> = Vec::new();
            let events = recieved_server_info.events;
            for event in events {
              //println!("{:?}", event);
              match event {
                GameEvent::AttackHit(object_type, owner, victim) => {
                  // if the bullet is ours
                  if owner == data.player.username {
                    let sound: &str = match object_type {
                      GameObjectType::HernaniBullet =>                    "audio/sword-hit.ogg",
                      GameObjectType::CynewynnSword =>                    "audio/sword-hit.ogg",
                      GameObjectType::FedyaProjectileRicochet =>          "audio/sword-hit.ogg",
                      GameObjectType::FedyaTurretProjectile =>            "audio/sword-hit.ogg",
                      GameObjectType::RaphaelleBullet =>                  "audio/sword-hit.ogg",
                      GameObjectType::RaphaelleBulletEmpowered =>         "audio/sword-hit.ogg",
                      GameObjectType::TemerityRocket =>                   "audio/explosion.ogg",
                      GameObjectType::TemerityRocketSecondary =>          "audio/sword-hit.ogg",
                      GameObjectType::WiroGunShot =>                      "audio/sword-hit.ogg",
                      GameObjectType::KoldoCannonBall =>                  "audio/sword-hit.ogg",
                      GameObjectType::KoldoCannonBallEmpowered =>         "audio/sword-hit.ogg",
                      GameObjectType::KoldoCannonBallEmpoweredUltimate => "audio/sword-hit.ogg",
                      _ => continue
                    };
                    sound_queue.push((sound, AudioTrack::SoundEffectSelf, 0.0));
                  }
                  // if it hit us
                  if victim == data.player.username {

                  }
                }
                GameEvent::AttackFired(object_type, owner) => {
                  let sound: &str = match object_type {
                    GameObjectType::HernaniBullet =>                    "audio/gunshot.ogg",
                    GameObjectType::CynewynnSword =>                    "audio/whoosh.ogg",
                    GameObjectType::FedyaProjectileRicochet =>          "audio/whoosh.ogg",
                    GameObjectType::FedyaTurretProjectile =>            "audio/whoosh.ogg",
                    GameObjectType::RaphaelleBullet =>                  "audio/whoosh.ogg",
                    GameObjectType::RaphaelleBulletEmpowered =>         "audio/whoosh.ogg",
                    GameObjectType::WiroGunShot =>                      "audio/whoosh.ogg",
                    GameObjectType::TemerityRocket =>                   "audio/rpgshot.ogg",
                    GameObjectType::TemerityRocketSecondary =>          "audio/rpgshot.ogg",
                    GameObjectType::KoldoCannonBall =>                  "audio/rpgshot.ogg",
                    GameObjectType::KoldoCannonBallEmpowered =>         "audio/rpgshot.ogg",
                    GameObjectType::KoldoCannonBallEmpoweredUltimate => "audio/rpgshot.ogg",
                    _ => {
                      continue;
                    }
                  };
                  if owner == data.username {
                    sound_queue.push((sound, AudioTrack::SoundEffectSelf, 0.0));
                    //println!("adding to sound queue");
                  }
                  else {
                    for other_player in data.players.clone() {
                      if other_player.username == owner {
                        let distance = (data.player.position - other_player.position).magnitude();
                        sound_queue.push((sound, AudioTrack::SoundEffectOther, distance));
                      }
                    }
                  }
                }
                GameEvent::WallHit(_object_type, _owner) => {

                }
              }
            }

            // play all sounds
            for (sound_path, track, distance) in sound_queue {
              // calculate volume from settings as a value from 0.0 to 1.0
              let sfx_self_vol = (data.settings.sfx_self_volume * data.settings.master_volume) / (100.0 * 100.0);
              let sfx_other_vol = (data.settings.sfx_other_volume * data.settings.master_volume) / (100.0 * 100.0);
              let music_vol = (data.settings.music_volume * data.settings.master_volume) / (100.0 * 100.0);

              let raw_volume = match track {
                AudioTrack::Music => music_vol,
                AudioTrack::SoundEffectOther => sfx_other_vol,
                AudioTrack::SoundEffectSelf => sfx_self_vol,
              };

              // reduce volume based on distance

              let cutoff = 7.0;
              let faloff = 0.05;

              //println!("dist: {:?}", distance);
              //println!("cut: {:?}", cutoff);

              let distance_volume_modifier = if distance < cutoff {
                1.0
              } else {
                //println!("{:?}", distance);
                1.0 / (faloff * (distance - cutoff) + 1.0)
              };
              //println!("{:?}", distance_volume_modifier);

              let volume = raw_volume * distance_volume_modifier;
              //println!("{:?}", volume);
              bevy_audio::play_sound(sound_path.to_string(), &mut com, asset_server.clone(), volume);
            }
          }
          // send our packet, at a lower frequency.
          if data.packet_timer.elapsed().as_secs_f32() > PACKET_INTERVAL {
            data.packet_timer = Instant::now();

            let client_packet = ClientPacket {
              position,
              movement: movement_raw,
              aim_direction: data.aim_direction,
              shooting_primary: data.player.shooting_primary,
              shooting_secondary: data.player.shooting_secondary,
              dashing: data.player.dashing,
              packet_interval: PACKET_INTERVAL,
              timestamp: SystemTime::now(),
            };

            // send data to server
            let serialized_packet: Vec<u8> = bincode::serialize(&client_packet).expect("Failed to serialize message");
            let mut nonce_bytes = [0u8; 12];
            nonce_bytes[8..].copy_from_slice(&data.nonce.to_be_bytes());
            
            let formatted_nonce = Nonce::from_slice(&nonce_bytes);
            let cipher_key = data.cipher_key.clone();
            let key = GenericArray::from_slice(&cipher_key);
            let cipher = ChaCha20Poly1305::new(&key);
            let ciphered = cipher.encrypt(&formatted_nonce, serialized_packet.as_ref()).expect("shit");
            
            let serialized_nonce: Vec<u8> = bincode::serialize::<u32>(&data.nonce).expect("oops");
            let serialized = [&serialized_nonce[..], &ciphered[..]].concat();
            let game_server_ip = data.game_server_ip.clone(); // .split(":").collect::<Vec<&str>>()[0];
            let game_server_ip = format!("{}:{}", game_server_ip, data.game_server_port);
            if let Some(ref mut game_socket) = data.game_socket {
              game_socket.send_to(&serialized, game_server_ip).expect("oops");
            }
            data.nonce += 1;
          }
        }
        // MARK: draw chat
      

        if data.paused {
          data.chat_open = false;
        }
        
        let chatbox_pos = bl_anchor + Vector2 {x: 5.0 * uiscale, y: - 100.0 * uiscale};
        let chatbox_size = Vector2 {x: 50.0 * uiscale, y: 80.0 * uiscale};
        if data.chat_open {
          let mut valid_msg = true;
          draw_rect(Color::Srgba(Srgba { red: 0.1, green: 0.1, blue: 0.1, alpha: 0.5 }), chatbox_pos, chatbox_size, CHAT_Z, &win, &mut com);
          
          data.chat_input.text_input(chatbox_pos - Vector2 {x: 0.0, y: -chatbox_size.y + 5.0*uiscale}, Vector2 { x: chatbox_size.x, y: 5.0*uiscale }, 4.0*uiscale, 20, uiscale, &mono_font, CHAT_Z+1, &mut com, &win, &m, &k, &mut ki);
          
          // cycle through friends
          // get a list of online friends (which we can chat to).
          let mut online_friends: Vec<String> = Vec::new();
          if data.current_menu == MenuScreen::Main(2) {
            online_friends.push(String::from("tc"));
            online_friends.push(String::from("ac"));
          }
          for friend in data.friend_list.clone() {
            if friend.1 == FriendShipStatus::Friends
            && friend.2 == true  {
              online_friends.push(friend.0);
            }
          }
          // cycle through friends if TAB is pressed
          if online_friends.len() >= 2 {
            if get_keys_pressed(&k).contains(&KeyCode::Tab) {
              data.selected_friend += 1;
              if data.selected_friend >= online_friends.len() {
                data.selected_friend = 0;
              }
            }
          } else {
            data.selected_friend = 0;
          }

          // draw selected friend indicator
          //let selected_friend_indicator_size = Vector2 {x: size.x}
          let peer_username;
          let mut message_type = ChatMessageType::Private;
          let mut displayed_selected_friend = if online_friends.len() > 0 {
            let split: Vec<&str> = (*online_friends[data.selected_friend]).split(":").collect();
            if *split[0] == username {
              peer_username = split[1];
            } else {
              peer_username = split[0];
            }
            peer_username
          } else {
            valid_msg = false; // don't bother the server.
            peer_username = "You";
            "No friends online."
          };
          if displayed_selected_friend == "tc" {
            message_type = ChatMessageType::Team;
            displayed_selected_friend = "Team";
          }
          if displayed_selected_friend == "ac" {
            message_type = ChatMessageType::All;
            displayed_selected_friend = "All";
          }
          let color = match message_type {
            ChatMessageType::Administrative => YELLOW,
            ChatMessageType::Private => PINK,
            ChatMessageType::Group => GREEN,
            ChatMessageType::Team => BLUE,
            ChatMessageType::All => ORANGE,
          };
          draw_text(&font, &format!("[TAB] Messaging: {}", displayed_selected_friend), Vector2 { x: chatbox_pos.x, y: chatbox_pos.y + chatbox_size.y - 9.0*uiscale }, Vector2 { x: 100.0*uiscale, y: 100.0*uiscale }, color, 3.0 * uiscale, CHAT_Z+3, Justify::Left, &win, &mut com);


          // message sending.
          if get_keys_pressed(&k).contains(&KeyCode::Enter) {
            if !data.chat_input.buffer.is_empty() {
              // send message
              let msg = data.chat_input.buffer.clone();
              // send message

              if valid_msg {

                data.packet_queue.push(
                  ClientToServer::SendChatMessage(peer_username.to_string(), msg.clone())
                );
              }
              let msg_type = match peer_username {
                "tc" => ChatMessageType::Team,
                "ac" => ChatMessageType::All,
                "gc" => ChatMessageType::Group,
                _ => ChatMessageType::Private,
              };
              let sender = match msg_type {
                ChatMessageType::Private => format!("You -> {}", peer_username),
                _ => "You".to_string(),
              };
              data.recv_messages_buffer.push((sender, msg, msg_type));
              data.chat_input.buffer.clear();
              //data.chat_input.selected = false;
              //data.chat_open = false;
              data.chat_timer = Instant::now();
            } 
          }
        }
        
        else {
          data.chat_input.selected = false;
        }
        
        // chat messages
        let chat_stay_open_time = 2.0;
        if data.chat_open || data.chat_timer.elapsed().as_secs_f32() < chat_stay_open_time && !data.paused {
          let font_size = 4.0 * uiscale;
          let y_step = font_size + 1.0 * uiscale;
          let mut current_y_step = 0;
          // in characters
          let max_line_len = 20;

          let mut messages = data.recv_messages_buffer.clone();
          messages.reverse();
          for recv_message in messages {
            let sender = recv_message.0;
            let message = recv_message.1;
            let message_type = recv_message.2;

            let mut text: String = format!("[{}] {}", sender, message);
            let line_count = text.len() / max_line_len + 1;

            let color: Srgba = match message_type {
              ChatMessageType::Administrative => YELLOW,
              ChatMessageType::All => ORANGE,
              ChatMessageType::Group => GREEN,
              ChatMessageType::Team => BLUE,
              ChatMessageType::Private => PINK,
            };
            
            current_y_step += line_count;
            for y in 0..line_count {
              let text_copy = text.clone();
              let line_text;
              if y < line_count-1 {
                let remaining_text;
                (line_text, remaining_text) = text_copy.split_at(max_line_len);
                text = remaining_text.to_string();
              } else {
                line_text = &text_copy;
              }
              let msg_y_bottom = chatbox_pos.y + chatbox_size.y - 14.0 * uiscale;
              let text_position = Vector2 {
                x: chatbox_pos.x,
                y: msg_y_bottom - (current_y_step - y - 1) as f32 * y_step + data.chat_scroll
              };
              if text_position.y <= msg_y_bottom && text_position.y > chatbox_pos.y {
                draw_text(&mono_font, &line_text, text_position, Vector2 { x: 100.0*vh, y: 100.0*vh }, color, font_size, CHAT_Z+2, Justify::Left, &win, &mut com);
              }
            }

          }

        }

        // chat bg
        if data.chat_timer.elapsed().as_secs_f32() < chat_stay_open_time && !data.chat_open{
          draw_rect(Color::Srgba(Srgba { red: 0.1, green: 0.1, blue: 0.1, alpha: 0.25 }), chatbox_pos, chatbox_size, CHAT_Z, &win, &mut com);
        }

        let scrollwheel = get_mouse_wheel(&mut mw);
        if data.chat_open {
          data.chat_scroll = (data.chat_scroll + scrollwheel.y * 300.0 * uiscale * delta_time).clamp(0.0, 10000.0*uiscale);
        }


        // talk to main server
        // MARK: Server Comm

        let mut buffer: [u8; 2048] = [0; 2048];
        let mut len = 0;
        if let Some(ref mut server_stream) = data.server_stream {
          match server_stream.read(&mut buffer) {
            Ok(0) => {
              data.notifications.push(
                Notification::new("Server has disconnected.", 2.0)
              );
              data.current_menu = MenuScreen::Login(0);
              return;
            }
            Ok(length) => {
              len = length;
            }
            Err(err) => {
              if err.kind() != ErrorKind::WouldBlock {
                data.notifications.push(
                  Notification::new(&format!("Network error: {:?}", err), 5.0)
                );
                return;
              }
            }
          }
        }

        let packets = network::tcp_decode_decrypt::<ServerToClientPacket>(buffer[..len].to_vec(), data.cipher_key.clone(), &mut data.last_nonce);
        let packets = match packets {
          Ok(packets) => packets,
          Err(_) => {
            println!("error decrypting");
            return;
          }
        };

        for packet in packets {
          match packet.information {
            // MARK: | Match assign
            ServerToClient::MatchAssignment(info) => {
              // a match assignment should override whatever we're doing.
              println!("Match assignment: {:?}", info);
              let selected_char = data.heroes_tabs.selected_tab();
              data.player.character = CHARACTER_LIST[selected_char];
              data.game_server_port = info.port;
              data.server_ip = String::from(CURRENT_SERVER_IP);
              data.game_id = info.game_id;
              data.queued = false;
              data.game_last_nonce = 0;
              data.background_tiles = load_background_tiles(32, 24);
              //let full_ip = get_ip();
              //let ip = full_ip.split(":").collect::<Vec<&str>>()[0];
              //let game_server_ip = format!("{}:{}", ip, info.port);

              // bind to udp socket for gameserver.
              let port = get_random_port();
              let ip = format!("0.0.0.0:{}", port);
              let udp_socket = UdpSocket::bind(ip);
              match udp_socket {
                Ok(socket) => {
                  socket.set_nonblocking(true).expect("oops");
                  data.game_socket = Some(socket);
                  // set game screen.
                  data.current_menu = MenuScreen::Main(2);
                }
                Err(err) => {
                  data.current_menu = MenuScreen::Main(0);
                  data.notifications.push(
                    Notification::new(&format!("Connection error: {:?}", err), 5.0),
                  );
                  return
                }
              }
            },
            ServerToClient::PlayerDataResponse(recv_player_stats) => {
              data.player_stats = recv_player_stats;
            }
            ServerToClient::FriendListResponse(recv_friend_list) => {
              data.friend_list = recv_friend_list;
            }
            ServerToClient::InteractionRefused(refusal_reason) => {
              let text = match refusal_reason {
                RefusalReason::FriendRequestAlreadySent => "Request Already Exists",
                RefusalReason::InternalError => "Internal Server Error",
                RefusalReason::UsernameInexistent => "User Inexistent",
                RefusalReason::AlreadyFriends => "Already friends",
                RefusalReason::UsersBlocked => "Users are blocked",
                RefusalReason::ThatsYouDummy => "That's you, silly goose!",
                RefusalReason::UserNotOnline => "User not online",
                RefusalReason::NotFriends => "Not friends with user",
                RefusalReason::InvalidInvite => "Invite expired/invalid",
                RefusalReason::AlreadyInPary => "Already in a party",
                RefusalReason::InvalidChannel => "Invalid selected channel",
                //there is no reason for these to exist here
                RefusalReason::InvalidUsername => "Unexpected Error (InvalidUsername)",
                RefusalReason::UsernameTaken => "Unexpected Error (UsernameTaken)",
              };
              data.notifications.push(Notification::new(text, 1.0));
            }
            ServerToClient::FriendRequestSuccessful => {
              data.notifications.push(Notification::new("Friend request sent", 1.0));
              // update friend list
              let buffer = data.friend_request_input.buffer.clone();

              data.friend_list.push(
                (buffer.clone(), get_friend_request_type(&username, &buffer), false)
              );
              data.friend_request_input.buffer = String::new();
            }
            ServerToClient::FriendshipSuccessful => {
              data.notifications.push(Notification::new("You are now friends!", 1.0));
              // update friend list
              for f_index in 0..data.friend_list.len() {
                if database::get_friend_name(&data.username, &data.friend_list[f_index].0) == data.friend_request_input.buffer {
                  data.friend_list[f_index].1 = FriendShipStatus::Friends;
                }
              }
              data.friend_request_input.buffer = String::new();
            }
            ServerToClient::ChatMessage(sender, message, message_type) => {
              // update friend list
              for f_index in 0..data.friend_list.len() {
                if sender == database::get_friend_name(&data.username, &data.friend_list[f_index].0) {
                  data.friend_list[f_index].2 = true;
                }
              }
              data.recv_messages_buffer.push((sender, message, message_type));
              data.chat_timer = Instant::now();
              data.chat_scroll = 0.0;
            }
            // lobby
            ServerToClient::LobbyInvite(inviting_user) => {
              data.lobby_invites.retain(|element| element != &inviting_user);
              data.lobby_invites.push(inviting_user.clone());
              data.notifications.push(
                Notification::new(&format!("{} invited you", inviting_user), 4.0)
              );
            }
            ServerToClient::LobbyUpdate(recvdata) => {
              //println!("{:?}", data);
              data.lobby = recvdata;
              // if we're in this list, delete us
              let username = data.username.clone();
              data.lobby.retain(|element| element.username != username);
            }
            ServerToClient::MatchEnded(result) => {
              println!("Match ended! {:?}", result);
            }
            _ => {}
          }
        }

        let packet_queue = data.packet_queue.clone();
        let cipher_key = data.cipher_key.clone();
        let mut nonce = data.nonce.clone();
        if let Some(ref mut server_stream) = data.server_stream {
          for packet in packet_queue {
            server_stream.write_all(
              &network::tcp_encode_encrypt(packet, cipher_key.clone(), nonce).expect("oops")
            ).expect("idk 1");
            nonce += 1;
          }
        }
        data.packet_queue = Vec::new();
        data.nonce = nonce;

        // settings screen
        if get_keys_pressed(&k).contains(&KeyCode::Escape) {
          if data.chat_open {
            data.chat_open = false;
            data.chat_input.buffer = String::new();
          }
          else {
            data.paused = !data.paused;
            if data.paused == false {
              data.settings_open = false;
            }
          }
        }
        if get_keys_pressed(&k).contains(&KeyCode::Enter) {
          data.chat_open = !data.chat_open;
          data.chat_scroll = 0.0;
          data.chat_input.selected = data.chat_open;
          data.chat_timer = Instant::now();
        }

        if data.paused {
          let (paused, quit) = draw_pause_menu(uiscale, vh, vw, &mut data, ESC_MENU_Z, &font, &mut win, &mut com, &m, k);
          data.paused = paused;
          if quit {
            // if in menus
            if mode != 2 {
              exit.write(AppExit::Success);
            }
            // if in-game
            else {
              data.current_menu = MenuScreen::Main(0);
            }
          }
        }
      }

      // MARK: Login/register Screen
      MenuScreen::Login(login_step) => {
        clear_background(WHITE, &win, &mut com);

        // login / register tabs
        data.tabs_login.update_size(tl_anchor + Vector2 { x: 35.0 * uiscale, y: 20.0 * uiscale}, Vector2 { x: 40.0*uiscale, y: 6.0*uiscale }, 4.0*uiscale);
        data.tabs_login.draw_and_process(uiscale, true, 0, &font, &win, &mut com, &m);
        let logging_in = match data.tabs_login.selected_tab() {
          0 => {true}
          _ => {false}
        };

        // input fields
        let input_size = Vector2 { x: 40.0*uiscale, y: 5.0 * uiscale };
        let user_input_pos = tl_anchor + Vector2 {x: 35.0 * uiscale, y: 35.0 * uiscale};
        let password_input_pos = tl_anchor + Vector2 {x: 35.0 * uiscale, y: 45.0 * uiscale};
        draw_text(&font, "Username", tl_anchor + Vector2 {x: 35.0 * uiscale, y: 32.0 * uiscale}, input_size, BLACK, 3.0 * uiscale, MENU_Z, Justify::Left, &win, &mut com);
        tooltip(user_input_pos, input_size, "3-20 characters.", Vector2 { x: 30.0*uiscale, y: 5.0*uiscale }, uiscale, vw, &font, mouse_pos, TOOLTIP_Z, &win, &mut com);
        data.username_input.text_input(user_input_pos, input_size, 4.0 * uiscale, 15, vh, &mono_font, MENU_Z, &mut com, &win, &m, &k, &mut ki);
        tooltip(password_input_pos, input_size, "8 characters minimum.", Vector2 { x: 30.0*uiscale, y: 10.0*uiscale }, uiscale, vw, &font, mouse_pos, TOOLTIP_Z, &win, &mut com);
        draw_text(&font, "Password", tl_anchor + Vector2 {x: 35.0 * uiscale, y: 42.0 * uiscale}, input_size, BLACK, 3.0 * uiscale, MENU_Z, Justify::Left, &win, &mut com);
        data.password_input.text_input(password_input_pos, input_size, 4.0 * uiscale, 15, vh, &mono_font, MENU_Z, &mut com, &win, &m, &k, &mut ki);

        // confirm button
        let mut confirm_button = Button::new(bl_anchor + Vector2 { x: 35.0*uiscale, y: -20.0*uiscale}, Vector2 { x: 20.0*uiscale, y: 5.0*uiscale }, if logging_in {"Login"} else {"Register"}, 4.0*uiscale);
        confirm_button.draw(uiscale, true, MENU_Z, &font, &win, &mut com);

        // remember me checkbox
        let credentials_checkbox_pos = tl_anchor + Vector2 {x: 35.0 * uiscale, y: 55.0 * uiscale};
        let credentials_checkbox_size = 5.0 * uiscale;
        let credentials_changed = checkbox(credentials_checkbox_pos, credentials_checkbox_size, "Remember me", 4.0*uiscale, vh, &mut data.settings.store_credentials, 0, &font, &win, &mut com, &m);
        if credentials_changed {
          data.settings.save();
        }
        tooltip(credentials_checkbox_pos, Vector2 { x: credentials_checkbox_size, y: credentials_checkbox_size }, "Stores your password in your OS keyring, like Safari.", Vector2 { x: 40.0*uiscale, y: 13.0*uiscale }, uiscale, vw, &font, mouse_pos, TOOLTIP_Z, &win, &mut com);

        // offline mode
        let mut offline_mode_button = Button::new(br_anchor - Vector2 {x: 33.0 * uiscale,y: 11.0 * uiscale }, Vector2 { x: 30.0*uiscale, y: 8.0*uiscale }, "Play offline", 4.0*uiscale);
        offline_mode_button.draw(uiscale, true, MENU_Z, &font, &win, &mut com);
        if offline_mode_button.was_released(&win, &m) {
          data.current_menu = MenuScreen::Main(0);
          data.server_ip = String::from("127.0.0.1:25569")
        }

        // keep these immutable.
        let password = data.password_input.buffer.clone();
        let username = data.username_input.buffer.clone();

        let mut rng = OsRng;
        match login_step {
          // MARK: Login/Register 0
          0 => {
            if confirm_button.was_released(&win, &m) {
              // check credential validity.
              let mut credentials_valid = true;
              if !valid_password(password.clone()) {
                data.notifications.push(
                  Notification::new("Unsafe password", 1.0)
                );
                credentials_valid = false;
                data.username_input.selected = false;
                data.password_input.selected = true;
              }
              if !valid_username(username.clone()) {
                data.notifications.push(
                  Notification::new("Invalid username", 1.0)
                );
                credentials_valid = false;
                data.username_input.selected = true;
                data.password_input.selected = false;
              }

              if credentials_valid {


                // store credentials.
                if data.settings.store_credentials {
                  data.settings.saved_username = data.username_input.buffer.clone();
                  save_password(&password, &data.settings.saved_username.clone(), &mut data.notifications);
                }

                // set next step.
                if logging_in {
                  data.current_menu = MenuScreen::Login(1)
                }
                else {
                  data.current_menu = MenuScreen::Login(3)
                }

                // set timer
                data.opake_data.timeout = Instant::now();
                
                // Attempt connection to server.
                draw_text(&font, "Attempting connection...", tl_anchor + Vector2 {x: 35.0 * uiscale, y: 55.0 * uiscale}, Vector2 { x: 40.0*uiscale, y: 5.0 * uiscale }, BLACK, 5.0 * uiscale, MENU_Z, Justify::Left, &win, &mut com);
                
                if data.server_stream.is_none() {
                  match TcpStream::connect(&data.server_ip) {
                    Ok(stream) => {
                      data.server_stream = Some(stream);
                      if let Some(ref server_stream) = data.server_stream {
                        server_stream.set_nonblocking(true).expect("idk");
                      }
                    }
                    Err(err) => {
                      // back to login screen.
                      data.current_menu = MenuScreen::Login(0);
                      data.notifications.push(
                        Notification::new(&format!("Connection to server failed. Reason: {:?}", err), 5.0)
                      )
                    }
                  }
                }
              }
            }
          }
          // MARK: login
          // OPAKE login step 1.
          1 => {
            let client_login_start_result: ClientLoginStartResult<DefaultCipherSuite> = ClientLogin::<DefaultCipherSuite>::start(&mut rng, password.as_bytes()).expect("Oops");
            let message = client_login_start_result.message.clone();
            let message = ClientToServerPacket {
              information: ClientToServer::LoginRequestStep1(username.clone(), message)
            };
            data.opake_data.client_login_start_result = Some(client_login_start_result);
            if let Some(ref mut server_stream) = data.server_stream {
              match server_stream.write_all(&network::tcp_encode(&message).expect("oops")) {
                Ok(_) => {
                  data.current_menu = MenuScreen::Login(2);
                }
                Err(err) => {
                  //registration failed.
                  data.current_menu = MenuScreen::Login(0);
                  data.notifications.push(
                    Notification::new(&format!("Connection failed. Reason: {:?}", err), 5.0)
                  )
                }
              }
            }
          }
          // OPAKE login step 2.
          2 => {

            // recieve packet
            let mut buffer: [u8; 2048] = [0; 2048];
            let mut len = 0;
            if let Some(ref mut server_stream) = data.server_stream {

              len = match server_stream.read(&mut buffer) {
                Ok(0) => {
                  // server disconnects us.
                  data.server_stream = None;
                  data.notifications.push(
                    Notification::new("Server has disconnected.", 2.0)
                  );
                  data.current_menu = MenuScreen::Login(0);
                  return;
                }
                Ok(n) => {n}
                Err(err) => {
                  match err.kind() {
                    // no message this time, try again.
                    ErrorKind::WouldBlock => {
                      // waited too long, timeout.
                      if data.opake_data.timeout.elapsed().as_secs_f32() > 3.0 {
                        data.notifications.push(Notification::new("Timed out", 1.0));
                        data.current_menu = MenuScreen::Login(0);
                        return;
                      }
                      // try again.
                      return;
                    }
                    // other errors
                    _ => {
                      data.notifications.push(
                        Notification::new(&format!("Unkown error during login. Reason: {:?}", err), 5.0)
                      );
                      data.current_menu = MenuScreen::Login(0);
                      return;
                    }
                  }
                }
              };
            }
            if len > 2048 {
              data.notifications.push(
                Notification::new("Buffer overflow.", 2.0)
              );
              data.current_menu = MenuScreen::Login(0);
              return;
            }
            let packets = network::tcp_decode::<ServerToClientPacket>(buffer[..len].to_vec());
            let packets = match packets {
              Ok(packets) => packets,
              Err(_err) => {
                data.current_menu = MenuScreen::Login(0);
                return;
              }
            };
            for packet in packets {
              match packet.information {
                
                // login finish (step 3)
                ServerToClient::LoginResponse1(server_response) => {
                  // placeholder.
                  let mut message: ClientToServerPacket = ClientToServerPacket { information: ClientToServer::LobbyLeave };
                  if let Some(ref client_login_start_result) = data.opake_data.client_login_start_result {
                    let client_login_finish_result = match client_login_start_result.clone().state.finish(
                      &mut rng,
                      password.as_bytes(),
                      server_response,
                      ClientLoginFinishParameters::default(),
                    ) {
                      Ok(result) => {result},
                      Err(_err) => {
                        data.notifications.push(
                          Notification::new("Wrong password", 2.0)
                        );
                        data.username_input.selected = false;
                        data.password_input.selected = true;
                        data.current_menu = MenuScreen::Login(0);
                        return;
                      }
                    };
                    let session_key = client_login_finish_result.session_key;
                    
                    // Shrink PAKE key
                    // put this in a function later
                    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, &[]);
                    let prk = salt.extract(&session_key);
                    let okm = prk.expand(&[], hkdf::HKDF_SHA256).unwrap();
                    let mut key_bytes = [0u8; 32];
                    okm.fill(&mut key_bytes).unwrap();
                    let key = Vec::from(&key_bytes);
                    data.cipher_key = key;
                    let credential_finalization = client_login_finish_result.message;
                    message = ClientToServerPacket {
                      information: ClientToServer::LoginRequestStep2(credential_finalization)
                    };
                  }
                  
                  if let Some(ref mut server_stream) = data.server_stream {

                    match server_stream.write_all(&network::tcp_encode(&message).expect("oops")) {
                      Ok(_) => {}
                      Err(_) => {
                        data.current_menu = MenuScreen::Login(0);
                        return;
                      }
                    }
                  }
                  // login successful, go to main screen.
                  data.username = data.username_input.buffer.clone();
                  data.current_menu = MenuScreen::Main(0);
                  data.packet_queue.push(ClientToServer::GetFriendList);
                  return;
                }

                // error
                ServerToClient::InteractionRefused(reason) => {
                  data.notifications.push(
                    Notification { start_time: Instant::now(), text: match reason {
                      RefusalReason::InternalError => String::from("Internal Server Error"),
                      RefusalReason::InvalidUsername => String::from("Invalid Username"),
                      RefusalReason::UsernameTaken => String::from("Username Taken"),
                      RefusalReason::UsernameInexistent => String::from("Incorrect Username"),
                      _ => String::from("Unexpected Error"),
                    }, duration: 1.5 }
                  );
                  data.username_input.selected = true;
                  data.password_input.selected = false;
                  data.current_menu = MenuScreen::Login(0);
                  return;
                }
                // any other packet
                _ => {}
              }
            }
          }

          // MARK: register
          // OPAKE register step 1.
          3 => {
            let client_registration_start_result = ClientRegistration::<DefaultCipherSuite>::start(&mut rng, password.as_bytes()).expect("oops");
            let message = client_registration_start_result.clone().message;
            // server packet
            let message = ClientToServerPacket {
              information: ClientToServer::RegisterRequestStep1(username.clone(), message)
            };
            data.opake_data.client_registration_start_result = Some(client_registration_start_result);
            if let Some(ref mut server_stream) = data.server_stream {
              match server_stream.write_all(&network::tcp_encode(message).expect("oops")) {
                Ok(_) => {
                  data.current_menu = MenuScreen::Login(4);
                }
                Err(err) => {
                  //registration failed.
                  data.current_menu = MenuScreen::Login(0);
                  data.notifications.push(
                    Notification::new(&format!("Connection failed. Reason: {:?}", err), 5.0)
                  )
                }
              }
            }
          }
          // OPAKE register step 3 (step 2 client POV, step 3 overall). 
          4 => {
            println!("1");
            let mut len: usize = 0;
            let mut buffer: [u8; 2048] = [0; 2048];
            if let Some(ref mut server_stream) = data.server_stream {
              // recieve packet.

              // get packet length.
              len = match server_stream.read(&mut buffer) {
                Ok(0) => {
                  // server disconnects us.
                  data.server_stream = None;
                  data.notifications.push(
                    Notification::new("Server has disconnected.", 2.0)
                  );
                  data.current_menu = MenuScreen::Login(0);
                  return;
                }
                Ok(n) => {n}
                Err(err) => {
                  match err.kind() {
                    // no message this time, try again.
                    ErrorKind::WouldBlock => {
                      println!("wouldblock");
                      // waited too long, timeout.
                      if data.opake_data.timeout.elapsed().as_secs_f32() > 3.0 {
                        data.notifications.push(Notification::new("Timed out", 1.0));
                        data.current_menu = MenuScreen::Login(0);
                        return;
                      }
                      // try again.
                      return;
                    }
                    // other errors
                    _ => {
                      data.notifications.push(
                        Notification::new(&format!("Unkown error during login. Reason: {:?}", err), 5.0)
                      );
                      data.current_menu = MenuScreen::Login(0);
                      return;
                    }
                  }
                }
              };
            }
            println!("2");

            if len > 2048 {
              data.notifications.push(
                Notification::new("Buffer overflow.", 2.0)
              );
              data.current_menu = MenuScreen::Login(0);
              return;
            }
            // read recieved packets.
            let packets = network::tcp_decode::<ServerToClientPacket>(buffer[..len].to_vec());
            let packets = match packets {
              Ok(packets) => packets,
              Err(_err) => {
                data.current_menu = MenuScreen::Login(0);
                return;
              }
            };
            let mut message: ClientToServerPacket = ClientToServerPacket {
              information: ClientToServer::LobbyLeave,
            }; // placeholder.
            for packet in packets {
              match packet.information {
                // register finish (step 3)
                ServerToClient::RegisterResponse1(server_message) => {
                  if let Some(ref mut client_registration_start_result) = data.opake_data.client_registration_start_result {
                    let client_registration_finish_result = client_registration_start_result.clone().state.finish(
                      &mut rng,
                      password.as_bytes(),
                      server_message,
                      ClientRegistrationFinishParameters::default(),
                    ).expect("oops");
                    let registration_upload = client_registration_finish_result.message;
                    message = ClientToServerPacket {
                      information: ClientToServer::RegisterRequestStep2(registration_upload)
                    };
                  }
                  // send the final result back to the server.
                  if let Some(ref mut server_stream) = data.server_stream {
                    match server_stream.write_all(&network::tcp_encode(&message).expect("oops")) {
                      Ok(_) => {}
                      Err(_) => {
                        //registration failed.
                        println!("hello");
                        data.current_menu = MenuScreen::Login(0);
                        continue;
                      }
                    }
                  }
                  // register successful
                  data.notifications.push(
                    Notification::new("Account created!", 1.5)
                  );
                  data.tabs_login.set_selected(0);  
                  data.current_menu = MenuScreen::Login(0);
                }
                
                // error
                ServerToClient::InteractionRefused(reason) => {
                  data.notifications.push(
                    Notification { start_time: Instant::now(), text: match reason {
                      RefusalReason::InternalError => String::from("Internal Server Error"),
                      RefusalReason::InvalidUsername => String::from("Invalid Username"),
                      RefusalReason::UsernameTaken => String::from("Username Taken"),
                      RefusalReason::UsernameInexistent => String::from("Incorrect Username"),
                      _ => String::from("Unexpected Error"),
                    }, duration: 1.0 }
                  );
                  data.username_input.selected = true;
                  data.password_input.selected = true;
                  data.current_menu = MenuScreen::Login(0);
                }
                // any other packet
                _ => {}
              }
            }
          }
          _ => {panic!();}
        }
      }
    }
    // MARK: Always running:

    // draw notifications
    for n_index in 0..data.notifications.len() {
      data.notifications[n_index].draw(uiscale, tr_anchor, 4.0*uiscale, n_index, 127, &font, &win, &mut com);
    }
    for n_index in 0..data.notifications.len() {
      if data.notifications[n_index].start_time.elapsed().as_secs_f32() > data.notifications[n_index].duration {
        data.notifications.remove(n_index); // you've overstayed your welcome, pal.
        break;
      }
    }
    // extra input keys
    if is_window_focused(&win) {
      let device_state: DeviceState = DeviceState::new();
      let keys: Vec<Keycode> = device_state.get_keys();
      
      // key binds
      if keys.is_empty() {
        data.fullscreen_pressed = false;
        data.chat_pressed = false;
      }

      for key in keys {
        let key = key as u16;
        if key == data.settings.keybinds.fullscreen.0 || key == data.settings.keybinds.fullscreen.1 {
          if !data.fullscreen_pressed {
            data.settings.fullscreen = !data.settings.fullscreen;
            data.fullscreen_pressed = true;
            set_fullscreen(data.settings.fullscreen, &mut win);
          }
        }
        //if key == data.settings.keybinds.open_chat.0 || key == data.settings.keybinds.open_chat.1 {
        //  if !data.chat_pressed && data.chat_timer.elapsed().as_secs_f32() > 0.5 {
        //    //data.chat_open = !data.chat_open;
        //    println!("chat open: {:?}", data.chat_open);
        //    data.chat_pressed = true;
        //    data.chat_open = true;
        //    data.chat_input.selected = true;
        //    data.chat_timer = Instant::now();
        //  }
        //  //data.chat_open = true;
        //  //data.chat_input.selected = true;
        //  //data.chat_pressed = true;
        //} else {
        //  data.chat_pressed = false;
        //}
      }
    }
    // debug
    draw_text(&font, &format!("fps: {:?}", (1.0 / time.delta().as_secs_f32()) as u16), Vector2 { x: 0.0, y: 0.0 }, Vector2 { x: 100.0, y: 100.0 }, BLACK, 10.0, 127, Justify::Left, &win, &mut com);
  }
}

// MARK: Clearer
fn sprite_clearer(mut commands: Commands, query: Query<Entity, With<DeleteAfterFrame>>) {
  for entity in query {    
    commands.entity(entity).despawn();
  }
}

// MARK: Setup
fn setup(mut commands: Commands) {
  commands.init_resource::<GameData>();
  commands.init_resource::<Settings>();
  commands.spawn(Camera2d);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuScreen {
  /// Login menu. Used as starting menu screen. The u8 starts at 0.
  /// 
  /// The u8 indicates the current login step of the OPAKE login.
  /// 0 means it hasn't started yet and we're waiting for confirmation.
  Login(u8),
  /// The main game. The u8 indicates which part of the game we're in.
  /// - 0 indicates the main menu.
  /// - 1 indicates the practice range.
  /// - 2 indicates the game itself.
  Main(u8),
}

pub struct OpakeData {
  pub timeout: Instant,
  pub client_registration_start_result: Option<ClientRegistrationStartResult<DefaultCipherSuite>>,
  pub client_login_start_result: Option<ClientLoginStartResult<DefaultCipherSuite>>,
}

pub const CHARACTER_LIST: [Character; 7]  = [
  Character::Cynewynn,
  Character::Fedya,
  Character::Hernani,
  Character::Koldo,
  Character::Raphaelle,
  Character::Temerity,
  Character::Wiro,
];

#[derive(Clone, Debug, Copy)]
pub enum AudioTrack {
  Music,
  SoundEffectSelf,
  SoundEffectOther,
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
      let random_num_raw = crappy_random();
      let mut random_num_f = (random_num_raw as f64) / u32::MAX as f64;
      random_num_f *= 6.0;
      let random_num = random_num_f.round() as usize;
      let pos_x: i16 = x.try_into().unwrap();
      let pos_x: f32 = (pos_x - extra_offset_x as i16) as f32;
      let pos_y: i16 = y.try_into().unwrap();
      let pos_y: f32 = (pos_y - extra_offset_y as i16) as f32 + 0.5;
      if (x + y) % 2 == 1 {
        tiles.push(BackGroundTile { position: Vector2 { x: pos_x, y: pos_y }, object_type: bright_tiles[random_num].clone() });
      } else {
        tiles.push(BackGroundTile { position: Vector2 { x: pos_x, y: pos_y }, object_type: dark_tiles[random_num].clone() });
      }
    }
  }
  return tiles;
}

fn exit_catcher(mut exit_events: MessageReader<AppExit>, settings: Res<Settings>) {
  for exit_event in exit_events.read() {
    match exit_event {
      AppExit::Success => {
        println!("App exited successfully.");
        // save settings
        settings.save();
      }
      AppExit::Error(err) => {
        println!("App exited with an error.");
      }
    }
  }
}