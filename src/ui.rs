use core::panic;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::time::Instant;
use std::fs::File;
use macroquad::prelude::*;
use crate::common::*;
use crate::database::FriendShipStatus;
use crate::gamedata::Character;
use crate::gamedata::CharacterDescription;
use crate::graphics;
use crate::graphics::draw_rectangle;
use crate::maths::*;
use crate::graphics::*;
use crate::mothership_common::ChatMessageType;
use crate::mothership_common::ClientToServer;
use crate::mothership_common::ClientToServerPacket;

// MARK: Buttons & fluff
pub struct Button {
  pub position:  Vector2,
  pub size:      Vector2,
  pub text:      String,
  pub font_size: f32,
  clickable:     bool,
}
impl Button {
  pub fn new(position: Vector2, size: Vector2, text: &str, font_size: f32) -> Button {
    return Button {
      position,
      size,
      text: text.to_string(),
      font_size,
      clickable: true,
    }
  }
  pub fn draw(&mut self, vh: f32, clickable: bool) {
    self.clickable = clickable;
    let position = self.position;
    let size = self.size;
    let text = self.text.as_str();
    let font_size = self.font_size;
    graphics::draw_rectangle(position, size, BLUE);
    let inner_shrink: f32 = 1.0 * vh;
    graphics::draw_rectangle(position + Vector2{x: inner_shrink, y: inner_shrink}, size - Vector2{x:  inner_shrink*2.0, y: inner_shrink*2.0}, SKYBLUE);
    draw_text(text, position.x + 1.0*vh, position.y + size.y * 0.65, font_size , BLACK);
    let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
    if self.clickable {
      if mouse.x > position.x && mouse.x < (position.x + size.x) {
        if mouse.y > position.y && mouse.y < (position.y + size.y) {
          graphics::draw_rectangle(position, size,GRAY);
          draw_text(text, position.x + 10.0, position.y + size.y * 0.65, font_size , BLACK);
        }
      }
    }
  }
  pub fn is_down(&self) -> bool {
    let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
    if mouse.x > self.position.x && mouse.x < (self.position.x + self.size.x) {
      if mouse.y > self.position.y && mouse.y < (self.position.y + self.size.y) {
        if is_mouse_button_down(MouseButton::Left) {
          return true & self.clickable;
        }
      }
    }
    return false;
  }
  pub fn was_pressed(&self) -> bool {
    let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
    if mouse.x > self.position.x && mouse.x < (self.position.x + self.size.x) {
      if mouse.y > self.position.y && mouse.y < (self.position.y + self.size.y) {
        if is_mouse_button_pressed(MouseButton::Left) {
          return true & self.clickable;
        }
      }
    }
    return false;
  }
}

