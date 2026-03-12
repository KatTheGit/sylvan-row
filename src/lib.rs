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

use std::{io::{ErrorKind, Read, Write}, net::TcpStream, time::Instant};
use bevy::{color::palettes::css::*, input::{keyboard::KeyboardInput, mouse::MouseWheel}, prelude::*, window::WindowResolution};
use bevy_immediate::*;
use bevy_graphics::*;
use maths::*;
use opaque_ke::{ClientLogin, ClientLoginFinishParameters, ClientRegistration, ClientRegistrationFinishParameters, ClientRegistrationStartResult};
use rand::rngs::OsRng;
use crate::{bevy_graphics::Button, const_params::DefaultCipherSuite, filter::{valid_password, valid_username}, mothership_common::{ClientToServer, ClientToServerPacket, RefusalReason, ServerToClient, ServerToClientPacket}};


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
struct GameData {
  pub current_menu: MenuScreen,
  pub tabs_login: Tabs,
  pub settings: Settings,
  pub chat_input: TextInput,
  pub username_input: TextInput,
  pub password_input: TextInput,
  pub server_stream: Option<TcpStream>,
  pub notifications: Vec<Notification>,
  pub opake_data: OpakeData,
}
impl Default for GameData {
  fn default() -> Self {
    return GameData {
      current_menu: MenuScreen::Login(0),
      tabs_login: Tabs::new(vec!["Login".to_string(), "Register".to_string()]),
      settings: Settings::load(),
      chat_input: TextInput {
        selected: false,
        buffer: String::new(),
        hideable: false,
        show_password: false,
      },
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
        client_registration_start_result: None
      }
    }
  }
}

struct OpakeData {
  pub timeout: Instant,
  pub client_registration_start_result: Option<ClientRegistrationStartResult<DefaultCipherSuite>>,
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
) {
  if let Some(mut data) = data {

    let server_ip = "127.0.0.1:25569";


    // MAIN LOOP
    let mut win = window.single_mut().expect("oops");
    let vw = win.width() / 100.0;
    let vh = win.height() / 100.0;

    // calculate UI scale
    let size_min = f32::min(vw, vh);
    let uiscale = if size_min < 5.0 {2.5} else if size_min < 10.0 {5.0} else {10.0};
    let tl_anchor = Vector2 {x: 0.0, y: 0.0};
    let tr_anchor = Vector2 {x: 100.0*vw, y: 0.0};
    let bl_anchor = Vector2 {x: 0.0, y: 100.0*vh};
    let br_anchor = Vector2 {x: 100.0*vw, y: 100.0*vh};

    let font: Handle<Font> = asset_server.load("fonts/Roboto-Black.ttf");

    match data.current_menu {
      MenuScreen::Login(login_step) => {
        // draw login screen

        // login / register tabs
        data.tabs_login.update_size(tl_anchor + Vector2 { x: 35.0 * uiscale, y: 20.0 * uiscale}, Vector2 { x: 40.0*uiscale, y: 6.0*uiscale }, 4.0*uiscale);
        data.tabs_login.draw_and_process(uiscale, true, 0, &font, &win, &mut com, &m);
        let logging_in = match data.tabs_login.selected_tab() {
          0 => {true}
          _ => {false}
        };

        // input fields
        draw_text(&font, "Username", tl_anchor + Vector2 {x: 35.0 * uiscale, y: 32.0 * uiscale}, Vector2 { x: 40.0*uiscale, y: 5.0 * uiscale }, 3.0 * uiscale, 0, &win, &mut com);
        data.username_input.text_input(tl_anchor + Vector2 {x: 35.0 * uiscale, y: 35.0 * uiscale}, Vector2 { x: 40.0*uiscale, y: 6.0 * uiscale }, 4.0 * uiscale, vh, &font, 0, &mut com, &win, &m, &k, &mut ki);
        draw_text(&font, "Password", tl_anchor + Vector2 {x: 35.0 * uiscale, y: 42.0 * uiscale}, Vector2 { x: 40.0*uiscale, y: 5.0 * uiscale }, 3.0 * uiscale, 0, &win, &mut com);
        data.password_input.text_input(tl_anchor + Vector2 {x: 35.0 * uiscale, y: 45.0 * uiscale}, Vector2 { x: 40.0*uiscale, y: 6.0 * uiscale }, 4.0 * uiscale, vh, &font, 0, &mut com, &win, &m, &k, &mut ki);

        // confirm button
        let mut confirm_button = Button::new(bl_anchor + Vector2 { x: 35.0*uiscale, y: -20.0*uiscale}, Vector2 { x: 20.0*uiscale, y: 5.0*uiscale }, if logging_in {"Login"} else {"Register"}, 4.0*uiscale);
        confirm_button.draw(uiscale, true, 0, &font, &win, &mut com);

        // remember me checkbox
        let credentials_changed = checkbox(tl_anchor + Vector2 {x: 35.0 * uiscale, y: 55.0 * uiscale}, 5.0 * uiscale, "Remember me", 4.0*uiscale, vh, &mut data.settings.store_credentials, 0, &font, &win, &mut com, &m);
        if credentials_changed {
          data.settings.save();
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
              draw_text(&font, "Attempting connection...", tl_anchor + Vector2 {x: 35.0 * uiscale, y: 55.0 * uiscale}, Vector2 { x: 40.0*uiscale, y: 5.0 * uiscale }, 5.0 * uiscale, 0, &win, &mut com);
              
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
                      Notification::new(&format!("Connection to server failed. Reason: {:?}", err), 2.0)
                    )
                  }
                }
              }
            }
          }
          // MARK: login
          1 => {

          }
          2 => {

          }


          // MARK: register
          // OPAKE register step 1.
          3 => {
            //if let Some(ref mut client_registration_start_result) = data.opake_data.client_registration_start_result {
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
                      Notification::new(&format!("Connection failed. Reason: {:?}", err), 2.0)
                    )
                  }
                }
              }
            //}
          }
          // OPAKE register step 3 (step 2 client POV, step 3 overall.)
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
                        Notification::new(&format!("Unkown error during login. Reason: {:?}", err), 2.0)
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
      // MARK: Main
      MenuScreen::Main(mode) => {
        if mode != 2 {
          // menu
        }
        if mode == 1 || mode == 2 {
          // game
        }
        // draw chat
        

        // talk to main server

        // settings screen

        // input
        if is_window_focused(&win) {

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
    draw_text(&font, &format!("fps: {:?}", (1.0 / time.delta().as_secs_f32()) as u16), Vector2 { x: 0.0, y: 0.0 }, Vector2 { x: 100.0, y: 100.0 }, 10.0, 127, &win, &mut com);
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
enum MenuScreen {
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