/// Functions and structs related to any form of maths
/// or logic, like `Vector2` or movement logic functions.
pub mod maths;
/// Constant parameters, like TILE_SIZE, DEFAULT_IP_ADDRESS, etc...
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
/// Abstraction for playing sounds & stuff.
pub mod audio;
///// The actual game, once it's connected to the server.
//pub mod game;
/// Immediate mode rendering wrapper for Bevy.
pub mod bevy_immediate;
/// Higher level wrapper for any graphics.
pub mod bevy_graphics;

use std::{collections::HashMap, io::{ErrorKind, Read, Write}, net::{TcpStream, UdpSocket}, time::Instant};
use bevy::{color::palettes::css::*, input::{keyboard::KeyboardInput, mouse::MouseWheel}, prelude::*, window::WindowResolution};
use bevy_immediate::*;
use bevy_graphics::*;
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce};
use maths::*;
use opaque_ke::{generic_array::GenericArray, ClientLogin, ClientLoginFinishParameters, ClientLoginStartResult, ClientRegistration, ClientRegistrationFinishParameters, ClientRegistrationStartResult};
use rand::rngs::OsRng;
use ring::hkdf;
use crate::{bevy_graphics::Button, const_params::{DefaultCipherSuite, PACKET_INTERVAL}, database::{get_friend_request_type, FriendShipStatus}, filter::{valid_password, valid_username}, gamedata::*, mothership_common::{ChatMessageType, ClientToServer, ClientToServerPacket, GameMode, LobbyPlayerInfo, MatchRequestData, PlayerStatistics, RefusalReason, ServerToClient, ServerToClientPacket}, network::get_ip};


