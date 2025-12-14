use core::panic;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::time::Instant;
use std::fs::File;
use macroquad::prelude::*;
use crate::common::*;
use crate::maths::*;
use crate::graphics::*;

pub fn button(position: Vector2, size: Vector2, text: &str, font_size: f32, vh: f32) -> bool {
  draw_rectangle(position.x, position.y, size.x, size.y, BLUE);
  let inner_shrink: f32 = 1.0 * vh;
  draw_rectangle(position.x + inner_shrink, position.y + inner_shrink, size.x - inner_shrink*2.0, size.y - inner_shrink*2.0, SKYBLUE);
  draw_text(text, position.x + 1.0*vh, position.y + size.y / 2.0, font_size , BLACK);
  let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
  if mouse.x > position.x && mouse.x < (position.x + size.x) {
    if mouse.y > position.y && mouse.y < (position.y + size.y) {
      draw_rectangle(position.x, position.y, size.x, size.y,GRAY);
      draw_text(text, position.x + 10.0, position.y + size.y / 2.0, font_size , BLACK);
      if is_mouse_button_down(MouseButton::Left) {
        return true;
      }
    }
  }
  return false;
}
pub fn button_was_pressed(position: Vector2, size: Vector2, text: &str, font_size: f32, vh: f32) -> bool {
  draw_rectangle(position.x, position.y, size.x, size.y, BLUE);
  let inner_shrink: f32 = 1.0 * vh;
  draw_rectangle(position.x + inner_shrink, position.y + inner_shrink, size.x - inner_shrink*2.0, size.y - inner_shrink*2.0, SKYBLUE);
  draw_text(text, position.x + 1.0*vh, position.y + size.y / 2.0, font_size , BLACK);
  let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
  if mouse.x > position.x && mouse.x < (position.x + size.x) {
    if mouse.y > position.y && mouse.y < (position.y + size.y) {
      draw_rectangle(position.x, position.y, size.x, size.y,GRAY);
      draw_text(text, position.x + 10.0, position.y + size.y / 2.0, font_size , BLACK);
      if is_mouse_button_pressed(MouseButton::Left) {
        return true;
      }
    }
  }
  return false;
}
pub fn one_way_button(position: Vector2, size: Vector2, text: &str, font_size: f32, vh: f32, selected: bool) -> bool {
  draw_rectangle(position.x, position.y, size.x, size.y, BLUE);
  let inner_shrink: f32 = 1.0 * vh;
  draw_rectangle(position.x + inner_shrink, position.y + inner_shrink, size.x - inner_shrink*2.0, size.y - inner_shrink*2.0, SKYBLUE);
  draw_text(text, position.x + 1.0*vh, position.y + size.y / 2.0, font_size , BLACK);
  if selected {
    draw_rectangle(position.x, position.y, size.x, size.y,GRAY);
    draw_text(text, position.x + 10.0, position.y + size.y / 2.0, font_size , BLACK);
  }
  let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
  if mouse.x > position.x && mouse.x < (position.x + size.x) {
    if mouse.y > position.y && mouse.y < (position.y + size.y)   {
      draw_rectangle(position.x, position.y, size.x, size.y,GRAY);
      draw_text(text, position.x + 10.0, position.y + size.y / 2.0, font_size , BLACK);
      if is_mouse_button_down(MouseButton::Left) {
        return true;
      }
    }
  }
  return false;
}
/// A checkbox.
/// 
/// Reads a `selected` boolean, returns the same bool if it wasn't
/// pressed, and returns the opposite if it was.
pub fn checkbox(position: Vector2, size: f32, text: &str, font_size: f32, vh: f32, selected: bool) -> bool {
draw_rectangle(position.x, position.y, size, size, BLUE);
  let inner_shrink: f32 = 0.2 * vh;
  draw_rectangle(position.x + inner_shrink, position.y + inner_shrink, size - inner_shrink*2.0, size - inner_shrink*2.0, SKYBLUE);
  draw_text(text, position.x + size + 1.0 *vh, position.y + size / 1.5, font_size , BLACK);

  
  if selected {
    draw_line(position.x, position.y + size/2.0, position.x + size/2.0, position.y + size, 0.5*vh, WHITE);
    draw_line(position.x + size/2.0, position.y + size, position.x + size,position.y, 0.5*vh, WHITE);
  }
  let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
  if mouse.x > position.x && mouse.x < (position.x + size) {
    if mouse.y > position.y && mouse.y < (position.y + size) {
      draw_rectangle(position.x, position.y, size, size,Color { r: 0.05, g: 0.0, b: 0.1, a: 0.2 });
      if is_mouse_button_pressed(MouseButton::Left) {
        return !selected;
      }
    }
  }
  return selected;
}
/// A text input field.
pub fn text_input(position: Vector2, size: Vector2, buffer: &mut String, active: &mut bool, font_size: f32, vh: f32) {

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
  draw_rectangle(position.x, position.y, size.x, size.y, bg);

  if *active {
    if let Some(ch) = get_char_pressed() {
      // extra check not to add backspace...
      if ch >= ' ' && ch <= '~' {
        buffer.push(ch);
      }
      // 8 = backspace
      if Some(ch) == char::from_u32(8) {
        buffer.pop();
      }
    }
    
  }

  draw_text(buffer.as_str(), position.x + 2.0 * vh, position.y + size.y * 0.65, font_size, WHITE);
}

