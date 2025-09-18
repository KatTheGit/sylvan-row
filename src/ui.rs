use core::panic;

use macroquad::prelude::*;
use crate::common::*;

pub fn button(position: Vector2, size: Vector2, text: &str) -> bool {
  draw_rectangle(position.x, position.y, size.x, size.y, LIGHTGRAY);
  draw_text(text, position.x + 10.0, position.y + size.y / 2.0, 40.0, BLACK);
  let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
  if mouse.x > position.x && mouse.x < (position.x + size.x) {
    if mouse.y > position.y && mouse.y < (position.y + size.y) {
      draw_rectangle(position.x, position.y, size.x, size.y,GRAY);
      draw_text(text, position.x + 10.0, position.y + size.y / 2.0, 40.0, BLACK);
      if is_mouse_button_down(MouseButton::Left) {
        return true;
      }
    }
  }
  return false;
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
  draw_text_ex("Player", (position.x) * vh, (position.y) * vh, TextParams { font: Some(font), font_size: (size * 0.5 * vh) as u16, color: color, ..Default::default() });
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