// Represents a row of tabs
pub struct Tabs {
  pub position: Vector2,
  pub size: Vector2,
  pub tab_names: Vec<String>,
  selected: Vec<bool>,
  font_size: f32,
}
impl Tabs {
  pub fn new(position: Vector2, size: Vector2, tab_names: Vec<String>, font_size: f32) -> Tabs {
    let mut selected: Vec<bool> = Vec::new();
    for i in 0..tab_names.len() {
      if i == 0 {
        selected.push(true);
      }
      else {
        selected.push(false);
      }
    }
    return Tabs {
      position,
      size,
      tab_names: tab_names,
      selected,
      font_size,
    }
  }
  pub fn update_size(&mut self, position: Vector2, size: Vector2, font_size: f32) {
    self.position = position;
    self.size = size;
    self.font_size = font_size;
  }
  pub fn set_selected(&mut self, index: usize) {
    for i in 0..self.selected.len() {
      self.selected[i] = i == index;
    }
  }
  pub fn draw_and_process(&mut self, vh: f32, clickable: bool) {
    fn one_way_button(position: Vector2, size: Vector2, text: &str, font_size: f32, vh: f32, selected: bool, clickable: bool) -> bool {
      graphics::draw_rectangle(position, size, BLUE);
      let inner_shrink: f32 = 1.0 * vh;
      graphics::draw_rectangle(position + Vector2{x: inner_shrink, y:inner_shrink}, size - Vector2{x: inner_shrink*2.0, y: inner_shrink*2.0}, SKYBLUE);
      draw_text(text, position.x + 1.0*vh, position.y + size.y * 0.65, font_size , BLACK);
      if selected {
        graphics::draw_rectangle(position, size,GRAY);
        draw_text(text, position.x + 10.0, position.y + size.y * 0.65, font_size , BLACK);
      }
      let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
      if clickable {
        if mouse.x > position.x && mouse.x < (position.x + size.x) {
          if mouse.y > position.y && mouse.y < (position.y + size.y)   {
            graphics::draw_rectangle(position, size,GRAY);
            draw_text(text, position.x + 10.0, position.y + size.y * 0.65, font_size , BLACK);
            if is_mouse_button_down(MouseButton::Left) {
              return true;
            }
          }
        }
      }
      return false;
    }
    let button_count = self.selected.len();
    let button_width = self.size.x / button_count as f32;

    let mut buttons: Vec<bool> = Vec::new();
    for i in 0..self.selected.len() {
      buttons.push(
        one_way_button(Vector2 { x: self.position.x + i as f32 * button_width, y: self.position.y }, Vector2 { x: button_width, y: self.size.y }, &self.tab_names[i], self.font_size, vh, self.selected[i], clickable)
      );
    };
    for i in 0..buttons.len() {
      if buttons[i] {
        for x in 0..buttons.len() {
          if x == i {
            self.selected[x] = true;
          }
          else {
            self.selected[x] = false;
          }
        }
        break;
      }
    };
  }
  pub fn selected_tab(&self) -> usize {
    for (i, tab_selected) in self.selected.clone().iter().enumerate() {
      if *tab_selected {
        return i;
      }
    }
    return 0;
  }
}
/// A checkbox.
/// 
/// Reads a `selected` boolean and modifies it. If the value was changed this frame,
/// returns `true`.
pub fn checkbox(position: Vector2, size: f32, text: &str, font_size: f32, vh: f32, selected: &mut bool) -> bool {
  graphics::draw_rectangle(position, Vector2 { x: size, y: size }, BLUE);
  let inner_shrink: f32 = 0.2 * vh;
  graphics::draw_rectangle(position + Vector2{x: inner_shrink,y: inner_shrink}, Vector2{x: size, y:size} - Vector2{ x: inner_shrink*2.0, y: inner_shrink*2.0}, SKYBLUE);
  draw_text(text, position.x + size + 1.0 *vh, position.y + size / 1.5, font_size , BLACK);

  
  if *selected {
    draw_line(position.x, position.y + size/2.0, position.x + size/2.0, position.y + size, 0.5*vh, WHITE);
    draw_line(position.x + size/2.0, position.y + size, position.x + size,position.y, 0.5*vh, WHITE);
  }
  let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
  if mouse.x > position.x && mouse.x < (position.x + size) {
    if mouse.y > position.y && mouse.y < (position.y + size) {
      graphics::draw_rectangle(position, Vector2 { x: size, y: size },Color { r: 0.05, g: 0.0, b: 0.1, a: 0.2 });
      if is_mouse_button_pressed(MouseButton::Left) {
        *selected = !*selected;
        return true;
      }
    }
  }
  return false;
}
/// slider.
/// 
/// Returns true if the value was modified.
pub fn slider(position: Vector2, size: Vector2, text: &str, font_size: f32, vh: f32, value: &mut f32, value_min: f32, value_max: f32) {
  let shrink = 1.0*vh;
  graphics::draw_rectangle(position, size, BLUE);
  graphics::draw_rectangle(position + Vector2 {x: shrink, y: shrink}, size - Vector2 {x: shrink*2.0, y:shrink*2.0}, SKYBLUE);
  let slider_width = 2.0 * vh;
  let slider_x_pos = position.x + (size.x - slider_width) * ((*value-value_min) / (value_max - value_min));
  draw_rectangle(Vector2 { x: slider_x_pos, y: position.y }, Vector2 { x: slider_width, y: size.y }, BLUE);
  let mut formatted_value: String = format!("{:.2}", value);
  if *value >= 1.0 {
    formatted_value = format!("{:.1}", value);
  }
  if *value >= 10.0 {
    formatted_value = format!("{:.0}", value);
  }
  draw_text(format!("{}: {}", text, formatted_value).as_str(), position.x + 2.0*vh, position.y + size.y * 0.65, font_size , BLACK);

  let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
  let margin = size.x * 0.1;
  if mouse.x > (position.x - margin) && mouse.x < (position.x + size.x + margin) {
    if mouse.y > position.y && mouse.y < (position.y + size.y) {
      if is_mouse_button_down(MouseButton::Left) {
        *value = (mouse.x - position.x) / size.x * (value_max - value_min) + value_min;
        if *value > value_max {
          *value = value_max
        }
        if *value < value_min {
          *value = value_min
        }
      }
    }
  }
}

