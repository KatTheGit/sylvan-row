use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::Instant;
use crate::database::FriendShipStatus;
use crate::gamedata::Camera;
use crate::mothership_common::{ChatMessageType, ClientToServerPacket, ClientToServer};
use crate::{gamedata::*, GameData};
use crate::maths::Vector2;
use crate::bevy_immediate::*;
use bevy::color::palettes::css::*;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
#[cfg(not(target_os="android"))]
use device_query::{DeviceQuery, DeviceState, Keycode};
use keyring;
//use kira::track::TrackHandle;

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
  pub fn draw(&mut self, vh: f32, clickable: bool, z: i8, font: &Handle<Font>, window: &Window, commands: &mut Commands) {
    self.clickable = clickable;
    let position = self.position;
    let size = self.size;
    let text = self.text.as_str();
    let font_size = self.font_size;
    draw_rect(Color::Srgba(BLUE), position, size, z, window, commands);
    let inner_shrink: f32 = 1.0 * vh;
    draw_rect(Color::Srgba(SKY_BLUE), position + Vector2{x: inner_shrink, y: inner_shrink}, size - Vector2{x:  inner_shrink*2.0, y: inner_shrink*2.0}, z+1, window, commands);
    draw_text(&font, text, Vector2 {x: position.x + 1.0*vh, y: position.y}, size, BLACK, font_size, z+3, Justify::Left, window, commands);
    let mouse: Vector2 = get_mouse_pos(&window);
    if self.clickable {
      if mouse.x > position.x && mouse.x < (position.x + size.x) {
        if mouse.y > position.y && mouse.y < (position.y + size.y) {
          draw_rect(Color::Srgba(GRAY), position, size, z+2, window, commands);
          //draw_text(&font, text, Vector2 { x: position.x + 1.0*vh + 10.0, y: position.y + size.y * 0.65 }, size, font_size, z, window, commands);
        }
      }
    }
  }
  pub fn is_down(&self, window: &Window, mouse_keys: &Res<ButtonInput<MouseButton>>) -> bool {
    let mouse: Vector2 = get_mouse_pos(window);
    if mouse.x > self.position.x && mouse.x < (self.position.x + self.size.x) {
      if mouse.y > self.position.y && mouse.y < (self.position.y + self.size.y) {
        if get_mouse_down(mouse_keys).contains(&MouseButton::Left) {
          return true & self.clickable;
        }
      }
    }
    return false;
  }
  pub fn was_pressed(&self, window: &Window, mouse_keys: &Res<ButtonInput<MouseButton>>) -> bool {
    let mouse: Vector2 = get_mouse_pos(window);
    if mouse.x > self.position.x && mouse.x < (self.position.x + self.size.x) {
      if mouse.y > self.position.y && mouse.y < (self.position.y + self.size.y) {
        if get_mouse_pressed(mouse_keys).contains(&MouseButton::Left) {
          return true & self.clickable;
        }
      }
    }
    return false;
  }
  pub fn was_released(&self, window: &Window, mouse_keys: &Res<ButtonInput<MouseButton>>) -> bool {
    let mouse: Vector2 = get_mouse_pos(window);
    if mouse.x > self.position.x && mouse.x < (self.position.x + self.size.x) {
      if mouse.y > self.position.y && mouse.y < (self.position.y + self.size.y) {
        if get_mouse_released(mouse_keys).contains(&MouseButton::Left) {
          return true & self.clickable;
        }
      }
    }
    return false;
  }
}


