use bevy::{input::{keyboard::KeyboardInput, mouse::MouseWheel}, prelude::*, text::{FontSmoothing, LineBreak, TextBounds}};
use crate::maths::Vector2;

// MARK: Keys
pub fn get_keys_pressed(keys: &Res<ButtonInput<KeyCode>>) -> Vec<KeyCode> {
  let mut keys_to_return: Vec<KeyCode> = Vec::new();
  for key in keys.get_just_pressed() {
    keys_to_return.push(*key);
  }
  return keys_to_return;
}
pub fn get_keys_down(keys: &Res<ButtonInput<KeyCode>>) -> Vec<KeyCode> {
  let mut keys_to_return: Vec<KeyCode> = Vec::new();
  for key in keys.get_pressed() {
    keys_to_return.push(*key);
  }
  return keys_to_return;
}
pub fn get_keys_released(keys: &Res<ButtonInput<KeyCode>>) -> Vec<KeyCode> {
  let mut keys_to_return: Vec<KeyCode> = Vec::new();
  for key in keys.get_just_released() {
    keys_to_return.push(*key);
  }
  return keys_to_return;
}
pub fn get_char_keys(keys: &mut MessageReader<KeyboardInput>) -> Vec<String> {
  let mut chars = Vec::new();
  for key in keys.read() {
    match key.logical_key.clone() {
      bevy::input::keyboard::Key::Character(input) => {
        chars.push(input.as_str().to_string());
      }
      _ => {}
    }
  }
  return chars;
}

// MARK: Mouse
pub fn get_mouse_pressed(mouse: &Res<ButtonInput<MouseButton>>) -> Vec<MouseButton> {
  let mut buttons_to_return: Vec<MouseButton> = Vec::new();
  for button in mouse.get_just_pressed() {
    buttons_to_return.push(*button);
  }
  return buttons_to_return;
}
pub fn get_mouse_down(mouse: &Res<ButtonInput<MouseButton>>) -> Vec<MouseButton> {
  let mut buttons_to_return: Vec<MouseButton> = Vec::new();
  for button in mouse.get_pressed() {
    buttons_to_return.push(*button);
  }
  return buttons_to_return;
}
pub fn get_mouse_released(mouse: &Res<ButtonInput<MouseButton>>) -> Vec<MouseButton> {
  let mut buttons_to_return: Vec<MouseButton> = Vec::new();
  for button in mouse.get_just_released() {
    buttons_to_return.push(*button);
  }
  return buttons_to_return;
}
pub fn get_mouse_pos(window: &Window) -> Vector2 {
  if let Some(pos) = window.cursor_position() {
    return Vector2 {x: pos.x, y: pos.y};
  }
  return Vector2 { x: 0.0, y: 0.0 }
}
pub fn set_mouse_pos(position: Vec2, window: &mut Window) {
  window.set_cursor_position(Some(position));
}
pub fn get_mouse_wheel(mouse_wheel: &mut MessageReader<MouseWheel>) -> Vector2 {
  let mut totall_scroll = Vector2::new();
  for event in mouse_wheel.read() {
    totall_scroll += Vector2 {x: event.x, y: event.y};
  }
  return totall_scroll;
}
// MARK: Touch
pub fn touch_drags(touches: &Res<Touches>) -> Vec<(Vec2, Vec2)> {
  let mut drags: Vec<(Vec2, Vec2)> = Vec::new();
  for finger in touches.iter() {
    drags.push((finger.position(), finger.start_position()))
  }
  return drags;
}
pub fn touch_released(touches: &Res<Touches>) -> Vec<Vec2> {
  let mut released: Vec<Vec2> = Vec::new();
  for finger in touches.iter_just_released() {
    released.push(finger.position());
  }
  return released;
}
pub fn touch_pressed(touches: &Res<Touches>) -> Vec<Vec2> {
  let mut pressed: Vec<Vec2> = Vec::new();
  for finger in touches.iter_just_pressed() {
    pressed.push(finger.position());
  }
  return pressed;
}

// MARK: Window
pub fn set_fullscreen(fullscreen: bool, window: &mut Window) {
  if fullscreen {
    window.mode = bevy::window::WindowMode::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current);
  }
  else {
    window.mode = bevy::window::WindowMode::Windowed;
  }
}
pub fn set_fullscreen_borderless(fullscreen: bool, window: &mut Window) {
  if fullscreen {
    window.mode = bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Current);
  }
  else {
    window.mode = bevy::window::WindowMode::Windowed;
  }
}
pub fn is_window_focused(window: &Window) -> bool {
  return window.focused;
}
pub fn set_vsync(vsync: bool, window: &mut Window) {
  if vsync {
    window.present_mode = bevy::window::PresentMode::AutoVsync;
  }
  else {
    window.present_mode = bevy::window::PresentMode::AutoNoVsync;
  }
}