// MARK: In-game

/// - ability index:
///   - 1: primary
///   - 2: secondary
///   - 3: dash
///   - 4: passive
/// - squished: whether to slightly shrink the icon to show the ability was used
/// - progress: cooldown / charge, 0.0-1.0
pub fn draw_ability_icon(position: Vector2, size: Vector2, ability_index: usize, squished: bool, progress: f32, vh: f32, vw: f32, font: &Font, character_descriptions: HashMap<Character, CharacterDescription>, character: Character) -> () {
  let icon: Texture2D = match ability_index {
    1 => {Texture2D::from_file_with_format(include_bytes!("../assets/ui/temp_ability_1.png"), None)},
    2 => {Texture2D::from_file_with_format(include_bytes!("../assets/ui/temp_ability_2.png"), None)},
    3 => {Texture2D::from_file_with_format(include_bytes!("../assets/ui/temp_ability_3.png"), None)},
    4 => {Texture2D::from_file_with_format(include_bytes!("../assets/ui/temp_ability_4.png"), None)},
    _ => {panic!()},
  };
  let squish_offset = match squished {
    true => 1.0,
    false => 0.0
  };
  draw_image(&icon,
    position.x + squish_offset/2.0,
    position.y + squish_offset/2.0,
    size.x - squish_offset,
    size.y - squish_offset,
    vh, Vector2::new(), WHITE
  );
  graphics::draw_rectangle(
    Vector2{x: (position.x + squish_offset/2.0) * vh, y:(position.y + squish_offset/2.0) * vh},
    Vector2{x: (size.x - squish_offset) * vh, y: ((size.y - squish_offset) * (1.0 - progress)) * vh},
    Color { r: 0.05, g: 0.0, b: 0.1, a: 0.4 },
  );
  let text = match ability_index {
    1 => " LMB ",
    2 => " RMB ",
    3 => "Space",
    4 => "",
    _ => "Unkown",
  };
  draw_text_ex(text, (position.x + size.y * 0.125) * vh, (position.y + size.y * 1.3) * vh, TextParams { font: Some(font), font_size: (size.x * 0.3 * vh) as u16, ..Default::default() });
  let ability = match ability_index {
    1 => character_descriptions[&character].primary.clone(),
    2 => character_descriptions[&character].secondary.clone(),
    3 => character_descriptions[&character].dash.clone(),
    4 => character_descriptions[&character].passive.clone(),
    _ => character_descriptions[&character].passive.clone(),
  };
  let text = ability.to_text();
  tooltip(position * vh, size * vh, &text, vh, vw, Vector2 {x: mouse_position().0, y: mouse_position().1});
}