// Represents a row of tabs
#[derive(Debug, Clone)]
pub struct Tabs {
  pub position: Vector2,
  pub size: Vector2,
  pub tab_names: Vec<String>,
  selected: Vec<bool>,
  font_size: f32,
}
impl Tabs {
  pub fn new(tab_names: Vec<String>) -> Tabs {
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
      position: Vector2::new(),
      size: Vector2::new(),
      tab_names: tab_names,
      selected,
      font_size: 10.0,
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
  pub fn draw_and_process(&mut self, vh: f32, clickable: bool, z: i8, font: &Handle<Font>, window: &Window, commands: &mut Commands, mouse_buttons: &Res<ButtonInput<MouseButton>>) {
    fn one_way_button(position: Vector2, size: Vector2, text: &str, font_size: f32, vh: f32, selected: bool, clickable: bool, z: i8, window: &Window, commands: &mut Commands, font: &Handle<Font>, mouse_buttons: &Res<ButtonInput<MouseButton>>) -> bool {
      draw_rect(Color::Srgba(BLUE), position, size, z, window, commands);
      let inner_shrink: f32 = 1.0 * vh;
      draw_rect(Color::Srgba(SKY_BLUE), position + Vector2{x: inner_shrink, y:inner_shrink}, size - Vector2{x: inner_shrink*2.0, y: inner_shrink*2.0}, z, window, commands);
      draw_text(&font, text, Vector2 {x: position.x + 1.0*vh, y: position.y}, size, BLACK, font_size, z+3, Justify::Left, window, commands);
      if selected {
        draw_rect(Color::Srgba(GRAY), position, size, z+1, window, commands);
      }
      let mouse: Vector2 = get_mouse_pos(window);
      if clickable {
        if mouse.x > position.x && mouse.x < (position.x + size.x) {
          if mouse.y > position.y && mouse.y < (position.y + size.y)   {
            draw_rect(Color::Srgba(GRAY), position, size, z+2, window, commands);
            //draw_text(&font, text, Vector2 {x: position.x + 1.0 *vh + 10.0, y: position.y}, size, font_size, z+2, window, commands);
            if get_mouse_released(mouse_buttons).contains(&MouseButton::Left) {
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
        one_way_button(
          Vector2 { x: self.position.x + i as f32 * button_width, y: self.position.y },
          Vector2 { x: button_width, y: self.size.y },
          &self.tab_names[i],
          self.font_size,
          vh,
          self.selected[i],
          clickable,
          z,
          window,
          commands,
          font,
          mouse_buttons,
        )
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
pub fn checkbox(position: Vector2, size: f32, text: &str, font_size: f32, vh: f32, selected: &mut bool, z: i8, font: &Handle<Font>, window: &Window, commands: &mut Commands, mouse_buttons: &Res<ButtonInput<MouseButton>>) -> bool {
  draw_rect(Color::Srgba(BLUE), position, Vector2 { x: size, y: size }, z, window, commands);
  let inner_shrink: f32 = 0.2 * vh;
  draw_rect(Color::Srgba(BLUE), position + Vector2{x: inner_shrink,y: inner_shrink}, Vector2 { x: size, y: size }- Vector2 { x: inner_shrink*2.0, y: inner_shrink*2.0}, z, window, commands);
  draw_text(&font, text, Vector2 {x: position.x + size + 1.0 *vh, y: position.y}, Vector2 { x: size + 300.0*vh, y: size }, BLACK, font_size , z, Justify::Left, window, commands);

  
  if *selected {
    draw_line(Vector2 {x: position.x, y: position.y + size/2.0}, Vector2{x: position.x + size/2.0, y: position.y + size}, 0.5*vh, WHITE, z, window, commands);
    draw_line(Vector2 {x: position.x + size/2.0, y: position.y + size}, Vector2{x: position.x + size, y: position.y}, 0.5*vh, WHITE, z, window, commands);
  }
  let mouse: Vector2 = get_mouse_pos(window);
  if mouse.x > position.x && mouse.x < (position.x + size) {
    if mouse.y > position.y && mouse.y < (position.y + size) {
      draw_rect(Color::Srgba(Srgba { red: 0.05, green: 0.0, blue: 0.1, alpha: 0.2 }), position, Vector2 { x: size, y: size }, z, window, commands);
      if get_mouse_released(mouse_buttons).contains(&MouseButton::Left) {
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
pub fn slider(position: Vector2, size: Vector2, text: &str, font_size: f32, vh: f32, value: &mut f32, value_min: f32, value_max: f32, font: &Handle<Font>, z: i8, window: &Window, commands: &mut Commands, mouse_buttons: &Res<ButtonInput<MouseButton>>) -> bool {
  let shrink = 1.0*vh;
  draw_rect(Color::Srgba(BLUE), position, size, z, window, commands);

  draw_rect(Color::Srgba(SKY_BLUE), position + Vector2 {x: shrink, y: shrink}, size - Vector2 {x: shrink*2.0, y:shrink*2.0}, z, window, commands);

  let slider_width = 2.0 * vh;
  let slider_x_pos = position.x + (size.x - slider_width) * ((*value-value_min) / (value_max - value_min));
  draw_rect(Color::Srgba(BLUE), Vector2 { x: slider_x_pos, y: position.y }, Vector2 { x: slider_width, y: size.y }, z+2, window, commands);
  let mut formatted_value: String = format!("{:.2}", value);
  if *value >= 1.0 {
    formatted_value = format!("{:.1}", value);
  }
  if *value >= 10.0 {
    formatted_value = format!("{:.0}", value);
  }
  draw_text(&font, format!("{}: {}", text, formatted_value).as_str(), Vector2 { x: position.x + 2.0*vh, y: position.y}, size, BLACK, font_size, z+1, Justify::Left, window, commands);
  let mouse: Vector2 = get_mouse_pos(window);
  let margin = size.x * 0.1;
  if mouse.x > (position.x - margin) && mouse.x < (position.x + size.x + margin) {
    if mouse.y > position.y && mouse.y < (position.y + size.y) {
      if get_mouse_down(mouse_buttons).contains(&MouseButton::Left) {
        *value = (mouse.x - position.x) / size.x * (value_max - value_min) + value_min;
        if *value > value_max {
          *value = value_max
        }
        if *value < value_min {
          *value = value_min
        }
        return true;
      }
    }
  }
  return false;
}

// MARK: In-game

/// - ability index:
///   - 1: primary
///   - 2: secondary
///   - 3: dash
///   - 4: passive
/// - squished: whether to slightly shrink the icon to show the ability was used
/// - progress: cooldown / charge, 0.0-1.0
pub fn draw_ability_icon(position: Vector2, size: Vector2, ability_index: usize, squished: bool, progress: f32, vh: f32, vw: f32, uiscale: f32, font: &Handle<Font>, character_descriptions: HashMap<Character, CharacterDescription>, character: Character, z: i8, texture: &Handle<Image>, window: &Window, commands: &mut Commands, settings: Settings) -> () {
  let squish_offset = match squished {
    true => 1.0,
    false => 0.0
  };
  draw_sprite(
    &Texture {
      image: texture.clone(),
      size: Vec2 {
        x: 400.0,
        y: 400.0,
      }
    },
    Vector2 {
      x: position.x + squish_offset/2.0,
      y: position.y + squish_offset/2.0,
    },
    Vector2 {
      x: size.x - squish_offset,
      y: size.y - squish_offset,
    },
    z, window, commands
  );
  draw_rect(Color::Srgba(Srgba { red: 0.05, green: 0.0, blue: 0.1, alpha: 0.4 }), Vector2{x: (position.x + squish_offset/2.0), y:(position.y + squish_offset/2.0)}, Vector2{x: (size.x - squish_offset), y: ((size.y - squish_offset) * (1.0 - progress))}, z+10, window, commands);
  let text = match ability_index {
    0 => "PASSIVE",
    1 => &format!("PRIMARY\n({})",
      if settings.keybinds.primary.2 != 255 {
        format!("MB{}", settings.keybinds.primary.2+1)
      } else {
        name_from_keycode_u16(settings.keybinds.primary.0)
      }
    ),
    2 => &format!("SECONDARY\n({})",
      if settings.keybinds.secondary.2 != 255 {
        format!("MB{}", settings.keybinds.secondary.2+1)
      } else {
        name_from_keycode_u16(settings.keybinds.secondary.0)
      }
    ),
    3 => &format!("DASH\n({})",
      if settings.keybinds.dash.2 != 255 {
        format!("MB{}", settings.keybinds.dash.2+1)
      } else {
        name_from_keycode_u16(settings.keybinds.dash.0)
      }
    ),
    _ => "Unkown",
  };
  draw_text(&font, text, Vector2 { x: (position.x), y: (position.y + size.y * 1.05)}, size, BLACK, size.x * 0.25, z, Justify::Left, window, commands);
  let ability = match ability_index {
    1 => character_descriptions[&character].primary.clone(),
    2 => character_descriptions[&character].secondary.clone(),
    3 => character_descriptions[&character].dash.clone(),
    4 => character_descriptions[&character].passive.clone(),
    _ => character_descriptions[&character].passive.clone(),
  };
  let text = ability.to_text();
  let mouse_pos = get_mouse_pos(window);
  //tooltip(position, size, &text, Vector2 { x: 55.0 * vh, y: 25.0 * vh }, vh, vw, font, mouse_pos, z+10, window, commands);
  ability_tooltip(ability_index, character, character_descriptions, position, size, uiscale, vh, vw, font, mouse_pos, z+10, settings, window, commands);
}

pub fn draw_player_info(position: Vector2, size: f32, player: ClientPlayer, font: &Handle<Font>, vh: f32, settings: Settings, z: i8, window: &Window, commands: &mut Commands) -> () {
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
  
  draw_text(&font, &displayed_name, Vector2 { x: position.x * vh, y: position.y * vh }, Vector2 { x: 100.0*vh, y: 100.0*vh }, BLACK, size * 0.5 * vh, z, Justify::Left, window, commands);
  draw_rect(Color::Srgba(Srgba {red: 0.0, green: 0.0, blue: 0.0, alpha: 0.5}), Vector2 {x: (position.x) * vh, y: (position.y + 1.5) * vh}, Vector2 {x: (size * (100.0 as f32 / 100.0) * 2.0 ) * vh, y: (size * 0.25 ) * vh}, z, window, commands);
  draw_rect(Color::Srgba(Srgba {red: 0.0, green: 1.0, blue: 0.1, alpha: 1.0}), Vector2 {x: (position.x) * vh, y: (position.y + 1.5) * vh}, Vector2{x: ( size * (player.health as f32 / 100.0) * 2.0 ) * vh, y: (size * 0.25 ) * vh}, z, window, commands);
}

// MARK: Esc Menu

/// Not actually a pause menu, but you get the point
/// 
/// This function is called both in-game and in the main menu
/// 
/// returns menu_paused and wants_to_quit
/// 
/// Track order:
/// - sfx self
/// - sfx other
/// - music
pub fn draw_pause_menu(uiscale: f32, vh: f32, vw: f32, data: &mut GameData/*, mut tracks: (&mut TrackHandle, &mut TrackHandle, &mut TrackHandle)*/, z: i8, font: &Handle<Font>, window: &mut Window, commands: &mut Commands, mouse_buttons: &Res<ButtonInput<MouseButton>>, keys: Res<ButtonInput<KeyCode>>) -> (bool, bool) {
  let mut menu_paused = true;
  let mut wants_to_quit = false;
  let button_y_separation: f32 = 15.0 * uiscale;
  let button_y_offset: f32 = 25.0 * uiscale;
  let button_font_size = 5.0 * uiscale;

  let button_size: Vector2 = Vector2 { x: 25.0 * uiscale, y: 9.0 * uiscale };
  // semi-transparent background
  draw_rect(Color::Srgba(Srgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 0.9 }), Vector2 {x:0.0, y: 0.0}, Vector2 {x: vw * 100.0, y: vh * 100.0}, z, window, commands);
  if !data.settings_open {
    // buttons
    let resume_button_position: Vector2 = Vector2 { x: vw * 50.0 - button_size.x/2.0, y: button_y_offset };
    let mut resume_button = Button::new(resume_button_position, button_size, "Resume", button_font_size);
    resume_button.draw(uiscale, true, z, font, window, commands);
    if resume_button.is_down(window, mouse_buttons) {
      menu_paused = false;
      data.settings_open = false;
    }
    
    
    let settings_button_position: Vector2 = Vector2 { x: vw * 50.0 - button_size.x/2.0, y: button_y_offset + button_y_separation };
    let mut settings_button = Button::new(settings_button_position, button_size, "Options", button_font_size);
    settings_button.draw(uiscale, true, z, font, window, commands);
    if settings_button.was_released(window, mouse_buttons) {
      data.settings_open = true;
      data.settings_timer = Instant::now();
    }

    // Quit button
    let quit_button_position: Vector2 = Vector2 { x: vw * 50.0 - button_size.x/2.0, y: button_y_offset + button_y_separation * 2.0 };
    let mut quit_button = Button::new(quit_button_position, button_size, "Quit", button_font_size);
    quit_button.draw(uiscale, true, z, font, window, commands);
    if quit_button.was_released(window, mouse_buttons) {
      wants_to_quit = true;
      menu_paused = false;
      data.settings_open = false;
    }

  }
  if data.settings_open {
    data.settings_tabs.update_size(Vector2 { x: 10.0*vw, y: 15.0*uiscale }, Vector2 { x: 80.0*vw, y: 6.0*uiscale }, 5.0*uiscale);
    data.settings_tabs.draw_and_process(uiscale, true, z, font, window, commands, mouse_buttons);
    let mut back_button = Button::new(Vector2 { x: vw * 50.0 - button_size.x/2.0, y: 3.0*uiscale }, Vector2 { x: 25.0 * uiscale, y: 9.0 * uiscale }, "Back", button_font_size);
    back_button.draw(uiscale, true, z, font, window, commands);
    if back_button.was_released(window, mouse_buttons) {
      data.settings_open = false;
    }

    let mut settings_modified: bool = false;

    // Gameplay
    if data.settings_tabs.selected_tab() == 0 {
      settings_modified |= checkbox(Vector2 { x: vw * 25.0, y: uiscale * 25.0 }, 4.0 * uiscale, "Camera smoothing", 4.0*uiscale, uiscale, &mut data.settings.camera_smoothing, z, font, window, commands, mouse_buttons);
      settings_modified |= checkbox(Vector2 { x: vw * 25.0, y: uiscale * 30.0 }, 4.0 * uiscale, "Display character names instead of usernames", 4.0*uiscale, uiscale, &mut data.settings.display_char_name_instead, z, font, window, commands, mouse_buttons);
    }
    // Video
    if data.settings_tabs.selected_tab() == 1 {
      let fullscreen_changed= checkbox(Vector2 { x: vw * 25.0, y: uiscale * 25.0 }, 4.0 * uiscale, "Fullscreen", 4.0*uiscale, uiscale, &mut data.settings.fullscreen, z, font, window, commands, mouse_buttons);
      if fullscreen_changed {
        set_fullscreen(data.settings.fullscreen, window);
      }
      settings_modified |= fullscreen_changed;
    }
    // Audio
    if data.settings_tabs.selected_tab() == 2 {
      settings_modified |= slider(Vector2 { x: vw * 25.0, y: uiscale * 25.0 }, Vector2 { x: 45.0*vw, y: 7.0*uiscale }, "Volume", 5.0*uiscale, uiscale, &mut data.settings.master_volume, 0.0, 100.0, font, z, window, commands, mouse_buttons);
      settings_modified |= slider(Vector2 { x: vw * 30.0, y: uiscale * 33.0 }, Vector2 { x: 40.0*vw, y: 7.0*uiscale }, "Music", 5.0*uiscale, uiscale, &mut data.settings.music_volume, 0.0, 100.0, font, z, window, commands, mouse_buttons);
      settings_modified |= slider(Vector2 { x: vw * 30.0, y: uiscale * 41.0 }, Vector2 { x: 40.0*vw, y: 7.0*uiscale }, "SFX (You)", 5.0*uiscale, uiscale, &mut data.settings.sfx_self_volume, 0.0, 100.0, font, z, window, commands, mouse_buttons);
      settings_modified |= slider(Vector2 { x: vw * 30.0, y: uiscale * 49.0 }, Vector2 { x: 40.0*vw, y: 7.0*uiscale }, "SFX (Others)", 5.0*uiscale, uiscale, &mut data.settings.sfx_other_volume, 0.0, 100.0, font, z, window, commands, mouse_buttons);
      if settings_modified {
        // update volumes.
        let sfx_self_volume = data.settings.master_volume * data.settings.sfx_self_volume / 100.0;
        let sfx_other_volume = data.settings.master_volume * data.settings.sfx_other_volume / 100.0;
        let music_volume = data.settings.master_volume * data.settings.music_volume / 100.0;
        //audio::set_volume(sfx_self_volume, &mut tracks.0);
        //audio::set_volume(sfx_other_volume, &mut tracks.1);
        //audio::set_volume(music_volume, &mut tracks.2);
      }
    }
    // Controls
    if data.settings_tabs.selected_tab() == 3 {
      let mut was_edited = false;
      #[cfg(not(target_os="android"))]
      {
        let base_y: f32 = 25.0 * uiscale;
        let size = 5.0 * uiscale;
        was_edited |= keybind_edit_buttons("Walk UP",    &mut data.settings.keybinds.walk_up,    Vector2 { x: 10.0 * vw, y: base_y + size * 0.0 }, size, uiscale, data.settings_timer.elapsed().as_secs_f32() > 0.2, font, z, window, commands, mouse_buttons);
        was_edited |= keybind_edit_buttons("Walk DOWN",  &mut data.settings.keybinds.walk_down,  Vector2 { x: 10.0 * vw, y: base_y + size * 1.0 }, size, uiscale, data.settings_timer.elapsed().as_secs_f32() > 0.2, font, z, window, commands, mouse_buttons);
        was_edited |= keybind_edit_buttons("Walk LEFT",  &mut data.settings.keybinds.walk_left,  Vector2 { x: 10.0 * vw, y: base_y + size * 2.0 }, size, uiscale, data.settings_timer.elapsed().as_secs_f32() > 0.2, font, z, window, commands, mouse_buttons);
        was_edited |= keybind_edit_buttons("Walk RIGHT", &mut data.settings.keybinds.walk_right, Vector2 { x: 10.0 * vw, y: base_y + size * 3.0 }, size, uiscale, data.settings_timer.elapsed().as_secs_f32() > 0.2, font, z, window, commands, mouse_buttons);
        was_edited |= keybind_edit_buttons("Primary",    &mut data.settings.keybinds.primary,    Vector2 { x: 10.0 * vw, y: base_y + size * 4.0 }, size, uiscale, data.settings_timer.elapsed().as_secs_f32() > 0.2, font, z, window, commands, mouse_buttons);
        was_edited |= keybind_edit_buttons("Secondary",  &mut data.settings.keybinds.secondary,  Vector2 { x: 10.0 * vw, y: base_y + size * 5.0 }, size, uiscale, data.settings_timer.elapsed().as_secs_f32() > 0.2, font, z, window, commands, mouse_buttons);
        was_edited |= keybind_edit_buttons("Dash",       &mut data.settings.keybinds.dash,       Vector2 { x: 10.0 * vw, y: base_y + size * 6.0 }, size, uiscale, data.settings_timer.elapsed().as_secs_f32() > 0.2, font, z, window, commands, mouse_buttons);
        was_edited |= keybind_edit_buttons("Fullscreen", &mut data.settings.keybinds.fullscreen, Vector2 { x: 10.0 * vw, y: base_y + size * 7.0 }, size, uiscale, data.settings_timer.elapsed().as_secs_f32() > 0.2, font, z, window, commands, mouse_buttons);
        was_edited |= keybind_edit_buttons("Open Chat",  &mut data.settings.keybinds.open_chat,  Vector2 { x: 10.0 * vw, y: base_y + size * 8.0 }, size, uiscale, data.settings_timer.elapsed().as_secs_f32() > 0.2, font, z, window, commands, mouse_buttons);
        was_edited |= keybind_edit_buttons("Cycle Friends (Chat)",  &mut data.settings.keybinds.cycle_friends,  Vector2 { x: 10.0 * vw, y: base_y + size * 9.0 }, size, uiscale, data.settings_timer.elapsed().as_secs_f32() > 0.2, font, z, window, commands, mouse_buttons);
      }
      
      if was_edited {
        data.settings_timer = Instant::now();
      }
      settings_modified |= was_edited;
    }
    // Other
    if data.settings_tabs.selected_tab() == 4 {
      let mut reset_button = Button::new(Vector2 { x: 40.0*vw, y: 30.0*uiscale }, Vector2 { x: 20.0*vw, y: 7.0*uiscale }, "Reset settings", 4.0*uiscale);
      reset_button.draw(uiscale, true, z, font, window, commands);
      if reset_button.was_released(window, mouse_buttons) {
        data.settings = Settings::new();
        settings_modified = true;
      }
      let mut reset_keybinds_button = Button::new(Vector2 { x: 40.0*vw, y: 40.0*uiscale }, Vector2 { x: 20.0*vw, y: 7.0*uiscale }, "Reset keybinds", 4.0*uiscale);
      reset_keybinds_button.draw(uiscale, true, z, font, window, commands);
      if reset_keybinds_button.was_released(window, mouse_buttons) {
        data.settings.keybinds = KeybindSettings::new();
        settings_modified = true;
      }
    }

    // if the settings were modified, save them.
    if settings_modified {
      data.settings.save();
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
#[derive(Debug)]
pub struct Notification {
  pub start_time: Instant,
  pub text: String,
  pub duration: f32,
}
impl Notification {
  pub fn draw(&self, vh: f32, tr_anchor: Vector2, font_size: f32, offset: usize, z: i8, font: &Handle<Font>, window: &Window, commands: &mut Commands) {
    let size: Vector2 = Vector2 { x: 60.0*vh, y: 20.0*vh };
    let position = tr_anchor + Vector2 {x: -size.x, y: offset as f32 * size.y};
    let inner_shrink: f32 = 1.0 * vh;
    draw_rect(Color::Srgba(BLUE), position, size, z, window, commands);
    draw_rect(Color::Srgba(SKY_BLUE), position + Vector2 {x: inner_shrink, y: inner_shrink}, size - Vector2 {x: inner_shrink*2.0, y: inner_shrink*2.0}, z, window, commands);
    draw_text(&font, self.text.as_str(), Vector2 {x: position.x + 2.0 * vh, y: position.y}, size, BLACK, font_size, z, Justify::Left, window, commands);
  }
  pub fn new(text: &str, duration: f32) -> Notification {
    return Notification { start_time: Instant::now(), text: String::from(text), duration }
  }
}

// MARK: Settings

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Resource)]
pub struct Settings {
  pub camera_smoothing: bool,
  /// If false, usernames are displayed.
  /// If true, character names are displayed.
  pub display_char_name_instead: bool,
  pub fullscreen: bool,
  pub saved_username: String,
  pub store_credentials: bool,
  pub master_volume: f32,
  pub music_volume: f32,
  pub sfx_self_volume: f32,
  pub sfx_other_volume: f32,
  pub keybinds: KeybindSettings,
}
impl Settings {
  pub fn new() -> Settings{
    return Settings {
      camera_smoothing: true,
      display_char_name_instead: true,
      fullscreen: false,
      saved_username: String::new(),
      store_credentials: false,
      master_volume: 80.0,
      music_volume: 100.0,
      sfx_self_volume: 100.0,
      sfx_other_volume: 50.0,
      keybinds: KeybindSettings::new(),
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
impl Default for Settings {
  fn default() -> Settings {
    return Settings::new();
  }
}
/// Holds keybinds for actions with the below tuple:
/// 
/// primary keybind, secondary keybind, mouse keybind, mouse keybind.
/// 
/// (keycode as u16, keycode as u16, mouse button index as u8, mouse button index as u8)
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct KeybindSettings {
  pub walk_up:       (u16, u16, u8, u8),
  pub walk_down:     (u16, u16, u8, u8),
  pub walk_left:     (u16, u16, u8, u8),
  pub walk_right:    (u16, u16, u8, u8),
  pub primary:       (u16, u16, u8, u8),
  pub secondary:     (u16, u16, u8, u8),
  pub dash:          (u16, u16, u8, u8),
  pub open_chat:     (u16, u16, u8, u8),
  pub fullscreen:    (u16, u16, u8, u8),
  pub cycle_friends: (u16, u16, u8, u8),
}
impl KeybindSettings {
  pub fn new() -> KeybindSettings {
    #[cfg(not(target_os="android"))]
    {
      return KeybindSettings {
        walk_up:       (device_query::Keycode::W as u16, u16::MAX, 255, 255),
        walk_down:     (device_query::Keycode::S as u16, u16::MAX, 255, 255),
        walk_left:     (device_query::Keycode::A as u16, u16::MAX, 255, 255),
        walk_right:    (device_query::Keycode::D as u16, u16::MAX, 255, 255),
        primary:       (u16::MAX, u16::MAX, 0, 255),
        secondary:     (u16::MAX, u16::MAX, 1, 255),
        dash:          (device_query::Keycode::Space as u16, u16::MAX, 255, 255),
        open_chat:     (device_query::Keycode::Enter as u16, u16::MAX, 255, 255),
        fullscreen:    (device_query::Keycode::F11 as u16, u16::MAX, 255, 255),
        cycle_friends: (device_query::Keycode::Tab as u16, u16::MAX, 255, 255),
      }
    }
    #[cfg(target_os="android")]
    {
      return KeybindSettings {
        walk_up:       (0, u16::MAX, 255, 255),
        walk_down:     (0, u16::MAX, 255, 255),
        walk_left:     (0, u16::MAX, 255, 255),
        walk_right:    (0, u16::MAX, 255, 255),
        primary:       (u16::MAX, u16::MAX, 0, 255),
        secondary:     (u16::MAX, u16::MAX, 1, 255),
        dash:          (0 as u16, u16::MAX, 255, 255),
        open_chat:     (0 as u16, u16::MAX, 255, 255),
        fullscreen:    (0 as u16, u16::MAX, 255, 255),
        cycle_friends: (0 as u16, u16::MAX, 255, 255),
      }
    }
  }
}
#[cfg(not(target_os="android"))]
pub fn keybind_edit_buttons(keybind_name: &str, keybind: &mut (u16, u16, u8, u8), position: Vector2, size: f32, vh: f32, clickable: bool, font: &Handle<Font>, z: i8, window: &Window, commands: &mut Commands, mouse_buttons: &Res<ButtonInput<MouseButton>>) -> bool {
  let mut font_size = size * 0.8;
  draw_text(&font, keybind_name, Vector2 { x: position.x, y: position.y}, Vector2 { x: size*10.0, y: size }, BLACK, font_size, z, Justify::Left, window, commands);
  for i in 0..2 {

    let keycode_name;
    if i == 0 {
      if keybind.0 == u16::MAX {
        if keybind.2 == 255 {
          keycode_name = "None".to_string();
        } else {
          if keybind.2 == u8::MAX-1 {
            keycode_name = "... (DEL to remove)".to_string();
            font_size = size * 0.7;
          } else {
            keycode_name = format!("MB{}", keybind.2+1);
          }
        }
      } else {
        if keybind.0 == u16::MAX-1 {
          keycode_name = "... (DEL to remove)".to_string();
          font_size = size * 0.7;
        } else {
          keycode_name = name_from_keycode_u16(keybind.0);
        }
      }
    } else {
      if keybind.1 == u16::MAX {
        if keybind.3 == 255 {
          keycode_name = "None".to_string();
        } else {
          if keybind.3 == u8::MAX-1 {
            keycode_name = "... (DEL to remove)".to_string();
            font_size = size * 0.7;
          } else {
            keycode_name = format!("MB{}", keybind.3+1);
          }
        }
      } else {
        if keybind.1 == u16::MAX-1 {
          keycode_name = "... (DEL to remove)".to_string();
          font_size = size * 0.7;
        } else {
          keycode_name = name_from_keycode_u16(keybind.1);
        }
      }
    }
      
    // set listening
    let x_size = 7.0*size;
    let x_offset = if i == 0 {50.0*vh} else {50.0*vh + x_size};
    let mut button = Button::new(position + Vector2 {x: x_offset, y: 0.0}, Vector2 { x: x_size, y: size }, &keycode_name, font_size);
    button.draw(vh, clickable, z, font, window, commands);
    if button.was_released(window, mouse_buttons) {
      if i == 0 {
        keybind.0 = u16::MAX-1;
        keybind.2 = u8::MAX -1;
      }
      else {
        keybind.1 = u16::MAX-1;
        keybind.3 = u8::MAX -1;
      }
    }

    // listening
    if i == 0 {
      if keybind.0 == u16::MAX-1 || keybind.2 == u8::MAX-1 {
        let device_state: DeviceState = DeviceState::new();
        let keys: Vec<device_query::Keycode> = device_state.get_keys();
        let mouse = get_mouse_pressed(&mouse_buttons);
        if !keys.is_empty() {
          if keys[0] == Keycode::Delete {
            keybind.0 = u16::MAX;
            keybind.2 = u8::MAX;
            return true;
          }
          keybind.0 = keys[0] as u16;
          keybind.2 = u8::MAX;
          return true;
        }
        else {
          for button in mouse {
            keybind.2 = mb_to_num(button);
            keybind.0 = u16::MAX;
            return true;
          }
        }
      }
    }
    else {
      if keybind.1 == u16::MAX-1 || keybind.3 == u8::MAX-1 {
        let device_state: DeviceState = DeviceState::new();
        let keys: Vec<device_query::Keycode> = device_state.get_keys();
        let mouse = get_mouse_pressed(&mouse_buttons);
        if !keys.is_empty() {
          if keys[0] == device_query::Keycode::Delete {
            keybind.1 = u16::MAX;
            keybind.3 = u8::MAX;
            return true;
          }
          keybind.1 = keys[0] as u16;
          keybind.3 = u8::MAX;
          return true;
        }
        else {
          for button in mouse {
            keybind.3 = mb_to_num(button);
            keybind.1 = u16::MAX;
            return true;
          }
        }
      }
    }

  }

  return false;
}
pub fn name_from_keycode_u16(keycode: u16) -> String {
  return match keycode {
    0 => "Key0".to_string(),
    1 => "Key1".to_string(),
    2 => "Key2".to_string(),
    3 => "Key3".to_string(),
    4 => "Key4".to_string(),
    5 => "Key5".to_string(),
    6 => "Key6".to_string(),
    7 => "Key7".to_string(),
    8 => "Key8".to_string(),
    9 => "Key9".to_string(),
    10 => "A".to_string(),
    11 => "B".to_string(),
    12 => "C".to_string(),
    13 => "D".to_string(),
    14 => "E".to_string(),
    15 => "F".to_string(),
    16 => "G".to_string(),
    17 => "H".to_string(),
    18 => "I".to_string(),
    19 => "J".to_string(),
    20 => "K".to_string(),
    21 => "L".to_string(),
    22 => "M".to_string(),
    23 => "N".to_string(),
    24 => "O".to_string(),
    25 => "P".to_string(),
    26 => "Q".to_string(),
    27 => "R".to_string(),
    28 => "S".to_string(),
    29 => "T".to_string(),
    30 => "U".to_string(),
    31 => "V".to_string(),
    32 => "W".to_string(),
    33 => "X".to_string(),
    34 => "Y".to_string(),
    35 => "Z".to_string(),
    36 => "F1".to_string(),
    37 => "F2".to_string(),
    38 => "F3".to_string(),
    39 => "F4".to_string(),
    40 => "F5".to_string(),
    41 => "F6".to_string(),
    42 => "F7".to_string(),
    43 => "F8".to_string(),
    44 => "F9".to_string(),
    45 => "F10".to_string(),
    46 => "F11".to_string(),
    47 => "F12".to_string(),
    48 => "F13".to_string(),
    49 => "F14".to_string(),
    50 => "F15".to_string(),
    51 => "F16".to_string(),
    52 => "F17".to_string(),
    53 => "F18".to_string(),
    54 => "F19".to_string(),
    55 => "F20".to_string(),
    56 => "Escape".to_string(),
    57 => "Space".to_string(),
    58 => "LControl".to_string(),
    59 => "RControl".to_string(),
    60 => "LShift".to_string(),
    61 => "RShift".to_string(),
    62 => "LAlt".to_string(),
    63 => "RAlt".to_string(),
    64 => "Command".to_string(),
    65 => "LOption".to_string(),
    66 => "ROption".to_string(),
    67 => "LMeta".to_string(),
    68 => "RMeta".to_string(),
    69 => "Enter".to_string(),
    70 => "Up".to_string(),
    71 => "Down".to_string(),
    72 => "Left".to_string(),
    73 => "Right".to_string(),
    74 => "Backspace".to_string(),
    75 => "CapsLock".to_string(),
    76 => "Tab".to_string(),
    77 => "Home".to_string(),
    78 => "End".to_string(),
    79 => "PageUp".to_string(),
    80 => "PageDown".to_string(),
    81 => "Insert".to_string(),
    82 => "Delete".to_string(),
    83 => "Numpad0".to_string(),
    84 => "Numpad1".to_string(),
    85 => "Numpad2".to_string(),
    86 => "Numpad3".to_string(),
    87 => "Numpad4".to_string(),
    88 => "Numpad5".to_string(),
    89 => "Numpad6".to_string(),
    90 => "Numpad7".to_string(),
    91 => "Numpad8".to_string(),
    92 => "Numpad9".to_string(),
    93 => "NumpadSubtract".to_string(),
    94 => "NumpadAdd".to_string(),
    95 => "NumpadDivide".to_string(),
    96 => "NumpadMultiply".to_string(),
    97 => "NumpadEquals".to_string(),
    98 => "NumpadEnter".to_string(),
    99 => "NumpadDecimal".to_string(),
    100 => "Grave".to_string(),
    101 => "Minus".to_string(),
    102 => "Equal".to_string(),
    103 => "LeftBracket".to_string(),
    104 => "RightBracket".to_string(),
    105 => "BackSlash".to_string(),
    106 => "Semicolon".to_string(),
    107 => "Apostrophe".to_string(),
    108 => "Comma".to_string(),
    109 => "Dot".to_string(),
    110 => "Slash".to_string(),
    _ => "???".to_string(),
  }
}
pub fn mb_to_num(mouse_button: MouseButton) -> u8 {
  match mouse_button {
    MouseButton::Left => {0}
    MouseButton::Right => {1}
    MouseButton::Middle => {2}
    MouseButton::Back => {3}
    MouseButton::Forward => {4}
    MouseButton::Other(i) => {5 + i as u8}
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
  chat_input:                &mut TextInput,
  vh:                        f32,
  username:                  String,
  packet_queue:              &mut Vec<ClientToServerPacket>,
  scroll_index:              &mut usize,
  is_in_game:                bool,
  timer:                     &mut Instant,
  settings:                  &Settings,
  window:                    &Window,
  z:                         i8,
  commands:                  &mut Commands,
  font:                      &Handle<Font>,
  mouse_wheel:               &mut MessageReader<MouseWheel>,
  key_events:                &mut MessageReader<KeyboardInput>,
  mouse_buttons:             &Res<ButtonInput<MouseButton>>,
  key_inputs:                &Res<ButtonInput<KeyCode>>,
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
  let mut cycle_friends = false;
  let mut open_chat = false;

  #[cfg(not(target_os="android"))]
  {
    let device_state: DeviceState = DeviceState::new();
    let keys: Vec<device_query::Keycode> = device_state.get_keys();
    
    // cycle through friends if TAB is pressed
    if online_friends.len() >= 2 {
      if is_window_focused(window) {
        for key in keys.clone() {
          if key as u16 == settings.keybinds.cycle_friends.0
          || key as u16 == settings.keybinds.cycle_friends.1 {
            cycle_friends = true;
          }
        }
      }
      if cycle_friends && timer.elapsed().as_secs_f32() > 0.4 {
        *timer = Instant::now();
        *selected_friend += 1;
        if *selected_friend >= online_friends.len() {
          *selected_friend = 0;
        }
      }
    } else {
      *selected_friend = 0;
    }
    
    // Open chat if ENTER is pressed
    if is_window_focused(window) {
      for key in keys {
        if key as u16 == settings.keybinds.cycle_friends.0
        || key as u16 == settings.keybinds.cycle_friends.1 {
          open_chat = true;
        }
      }
    }
    if open_chat && timer.elapsed().as_secs_f32() > 0.4 {
      *timer = Instant::now();
    } else {
      open_chat = false;
    }
  }
  if !*is_chatbox_open {
    *is_chatbox_open = true;
    chat_input.selected = true;
    return
  }
  // Close chat if ENTER is pressed and buffer is empty
  if *is_chatbox_open {
    if open_chat
    && (chat_input.buffer.is_empty() || !chat_input.selected) {
      *is_chatbox_open = false;
      chat_input.selected = false;
      return
    }
  }

  let text_input_box_size = Vector2 {x: size.x, y: 5.5 * vh};
  if *is_chatbox_open {
    // draw frame
    draw_rect(Color::Srgba(Srgba { red: 0.025, green: 0.0, blue: 0.05, alpha: 0.45 }), position, size, z, window, commands);
  }
  if timer.elapsed().as_secs_f32() < chat_remain_displayed_time && !*is_chatbox_open {
    draw_rect(Color::Srgba(Srgba { red: 0.025, green: 0.0, blue: 0.05, alpha: 0.2 }), Vector2 { x: position.x, y: position.y + size.y - 9.0 * vh }, Vector2 { x: size.x, y: -16.0 * vh }, z, window, commands);
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
      ChatMessageType::Team => SKY_BLUE,
      ChatMessageType::All => ORANGE,
    };
    draw_text(&font, &format!("[TAB] Messaging: {}", displayed_selected_friend), position, size, BLACK, 3.0 * vh, z, Justify::Left, window, commands);
    // draw input textbox
    chat_input.text_input(position + Vector2 {x: 0.0, y: size.y - text_input_box_size.y}, text_input_box_size, 3.0*vh, vh, font, z, commands, window, mouse_buttons, key_inputs.clone(), key_events);
    
    // Send message if ENTER is pressed and buffer is not empty
    // and a friend can be messaged and input field selected.
    if !chat_input.buffer.is_empty()
    && !online_friends.is_empty()
    && chat_input.selected
    && open_chat {

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
          information: ClientToServer::SendChatMessage(String::from(peer_username), chat_input.buffer.clone())
        }
      );

      // reset things
      recv_messages_buffer.push((username, chat_input.buffer.clone(), message_type));
      chat_input.buffer = String::new();
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
      //while measure_text(&formatted_message, TextParams::default().font, (3.0 * vh) as u16, 1.0).width > size.x - margin {
      //  let mut new_message = String::new();
      //  while measure_text(&new_message, TextParams::default().font, (3.0 * vh) as u16, 1.0).width < size.x - margin {
      //    new_message.insert(0, formatted_message.pop().expect("oopsies"));
      //  }
      //  formatted_messages.push((new_message, message.2.clone()));
      //}
      formatted_messages.push((formatted_message, message.2.clone()));
    }

    if *scroll_index > formatted_messages.len() {
      *scroll_index = formatted_messages.len() -1;
    }
    if !*is_chatbox_open {
      formatted_messages.truncate(5);
    }
    if formatted_messages.len() > 0 {
      let mut y_offset: f32 = 0.0;
      for m_index in *scroll_index..(*formatted_messages).len() {
        let pos_y = y_start - (y_offset - *scroll_index as f32) * 3.0 * vh;
        if pos_y < position.y {
          break;
        }
        let color = match formatted_messages[m_index].1 {
          ChatMessageType::Administrative => YELLOW,
          ChatMessageType::Private => PINK,
          ChatMessageType::Group => GREEN,
          ChatMessageType::Team => SKY_BLUE,
          ChatMessageType::All => ORANGE,
        };
        let current_y_size = (formatted_messages[m_index].0.len() / 14) as f32;
        y_offset += current_y_size;
        draw_text(&font, &formatted_messages[m_index].0, Vector2 { x: position.x, y: pos_y }, Vector2 { x: size.x, y: current_y_size }, BLACK, 3.0 * vh, z, Justify::Left, window, commands);
      }
    }
    let mouse_wheel = get_mouse_wheel(mouse_wheel);
    if mouse_wheel.y > 0.0
    && *scroll_index < formatted_messages.len() {
      *scroll_index += 1;
    }
    if mouse_wheel.y < 0.0
    && *scroll_index > 0 {
      *scroll_index -= 1;
    }
  }
}

#[derive(Clone, Debug)]
// MARK: Text input
pub struct TextInput {
  pub selected: bool,
  pub buffer: String,
  /// i.e. for passwords.
  pub hideable: bool,
  pub show_password: bool,
}
impl TextInput {

  /// A text input field.
  pub fn text_input(&mut self, position: Vector2, size: Vector2, font_size: f32, vh: f32, font: &Handle<Font>, z: i8, commands: &mut Commands, window: &Window, mouse_buttons: &Res<ButtonInput<MouseButton>>, keys: &Res<ButtonInput<KeyCode>>, key_events: &mut MessageReader<KeyboardInput>) {
    let margin: f32 = 2.0 * vh;
    let mouse = get_mouse_pos(window);
    
    if get_mouse_down(mouse_buttons).contains(&MouseButton::Left) {
      let is_inside =
      mouse.x > position.x && mouse.x < position.x + size.x &&
      mouse.y > position.y && mouse.y < position.y + size.y;
      self.selected = is_inside;
    }
    
    let bg = if self.selected { DARK_GRAY } else { GRAY };
    draw_rect(Color::Srgba(bg), position, size, z, window, commands);
    
    if self.hideable {
      checkbox(Vector2 { x: position.x + size.x + margin, y: position.y + size.y * 0.15 }, size.y * 0.7, "show", font_size, vh, &mut self.show_password, z, font, window, commands, mouse_buttons);
    }
    
    if self.selected {
      for key_event in key_events.read() {
        if key_event.state.is_pressed() {
          let key = key_event.text.clone();
          if let Some(chars) = key {
            let string = chars.as_str();
            for ch in string.chars() {
              if ch >= ' ' /* && ch <= '~' */ {
                self.buffer.push(ch);
              }
              // 8 = backspace
              if Some(ch) == char::from_u32(8) {
                self.buffer.pop();
              }
            }
          }
        }
      }
    }
    let mut text_to_draw = self.buffer.clone();
    if self.hideable && !self.show_password {
      let len = text_to_draw.len();
      text_to_draw = String::new();
      for _ in 0..len {
        text_to_draw.push('*');
      }
    }
    while text_to_draw.len() > 10 {
      text_to_draw.remove(0);
    }
    draw_text(&font, &text_to_draw, Vector2 {x: position.x + margin, y: position.y}, size, BLACK, font_size, z, Justify::Left, window, commands);
  }
}

// MARK: Tooltip
/// When the mouse hovers over the given rectangle with `position`
/// and `size`, it will display the given text.
pub fn tooltip(position: Vector2, size: Vector2, text: &str, tooltip_size: Vector2, vh: f32, vw: f32, font: &Handle<Font>, mouse_pos: Vector2, z: i8, window: &Window, commands: &mut Commands) {
  let font_size = 3.5 * vh;
  if mouse_pos.x < position.x + size.x
  && mouse_pos.x > position.x
  && mouse_pos.y < position.y + size.y
  && mouse_pos.y > position.y {
    //let lines: Vec<&str> = text.split("\n").collect();
    let y_offset = 4.0 * vh;

    let visibility_x_offset: f32 = if mouse_pos.x > 60.0 * vw {
      - tooltip_size.x - 10.0
    } else {
      10.0
    };
    let visibility_y_offset: f32 = if mouse_pos.y > 60.0 * vh {
      - tooltip_size.y - 10.0
    } else {
      10.0
    };
    draw_rect(Color::Srgba(BLUE), mouse_pos - Vector2 {x: 0.5*vh, y: 0.5*vh} + Vector2 {x: visibility_x_offset, y: visibility_y_offset}, Vector2 { x: tooltip_size.x + 1.0*vh, y: tooltip_size.y + 1.0*vh }, z, window, commands);
    draw_rect(Color::Srgba(SKY_BLUE), mouse_pos                              + Vector2 {x: visibility_x_offset, y: visibility_y_offset}, tooltip_size, z, window, commands);
    draw_text(&font, text, mouse_pos - Vector2 {x: 0.5*vh, y: 0.5*vh} + Vector2 {x: visibility_x_offset + 1.0 * vh, y: visibility_y_offset}, tooltip_size + Vector2 {x: -2.0*vh, y: 0.0}, BLACK, font_size, z+1, Justify::Left, window, commands);
  }
}

/// When the mouse hovers over the given rectangle with `position`
/// and `size`, it will display the given character ability info.
pub fn ability_tooltip(ability: usize, character: Character, character_descriptions: HashMap<Character, CharacterDescription>, position: Vector2, size: Vector2, uiscale: f32, vh: f32, vw: f32, font: &Handle<Font>, mouse_pos: Vector2, z: i8, settings: Settings, window: &Window, commands: &mut Commands) {
  let font_size = 3.5 * uiscale;
  if mouse_pos.x < position.x + size.x
  && mouse_pos.x > position.x
  && mouse_pos.y < position.y + size.y
  && mouse_pos.y > position.y {
    //let lines: Vec<&str> = text.split("\n").collect();
    let y_offset = 4.0 * uiscale;

    let tooltip_size = Vector2 { x: 65.0 * uiscale, y: 25.0 * uiscale };

    let visibility_x_offset: f32 = if mouse_pos.x > 60.0 * vw {
      - tooltip_size.x - 10.0
    } else {
      10.0
    };
    let visibility_y_offset: f32 = if mouse_pos.y > 60.0 * vh {
      - tooltip_size.y - 10.0
    } else {
      10.0
    };
    draw_rect(Color::Srgba(BLUE), mouse_pos - Vector2 {x: 0.5*uiscale, y: 0.5*uiscale} + Vector2 {x: visibility_x_offset, y: visibility_y_offset}, Vector2 { x: tooltip_size.x + 1.0*uiscale, y: tooltip_size.y + 1.0*uiscale }, z, window, commands);
    draw_rect(Color::Srgba(SKY_BLUE), mouse_pos                              + Vector2 {x: visibility_x_offset, y: visibility_y_offset}, tooltip_size, z, window, commands);
    let ability_description = match ability {
      1 => character_descriptions[&character].primary.clone(),
      2 => character_descriptions[&character].secondary.clone(),
      3 => character_descriptions[&character].dash.clone(),
      _ => character_descriptions[&character].passive.clone(),
    };
    let text = ability_description.to_text();
    draw_text(&font, &text, mouse_pos - Vector2 {x: 0.5*uiscale, y: 0.5*uiscale} + Vector2 {x: visibility_x_offset + 1.0 * uiscale, y: visibility_y_offset}, tooltip_size + Vector2 {x: -2.0*uiscale, y: 0.0}, BLACK, font_size, z+1, Justify::Left, window, commands);
    
    if ability_description.cooldown > 0.0 {
      let mut subtext = format!("CD: {}s", ability_description.cooldown);
      // secondary
      if ability == 2 {
        subtext = format!("Cost: {}", ability_description.cooldown);
      }

      draw_text(&font, &subtext, mouse_pos - Vector2 {x: 0.5*uiscale, y: 0.5*uiscale - tooltip_size.y + 4.0*uiscale} + Vector2 {x: visibility_x_offset + 1.0 * uiscale, y: visibility_y_offset}, tooltip_size + Vector2 {x: -2.0*uiscale, y: 0.0}, BLACK, font_size, z+1, Justify::Left, window, commands);
      
      if ability != 0 {
        
        let subtext = format!("{}", get_keybind_name(settings, ability));
        
        draw_text(&font, &subtext, mouse_pos - Vector2 {x: 0.5*uiscale, y: 0.5*uiscale - tooltip_size.y + 4.0*uiscale} + Vector2 {x: visibility_x_offset + 1.0 * uiscale, y: visibility_y_offset}, tooltip_size + Vector2 {x: -2.0*uiscale, y: 0.0}, BLACK, font_size, z+1, Justify::Right, window, commands);
      }
    
    }
  }
}

//MARK:  Credential store
const SERVICE_NAME: &str = "SYLVAN_ROW";
/// Tries to store password in keyring.
pub fn save_password(password: &str, username: &str, notifications: &mut Vec<Notification>) {
  let entry = match keyring::Entry::new(SERVICE_NAME, username) {
    Ok(entry) => entry,
    Err(err) => {
      notifications.push(
        Notification::new(&format!("Failed to access keyring. Reason: {:?}", err), 2.0)
      );
      println!("1 {:?}", err);
      return;
    }
  };
  match entry.set_password(password) {
    Ok(_) => {},
    Err(err) => {
      notifications.push(
        Notification::new(&format!("Failed to save to keyring. Reason: {:?}", err), 2.0)
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

/// This function does not include the necessary multiplication by VH.
pub fn world_to_screen(world_position: Vector2, camera: Camera, vh: f32, vw: f32) -> Vector2 {
  let screen_position = (world_position - camera.position) * camera.zoom + Vector2 {x: 50.0 * (vw/vh), y: 50.0};
  return screen_position;
}
pub fn screen_to_world(screen_position: Vector2, camera: Camera, vh: f32, vw: f32) -> Vector2 {
  // screen_position = (world_position - camera.position) * camera.zoom - Vector2 {x: 50.0 * vw, y: 50.0 * vh};
  // (screen_position + Vector2 {x: 50.0 * vw, y: 50.0 * vh})/camera.zoom + camera.position = world_position
  let world_position = (screen_position - Vector2 {x: 50.0 * (vw/vh), y: 50.0})/camera.zoom + camera.position;
  return world_position;
}


/// same as draw_image but draws relative to a ceratain position and centers it.
/// The x and y parameters are still world coordinates.
pub fn draw_image_relative(texture: &Texture, x: f32, y: f32, w: f32, h: f32, vh: f32, vw: f32, camera: Camera, z: i8, window: &Window, commands: &mut Commands) -> () {

  // draw relative to position and centered.
  let relative_position = world_to_screen(Vector2 { x: x, y: y }, camera.clone(), vh, vw);
  //let relative_position_x = (x - camera.position.x) * camera.zoom + (50.0 * (16.0/9.0));
  //let relative_position_y = (y - camera.position.y) * camera.zoom + (50.0);
  draw_sprite(texture, relative_position * vh, Vector2 { x: w * camera.zoom * vh, y: h * camera.zoom * vh }, z, window, commands);
}
/// same as draw_image_ex but draws relative to a ceratain position and centers it.
/// The x and y parameters are still world coordinates.
pub fn draw_image_relative_ex(texture: &Texture, x: f32, y: f32, w: f32, h: f32, rotation: Vector2, vh: f32, vw: f32, camera: Camera, z: i8, window: &Window, commands: &mut Commands) -> () {
  // draw relative to position and centered.
  let relative_position = world_to_screen(Vector2 { x: x, y: y }, camera.clone(), vh, vw);
  //let relative_position_x = (x - camera.position.x) * camera.zoom + (50.0 * (16.0/9.0));
  //let relative_position_y = (y - camera.position.y) * camera.zoom + (50.0);
  draw_sprite_ex(texture, relative_position * vh, rotation, Vector2 { x: w * camera.zoom * vh, y: h * camera.zoom * vh }, z, window, commands);
}
pub fn draw_line_relative(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Srgba, camera: Camera, vh:f32, vw: f32, z: i8, window: &Window, commands: &mut Commands) -> () {
  let relative_position_1 = world_to_screen(Vector2 { x: x1, y: y1 }, camera.clone(), vh, vw);
  let relative_position_2 = world_to_screen(Vector2 { x: x2, y: y2 }, camera.clone(), vh, vw);
  let relative_thickness = thickness * camera.zoom * vh;
  draw_line(relative_position_1 * vh, relative_position_2 * vh, relative_thickness, color, z, window, commands);
}
pub fn draw_rectangle_relative(x1: f32, y1: f32, w: f32, h: f32, color: Srgba, camera: Camera, vh:f32, vw: f32, z: i8, window: &Window, commands: &mut Commands) -> () {
  let relative_position = world_to_screen(Vector2 { x: x1, y: y1 }, camera.clone(), vh, vw);

  draw_rect(Color::Srgba(color), relative_position * vh, Vector2 { x: w*vh*camera.zoom, y: h*vh*camera.zoom }, z, window, commands);
}
pub fn draw_text_relative(text: &str, position: Vector2, size: Vector2, font: &Handle<Font>, color: Srgba, font_size: f32, vh: f32, vw: f32, camera: Camera, z: i8, alignment: Justify, window: &Window, commands: &mut Commands) -> () {
  let relative_position = world_to_screen(position, camera.clone(), vh, vw);
  let relative_size = size * camera.zoom * vh;
  draw_text(&font, text, relative_position * vh, relative_size, color, font_size * vh * camera.zoom, z, alignment, window, commands);
}
pub fn draw_lines(positions: Vec<Vector2>, camera: Camera, vh: f32, vw: f32, team: Team, y_offset: f32, alpha: f32, z: i8, window: &Window, commands: &mut Commands) -> () {
  if positions.len() < 2 { return; }
  for position_index in 0..positions.len()-1 {
    draw_line_relative(positions[position_index].x, positions[position_index].y + y_offset, positions[position_index+1].x, positions[position_index+1].y + y_offset, 0.1, match team {Team::Blue => Srgba { red: 0.2, green: 1.0-(position_index as f32 / positions.len() as f32), blue: 0.8, alpha: alpha }, Team::Red => Srgba { red: 0.8, green: 0.7-0.3*(position_index as f32 / positions.len() as f32), blue: 0.2, alpha: alpha }}, camera.clone(), vh, vw, z, window, commands);
  }
}


//#[macro_export]
//macro_rules! load {
//  ($file:expr $(,)?) => {
//    Texture2D::from_file_with_format(include_bytes!(concat!("../assets/", $file)), None)
//  };
//}
pub fn load_game_object_animations(asset_server: AssetServer) -> HashMap<GameObjectType, AnimationState>  {
  let game_object_animations: HashMap<GameObjectType, AnimationState> = HashMap::from([
    (GameObjectType::Wall,                             AnimationState::new(vec![asset_server.load("gameobjects/wall.png")],Vec2 {x: 200.0, y: 400.0},1.0, 0)),
    (GameObjectType::HernaniWall,                      AnimationState::new(vec![asset_server.load("characters/hernani/textures/wall.png")],Vec2 {x: 200.0, y: 400.0},1.0, 0)),
    (GameObjectType::RaphaelleAura,                    AnimationState::new(vec![asset_server.load("characters/raphaelle/textures/secondary.png")],Vec2 {x: 1000.0, y: 1000.0},1.0, 0)),
    (GameObjectType::UnbreakableWall,                  AnimationState::new(vec![asset_server.load("gameobjects/unbreakable_wall.png")],Vec2 {x: 200.0, y: 400.0},1.0, 0)),
    (GameObjectType::HernaniBullet,                    AnimationState::new(vec![asset_server.load("characters/hernani/textures/bullet.png")],Vec2 {x: 1000.0, y: 400.0},1.0, 0)),
    (GameObjectType::RaphaelleBullet,                  AnimationState::new(vec![asset_server.load("characters/raphaelle/textures/bullet.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::RaphaelleBulletEmpowered,         AnimationState::new(vec![asset_server.load("characters/raphaelle/textures/bullet-empowered.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::CynewynnSword,                    AnimationState::new(vec![asset_server.load("characters/cynewynn/textures/bullet.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::HernaniLandmine,                  AnimationState::new(vec![asset_server.load("characters/hernani/textures/trap.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::FedyaProjectileRicochet,          AnimationState::new(vec![asset_server.load("characters/hernani/textures/bullet.png")],Vec2 {x: 1000.0, y: 400.0},1.0, 0)),
    (GameObjectType::FedyaProjectileGround,            AnimationState::new(vec![asset_server.load("characters/hernani/textures/trap.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::FedyaProjectileGroundRecalled,    AnimationState::new(vec![asset_server.load("characters/hernani/textures/trap.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::FedyaTurret,                      AnimationState::new(vec![asset_server.load("ui/temp_ability_1.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::FedyaTurretProjectile,            AnimationState::new(vec![asset_server.load("characters/hernani/textures/bullet.png")],Vec2 {x: 1000.0, y: 400.0},1.0, 0)),
    (GameObjectType::Grass1,                           AnimationState::new(vec![asset_server.load("gameobjects/grass-1.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass2,                           AnimationState::new(vec![asset_server.load("gameobjects/grass-2.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass3,                           AnimationState::new(vec![asset_server.load("gameobjects/grass-3.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass4,                           AnimationState::new(vec![asset_server.load("gameobjects/grass-4.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass5,                           AnimationState::new(vec![asset_server.load("gameobjects/grass-5.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass6,                           AnimationState::new(vec![asset_server.load("gameobjects/grass-6.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass7,                           AnimationState::new(vec![asset_server.load("gameobjects/grass-7.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass1Bright,                     AnimationState::new(vec![asset_server.load("gameobjects/grass-1-b.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass2Bright,                     AnimationState::new(vec![asset_server.load("gameobjects/grass-2-b.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass3Bright,                     AnimationState::new(vec![asset_server.load("gameobjects/grass-3-b.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass4Bright,                     AnimationState::new(vec![asset_server.load("gameobjects/grass-4-b.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass5Bright,                     AnimationState::new(vec![asset_server.load("gameobjects/grass-5-b.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass6Bright,                     AnimationState::new(vec![asset_server.load("gameobjects/grass-6-b.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Grass7Bright,                     AnimationState::new(vec![asset_server.load("gameobjects/grass-7-b.png")],Vec2 {x: 200.0, y: 200.0},1.0, 0)),
    (GameObjectType::Water1,                           AnimationState::new(vec![asset_server.load("gameobjects/water-edge.png")],Vec2 {x: 200.0, y: 400.0},1.0, 0)),
    (GameObjectType::Water2,                           AnimationState::new(vec![asset_server.load("gameobjects/water-full.png")],Vec2 {x: 200.0, y: 400.0},1.0, 0)),
    (GameObjectType::CenterOrb,                        AnimationState::new(vec![asset_server.load("gameobjects/orb.png")],Vec2 {x: 800.0, y: 800.0},1.0, 0)),
    (GameObjectType::CenterOrbSpawnPoint,              AnimationState::new(vec![asset_server.load("empty.png")],Vec2 {x: 1.0, y: 1.0},1.0, 0)),
    (GameObjectType::WiroShield,                       AnimationState::new(vec![asset_server.load("ui/temp_ability_1.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::WiroGunShot,                      AnimationState::new(vec![asset_server.load("ui/temp_ability_1.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::WiroDashProjectile,               AnimationState::new(vec![asset_server.load("empty.png")],Vec2 {x: 1.0, y: 1.0},1.0, 0)),
    (GameObjectType::TemerityRocket,                   AnimationState::new(vec![asset_server.load("ui/temp_ability_1.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::TemerityRocketSecondary,          AnimationState::new(vec![asset_server.load("ui/temp_ability_1.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::KoldoCannonBall,                  AnimationState::new(vec![asset_server.load("characters/raphaelle/textures/bullet.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::KoldoCannonBallEmpowered,         AnimationState::new(vec![asset_server.load("characters/raphaelle/textures/bullet-empowered.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
    (GameObjectType::KoldoCannonBallEmpoweredUltimate, AnimationState::new(vec![asset_server.load("characters/raphaelle/textures/bullet-empowered.png")],Vec2 {x: 400.0, y: 400.0},1.0, 0)),
  ]);
  return game_object_animations;
}
pub fn load_character_textures(asset_server: AssetServer) -> HashMap<Character, Handle<Image>> {
  let player_textures = HashMap::from([
    (Character::Cynewynn,  asset_server.load("characters/cynewynn/textures/main.png")),
    (Character::Raphaelle, asset_server.load("characters/raphaelle/textures/main.png")),
    (Character::Hernani,   asset_server.load("characters/hernani/textures/main.png")),
    (Character::Fedya,     asset_server.load("characters/dummy/textures/template1.png")),
    (Character::Wiro,      asset_server.load("characters/dummy/textures/template2.png")),
    (Character::Dummy,     asset_server.load("characters/dummy/textures/template.png")),
    (Character::Temerity,  asset_server.load("characters/dummy/textures/template3.png")),
    (Character::Koldo,     asset_server.load("characters/koldo/textures/template4.png")),
  ]);
  return player_textures;
}

/// Describes an animation, its current state and how it behaves.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationState {
  pub frames: Vec<Texture>,
  pub frame_rate: f32,
  /// Animation start instant
  pub timer: Instant,
  /// Animation priority - comparison to know whether can be overwritten before its end.
  /// - 0: idle
  /// - 1: walk
  /// - 2: fire
  pub animation_prio: u8,
}
impl AnimationState {
  /// gets the current frame of the animation.
  pub fn current_frame(&self) -> Result<Texture, ()> {
    if self.frames.len() == 0 {
      return Err(());
    }
    let elapsed = self.timer.elapsed().as_secs_f32();
    let mut current_frame = (elapsed * self.frame_rate) as usize;
    let max_len = self.frames.len()-1;
    if current_frame > max_len {
      current_frame = max_len;
    }
    return Ok(self.frames[current_frame].clone());
  }
  pub fn is_finished(&self) -> bool {
    let elapsed = self.timer.elapsed().as_secs_f32();

    let current_frame = (elapsed * self.frame_rate) as usize;
    if self.frames.len() <= current_frame {
      return true
    }
    else {
      return false
    }
  }
  pub fn from_start(&self) -> AnimationState {

    return AnimationState {
      frames: self.frames.clone(),
      frame_rate: self.frame_rate,
      timer: Instant::now(),
      animation_prio: self.animation_prio,
    };
  }
  pub fn new(frames: Vec<Handle<Image>>, size: Vec2, frame_rate: f32, priority_level: u8) -> AnimationState {
    let mut texture_frames = Vec::new();
    for frame in frames {
      texture_frames.push(
        Texture { image: frame, size: size }
      )
    };
    return AnimationState {
      frames: texture_frames,
      frame_rate: frame_rate,
      timer: Instant::now(),
      animation_prio: priority_level,
    }
  }
}
pub fn load_character_animations(asset_server: AssetServer) -> HashMap<Character, Vec<AnimationState>> {
  return HashMap::from([
    (Character::Cynewynn, vec![
      AnimationState::new(vec![
        asset_server.load("characters/cynewynn/textures/main.png"),
      ], Vec2 { x: 800.0, y: 1200.0 }, 1.0, 0)
    ]),
    (Character::Raphaelle, vec![
      AnimationState::new(vec![
        asset_server.load("characters/raphaelle/textures/main.png"),
      ], Vec2 { x: 800.0, y: 1200.0 }, 1.0, 0)
    ]),
    (Character::Hernani, vec![
      AnimationState::new(vec![
        asset_server.load("characters/hernani/textures/main.png"),
      ], Vec2 { x: 800.0, y: 1200.0 }, 1.0, 0)
    ]),
    (Character::Fedya, vec![
      AnimationState::new(vec![
        asset_server.load("characters/dummy/textures/template1.png"),
      ], Vec2 { x: 800.0, y: 1200.0 }, 1.0, 0)
    ]),
    (Character::Wiro, vec![
      AnimationState::new(vec![
        asset_server.load("characters/dummy/textures/template2.png"),
      ], Vec2 { x: 800.0, y: 1200.0 }, 1.0, 0)
    ]),
    (Character::Dummy, vec![
      AnimationState::new(vec![
        asset_server.load("characters/dummy/textures/template.png"),
      ], Vec2 { x: 800.0, y: 1200.0 }, 1.0, 0)
    ]),
    (Character::Temerity, vec![
      AnimationState::new(vec![
        asset_server.load("characters/dummy/textures/template3.png"),
      ], Vec2 { x: 800.0, y: 1200.0 }, 1.0, 0)
    ]),
    (Character::Koldo, vec![
      AnimationState::new(vec![
        asset_server.load("characters/koldo/textures/template4.png"),
      ], Vec2 { x: 800.0, y: 1200.0 }, 1.0, 0)
    ]),
  ]);
}

fn get_keybind_name(settings: Settings, ability_index: usize) -> String {
  let text = match ability_index {
    0 => String::new(),
    1 => {
      if settings.keybinds.primary.2 != 255 {
        format!("MB{}", settings.keybinds.primary.2+1)
      } else {
        name_from_keycode_u16(settings.keybinds.primary.0).to_string()
      }
    }
    2 => {
      if settings.keybinds.secondary.2 != 255 {
        format!("MB{}", settings.keybinds.secondary.2+1)
      } else {
        name_from_keycode_u16(settings.keybinds.secondary.0)
      }
    }
    3 => {
      if settings.keybinds.dash.2 != 255 {
        format!("MB{}", settings.keybinds.dash.2+1)
      } else {
        name_from_keycode_u16(settings.keybinds.dash.0)
      }
    }
    _ => "Unkown".to_string(),
  };
  return text;
}