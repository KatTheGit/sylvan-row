/// Common functions and structs used by both client and server.
/// Utility functions too.

use macroquad::prelude::*;
use std::time::Instant;
use strum_macros::EnumIter;

/// Any client sending packets faster than this will be ignored.
pub const MAX_PACKET_INTERVAL: f64 = 1.0 / 1000000.0;
/// A client sending packets slower than this will be penalised, as this could be a cheating attempt.
pub const MIN_PACKET_INTERVAL: f64 = 1.0 / 20.0;
pub const PACKET_INTERVAL_ERROR_MARGIN: f64 = 0.01;

/// how many packets are averaged when calculating legality of player position.
pub const PACKET_AVERAGE_SAMPLES: u8 = 5;

// TODO: later do this dynamically for client at least
pub const CLIENT_SEND_PORT:   u32 = 25566;
pub const CLIENT_LISTEN_PORT: u32 = 25567;
pub const SERVER_SEND_PORT:   u32 = 25568;
pub const SERVER_LISTEN_PORT: u32 = 25569;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum Character {
  SniperGirl,
}

/// Information held by client about self and other players.
/// Sent by server to client as well.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ClientPlayer {
  pub health: u8,
  pub position: Vector2,
  pub aim_direction: Vector2,
  pub character: Character,
  pub secondary_charge: u8,
  pub movement_direction: Vector2,
  pub shooting_primary: bool,
  pub shooting_secondary: bool,
  pub team: Team,
}

impl ClientPlayer {
  pub fn draw(&self, texture: &Texture2D, vh: f32) {
    // TODO: animations
    draw_image(&texture, self.position.x -7.5, self.position.y - (7.5 * (8.0/5.0)), 15.0, 15.0 * (8.0/5.0), vh);
  }
  pub fn draw_crosshair(&self, vh: f32) {
    draw_line(
      self.position.x * vh, self.position.y * vh,
      (self.aim_direction.normalize().x * 50.0 * vh) + (self.position.x * vh),
      (self.aim_direction.normalize().y * 50.0 * vh) + (self.position.y * vh),
      2.0, Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }
    );
  }
  pub fn new() -> ClientPlayer {
    return ClientPlayer {
      health: 255,
      position: Vector2::new(),
      aim_direction: Vector2::new(),
      character: Character::SniperGirl,
      secondary_charge: 0,
      movement_direction: Vector2::new(),
      shooting_primary: false,
      shooting_secondary: false,
      team: Team::Blue,
    };
  }
}

/// information sent by client to server
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub struct ClientPacket {
  pub position:           Vector2,
  pub movement:           Vector2,
  pub aim_direction:      Vector2,
  pub shooting_primary:   bool,
  pub shooting_secondary: bool,
}

/// information held by server.
#[derive(Debug, Clone)]
pub struct ServerPlayer {
  pub ip:                            String,
  pub team:                          Team,
  pub health:                        u8,
  pub position:                      Vector2,
  pub shooting:                      bool,
  pub aim_direction:                 Vector2,
  pub move_direction:                Vector2,
  pub last_update_time:              Instant,
  pub secondary_charge:              u8,
  pub shooting_secondary:            bool,
  pub had_illegal_position:          bool,
  pub traveled_distance:             f32,
  pub packet_average_counter:        u8,
  pub position_before_checks:        Vector2,
  pub time_at_beginning_of_average:  Instant,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub enum Team {
  Red = 0,
  Blue = 1,
}

/// Information sent by srever to client about themself
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub struct ServerRecievingPlayerPacket {
  pub health: u8,
  pub override_position: bool,
  pub position_override: Vector2,
  pub shooting_primary: bool,
  pub shooting_secondary: bool,
}

/// information sent by server to client
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ServerPacket {
  pub player_packet_is_sent_to: ServerRecievingPlayerPacket,
  pub players:      Vec<ClientPlayer>,
  pub game_objects: Vec<GameObject>
}
/// defines any non-player gameplay element
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub struct GameObject {
  pub object_type: GameObjectType,
  pub position: Vector2,
}
/// enumerates all possible gameobjects. Their effects are then handled by the server.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, EnumIter, PartialEq, Hash, Eq)]
pub enum GameObjectType {
  Wall,
  UnbreakableWall,
  SniperGirlBullet,
}

// utility

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct Vector2 {
  pub x: f32,
  pub y: f32,
}
impl Vector2 {
  pub fn normalize(&self) -> Vector2 {
    let magnitude: f32 = self.magnitude();
    return Vector2 {
      x: self.x / magnitude,
      y: self.y / magnitude,
    };
  }
  pub fn magnitude(&self) -> f32 {
    return vector_distance(Vector2 { x: 0.0, y: 0.0 }, *self);
  }
  pub fn as_vec2(&self) -> Vec2 {
    return Vec2 { x: self.x, y: self.y };
  }
  pub fn new() -> Vector2 {
    return Vector2 {x: 0.0, y: 0.0};
  }
}

pub fn vector_distance(vec1: Vector2, vec2: Vector2) -> f32 {
  return f32::sqrt(f32::powi(vec1.x - vec2.x, 2) + f32::powi(vec1.y - vec2.y, 2))
}

pub fn vector_difference(vec1: Vector2, vec2: Vector2) -> Vector2 {
  return Vector2 {
    x: vec2.x - vec1.x,
    y: vec2.y - vec1.y,
  };
}

/// wrapper function for `draw_texture_ex`, simplifies it.
pub fn draw_image(texture: &Texture2D, x: f32, y: f32, w: f32, h: f32, vh: f32) -> () {
  draw_texture_ex(texture, x * vh, y * vh, WHITE, DrawTextureParams {
    dest_size: Some(Vec2 { x: w * vh, y: h * vh}),
    source: None,
    rotation: 0.0,
    flip_x: false,
    flip_y: false,
    pivot: Some(Vec2 { x: 0.0, y: 0.0 })
  });
}

/// same as draw_image but draws relative to a ceratain position and centers it.
/// The x and y parameters are still world coordinates.
pub fn draw_image_relative(texture: &Texture2D, x: f32, y: f32, w: f32, h: f32, vh: f32, position: Vector2) -> () {

  // draw relative to position and centered.
  let relative_position_x = position.x - x + w / 2.0;
  let relative_position_y = position.y - y + h / 2.0;

  draw_image(texture, relative_position_x, relative_position_y, w, h, vh);
}