pub fn draw_player_info(position: Vector2, size: f32, player: ClientPlayer, font: &Font, vh: f32, settings: Settings) -> () {
  let color = match player.team {
    Team::Red => RED,
    Team::Blue => BLUE,
  };
  let displayed_name =
    if settings.display_char_name_instead {
      player.character.name()
    }
    else {
      player.username.clone()
    };
  draw_text_ex(&displayed_name, (position.x) * vh, (position.y) * vh, TextParams { font: Some(font), font_size: (size * 0.5 * vh) as u16, color: color, ..Default::default() });
  graphics::draw_rectangle(
    Vector2 {x: (position.x) * vh, y: (position.y + 1.5) * vh},
    Vector2 {x: (size * (100.0 as f32 / 100.0) * 2.0 ) * vh, y: (size * 0.25 ) * vh},
    Color { r: 0.0, g: 0.0, b: 0.0, a: 0.5 },
  );
  graphics::draw_rectangle(
    Vector2 {x: (position.x) * vh, y: (position.y + 1.5) * vh},
    Vector2{x:( size * (player.health as f32 / 100.0) * 2.0 ) * vh, y: (size * 0.25 ) * vh},
    Color { r: 0.0, g: 1.0, b: 0.1, a: 1.0 },
  );
}

// MARK: Esc Menu

/// Not actually a pause menu, but you get the point
/// 
/// This function is called both in-game and in the main menu
/// 
/// returns menu_paused and wants_to_quit
pub fn draw_pause_menu(vh: f32, vw: f32, settings: &mut Settings, settings_open: &mut bool) -> (bool, bool) {
  let mut menu_paused = true;
  let mut wants_to_quit = false;
  let button_y_separation: f32 = 15.0 * vh;
  let button_y_offset: f32 = 25.0 * vh;
  let button_font_size = 5.0 * vh;

  let button_size: Vector2 = Vector2 { x: 25.0 * vh, y: 9.0 * vh };
  // semi-transparent background
  graphics::draw_rectangle(Vector2 {x:0.0, y: 0.0}, Vector2 {x: vw * 100.0, y: vh * 100.0}, Color { r: 1.0, g: 1.0, b: 1.0, a: 0.75 });
  if !*settings_open {
    // buttons
    let resume_button_position: Vector2 = Vector2 { x: vw * 50.0 - button_size.x/2.0, y: button_y_offset };
    let mut resume_button = Button::new(resume_button_position, button_size, "Resume", button_font_size);
    resume_button.draw(vh, true);
    if resume_button.is_down() {
      menu_paused = false;
      *settings_open = false;
    }
    
    
    let settings_button_position: Vector2 = Vector2 { x: vw * 50.0 - button_size.x/2.0, y: button_y_offset + button_y_separation };
    let mut settings_button = Button::new(settings_button_position, button_size, "Options", button_font_size);
    settings_button.draw(vh, true);
    if settings_button.is_down() {
      *settings_open = true;
    }

    // Quit button
    let quit_button_position: Vector2 = Vector2 { x: vw * 50.0 - button_size.x/2.0, y: button_y_offset + button_y_separation * 2.0 };
    let mut quit_button = Button::new(quit_button_position, button_size, "Quit", button_font_size);
    quit_button.draw(vh, true);
    if quit_button.is_down() {
      wants_to_quit = true;
      menu_paused = false;
      *settings_open = false;
    }

  }
  if *settings_open {
    let mut back_button = Button::new(Vector2 { x: vw * 50.0 - button_size.x/2.0, y: 15.0*vh }, Vector2 { x: 25.0 * vh, y: 9.0 * vh }, "Back", button_font_size);
    back_button.draw(vh, true);
    if back_button.is_down() {
      *settings_open = false;
    }
    let c1 = checkbox(Vector2 { x: vw * 25.0, y: vh * 30.0 }, 4.0 * vh, "Camera smoothing", 5.0*vh, vh, &mut settings.camera_smoothing);
    let c2 = checkbox(Vector2 { x: vw * 25.0, y: vh * 35.0 }, 4.0 * vh, "Display character names instead of usernames", 5.0*vh, vh, &mut settings.display_char_name_instead);

    let c3 = checkbox(Vector2 { x: vw * 25.0, y: vh * 40.0 }, 4.0 * vh, "Fullscreen", 5.0*vh, vh, &mut settings.fullscreen);
    if c3 {
      set_fullscreen(settings.fullscreen);
    }
    // if the settings were modified, save them.
    if c1 || c2 || c3 {
      settings.save();
    }
  }

  return (menu_paused, wants_to_quit)
}

