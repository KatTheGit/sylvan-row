/// Common functions and structs used by both client and server.
/// Utility functions too.

use macroquad::prelude::*;

pub static mut VH: f32 = 10.0;
pub static mut VW: f32 = 10.0;

pub const MAX_PACKET_INTERVAL: f64 = 1.0 / 20.0; // 20Hz, T=0.05s
pub const PACKET_INTERVAL_ERROR_MARGIN: f64 = 0.01;

// TODO: do this dynamically for client at least
pub const CLIENT_SEND_PORT:   u32 = 25566;
pub const CLIENT_LISTEN_PORT: u32 = 25567;
pub const SERVER_SEND_PORT:   u32 = 25568;
pub const SERVER_LISTEN_PORT: u32 = 25569;

/// Information held by client.
#[derive(Debug,Clone)]
pub struct ClientPlayer {
  pub health: u8,
  pub position: Vec2,
  pub aim_direction: Vec2,
  pub character: Character,
  pub secondary_charge: u8,
}
impl ClientPlayer {
  pub fn draw(&self) {
    // TODO: animations
    let texture = Texture2D::from_file_with_format(include_bytes!("../assets/player/player1.png"), None); // temporary
    draw_image(&texture, self.position.x -7.5, self.position.y - (7.5 * (8.0/5.0)), 15.0, 15.0 * (8.0/5.0));
  }
  pub fn draw_crosshair(&self) {
    unsafe {
      draw_line(
        self.position.x * VH, self.position.y * VH,
        (self.aim_direction.normalize().x * 50.0 * VH) + (self.position.x * VH),
        (self.aim_direction.normalize().y * 50.0 * VH) + (self.position.y * VH),
        2.0, Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }
      );
    }
  }
}
#[derive(Debug,Clone)]
pub enum Character {
  SniperGirl,
}

/// information send by client to server
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub struct ClientPacket {
  pub position:           Vector2,
  pub aim_direction:      Vector2,
  pub shooting:           bool,
  pub shooting_secondary: bool,
}

/// information held by server
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ServerPlayer{
  pub ip:                     String,
  pub team:                   Team,
  pub health:                 u8,
  pub position:               Vector2,
  pub shooting:               bool,
  pub aim_direction:          Vector2,
  pub move_direction:         Vector2,
  pub secondary_charge:       u8,
  pub shooting_secondary:     bool,
  pub had_illegal_position:   bool,
  pub time_since_last_packet: f64,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub enum Team {
  Red = 0,
  Blue = 1,
}

/// Information sent by server to client about other players
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub struct ServerPlayerPacket {
  pub health: u8,
  pub position: Vector2,
  pub secondary_charge: u8,
  pub aim_direction: Vector2,
}

/// Information sent by srever to client about themself
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub struct ServerRecievingPlayerPacket {
  pub health: u8,
  pub override_position: bool,
  pub position_override: Vector2,
}

/// information sent by server to client
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ServerPacket {
  pub player_packet_is_sent_to: ServerRecievingPlayerPacket,
  pub players:      Vec<ServerPlayerPacket>,
  pub game_objects: Vec<GameObject>
}
/// defines any non-player gameplay element
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub struct GameObject {
  pub object_type: GameObjectType,
  pub position: Vector2,
}
/// enumerates all possible gameobjects. Their effects are then handled by the server.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub enum GameObjectType {
  Wall,
  UnbreakableWall,
  SniperGirlBullet,
}

// utility

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Vector2 {
  pub x: f32,
  pub y: f32,
}
impl Vector2 {
  pub fn normalize(mut self) {
    let magnitude: f32 = vector_distance(Vector2 { x: 0.0, y: 0.0 }, self);
    self.x = self.x / magnitude;
    self.y = self.y / magnitude;
  }
  pub fn magnitude(&self) -> f32 {
    return vector_distance(Vector2 { x: 0.0, y: 0.0 }, *self);
  }
  pub fn as_vec2(&self) -> Vec2 {
    return Vec2 { x: self.x, y: self.y };
  }
}

pub fn draw_image(texture: &Texture2D, x: f32, y: f32, w: f32, h: f32) {
  unsafe {
    draw_texture_ex(texture, x * VH, y * VH, WHITE, DrawTextureParams {
      dest_size: Some(Vec2 { x: w * VH, y: h * VH}),
      source: None,
      rotation: 0.0,
      flip_x: false,
      flip_y: false,
      pivot: Some(Vec2 { x: 0.0, y: 0.0 })
    });
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