/// - ability index:
///   - 1: primary
///   - 2: secondary
///   - 3: dash/movement
/// - squished: whether to slightly shrink the icon to show the ability was used
/// - progress: cooldown / charge, 0.0-1.0
pub fn draw_ability_icon(position: Vector2, size: Vector2, ability_index: usize, squished: bool, progress: f32, vh: f32, font: &Font) -> () {
  let icon: Texture2D = match ability_index {
    1 => {Texture2D::from_file_with_format(include_bytes!("../assets/ui/temp_ability_1.png"), None)},
    2 => {Texture2D::from_file_with_format(include_bytes!("../assets/ui/temp_ability_2.png"), None)},
    3 => {Texture2D::from_file_with_format(include_bytes!("../assets/ui/temp_ability_3.png"), None)},
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
  draw_rectangle(
    (position.x + squish_offset/2.0) * vh,
    (position.y + squish_offset/2.0) * vh,
    (size.x - squish_offset) * vh,
    ((size.y - squish_offset) * (1.0 - progress)) * vh,
    Color { r: 0.05, g: 0.0, b: 0.1, a: 0.4 },
  );
  let text = match ability_index {
    1 => " LMB ",
    2 => " RMB ",
    3 => "Space",
    _ => "Unkown",
  };
  draw_text_ex(text, (position.x + size.y * 0.125) * vh, (position.y + size.y * 1.3) * vh, TextParams { font: Some(font), font_size: (size.x * 0.3 * vh) as u16, ..Default::default() });
}

pub fn draw_player_info(position: Vector2, size: f32, player: ClientPlayer, font: &Font, vh: f32) -> () {
  let color = match player.team {
    Team::Red => RED,
    Team::Blue => BLUE,
  };
  draw_text_ex(player.username.as_str(), (position.x) * vh, (position.y) * vh, TextParams { font: Some(font), font_size: (size * 0.5 * vh) as u16, color: color, ..Default::default() });
  draw_rectangle(
    (position.x) * vh,
    (position.y + 1.5) * vh,
    (size * (100.0 as f32 / 100.0) * 2.0 ) * vh,
    (size * 0.25 ) * vh,
    Color { r: 0.0, g: 0.0, b: 0.0, a: 0.5 },
  );
  draw_rectangle(
    (position.x) * vh,
    (position.y + 1.5) * vh,
    (size * (player.health as f32 / 100.0) * 2.0 ) * vh,
    (size * 0.25 ) * vh,
    Color { r: 0.0, g: 1.0, b: 0.1, a: 1.0 },
  );
}

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
  draw_rectangle(0.0, 0.0, vw * 100.0, vh * 100.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.3 });
  if !*settings_open {
    // buttons
    let resume_button_position: Vector2 = Vector2 { x: vw * 50.0 - button_size.x/2.0, y: button_y_offset };
    let resume_button = button(resume_button_position, button_size, "Resume", button_font_size, vh);
    if resume_button {
      menu_paused = false;
      *settings_open = false;
    }
    
    
    let settings_button_position: Vector2 = Vector2 { x: vw * 50.0 - button_size.x/2.0, y: button_y_offset + button_y_separation };
    let settings_button = button(settings_button_position, button_size, "Options", button_font_size, vh);
    if settings_button {
      *settings_open = true;
    }

    // Quit button
    let quit_button_position: Vector2 = Vector2 { x: vw * 50.0 - button_size.x/2.0, y: button_y_offset + button_y_separation * 2.0 };
    let quit_button = button(quit_button_position, button_size, "Quit", button_font_size, vh);
    if quit_button {
      wants_to_quit = true;
      menu_paused = false;
      *settings_open = false;
    }

  }
  if *settings_open {
    let back_button = button(Vector2 { x: vw * 50.0 - button_size.x/2.0, y: 15.0*vh }, Vector2 { x: 25.0 * vh, y: 9.0 * vh }, "Back", button_font_size, vh);
    if back_button {
      *settings_open = false;
      settings.save();
    }
    settings.camera_smoothing = checkbox(Vector2 { x: vw * 40.0, y: vh * 30.0 }, 4.0 * vh, "Camera smoothing", 5.0*vh, vh, settings.camera_smoothing);
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
    draw_rectangle(position.x, position.y, size.x, size.y, BLUE);
    draw_rectangle(position.x + inner_shrink, position.y + inner_shrink, size.x - inner_shrink*2.0, size.y - inner_shrink*2.0, SKYBLUE);
    draw_text(self.text.as_str(), position.x + 2.0 * vh, position.y + size.y * 0.65, font_size, BLACK);
  }
  pub fn new(text: &str, duration: f32) -> Notification {
    return Notification { start_time: Instant::now(), text: String::from(text), duration }
  }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Settings {
  pub camera_smoothing: bool,
}
impl Settings {
  pub fn new() -> Settings{
    return Settings {
      camera_smoothing: true,
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