/// Used to place items in relation to itself. In other words, a sort of "container".
pub struct DivBox {
  pub position: Vector2,
  pub nested: Vec<DivBox>,
}
impl DivBox {
  pub fn rel_pos(&self, position: Vector2) -> Vector2 {
    return Vector2 { x: self.position.x + position.x, y: self.position.y + position.y }
  }
}

// MARK: Notification
pub struct Notification {
  pub start_time: Instant,
  pub text: String,
  pub duration: f32,
}
impl Notification {
  pub fn draw(&self, vh: f32, vw: f32, font_size: f32, offset: usize) {
    let position: Vector2 = Vector2 { x: 65.0*vw, y: 10.0 * vh + offset as f32 * 10.0 * vh };
    let size: Vector2 = Vector2 { x: 30.0*vw, y: 10.0*vh };
    let inner_shrink: f32 = 1.0 * vh;
    graphics::draw_rectangle(position, size, BLUE);
    graphics::draw_rectangle(position + Vector2 {x: inner_shrink, y: inner_shrink}, size - Vector2 {x: inner_shrink*2.0, y: inner_shrink*2.0}, SKYBLUE);
    draw_text(self.text.as_str(), position.x + 2.0 * vh, position.y + size.y * 0.65, font_size, BLACK);
  }
  pub fn new(text: &str, duration: f32) -> Notification {
    return Notification { start_time: Instant::now(), text: String::from(text), duration }
  }
}

// MARK: Settings

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Settings {
  pub camera_smoothing: bool,
  /// If false, usernames are displayed.
  /// If true, character names are displayed.
  pub display_char_name_instead: bool,
  pub fullscreen: bool,
  pub saved_username: String,
  pub store_credentials: bool,
}
impl Settings {
  pub fn new() -> Settings{
    return Settings {
      camera_smoothing: true,
      display_char_name_instead: true,
      fullscreen: false,
      saved_username: String::new(),
      store_credentials: false,
    }
  }
  pub fn load() -> Settings {
    let settings_file_name = "moba_settings";
    let settings_file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .open(settings_file_name);

    match settings_file {
      Ok(mut file) => {
        let mut data = vec![];
        match file.read_to_end(&mut data) {
          Ok(_) => {
            match bincode::deserialize::<Settings>(&data) {
              Ok(settings) => {
                return settings
              },
              Err(_) => {
                file.set_len(0).ok(); // clear
                file.write_all(&bincode::serialize(&Settings::new()).expect("oops")).expect("oops");
                return Settings::new();
              }
            }
          }
          Err(_) => {
            println!("Couldn't read settings file.");
            file.set_len(0).ok();
            match file.write_all(&bincode::serialize(&Settings::new()).expect("oops")) {
              Ok(_) => {}
              Err(_) => {
                return Settings::new();
              }
            }
            return Settings::new();
          }
        }
      }
      Err(_) => {
        println!("Error loading settings file, using default settings.");
        return Settings::new();
      }
    }
  }
  pub fn save(&self) {
    let settings_file_name = "moba_settings";
    let settings_file = File::create(settings_file_name);
    match settings_file {
      Ok(mut file) => {
        file.write_all(&bincode::serialize::<Settings>(&self).expect("Serialization failure.")).expect("oops");
      }
      Err(_) => { }
    }
  }
}

// MARK: Chat

