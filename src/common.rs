/// Common functions and structs used by both client and server.
/// Utility functions too.

use macroquad::prelude::*;

pub static mut VH: f32 = 10.0;
pub static mut VW: f32 = 10.0;

/// Information held by client.
#[derive(Debug,Clone)]
pub struct ClientPlayer {
  pub health: u8,
  pub position: Vec2,
  pub aim_direction: Vec2,
  pub textures: Vec<Texture2D>,
  pub secondary_charge: u8,
}
impl ClientPlayer {
  pub fn draw(&self) {
    // TODO: animations
    draw_image(&self.textures[0], self.position.x -7.5, self.position.y - (7.5 * (8.0/5.0)), 15.0, 15.0 * (8.0/5.0));
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
  pub ip: String,
  pub team: Team,
  pub health: u8,
  pub position: Vector2,
  pub aim_direction: Vector2,
  pub shooting: bool,
  pub shooting_secondary: bool,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub enum Team {
  Red = 0,
  Blue = 1,
}

/// information sent by server to client
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub struct ServerPlayerPacket {
  health: u8,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ServerPacket {
  pub players:      Vec<ServerPlayerPacket>,
  pub game_objects: Vec<GameObject>

}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub struct GameObject {
  pub object_type: GameObjectType,
  pub position: Vector2,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy)]
pub enum GameObjectType {
  Wall,
  UnbreakableWall,
  SniperGirlBullet,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Vector2 {
  pub x: f32,
  pub y: f32,
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