// MARK: Draw
pub fn draw_sprite(
  texture: &Texture,
  position: Vector2,
  size: Vector2,
  z: i8,
  window: &Window,
  commands: &mut Commands
) {
  commands.spawn(
    (
      DeleteAfterFrame {},
      Sprite {
        image: texture.image.clone(),
        ..Default::default()
      },
      Transform {
        translation: Vec3 {
          x: position.x - window.width() / 2.0 + size.x / 2.0,
          y: window.height() / 2.0 - position.y - size.y / 2.0,
          z: z as f32
        },
        scale: Vec3 {
          x: size.x / texture.size.x,
          y: size.y / texture.size.y,
          z: 1.0
        },
        ..Default::default()
      }
    )
  );
}
pub fn draw_rect(
  color: Color,
  position: Vector2,
  size: Vector2,
  z: i8,
  window: &Window,
  commands: &mut Commands
) {
  commands.spawn((
    DeleteAfterFrame {},
    Sprite {
      color,
      ..Default::default()
    },
    Transform {
      translation: Vec3 { x: position.x - window.width() / 2.0 + size.x / 2.0, y: window.height() / 2.0 - position.y - size.y / 2.0, z: z as f32 },
      scale: Vec3 { x: size.x, y: size.y, z: 1.0 },
      ..Default::default()
    }
  ));
}
pub fn draw_text(
  font: &Handle<Font>,
  text: &str,
  position: Vector2,
  size: Vector2,
  font_size: f32,
  z: i8,
  window: &Window,
  commands: &mut Commands,
) {
  let box_position = Vec2::new(position.x - window.width() / 2.0 + size.x / 2.0, -position.y - size.y / 2.0 + window.height() / 2.0);
  let slightly_smaller_text_font = TextFont {
    font: font.clone(),
    font_size: font_size,
    ..default()
  };

  commands.spawn((
    DeleteAfterFrame {},
    Visibility::Visible,
    Transform::from_translation(box_position.extend(z as f32)),
    children![(
      Text2d::new(text),
      slightly_smaller_text_font.clone()
        .with_font_smoothing(FontSmoothing::None),
      TextLayout::new(Justify::Left, LineBreak::WordBoundary),
      TextBounds::from(size.as_vec2()),
      Transform::from_translation(Vec3::Z),
    )],
  ));
}
pub fn draw_line(
  start: Vector2,
  end: Vector2,
  thickness: f32,
  color: Color,
  z: i8,
  window: &Window,
  commands: &mut Commands,
) {
  let difference = end - start;
  let magnitude = difference.magnitude();
  let angle = difference.y.atan2(difference.x);
  let center = (start + end) / 2.0;

  commands.spawn((
    DeleteAfterFrame {},
    Sprite {
      color,
      ..Default::default()
    },
    Transform {
      translation: Vec3 {
        x: center.x - window.width() / 2.0,
        y: window.height() / 2.0 - center.y,
        z: z as f32,
      },
      rotation: Quat::from_rotation_z(-angle),
      scale: Vec3 { x: magnitude, y: thickness, z: 1.0 },
    },
  ));
}

pub fn clear_background(
  color: Srgba,
  window: &Window,
  commands: &mut Commands,
) {
  commands.spawn((
    DeleteAfterFrame {},
    Sprite {
      color: Color::Srgba(color),
      ..Default::default()
    },
    Transform {
      translation: Vec3 { x: 0.0, y: 0.0, z: -129.0 },
      scale: Vec3 { x: window.width(), y: window.height(), z: -129.0 },
      ..Default::default()
    }
  ));
}

/// Component added to any entity that should be deleted
/// at the beginning of the next frame.
#[derive(Component, Debug)]
pub struct DeleteAfterFrame {}

// MARK: Texture
#[derive(Clone, Debug)]
pub struct Texture {
  pub image: Handle<Image>,
  pub size: Vec2,
}
impl Texture {
  /// Returns the width (x) divided by the height(y) of the image.
  fn aspect_ratio(&self) -> f32 {
    return self.size.x / self.size.y;
  }
}