pub fn chatbox(
  position:                  Vector2,
  size:                      Vector2,
  friends:                   Vec<(String, FriendShipStatus, bool)>,
  is_chatbox_open:           &mut bool,
  selected_friend:           &mut usize,
  recv_messages_buffer:      &mut Vec<(String, String, ChatMessageType)>,
  chat_input_buffer:         &mut String,
  chat_input_field_selected: &mut bool,
  vh:                        f32,
  username:                  String,
  packet_queue:              &mut Vec<ClientToServerPacket>,
  scroll_index:              &mut usize,
  is_in_game:                bool,
  timer:                     &mut Instant,
) {

  let margin: f32 = 1.5 * vh;

  let mut message_type = ChatMessageType::Private;

  let chat_remain_displayed_time = 10.0;

  // get a list of online friends (which we can chat to).
  let mut online_friends: Vec<String> = Vec::new();
  if is_in_game {
    online_friends.push(String::from("tc"));
    online_friends.push(String::from("ac"));
  }
  for friend in friends {
    if friend.1 == FriendShipStatus::Friends
    && friend.2 == true  {
      online_friends.push(friend.0);
    }
  }
  // cycle through friends if TAB is pressed
  if online_friends.len() >= 2 {
    if get_keys_pressed().contains(&KeyCode::Tab) {
      *selected_friend += 1;
      if *selected_friend >= online_friends.len() {
        *selected_friend = 0;
      }
    }
  } else {
    *selected_friend = 0;
  }

  // Open chat if ENTER is pressed
  if !*is_chatbox_open {
    if get_keys_pressed().contains(&KeyCode::Enter) {
      clear_input_queue();
      *is_chatbox_open = true;
      *chat_input_field_selected = true;
      return
    }
  }
  // Close chat if ENTER is pressed and buffer is empty
  if *is_chatbox_open {
    if get_keys_pressed().contains(&KeyCode::Enter)
    && (chat_input_buffer.is_empty() || !*chat_input_field_selected) {
      *is_chatbox_open = false;
      *chat_input_field_selected = false;
      return
    }
  }

  let text_input_box_size = Vector2 {x: size.x, y: 5.5 * vh};
  if *is_chatbox_open {
    // draw frame
    graphics::draw_rectangle(position, size, Color { r: 0.025, g: 0.0, b: 0.05, a: 0.45 });
  }
  if timer.elapsed().as_secs_f32() < chat_remain_displayed_time && !*is_chatbox_open {
    graphics::draw_rectangle(Vector2 { x: position.x, y: position.y + size.y - 9.0 * vh }, Vector2 { x: size.x, y: -16.0 * vh }, Color { r: 0.025, g: 0.0, b: 0.05, a: 0.20 });
  }
  if *is_chatbox_open {
    
    // draw selected friend indicator
    //let selected_friend_indicator_size = Vector2 {x: size.x}
    let mut displayed_selected_friend = if online_friends.len() > 0 {
      let peer_username;
      let split: Vec<&str> = (*online_friends[*selected_friend]).split(":").collect();
      if *split[0] == username {
        peer_username = split[1];
      } else {
        peer_username = split[0];
      }
      peer_username
    } else {
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
      ChatMessageType::Team => SKYBLUE,
      ChatMessageType::All => ORANGE,
    };
    draw_text(&format!("[TAB] Messaging: {}", displayed_selected_friend), position.x, position.y + size.y - text_input_box_size.y - 1.0 *vh, 3.0 * vh, color);
    
    // draw input textbox
    text_input(position + Vector2 {x: 0.0, y: size.y - text_input_box_size.y}, text_input_box_size, chat_input_buffer, chat_input_field_selected, 3.0*vh, vh, false, &mut false);
    
    // Send message if ENTER is pressed and buffer is not empty
    // and a friend can be messaged and input field selected.
    if !chat_input_buffer.is_empty()
    && !online_friends.is_empty()
    && *chat_input_field_selected
    && get_keys_pressed().contains(&KeyCode::Enter) {

      // send message
      let peer_username;
      let split: Vec<&str> = (*online_friends[*selected_friend]).split(":").collect();
      if *split[0] == username {
        peer_username = split[1];
      } else {
        peer_username = split[0];
      }
      packet_queue.push(
        ClientToServerPacket {
          information: ClientToServer::SendChatMessage(String::from(peer_username), chat_input_buffer.clone())
        }
      );

      // reset things
      recv_messages_buffer.push((username, chat_input_buffer.clone(), message_type));
      *chat_input_buffer = String::new();
      *timer = Instant::now();
      *is_chatbox_open = false;
    }
  }
  if *is_chatbox_open || timer.elapsed().as_secs_f32() < chat_remain_displayed_time {
    // draw all messages
    let y_start = position.y + size.y - text_input_box_size.y - 5.0 * vh;
    let mut formatted_messages: Vec<(String, ChatMessageType)> = Vec::new();
    let mut reversed_recv_messages: Vec<(String, String, ChatMessageType)> = recv_messages_buffer.clone();
    reversed_recv_messages.reverse();
    for message in reversed_recv_messages {
      let message_type = match message.2 {
        ChatMessageType::Administrative => {"Admin"},
        ChatMessageType::Private => {"Message"},
        ChatMessageType::Group => {"Group"},
        ChatMessageType::Team => {"Team"},
        ChatMessageType::All => {"All"},
      };
      let mut formatted_message = format!("[{}] {}: {}", message_type, message.0, message.1);
      while measure_text(&formatted_message, TextParams::default().font, (3.0 * vh) as u16, 1.0).width > size.x - margin {
        let mut new_message = String::new();
        while measure_text(&new_message, TextParams::default().font, (3.0 * vh) as u16, 1.0).width < size.x - margin {
          new_message.insert(0, formatted_message.pop().expect("oopsies"));
        }
        formatted_messages.push((new_message, message.2.clone()));
      }
      formatted_messages.push((formatted_message, message.2.clone()));
    }

    if *scroll_index > formatted_messages.len() {
      *scroll_index = formatted_messages.len() -1;
    }
    if !*is_chatbox_open {
      formatted_messages.truncate(5);
    }
    if formatted_messages.len() > 0 {
      for m_index in *scroll_index..(*formatted_messages).len() {
        let pos_y = y_start - ((m_index - *scroll_index) as f32) * 3.0 * vh;
        if pos_y < position.y {
          break;
        }
        let color = match formatted_messages[m_index].1 {
          ChatMessageType::Administrative => YELLOW,
          ChatMessageType::Private => PINK,
          ChatMessageType::Group => GREEN,
          ChatMessageType::Team => SKYBLUE,
          ChatMessageType::All => ORANGE,
        };
        draw_text(&formatted_messages[m_index].0, position.x, pos_y, 3.0 * vh, color);
      }
    }
    let mouse_wheel = mouse_wheel();
    if mouse_wheel.1 > 0.0
    && *scroll_index < formatted_messages.len() {
      *scroll_index += 1;
    }
    if mouse_wheel.1 < 0.0
    && *scroll_index > 0 {
      *scroll_index -= 1;
    }
  }
}

