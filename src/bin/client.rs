// Don't show console window on Windows
#![windows_subsystem = "windows"]
#![allow(unused_parens)]

use std::{io::{ErrorKind, Read, Write}, net::{TcpStream}, process::exit, time::{Duration, Instant}};
use kira::{track::TrackBuilder, AudioManager, AudioManagerSettings, DefaultBackend};
use sylvan_row::{game, audio, const_params::*, database::{self, get_friend_request_type, FriendShipStatus}, filter::{self, valid_password, valid_username}, gamedata::*, gameserver::game_server, graphics::{self, draw_image}, maths::*, mothership_common::*, network, ui::{self, load_password, save_password, Notification, Settings}};
use miniquad::{conf::Icon, window::set_mouse_cursor};
use device_query::{DeviceQuery, DeviceState, Keycode};
use macroquad::prelude::*;
use ring::hkdf;
use ::rand::rngs::OsRng;
use opaque_ke::{
  ClientLogin, ClientLoginFinishParameters, ClientRegistration, ClientRegistrationFinishParameters
};
use sylvan_row::game::MainServerInteraction;

fn window_conf() -> Conf {
  Conf {
    window_title: "Sylvan Row".to_owned(),
    fullscreen: false,
    icon: Some(Icon {
      small:  Image::from_file_with_format(include_bytes!("../../assets/icon/icon-small.png"), None).expect("File not found").bytes.as_slice().try_into().expect("womp womp"),
      medium: Image::from_file_with_format(include_bytes!(concat!("../../assets/icon/icon-medium.png")), None).expect("File not found").bytes.as_slice().try_into().expect("womp womp"),
      big:    Image::from_file_with_format(include_bytes!(concat!("../../assets/icon/icon-big.png")), None).expect("File not found").bytes.as_slice().try_into().expect("womp womp"),
    }),
    window_resizable: true,
    window_height: 504,
    window_width: 896,
    ..Default::default()
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  let port = get_random_port();
  let mut vw: f32 = 1.0;
  let mut vh: f32 = 1.0;
  let mut selected_char = 0;
  let characters: Vec<Character> = vec![
    Character::Hernani,
    Character::Raphaelle,
    Character::Cynewynn,
    Character::Wiro,
    Character::Elizabeth,
    Character::Temerity,
  ];
  let descriptions: Vec<&str> = vec![
    include_str!("../../assets/characters/hernani/description.txt"),
    include_str!("../../assets/characters/raphaelle/description.txt"),
    include_str!("../../assets/characters/cynewynn/description.txt"),
    include_str!("../../assets/characters/wiro/description.txt"),
    include_str!("../../assets/characters/elizabeth/description.txt"),
    include_str!("../../assets/characters/temerity/description.txt"),
  ];
  let temporary_profiles: Vec<Texture2D> = vec![
    Texture2D::from_file_with_format(include_bytes!("../../assets/characters/hernani/textures/mini-profile.png"), None),
    Texture2D::from_file_with_format(include_bytes!("../../assets/characters/raphaelle/textures/mini-profile.png"), None),
    Texture2D::from_file_with_format(include_bytes!("../../assets/characters/cynewynn/textures/mini-profile.png"), None),
    Texture2D::from_file_with_format(include_bytes!("../../assets/characters/wiro/textures/mini-profile.png"), None),
    Texture2D::from_file_with_format(include_bytes!("../../assets/characters/elizabeth/textures/mini-profile.png"), None),
    Texture2D::from_file_with_format(include_bytes!("../../assets/characters/temerity/textures/mini-profile.png"), None),
  ];


  let mut settings = Settings::load();

  let mut fullscreen: bool = settings.fullscreen;
  set_fullscreen(fullscreen);

  // whether we're queueing
  let mut queue: bool = false;

  //let mut offline_mode: bool = false;

  // MARK: main menu
  let mut tab_stats_refresh_flag: bool = false;
  let mut tab_friends_refresh_flag: bool = false;
  let mut menu_paused = false;
  let mut escape_already_pressed: bool = false;

  let mut settings_open_flag: bool = false;
  let mut startup_happened: bool = false;

  let mut checkbox_1v1 = true;
  let mut checkbox_2v2 = true;

  let server_ip = network::get_ip();

  let mut friend_request_input: String = String::from("");
  let mut friend_request_input_selected: bool = false;

  let mut chat_timer: Instant = Instant::now().checked_sub(Duration::from_secs_f32(10.0)).expect("oops");

  // login fields
  let mut username: String = if settings.store_credentials {settings.saved_username.clone()} else {String::new()};
  let mut username_selected: bool = true;
  let mut password: String = String::from("");
  let mut password_selected: bool = false;
  let mut show_password: bool = false;

  // load password from keyring.
  if settings.store_credentials {
    password = load_password(&username);
  }

  let mut registering: bool = false;

  let mut notifications: Vec<ui::Notification> = Vec::new();

  let mut logged_in: bool = false;
  let mut cipher_key: Vec<u8> = Vec::new();
  // counter used for cipher nonce.
  let mut nonce: u32 = 1;
  let mut last_nonce: u32 = 0;


  let mut player_stats: PlayerStatistics = PlayerStatistics::new();
  let mut packet_queue: Vec<ClientToServerPacket> = Vec::new();

  let mut main_tabs = ui::Tabs::new(vec!["Play".to_string(), "Heroes".to_string(), "Tutorial".to_string(), "Stats".to_string(), "Friends".to_string()], 5.0*vh);
  let mut login_tabs = ui::Tabs::new(vec!["Login".to_string(), "Register".to_string()], 5.0*vh);
  let mut heroes_tabs = ui::Tabs::new(characters.iter().map(|x| x.name()).collect(), 5.0*vh);
  let mut settings_tabs = ui::Tabs::new(vec!["Gameplay".to_string(), "Video".to_string(), "Audio".to_string(), "Controls".to_string()], 5.0*vh);

  // for anything shared between game and main menu.
  let mut server_interaction = MainServerInteraction {
    server_stream: None,
    is_chatbox_open: false,
    selected_friend: 0,
    recv_messages_buffer: Vec::new(),
    chat_input_buffer: String::new(),
    chat_selected: false,
    chat_scroll_index: 0,
    friend_list: Vec::new(),
    lobby_invites: Vec::new(),
    lobby: Vec::new(),
  };

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
  
  loop {
    main_tabs.update_size(Vector2 { x: 5.0 * vw, y: 5.0 * vh}, Vector2 { x: 90.0*vw, y: 6.0*vh }, 5.0*vh);
    login_tabs.update_size(Vector2 { x: 35.0 * vh, y: 20.0 * vh}, Vector2 { x: 40.0*vw, y: 6.0*vh }, 5.0*vh);
    heroes_tabs.update_size(Vector2 { x: 5.0 * vw, y: 75.0 * vh}, Vector2 { x: 90.0*vw, y: 15.0*vh }, 5.0*vh);
    let mouse_pos = Vector2 { x: mouse_position().0, y: mouse_position().1 };

    if get_keys_pressed().contains(&KeyCode::F11) {
      fullscreen = !fullscreen;
      set_fullscreen(fullscreen);
    }

    set_mouse_cursor(miniquad::CursorIcon::Default);
    vw = screen_width() / 100.0;
    vh = screen_height() / 100.0;
    clear_background(WHITE);
    let br_anchor: Vector2 = Vector2 { x: screen_width(), y: screen_height() };
    //let tr_anchor: Vector2 = Vector2 { x: screen_width(), y: 0.0 };
    //let bl_anchor: Vector2 = Vector2 { x: 0.0,            y: screen_height() };
    //let tl_anchor: Vector2 = Vector2 { x: 0.0,            y: 0.0 };


    // show login window
    // MARK: Login
    if !logged_in {

      draw_text("Username", 35.0 * vh, 32.0*vh, 4.0*vh, BLACK);
      draw_text("Password", 35.0 * vh, 47.0*vh, 4.0*vh, BLACK);

      login_tabs.draw_and_process(vh, true);
      // register
      if login_tabs.selected_tab() == 1 {
        registering = true;
      }
      // login
      if login_tabs.selected_tab() == 0 {
        registering = false;
      }

      // username
      let username_input_position = Vector2 { x: 35.0*vh, y: 35.0*vh };
      let username_input_size = Vector2 { x: 30.0*vw, y: 7.0*vh };

      ui::text_input(
        username_input_position,
        username_input_size, &mut username, &mut username_selected, 4.0*vh, vh,
        false, &mut false,
      );
      username.truncate(20);
      // password
      let password_input_position = Vector2 { x: 35.0*vh, y: 50.0*vh };
      let password_input_size = Vector2 { x: 30.0*vw, y: 7.0*vh };
      ui::text_input(
        password_input_position,
        password_input_size, &mut password, &mut password_selected, 4.0*vh, vh,
        true, &mut show_password
      );

      let save_pass_check_position = Vector2 { x: 35.0*vh, y: 59.0*vh };
      let save_pass_check_size = 5.0*vh;
      let credentials_checkbox_changed = ui::checkbox(save_pass_check_position, save_pass_check_size, "Remember me", 4.0*vh, vh, &mut settings.store_credentials);
      // credentials tooltip
      ui::tooltip(save_pass_check_position, Vector2 { x: save_pass_check_size, y: save_pass_check_size }, "Stores the credentials in your OS keyring,\nlike most browers.", vh, vw, mouse_pos);
      // save;
      if credentials_checkbox_changed {
        settings.save();
      }
      // confirm button for either action
      let mut confirm_button = ui::Button::new(Vector2 { x: 35.0*vh, y: 70.0*vh }, Vector2 { x: 20.0*vh, y: 5.0*vh }, if registering {"register"} else {"log in"}, 5.0*vh);
      confirm_button.draw(vh, !menu_paused);
      let mut confirm = confirm_button.was_pressed();

      // tooltips
      ui::tooltip(username_input_position, username_input_size, "3-20 characters", vh, vw, mouse_pos);
      ui::tooltip(password_input_position, password_input_size, "8 characters minimum", vh, vw, mouse_pos);


      if get_keys_pressed().contains(&KeyCode::Enter) {
        if username_selected {
          username_selected = false;
          password_selected = true;
        }
        else if password_selected {
          confirm = true;
        }
        else {
          confirm = true;
        }
      }
      
      // if button pressed
      if confirm {
        draw_text("Attempting connection...", 35.0*vh, 80.0*vh, 5.0*vh, BLACK);
        next_frame().await;

        if server_interaction.server_stream.is_none() {
  
          match TcpStream::connect(&server_ip) {
            Ok(stream) => {
              server_interaction.server_stream = Some(stream);
              if let Some(ref server_stream) = server_interaction.server_stream {
                server_stream.set_nonblocking(true).expect("idk");
              }
            },
            Err(_err) => {
              //println!("{:?}", err);
              notifications.push(
                ui::Notification { start_time: Instant::now(), text: String::from("No connection."), duration: 3.0 }
              );
            },
          };
        }

        if let Some(ref mut server_stream) = server_interaction.server_stream {
          // save credentials
          if settings.store_credentials {
            settings.saved_username = username.clone();
            save_password(&password, &username, &mut notifications);
            settings.save();
            //notifications.push(
            //  Notification::new("Credentials saved.", 0.5)
            //);
          }

          let timeout_timer: Instant = Instant::now();

          // all data
          let mut client_rng = OsRng;

          // used inside the listener loops to perform
          // a double break
          let mut error_occurred: bool = false;

          // register
          if registering {
            // password check
            // this is only a clientside check because:
            // 1. the server doesn't know the plaintext password
            // 2. if you bypass this just to have a shit password, you do you...
            if !valid_password(password.clone()) {
              notifications.push(
                Notification::new("Unsafe password", 1.0)
              );
              username_selected = false;
              password_selected = true;
              next_frame().await;
              continue;
            }
            if !valid_username(username.clone()) {
              notifications.push(
                Notification::new("Invalid username", 1.0)
              );
              username_selected = true;
              password_selected = false;
              next_frame().await;
              continue;
            }
            // step 1
            let client_registration_start_result = ClientRegistration::<DefaultCipherSuite>::start(&mut client_rng, password.as_bytes()).expect("oops");
            let message = client_registration_start_result.clone().message;
            let message = ClientToServerPacket {
              information: ClientToServer::RegisterRequestStep1(username.clone(), message)
            };
            match server_stream.write_all(&network::tcp_encode(message).expect("oops")) {
              Ok(_) => {}
              Err(_) => {
                //registration failed.
                next_frame().await;
                continue;
              }
            }
            loop {
              clear_background(WHITE);
              draw_text("Processing...", 35.0*vh, 80.0*vh, 5.0*vh, BLACK);

              // recieve packet
              let mut buffer: [u8; 2048] = [0; 2048];
              let len: usize = match server_stream.read(&mut buffer) {
                Ok(0) => {
                  server_interaction.server_stream = None;
                  break;
                }
                Ok(n) => {n}
                Err(err) => {
                  match err.kind() {
                    ErrorKind::WouldBlock => {
                      if timeout_timer.elapsed().as_secs_f32() > 10.0 {
                        notifications.push(Notification::new("Timed out", 1.0));
                        break;
                      }
                      next_frame().await;
                      continue;
                    }
                    _ => {
                      break;
                    }
                  }
                }
              };
              if len > 2048 {
                next_frame().await;
                continue;
              }
              let packets = network::tcp_decode::<ServerToClientPacket>(buffer[..len].to_vec());
              let packets = match packets {
                Ok(packets) => packets,
                Err(_err) => {
                  next_frame().await;
                  continue;
                }
              };
              for packet in packets {
                match packet.information {

                  // register finish (step 3)
                  ServerToClient::RegisterResponse1(server_message) => {
                    let client_registration_finish_result = client_registration_start_result.clone().state.finish(
                      &mut client_rng,
                      password.as_bytes(),
                      server_message,
                      ClientRegistrationFinishParameters::default(),
                    ).expect("idgaf");
                    let registration_upload = client_registration_finish_result.message;
                    let message = ClientToServerPacket {
                      information: ClientToServer::RegisterRequestStep2(registration_upload)
                    };
                    match server_stream.write_all(&network::tcp_encode(&message).expect("oops")) {
                      Ok(_) => {}
                      Err(_) => {
                        //registration failed.
                        println!("hello");
                        continue;
                      }
                    }
                    // register successful
                    notifications.push(
                      Notification::new("Account created!", 1.5)
                    );
                    registering = false;
                    login_tabs.set_selected(0);  
                  }

                  // error
                  ServerToClient::InteractionRefused(reason) => {
                    notifications.push(
                      Notification { start_time: Instant::now(), text: match reason {
                        RefusalReason::InternalError => String::from("Internal Server Error"),
                        RefusalReason::InvalidUsername => String::from("Invalid Username"),
                        RefusalReason::UsernameTaken => String::from("Username Taken"),
                        RefusalReason::UsernameInexistent => String::from("Incorrect Username"),
                        _ => String::from("Unexpected Error"),
                      }, duration: 1.0 }
                    );
                    error_occurred = true;
                    username_selected = true;
                    password_selected = false;
                    break;
                  }
                  // any other packet
                  _ => {}
                }
              }
              if error_occurred {
                break;
              }
              if registering == false {
                break;
              }
              next_frame().await;
            }
          }
          // login
          else {
            let client_login_start_result = ClientLogin::<DefaultCipherSuite>::start(&mut client_rng, password.as_bytes()).expect("Oops");

            let message = client_login_start_result.message.clone();
            let message = ClientToServerPacket {
              information: ClientToServer::LoginRequestStep1(username.clone(), message)
            };
            match server_stream.write_all(&network::tcp_encode(&message).expect("oops")) {
              Ok(_) => {}
              Err(_) => {
                //registration failed.
                println!("hello");
                continue;
              }
            }

            loop {
              clear_background(WHITE);
              draw_text("Processing...", 35.0*vh, 80.0*vh, 5.0*vh, BLACK);

              // recieve packet
              let mut buffer: [u8; 2048] = [0; 2048];
              let len: usize = match server_stream.read(&mut buffer) {
                Ok(0) => {
                  server_interaction.server_stream = None;
                  break;
                }
                Ok(n) => {n}
                Err(err) => {
                  match err.kind() {
                    ErrorKind::WouldBlock => {
                      if timeout_timer.elapsed().as_secs_f32() > 10.0 {
                        notifications.push(Notification::new("Timed out", 1.0));
                        break;
                      }
                      next_frame().await;
                      continue;
                    }
                    _ => {
                      break;
                    }
                  }
                }
              };
              if len > 2048 {
                next_frame().await;
                continue;
              }
              let packets = network::tcp_decode::<ServerToClientPacket>(buffer[..len].to_vec());
              let packets = match packets {
                Ok(packets) => packets,
                Err(_err) => {
                  next_frame().await;
                  continue;
                }
              };
              for packet in packets {
                match packet.information {

                  // login finish (step 3)
                  ServerToClient::LoginResponse1(server_response) => {
                    println!("reached step 3");
                    let client_login_finish_result = match client_login_start_result.clone().state.finish(
                      &mut client_rng,
                      password.as_bytes(),
                      server_response,
                      ClientLoginFinishParameters::default(),
                    ) {
                      Ok(result) => {result},
                      Err(_err) => {
                        notifications.push(
                          Notification { start_time: Instant::now(), text: String::from("Wrong password"), duration: 1.5 }
                        );
                        username_selected = false;
                        password_selected = true;
                        println!("yo, wrong password");
                        next_frame().await;
                        error_occurred = true;
                        break;
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
                    cipher_key = key;
                    
                    let credential_finalization = client_login_finish_result.message;
                    let message = ClientToServerPacket {
                      information: ClientToServer::LoginRequestStep2(credential_finalization)
                    };
                    match server_stream.write_all(&network::tcp_encode(&message).expect("oops")) {
                      Ok(_) => {}
                      Err(_) => {
                        error_occurred = true;
                        break;
                      }
                    }
                    logged_in = true;
                    // break out of this loop, to complete login by going back
                    // to the main loop
                    break;
                  }

                  // error
                  ServerToClient::InteractionRefused(reason) => {
                    notifications.push(
                      Notification { start_time: Instant::now(), text: match reason {
                        RefusalReason::InternalError => String::from("Internal Server Error"),
                        RefusalReason::InvalidUsername => String::from("Invalid Username"),
                        RefusalReason::UsernameTaken => String::from("Username Taken"),
                        RefusalReason::UsernameInexistent => String::from("Incorrect Username"),
                        _ => String::from("Unexpected Error"),
                      }, duration: 1.5 }
                    );
                    username_selected = true;
                    password_selected = false;
                    error_occurred = true;
                    break;
                  }
                  // any other packet
                  _ => {}
                }
              }
              if error_occurred {
                break;
              }
              if logged_in {
                break;
              }
              next_frame().await;
            }
          }
        }
      }
      // draw notifications

      let mut offline_button = ui::Button::new(Vector2 { x: 80.0*vw, y: 90.0*vh }, Vector2 { x: 19.0*vw, y: 8.0*vh }, "Offline Mode", 5.0*vh);
      offline_button.draw(vh, !menu_paused);
      if offline_button.was_pressed() {
        //offline_mode = true;
        username = "Player".to_string();
        cipher_key = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
        logged_in = true;
      }

      for n_index in 0..notifications.len() {
        notifications[n_index].draw(vh, vw, 5.0*vh, n_index);
      }
      for n_index in 0..notifications.len() {
        if notifications[n_index].start_time.elapsed().as_secs_f32() > notifications[n_index].duration {
          notifications.remove(n_index);
          break;
        }
      }
      // for some reason on windows when there's no connection to the server the
      // client spins a lot and lags out the whole user's desktop which isn't
      // very "i promise my game is not malware"-like.
      // this is an ugly fix.
      //if get_frame_time() < (1.0 / 30.0) {
      //  std::thread::sleep(Duration::from_secs_f32(1.0/30.0 - get_frame_time()));
      //}
      next_frame().await;
      // end early
      continue;
    }
    // MARK: Main (logged in)
    // Startup
    if !startup_happened {
      startup_happened = true;

      // request friend list.
      packet_queue.push(
        ClientToServerPacket { information: ClientToServer::GetFriendList }
      );
    }

    main_tabs.draw_and_process(vh, !menu_paused);

    let mut start_game = false;
    let mut game_port = 0;
    let mut game_id = 0;

    // MARK: Listen-Write
    if let Some(ref mut server_stream) = server_interaction.server_stream {
      let mut buffer: [u8; 2048] = [0; 2048];
      match server_stream.read(&mut buffer) {
        Ok(0) => {
          logged_in = false;
        }
        Ok(len) => {
          let packets = network::tcp_decode_decrypt::<ServerToClientPacket>(buffer[..len].to_vec(), cipher_key.clone(), &mut last_nonce);
          let packets = match packets {
            Ok(packets) => packets,
            Err(_) => {
              continue;
            }
          };

          for packet in packets {
            
            match packet.information {
              ServerToClient::MatchAssignment(info) => {
                if info.port != 0 {
                  start_game = true;
                  game_port = info.port;
                  game_id = info.game_id;
                }
                queue = false;
              },
              ServerToClient::PlayerDataResponse(recv_player_stats) => {
                player_stats = recv_player_stats;
              }
              ServerToClient::FriendListResponse(recv_friend_list) => {
                server_interaction.friend_list = recv_friend_list;
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
                notifications.push(Notification::new(text, 1.0));
              }
              ServerToClient::FriendRequestSuccessful => {
                notifications.push(Notification::new("Friend request sent", 1.0));
                // update friend list
                server_interaction.friend_list.push(
                  (friend_request_input.clone(), get_friend_request_type(&username, &friend_request_input), false)
                );
                friend_request_input = String::new();
              }
              ServerToClient::FriendshipSuccessful => {
                notifications.push(Notification::new("You are now friends!", 1.0));
                // update friend list
                for f_index in 0..server_interaction.friend_list.len() {
                  if database::get_friend_name(&username, &server_interaction.friend_list[f_index].0) == friend_request_input {
                    server_interaction.friend_list[f_index].1 = FriendShipStatus::Friends;
                  }
                }
                friend_request_input = String::new();
              }
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
              // lobby
              ServerToClient::LobbyInvite(inviting_user) => {
                server_interaction.lobby_invites.retain(|element| element != &inviting_user);
                server_interaction.lobby_invites.push(inviting_user.clone());
                notifications.push(
                  Notification::new(&format!("{} invited you", inviting_user), 4.0)
                );
              }
              ServerToClient::LobbyUpdate(data) => {
                //println!("{:?}", data);
                server_interaction.lobby = data;
                // if we're in this list, delete us
                server_interaction.lobby.retain(|element| element.username != username);
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
          &network::tcp_encode_encrypt(packet, cipher_key.clone(), nonce).expect("oops")
        ).expect("idk 1");
        nonce += 1;
      }
      packet_queue = Vec::new();
    }

    if start_game {
      game::game(server_ip.clone(), characters[selected_char], port, game_port, cipher_key.clone(), username.clone(), &mut settings, &mut server_interaction, &mut nonce, &mut last_nonce, &mut fullscreen, game_id, &mut settings_tabs).await;
    }

    // PLAY TAB
    if main_tabs.selected_tab() == 0 {
      if !queue {
        ui::checkbox(br_anchor - Vector2 {x: 30.0*vh, y: 21.0*vh }, 4.0*vh, "1v1", 4.0*vh, vh, &mut checkbox_1v1);
        ui::checkbox(br_anchor - Vector2 {x: 17.5*vh, y: 21.0*vh }, 4.0*vh, "2v2", 4.0*vh, vh, &mut checkbox_2v2);
      } if queue {
        ui::checkbox(br_anchor - Vector2 {x: 30.0*vh, y: 21.0*vh }, 4.0*vh, "1v1", 4.0*vh, vh, &mut checkbox_1v1.clone()); // clone to disable writes
        ui::checkbox(br_anchor - Vector2 {x: 17.5*vh, y: 21.0*vh }, 4.0*vh, "2v2", 4.0*vh, vh, &mut checkbox_2v2.clone());
      }


      let mut play_button = ui::Button::new(br_anchor - Vector2 { x: 30.0*vh, y: 15.0*vh }, Vector2 { x: 25.0*vh, y: 13.0*vh }, "Play", 8.0*vh);
      play_button.draw(vh, !menu_paused);
      if queue {
        draw_text("In queue...", br_anchor.x - 30.0*vh, br_anchor.y - 24.0*vh, 5.0*vh, BLACK);
      }
      if play_button.was_pressed() {
        queue = !queue;
        if queue {
          let mut selected_gamemodes: Vec<GameMode> = Vec::new();
          if checkbox_1v1 {selected_gamemodes.push(GameMode::Standard1V1)}
          if checkbox_2v2 {selected_gamemodes.push(GameMode::Standard2V2)}
          if selected_gamemodes.is_empty() {
            notifications.push(Notification::new("Pick a gamemode!", 1.0));
            queue = false;
            continue;
          }
          // Send a match request packet
          packet_queue.push(
            ClientToServerPacket {
              information: ClientToServer::MatchRequest(MatchRequestData {
                gamemodes: selected_gamemodes,
                character: characters[selected_char],
              }),
            }
          );
        } else {
          // Send a match cancel packet
          packet_queue.push(
            ClientToServerPacket {
              information: ClientToServer::MatchRequestCancel,
            },
          )
        }
      }
      // draw lobby

      let mut lobby = server_interaction.lobby.clone();
      // insert self
      lobby.insert(
        0, 
        LobbyPlayerInfo {
          username: username.clone(),
          is_ready: queue,
        }
      );

      let lobby_position: Vector2 = Vector2 { x: 5.0*vw, y: 19.0*vh };
      let lobby_size: Vector2 = Vector2 { x: 30.0*vw, y: 7.0*vh };
      let y_offset = lobby_size.y;
      let inner_shrink: f32 = 1.0 * vh;
      draw_text("Lobby", lobby_position.x, lobby_position.y-1.0*vh, 5.0*vh, BLACK);
      for (i, player) in lobby.iter().enumerate() {
        graphics::draw_rectangle(lobby_position + Vector2 {x: 0.0, y: (i as f32)*y_offset}, lobby_size, BLUE);
        graphics::draw_rectangle(lobby_position + Vector2{x: inner_shrink, y:inner_shrink} + Vector2 {x: 0.0, y: (i as f32)*y_offset}, lobby_size - Vector2{x: inner_shrink*2.0, y:inner_shrink*2.0}, SKYBLUE);
        let is_ready_color = if player.is_ready {LIME} else {RED};
        let is_ready_text = if player.is_ready {"Ready"} else {"Not Ready"};
        draw_text(&format!("{}", player.username), lobby_position.x + 2.0*vh, lobby_position.y + (i as f32)*y_offset + lobby_size.y*0.65, 4.0*vh, BLACK);
        draw_text(&format!("{}", is_ready_text), lobby_position.x + lobby_size.x * 0.67, lobby_position.y + (i as f32)*y_offset + lobby_size.y*0.65, 4.0*vh, is_ready_color);
      }
      // lobby leave button
      if lobby.len() > 1 {
        let mut leave_button = ui::Button::new(lobby_position + Vector2 {x: 0.0, y: y_offset * (lobby.len() as f32) + inner_shrink}, Vector2 { x: lobby_size.x/2.0, y: lobby_size.y - inner_shrink }, "Leave", 5.0*vh);
        leave_button.draw(vh, !menu_paused);
        if leave_button.was_pressed() {
          packet_queue.push(
            ClientToServerPacket {
              information: ClientToServer::LobbyLeave,
            },
          );
          server_interaction.lobby = Vec::new();
          notifications.push(
            Notification::new("Left the party.", 1.0)
          )
        }
      }
    }

    // HEROES TAB
    if main_tabs.selected_tab() == 1 {

      heroes_tabs.draw_and_process(vh, !menu_paused);
      let max = characters.len();

      selected_char = heroes_tabs.selected_tab();

      draw_multiline_text_ex(descriptions[selected_char],20.0*vh, 15.0*vh, Some(0.7), 
        TextParams { font: None, font_size: 16, font_scale: 0.25*vh, font_scale_aspect: 1.0, rotation: 0.0, color: BLACK }
      );
      let image_size = 45.0;
      draw_image(&temporary_profiles[selected_char], (71.0/vh)*vw, 16.0, image_size*0.9, image_size, vh, Vector2::new(), WHITE);
      draw_text("Selected", 7.5 * vw + (selected_char as f32) * (90.0/(max) as f32) * vw, 95.0 * vh, 4.0 * vh, BLACK);
      let mut practice_button = ui::Button::new(Vector2 { x: 71.0*vw, y: 62.0*vh }, Vector2 { x: 35.0*vh, y: 9.0*vh }, "Practice Range", 4.0 * vh);
      practice_button.draw(vh, !menu_paused);
      if practice_button.was_pressed() {
        let game_port = get_random_port();
        let practice_game_port = game_port.clone();
        let practice_username = username.clone();
        let session_key = cipher_key.clone();
        let practice_character = characters[selected_char].clone();
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
        game::game(String::from("127.0.0.1"), characters[selected_char], port, game_port, cipher_key.clone(), username.clone(), &mut settings, &mut server_interaction, &mut nonce, &mut last_nonce, &mut fullscreen, game_id, &mut settings_tabs).await;
      }
    }
    if main_tabs.selected_tab() == 2 {
      let text: &str =
        "(LMB)   PRIMARY   - Ability on short cooldown.\n(RMB)   SECONDARY - Ability that requires charge. Build charge by hitting opponents.\n(Space) DASH      - Cooldown ability.\n(WASD)  Move";
      draw_multiline_text_ex(text,20.0*vh, 15.0*vh, Some(0.7), 
        TextParams { font: None, font_size: 16, font_scale: 0.25*vh, font_scale_aspect: 1.0, rotation: 0.0, color: BLACK }
      );
    }
    // stats
    if main_tabs.selected_tab() == 3 {
      // runs once when we click this tab
      if tab_stats_refresh_flag == false {
        tab_stats_refresh_flag = true;
        // ask the server for our stats
        packet_queue.push(
          ClientToServerPacket {
            information: ClientToServer::PlayerDataRequest,
          }
        );
      
      }
      let username = username.clone();
      let stats = player_stats.clone();
      draw_text(&format!("{}'s beautiful stats:", username), 20.0*vh, 20.0*vh, 5.0*vh, BLACK);
      draw_text(&format!("Wins: {}", stats.wins), 20.0*vh, 27.0*vh, 5.0*vh, BLACK);
    } if !main_tabs.selected_tab() == 3 {
      tab_stats_refresh_flag = false;
    }
    // friends
    if main_tabs.selected_tab() == 4 {
      // runs once when we click this tab
      if tab_friends_refresh_flag == false {
        tab_friends_refresh_flag = true;
        // ask the server for our friendship data.
        packet_queue.push(
          ClientToServerPacket {
            information: ClientToServer::GetFriendList,
          }
        );
      }
      // Friend request form
      ui::text_input(
        Vector2 { x: 15.0*vw, y: 15.0*vh },
        Vector2 { x: 45.0*vw, y: 7.0*vh }, &mut friend_request_input, &mut friend_request_input_selected, 4.0*vh, vh,
        false, &mut false,
      );
      let mut send_request_button = ui::Button::new(Vector2 { x: 60.0*vw, y: 15.0*vh }, Vector2 { x: 25.0*vw, y: 7.0*vh }, "Send friend request", 5.0*vh);
      send_request_button.draw(vh, !menu_paused);
      if send_request_button.was_pressed() {
        let username_requested = friend_request_input.as_str();
        if username_requested.is_empty() {
          notifications.push(
            Notification::new("Enter a username.", 1.0)
          );
          next_frame().await;
          continue;
        }
        if !filter::valid_username(String::from(username_requested)) {
          notifications.push(
            Notification::new("User doesn't exist.", 1.0)
          );
          next_frame().await;
          continue;
        }
        // the server also refuses this interaction if the check is bypassed
        if username_requested == username {
          notifications.push(
            Notification::new("That's you dummy!", 1.0)
          );
          next_frame().await;
          continue;
        }
        packet_queue.push(
          ClientToServerPacket {
            information: ClientToServer::SendFriendRequest(String::from(username_requested)),
          }
        );
      }

      // FRIEND LIST
      let y_offset = 6.0 * vh;
      for f_index in 0..server_interaction.friend_list.len() {
        let friend = &server_interaction.friend_list[f_index];
        let current_offset = y_offset * f_index as f32;
        let peer_username;
        let split: Vec<&str> = friend.0.split(":").collect();
        if *split[0] == username {
          peer_username = split[1];
        } else {
          peer_username = split[0];
        }

        draw_text(peer_username, 10.0*vw, 30.0*vh + current_offset, 5.0*vh, BLACK);
        let status: &str;
        match friend.1 {
          FriendShipStatus::PendingForA | FriendShipStatus::PendingForB => {
            let pending_for_you_status = database::get_friend_request_type(&username, &peer_username);
            if pending_for_you_status != friend.1 {
              // This user is requesting to be friends.
              status = "Awaiting your response.";
              let accept_button = ui::Button::new(Vector2 { x: 70.0*vw, y: 26.0*vh + current_offset }, Vector2 { x: 15.0*vw, y: 6.0*vh }, "Accept", 5.0*vh);
              if accept_button.was_pressed() {
                // Accept the friend request by sending a friend request to this user, which the
                // server processes as an accept.
                packet_queue.push(
                  ClientToServerPacket {
                    information: ClientToServer::SendFriendRequest(String::from(peer_username)),
                  }
                );
                packet_queue.push(
                  ClientToServerPacket {
                    information: ClientToServer::GetFriendList,
                  }
                );
                friend_request_input = peer_username.to_string();
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
            draw_text(match online {true => "Online", false => "Offline"}, 70.0*vw, 30.0*vh + current_offset, 5.0*vh, BLACK);
            // if we were invited by this user, show accept button
            if server_interaction.lobby_invites.contains(&String::from(peer_username)) {
              let mut accept_button = ui::Button::new(
                Vector2 { x: 70.0*vw, y: 26.0*vh + current_offset }, Vector2 { x: 15.0*vw, y: 6.0*vh }, "Join", 5.0*vh
              );
              accept_button.draw(vh, !menu_paused);
              if accept_button.was_pressed() {
                packet_queue.push(
                  ClientToServerPacket { information: ClientToServer::LobbyInviteAccept(String::from(peer_username)) }
                );
                server_interaction.lobby_invites.retain(|element| element != peer_username);  
              }
            }
            // invite user button
            else {
              if online {
                let mut invite_button = ui::Button::new(
                  Vector2 { x: 70.0*vw, y: 26.0*vh + current_offset }, Vector2 { x: 15.0*vw, y: 6.0*vh }, "Invite", 5.0*vh
                );
                invite_button.draw(vh, !menu_paused);
                if invite_button.was_pressed() {
                  packet_queue.push(
                    ClientToServerPacket { information: ClientToServer::LobbyInvite(String::from(peer_username)) }
                  );
                  notifications.push(
                    Notification::new(&format!("Invited {} to lobby.", peer_username), 1.5)
                  );
                }
              }
            }
          }
        }
        draw_text(status, 40.0*vw, 30.0*vh + current_offset, 5.0*vh, BLACK);
      }
      // friends
    } if !main_tabs.selected_tab() == 4 {
      tab_friends_refresh_flag = false;
    }

    // chat box
    let chatbox_position = Vector2{x: 5.0 * vw, y: 20.0 * vh};
    let chatbox_size = Vector2{x: 30.0 * vw, y: 70.0 * vh};
    ui::chatbox(chatbox_position, chatbox_size, server_interaction.friend_list.clone(), &mut server_interaction.is_chatbox_open, &mut server_interaction.selected_friend, &mut server_interaction.recv_messages_buffer, &mut server_interaction.chat_input_buffer, &mut server_interaction.chat_selected, vh, username.clone(), &mut packet_queue, &mut server_interaction.chat_scroll_index, false, &mut chat_timer);

    // draw NOTIFICATIONS
    for n_index in 0..notifications.len() {
      notifications[n_index].draw(vh, vw, 5.0*vh, n_index);
    }
    for n_index in 0..notifications.len() {
      if notifications[n_index].start_time.elapsed().as_secs_f32() > notifications[n_index].duration {
        notifications.remove(n_index);
        break;
      }
    }
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
    let mut quit: bool = false;
    if menu_paused {
      (menu_paused, quit) = ui::draw_pause_menu(vh, vw, &mut settings, &mut settings_open_flag, &mut settings_tabs, (&mut sfx_self_track, &mut sfx_other_track, &mut music_track));
    }
    if quit {
      exit(0);
    }
    next_frame().await;
  }
}