#[bevy_main]
pub fn main() {
  App::new()
    .add_systems(Startup, setup)
    .add_systems(PreUpdate, sprite_clearer)
    .add_systems(Update, main_thread)
    .add_plugins(
      DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
          title: "Sylvan Row".into(),
          name: Some("sylvan.row".into()),
          resolution: WindowResolution::new(720, 480),
          present_mode: bevy::window::PresentMode::AutoNoVsync, // vsync fucking sucks
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
  pub chat_input: TextInput,
  pub friend_request_input: TextInput,
  pub recv_messages_buffer: Vec<(String, String, ChatMessageType)>,
  pub chat_timer: Instant,
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

  pub game_server_port: u16,
  pub game_id: u128,
  pub game_socket: Option<UdpSocket>,
  pub character_properties: HashMap<Character, CharacterProperties>,
  pub packet_timer: Instant,
  pub position:           Vector2,
  /// Raw movement vector
  pub movement:           Vector2,
  pub aim_direction:      Vector2,
  pub shooting_primary:   bool,
  pub shooting_secondary: bool,
  pub dashing:         bool,
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
      friend_list: Vec::new(),
      username: "Player".to_string(),
      friend_request_input: TextInput {
        selected: false,
        buffer: String::new(),
        hideable: false,
        show_password: false,
      },
      chat_input: TextInput {
        selected: false,
        buffer: String::new(),
        hideable: false,
        show_password: false,
      },
      recv_messages_buffer: Vec::new(),
      chat_timer: Instant::now(),
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
      game_server_port: 0,
      game_id: 0,
      game_socket: None,
      character_properties: load_characters(),
      packet_timer: Instant::now(),
      movement: Vector2::new(),
      position: Vector2::new(),
      aim_direction: Vector2::new(),
      shooting_primary: false,
      shooting_secondary: false,
      dashing: false,
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
  mw: MessageReader<MouseWheel>,
  t: Res<Touches>,
  mut exit: MessageWriter<AppExit>,
) {
  if let Some(mut data) = data {
    // MAIN LOOP
    let mut win = window.single_mut().expect("oops");
    let vw = win.width() / 100.0;
    let vh = win.height() / 100.0;
    
    if data.startup {
      data.startup = false;
      set_fullscreen(data.settings.fullscreen, &mut win);
    }


    let server_ip = "127.0.0.1:25569";

    // calculate UI scale
    let size_min = f32::min(vw, vh);
    let uiscale = if size_min < 5.0 {2.5} else if size_min < 10.0 {5.0} else {10.0};
    let tl_anchor = Vector2 {x: 0.0, y: 0.0};
    let tr_anchor = Vector2 {x: 100.0*vw, y: 0.0};
    let bl_anchor = Vector2 {x: 0.0, y: 100.0*vh};
    let br_anchor = Vector2 {x: 100.0*vw, y: 100.0*vh};

    let font: Handle<Font> = asset_server.load("fonts/Roboto-Black.ttf");
    let mouse_pos = get_mouse_pos(&win);

    let username = data.username.clone();
    match data.current_menu {
      // MARK: Main
      MenuScreen::Main(mode) => {
        // menu
        if mode != 2 {
          clear_background(WHITE, &win, &mut com);
          let paused = data.paused;
          data.main_tabs.update_size(tl_anchor + Vector2 { x: 5.0 * vw, y: 5.0 * uiscale}, Vector2 { x: 90.0*vw, y: 8.0*uiscale }, 6.0*uiscale);
          data.main_tabs.draw_and_process(uiscale, !paused, 0, &font, &win, &mut com, &m);
          
          // play
          if data.main_tabs.selected_tab() == 0 {
            let selected_char = data.heroes_tabs.selected_tab();
            if !data.queued {
              checkbox(br_anchor - Vector2 {x: 30.0*uiscale, y: 21.0*uiscale }, 4.0*uiscale, "1v1", 4.0*uiscale, uiscale, &mut data.checkbox_1v1, 0, &font, &win, &mut com, &m);
              checkbox(br_anchor - Vector2 {x: 17.5*uiscale, y: 21.0*uiscale }, 4.0*uiscale, "2v2", 4.0*uiscale, uiscale, &mut data.checkbox_2v2, 0, &font, &win, &mut com, &m);
            } if data.queued {
              checkbox(br_anchor - Vector2 {x: 30.0*uiscale, y: 21.0*uiscale }, 4.0*uiscale, "1v1", 4.0*uiscale, uiscale, &mut data.checkbox_1v1.clone(), 0, &font, &win, &mut com, &m); // clone to disable writes
              checkbox(br_anchor - Vector2 {x: 17.5*uiscale, y: 21.0*uiscale }, 4.0*uiscale, "2v2", 4.0*uiscale, uiscale, &mut data.checkbox_2v2.clone(), 0, &font, &win, &mut com, &m);
            }


            let mut play_button = Button::new(br_anchor - Vector2 { x: 30.0*uiscale, y: 15.0*uiscale }, Vector2 { x: 25.0*uiscale, y: 13.0*uiscale }, "Play", 8.0*uiscale);
            play_button.draw(uiscale, !paused, 0, &font, &win, &mut com);
            if data.queued {
              draw_text(&font, "In queue...", br_anchor + Vector2 {x: - 30.0*uiscale, y: - 24.0*uiscale}, Vector2 { x: 100.0*uiscale, y: 100.0*uiscale }, BLACK, 5.0*uiscale, 0, &win, &mut com);
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
            draw_text(&font, "Lobby", Vector2 {x: lobby_position.x, y: lobby_position.y-3.0*uiscale}, Vector2 {x: 100.0*vh, y: 100.0*vh}, BLACK, 3.0*uiscale, 0, &win, &mut com);
            for (i, player) in lobby.iter().enumerate() {
              draw_rect(Color::Srgba(BLUE), lobby_position + Vector2 {x: 0.0, y: (i as f32)*y_offset}, lobby_size, 0, &win, &mut com );
              draw_rect(Color::Srgba(SKY_BLUE), lobby_position + Vector2{x: inner_shrink, y:inner_shrink} + Vector2 {x: 0.0, y: (i as f32)*y_offset}, lobby_size - Vector2{x: inner_shrink*2.0, y:inner_shrink*2.0}, 0, &win, &mut com);
              let is_ready_color = if player.is_ready {LIME} else {RED};
              let is_ready_text = if player.is_ready {"Ready"} else {"Not Ready"};
              draw_text(&font, &format!("{}", player.username), Vector2 {x: lobby_position.x + 2.0*vh, y: lobby_position.y + (i as f32)*y_offset}, Vector2{x: 100.0*vh, y: 100.0*vh}, BLACK, 3.0*uiscale, 0, &win, &mut com);
              draw_text(&font, &format!("{}", is_ready_text), Vector2 {x: lobby_position.x + lobby_size.x * 0.67, y: lobby_position.y + (i as f32)*y_offset}, Vector2{x: 100.0*vh, y: 100.0*vh}, is_ready_color, 3.0*uiscale, 0, &win, &mut com);
            }
            // lobby leave button
            if lobby.len() > 1 {
              let mut leave_button = Button::new(lobby_position + Vector2 {x: 0.0, y: y_offset * (lobby.len() as f32) + inner_shrink}, Vector2 { x: lobby_size.x/2.0, y: lobby_size.y - inner_shrink }, "Leave", 5.0*vh);
              leave_button.draw(vh, !paused, 0, &font, &win, &mut com);
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
            data.heroes_tabs.draw_and_process(uiscale, !paused, 0, &font, &win, &mut com, &m);

            let selected = data.heroes_tabs.selected_tab();
            let selected_character = CHARACTER_LIST[selected];
            let character_descriptions = CharacterDescription::create_all_descriptions(data.character_properties.clone());
            for i in 0..4 {
              let texture = asset_server.load(format!("ui/temp_ability_{}.png", i+1));
              let size = Vector2 { x: 10.0*uiscale, y: 10.0*uiscale };
              draw_ability_icon(tl_anchor + Vector2 { x: 10.0*uiscale + (size.x + 4.0*uiscale) * i as f32, y: 67.5*uiscale }, size, i, false, 1.0, uiscale, vw, &font, character_descriptions.clone(), selected_character, 5, &texture, &win, &mut com);
            }
            let profile_texture = asset_server.load(format!("characters/{}/textures/mini-profile.png", selected_character.name().to_lowercase() ));
            let profile_texture = Texture {
              image: profile_texture,
              size: Vec2 { x: 900.0, y: 1000.0 }
            };
            draw_sprite(&profile_texture, tl_anchor + Vector2 {x: 10.0*uiscale, y: 15.0*uiscale}, Vector2 { x: 55.0*uiscale, y: 55.0*uiscale*profile_texture.aspect_ratio() }, 0, &win, &mut com);
          }
          
          // tutorial
          if data.main_tabs.selected_tab() == 2 {
            
          }
          
          // stats
          if data.main_tabs.selected_tab() == 3 {
            
          }
          
          // friends
          if data.main_tabs.selected_tab() == 4 {
            
          }
        }
        // MARK: Game
        if mode == 1 || mode == 2 {
          println!("game");
          



          // MARK: | game graphics



          

          // MARK: | game input




          // MARK: | game net
          let mut buffer: [u8; 2048] = [0; 2048];
          let mut len = 0;
          if let Some(ref mut game_socket) = data.game_socket {
            match game_socket.recv_from(&mut buffer) {
              Ok((length, _addr)) => {
                len = length;
              }
              Err(err) => {
                //if err.kind() == std::io::ErrorKind::WouldBlock {
                //                    
                //}
                println!("{:?}", err);
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
            if recv_nonce <= data.last_nonce {
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
            data.last_nonce = recv_nonce;
            let recieved_server_info = match bincode::deserialize::<ServerPacket>(&deciphered) {
              Ok(packet) => packet,
              Err(_err) => {
                return; // ignore invalid packet
              }
            };

            println!("{:?}", recieved_server_info);

            // send out a udp packet
            if data.packet_timer.elapsed().as_secs_f32() > PACKET_INTERVAL {
              data.packet_timer = Instant::now();

              let client_packet = ClientPacket {
                position: todo!(),
                movement: todo!(),
                aim_direction: todo!(),
                shooting_primary: todo!(),
                shooting_secondary: todo!(),
                packet_interval: todo!(),
                dashing: todo!(),
                timestamp: todo!(),
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
              if let Some(ref mut game_socket) = data.game_socket {
                game_socket.send_to(&serialized, server_ip.clone()).expect("oops");
              }
              data.nonce += 1;
            }
          }
        }
        // MARK: draw chat
        

        // talk to main server
        // MARK: Server Comm
        println!("hi4");

        let mut buffer: [u8; 2048] = [0; 2048];
        let mut len = 0;
        if let Some(ref mut server_stream) = data.server_stream {
          match server_stream.read(&mut buffer) {
            Ok(0) => {
              println!("hi5");
              data.notifications.push(
                Notification::new("Server has disconnected.", 2.0)
              );
              data.current_menu = MenuScreen::Login(0);
              return;
            }
            Ok(length) => {
              println!("hi6");
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
        println!("hi3");

        let packets = network::tcp_decode_decrypt::<ServerToClientPacket>(buffer[..len].to_vec(), data.cipher_key.clone(), &mut data.last_nonce);
        let packets = match packets {
          Ok(packets) => packets,
          Err(_) => {
            return;
          }
        };
        println!("hi2");

        for packet in packets {
          
          match packet.information {
            // MARK: | Match assign
            ServerToClient::MatchAssignment(info) => {
              data.game_server_port = info.port;
              data.game_id = info.game_id;
              data.queued = false;
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
                  println!("hi");
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
            _ => {}
          }
        }
        println!("hi1");

        let packet_queue = data.packet_queue.clone();
        println!("q: {:?}", packet_queue);
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
        if get_keys_pressed(&k).contains(&KeyCode::Escape){
          data.paused = !data.paused;
          if data.paused == false {
            data.settings_open = false;
          }
        }
        if data.paused {
          let (paused, quit) = draw_pause_menu(uiscale, vh, vw, &mut data, 50, &font, &mut win, &mut com, &m, k);
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

        // input
        if is_window_focused(&win) {

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
        draw_text(&font, "Username", tl_anchor + Vector2 {x: 35.0 * uiscale, y: 32.0 * uiscale}, input_size, BLACK, 3.0 * uiscale, 0, &win, &mut com);
        tooltip(user_input_pos, input_size, "3-20 characters.", Vector2 { x: 30.0*uiscale, y: 5.0*uiscale }, uiscale, vw, &font, mouse_pos, 1, &win, &mut com);
        data.username_input.text_input(user_input_pos, input_size, 4.0 * uiscale, vh, &font, 0, &mut com, &win, &m, &k, &mut ki);
        tooltip(password_input_pos, input_size, "8 characters minimum.", Vector2 { x: 30.0*uiscale, y: 10.0*uiscale }, uiscale, vw, &font, mouse_pos, 1, &win, &mut com);
        draw_text(&font, "Password", tl_anchor + Vector2 {x: 35.0 * uiscale, y: 42.0 * uiscale}, input_size, BLACK, 3.0 * uiscale, 0, &win, &mut com);
        data.password_input.text_input(password_input_pos, input_size, 4.0 * uiscale, vh, &font, 0, &mut com, &win, &m, &k, &mut ki);

        // confirm button
        let mut confirm_button = Button::new(bl_anchor + Vector2 { x: 35.0*uiscale, y: -20.0*uiscale}, Vector2 { x: 20.0*uiscale, y: 5.0*uiscale }, if logging_in {"Login"} else {"Register"}, 4.0*uiscale);
        confirm_button.draw(uiscale, true, 0, &font, &win, &mut com);

        // remember me checkbox
        let credentials_checkbox_pos = tl_anchor + Vector2 {x: 35.0 * uiscale, y: 55.0 * uiscale};
        let credentials_checkbox_size = 5.0 * uiscale;
        let credentials_changed = checkbox(credentials_checkbox_pos, credentials_checkbox_size, "Remember me", 4.0*uiscale, vh, &mut data.settings.store_credentials, 0, &font, &win, &mut com, &m);
        if credentials_changed {
          data.settings.save();
        }
        tooltip(credentials_checkbox_pos, Vector2 { x: credentials_checkbox_size, y: credentials_checkbox_size }, "Stores the password in your OS keyring, like most browsers do.", Vector2 { x: 40.0*uiscale, y: 13.0*uiscale }, uiscale, vw, &font, mouse_pos, 1, &win, &mut com);

        // offline mode
        let mut offline_mode_button = Button::new(br_anchor - Vector2 {x: 28.0 * uiscale,y: 7.0 * uiscale }, Vector2 { x: 26.0*uiscale, y: 5.0*uiscale }, "Play offline", 4.0*uiscale);
        offline_mode_button.draw(uiscale, true, 0, &font, &win, &mut com);
        if offline_mode_button.was_released(&win, &m) {
          data.current_menu = MenuScreen::Main(0);
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

              if !valid_password(password.clone()) {
                data.notifications.push(
                  Notification::new("Unsafe password", 1.0)
                );
                data.username_input.selected = false;
                data.password_input.selected = true;
              }
              if !valid_username(username.clone()) {
                data.notifications.push(
                  Notification::new("Invalid username", 1.0)
                );
                data.username_input.selected = true;
                data.password_input.selected = false;
              }

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
              draw_text(&font, "Attempting connection...", tl_anchor + Vector2 {x: 35.0 * uiscale, y: 55.0 * uiscale}, Vector2 { x: 40.0*uiscale, y: 5.0 * uiscale }, BLACK, 5.0 * uiscale, 0, &win, &mut com);
              
              if data.server_stream.is_none() {
                match TcpStream::connect(&server_ip) {
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
            println!("hi");
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
    // debug
    draw_text(&font, &format!("fps: {:?}", (1.0 / time.delta().as_secs_f32()) as u16), Vector2 { x: 0.0, y: 0.0 }, Vector2 { x: 100.0, y: 100.0 }, BLACK, 10.0, 127, &win, &mut com);
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
  commands.spawn(Camera2d);
}

#[derive(Debug, Clone, Copy)]
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