// MARK: Text input

/// A text input field.
pub fn text_input(position: Vector2, size: Vector2, buffer: &mut String, active: &mut bool, font_size: f32, vh: f32, hideable: bool, show_password: &mut bool) {
  let margin: f32 = 2.0 * vh;
  let mouse = Vector2 { x: mouse_position().0, y: mouse_position().1 };

  if is_mouse_button_pressed(MouseButton::Left) {
    let is_inside =
      mouse.x > position.x && mouse.x < position.x + size.x &&
      mouse.y > position.y && mouse.y < position.y + size.y;

    *active = is_inside;
    if is_inside {
      // empty the input queue
      clear_input_queue();
    }
  }

  let bg = if *active { DARKGRAY } else { GRAY };
  graphics::draw_rectangle(position, size, bg);

  if hideable {
    checkbox(Vector2 { x: position.x + size.x + margin, y: position.y + size.y * 0.15 }, size.y * 0.7, "show", font_size, vh, show_password);
  }

  if *active {
    if let Some(ch) = get_char_pressed() {
      // extra check not to allow goofy characters like backspace...
      if ch >= ' ' /* && ch <= '~' */ {
        buffer.push(ch);
      }
      // 8 = backspace
      if Some(ch) == char::from_u32(8) {
        buffer.pop();
      }
    }
  }
  let mut text_to_draw = buffer.clone();
  if hideable && !*show_password {
    let len = text_to_draw.len();
    text_to_draw = String::new();
    for _ in 0..len {
      text_to_draw.push('*');
    }
  }
  let mut text_size = measure_text(&text_to_draw, TextParams::default().font, font_size as u16, 1.0);
  while text_size.width > size.x - margin * 2.0 {
    text_size = measure_text(&text_to_draw, TextParams::default().font, font_size as u16, 1.0);
    text_to_draw.remove(0);
  }
  draw_text(text_to_draw.as_str(), position.x + margin, position.y + size.y * 0.65, font_size, WHITE);
}

// MARK: Tooltip
/// When the mouse hovers over the given rectangle with `position`
/// and `size`, it will display the given text.
pub fn tooltip(position: Vector2, size: Vector2, text: &str, vh: f32, vw: f32, mouse_pos: Vector2) {
  let font_size = 4.0 * vh;
  if mouse_pos.x < position.x + size.x
  && mouse_pos.x > position.x
  && mouse_pos.y < position.y + size.y
  && mouse_pos.y > position.y {
    //show_mouse(false);
    let lines: Vec<&str> = text.split("\n").collect();
    let y_offset = 4.0 * vh;
    let y_size = y_offset * (lines.len() as f32) + 2.0 * vh;
    let mut x_size = 0f32;
    for line in lines.clone() {
      let line_len = measure_text(line, TextParams::default().font, font_size as u16, 1.0).width;
      if line_len > x_size {
        x_size = line_len;
      }
    }
    // margin
    x_size += 2.0 * vh;
    let visibility_x_offset: f32 = if mouse_pos.x > 60.0 * vw {
      - x_size - 10.0
    } else {
      10.0
    };
    draw_rectangle(mouse_pos - Vector2 {x: 0.5*vh, y: 0.5*vh} + Vector2 {x: visibility_x_offset, y: 0.0}, Vector2 { x: x_size + 1.0*vh, y: y_size + 1.0*vh }, BLUE);
    draw_rectangle(mouse_pos                                  + Vector2 {x: visibility_x_offset, y: 0.0}, Vector2 { x: x_size,          y: y_size          }, SKYBLUE);
    let text_size_y = measure_text(lines[0], TextParams::default().font, font_size as u16, 1.0).height;
    let initial_pos = mouse_pos + Vector2 {x: 1.0 *vh + visibility_x_offset, y: 1.0*vh + text_size_y};
    for (i, line) in lines.iter().enumerate() {
      draw_text(line, initial_pos.x, initial_pos.y + (i as f32) * y_offset, font_size, BLACK);
    }
  }
}

//MARK:  Credential store
use keyring;
const SERVICE_NAME: &str = "SYLVAN_ROW";
/// Tries to store password in keyring.
pub fn save_password(password: &str, username: &str, notifications: &mut Vec<Notification>) {
  let entry = match keyring::Entry::new(SERVICE_NAME, username) {
    Ok(entry) => entry,
    Err(err) => {
      notifications.push(
        Notification::new("Failed to access keyring.", 1.0)
      );
      println!("1 {:?}", err);
      return;
    }
  };
  match entry.set_password(password) {
    Ok(_) => {},
    Err(err) => {
      notifications.push(
        Notification::new("Failed to add password to keyring.", 1.0)
      );
      println!("2 {:?}", err);
    }
  };
}
/// Attempts to load the password from they keyring. If it fails, it returns nothing.
pub fn load_password(username: &str) -> String {

  let entry = match keyring::Entry::new(SERVICE_NAME, username) {
    Ok(entry) => {entry}
    Err(err) => {
      println!("3 {:?}", err);
      return String::new();
    }
  };
  match entry.get_password() {
    Ok(password) => {
      return password;
    },
    Err(err) => {
      println!("4 {:?}", err);
      return String::new();
    }
  }
}