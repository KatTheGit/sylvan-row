/// Common functions and structs used by both client and server.
/// Utility functions too.

use macroquad::prelude::*;
use rusty_pkl::*;
use std::collections::HashMap;
use strum_macros::EnumIter;
use strum::IntoEnumIterator;

pub const TILE_SIZE: f32 = 10.0;

// this is bs
/// Any client sending packets faster than this will be ignored, as this could be a cheating attempt.
pub const MAX_PACKET_INTERVAL: f64 = 1.0 / 65.0;
/// A client sending packets slower than this will be ignored, as this could be a cheating attempt.
pub const MIN_PACKET_INTERVAL: f64 = 1.0 / 55.0;
pub const PACKET_INTERVAL_ERROR_MARGIN: f64 = 0.01;

/// how many packets are averaged when calculating legality of player position.
pub const PACKET_AVERAGE_SAMPLES: u8 = 5;

// TODO: later do this dynamically for client at least
pub const CLIENT_SEND_PORT:   u32 = 25566;
pub const CLIENT_LISTEN_PORT: u32 = 25567;
pub const SERVER_SEND_PORT:   u32 = 25568;
pub const SERVER_LISTEN_PORT: u32 = 25569;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, EnumIter, PartialEq, Eq, Hash)]
pub enum Character {
  SniperGirl,
  HealerGirl,
  ThrowerGuy,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct CharacterProperties {
  pub health: u8,

  pub speed: f32,

  pub primary_damage:     u8,
  pub primary_heal:       u8,
  pub primary_cooldown:   f32,
  pub primary_range:      f32,
  pub primary_shot_speed: f32,

  pub secondary_damage:         u8,
  pub secondary_heal:           u8,
  pub secondary_hit_charge:     u8,
  pub secondary_heal_charge:    u8,
  pub secondary_passive_charge: u8,
  pub secondary_range:          f32,
  pub primary_hit_radius:       f32,
}

pub fn load_characters() -> HashMap<Character, CharacterProperties> {
  let mut characters: HashMap<Character, CharacterProperties> = HashMap::new();
  for character in Character::iter() {
    let character_properties: CharacterProperties = match character {
      Character::SniperGirl => CharacterProperties::from_pkl(include_str!("../assets/characters/sniper_girl/properties.pkl")),
      Character::HealerGirl => CharacterProperties::from_pkl(include_str!("../assets/characters/healer_girl/properties.pkl")),
      Character::ThrowerGuy => CharacterProperties::from_pkl(include_str!("../assets/characters/sniper_girl/properties.pkl")),
    };

    characters.insert(character, character_properties);
  }
  return characters;
}
impl CharacterProperties {
  /// Create a character properties struct from a given pkl string.
  pub fn from_pkl(pkl_data: &str) -> CharacterProperties {
    let pkl: PklValue = parse_pkl_string(pkl_data).expect("could not parse pkl");
    return CharacterProperties {
      health:                   pkl_u8( find_parameter(&pkl, "health").unwrap()),
      speed:                    pkl_f32(find_parameter(&pkl, "speed").unwrap()),
      primary_damage:           pkl_u8( find_parameter(&pkl, "primary_damage").unwrap()),
      primary_heal:             pkl_u8( find_parameter(&pkl, "primary_heal").unwrap()),
      primary_cooldown:         pkl_f32(find_parameter(&pkl, "primary_cooldown").unwrap()),
      primary_range:            pkl_f32(find_parameter(&pkl, "primary_range").unwrap()),
      primary_shot_speed:       pkl_f32(find_parameter(&pkl, "primary_shot_speed").unwrap()),
      primary_hit_radius:       pkl_f32(find_parameter(&pkl, "primary_hit_radius").unwrap()),
      secondary_damage:         pkl_u8( find_parameter(&pkl, "secondary_damage").unwrap()),
      secondary_heal:           pkl_u8( find_parameter(&pkl, "secondary_heal").unwrap()),
      secondary_hit_charge:     pkl_u8( find_parameter(&pkl, "secondary_hit_charge").unwrap()),
      secondary_heal_charge:    pkl_u8( find_parameter(&pkl, "secondary_heal_charge").unwrap()),
      secondary_passive_charge: pkl_u8( find_parameter(&pkl, "secondary_passive_charge").unwrap()),
      secondary_range:          pkl_f32(find_parameter(&pkl, "secondary_range").unwrap()),
    }
  }
}

pub fn pkl_u8(pkl_value: PklValue) -> u8 {
  return match pkl_value {
    PklValue::Integer(value) => value as u8,
    _ => panic!("Pkl value parser could not parse that")
  }
}
pub fn pkl_f32(pkl_value: PklValue) -> f32 {
  return match pkl_value {
    PklValue::Float(value) => value as f32,
    _ => panic!("Pkl value parser could not parse that")
  }
}
pub fn parse_pkl_string(pkl_string: &str) -> Result<PklValue, String> {
  let content = pkl_string;
  let mut lines = content.lines();
  let root_object = parse_object(&mut lines)?;
  Ok(root_object)
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
  pub fn draw(&self, texture: &Texture2D, vh: f32, position: Vector2) {
    // TODO: animations
    let size: f32 = 12.0;
    draw_image_relative(&texture, self.position.x -(size/2.0), self.position.y - ((size/2.0)* (8.0/5.0)), size, size * (8.0/5.0), vh, position);
  }
  pub fn draw_crosshair(&self, vh: f32, center_position: Vector2, range: f32) {
    let relative_position_x = self.position.x - center_position.x + (50.0 * (16.0/9.0)); //+ ((vh * (16.0/9.0)) * 100.0 )/ 2.0;
    let relative_position_y = self.position.y - center_position.y + 50.0; //+ (vh * 100.0) / 2.0;
    draw_line(
      relative_position_x * vh, relative_position_y * vh,
      (self.aim_direction.normalize().x * range * vh) + (relative_position_x * vh),
      (self.aim_direction.normalize().y * range * vh) + (relative_position_y * vh),
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
  /// Raw movement vector
  pub movement:           Vector2,
  pub aim_direction:      Vector2,
  pub shooting_primary:   bool,
  pub shooting_secondary: bool,
  pub packet_interval:    f32,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq)]
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
/// Contains fields that can describe all necessary information for most game objects.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct GameObject {
  pub object_type: GameObjectType,
  pub position: Vector2,
  pub direction: Vector2,
  pub to_be_deleted: bool,
  /// pikmin
  pub owner_index: usize,
  pub hitpoints: u8,
  /// Object's left lifetime in seconds.
  pub lifetime: f32,
  // stuff for bullets for example
  /// buffer primarily used by bullets to keep track of hit players
  pub players: Vec<usize>,
  pub traveled_distance: f32,
}
/// enumerates all possible gameobjects. Their effects are then handled by the server.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, EnumIter, PartialEq, Hash, Eq)]
pub enum GameObjectType {
  Wall,
  UnbreakableWall,
  SniperGirlBullet,
  HealerGirlPunch,
  ThrowerGuyProjectile,
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
    if magnitude == 0.0 {
      return Vector2 {x: 0.0, y: 0.0};
    }
    return Vector2 {
      x: self.x / magnitude,
      y: self.y / magnitude,
    };
  }
  pub fn magnitude(&self) -> f32 {
    return Vector2::distance(Vector2 { x: 0.0, y: 0.0 }, *self);
  }
  pub fn as_vec2(&self) -> Vec2 {
    return Vec2 { x: self.x, y: self.y };
  }
  pub fn new() -> Vector2 {
    return Vector2 {x: 0.0, y: 0.0};
  }
  pub fn distance(vec1: Vector2, vec2: Vector2) -> f32 {
    return f32::sqrt(f32::powi(vec1.x - vec2.x, 2) + f32::powi(vec1.y - vec2.y, 2))
  }
  pub fn difference(vec1: Vector2, vec2: Vector2) -> Vector2 {
    return Vector2 {
      x: vec2.x - vec1.x,
      y: vec2.y - vec1.y,
    };
  }
  pub fn from(vec2: Vec2) -> Vector2 {
    return Vector2 { x: vec2.x, y: vec2.y };
  }
  pub fn clean(&mut self) {
    self.x.clean();
    self.y.clean();
  }
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
pub fn draw_image_relative(texture: &Texture2D, x: f32, y: f32, w: f32, h: f32, vh: f32, center_position: Vector2) -> () {

  // draw relative to position and centered.
  let relative_position_x = x - center_position.x + (50.0 * (16.0/9.0)); //+ ((vh * (16.0/9.0)) * 100.0 )/ 2.0;
  let relative_position_y = y - center_position.y + 50.0; //+ (vh * 100.0) / 2.0;

  draw_image(texture, relative_position_x, relative_position_y, w, h, vh);
}

pub fn load_map_from_file(map: &str) -> Vec<GameObject> {
  let mut map_to_return: Vec<GameObject> = Vec::new();
  for line in map.lines() {
    let opcodes: Vec<&str> = line.split(" ").collect();
    let gameobject_type = opcodes[0].to_lowercase();
    let gameobject_type = gameobject_type.as_str();
    let pos_x: f32 = opcodes[1].parse().unwrap();
    let pos_y: f32 = opcodes[2].parse().unwrap();

    map_to_return.push(GameObject {
      object_type: match gameobject_type {
        "wall"            => {GameObjectType::Wall},
        "unbreakablewall" => {GameObjectType::UnbreakableWall},
        _                 => {panic!("Unexpected ojbect in map file.")},
      },
      position: Vector2 { x: pos_x, y: pos_y },
      direction: Vector2::new(),
      to_be_deleted: false,
      owner_index: 200,
      hitpoints: 255,
      lifetime: f32::INFINITY,
      players: Vec::new(),
      traveled_distance: 0.0,
    });
  }
  return map_to_return;
}

/// Apply movement taking into account interactions with walls.
/// Returns the new raw movement vector and the new movement vector.
pub fn object_aware_movement(
  current_player_position: Vector2,
  raw_movement: Vector2,
  movement: Vector2,
  game_objects: Vec<GameObject>,
) -> (Vector2, Vector2) {

  let mut adjusted_raw_movement = raw_movement;
  let mut adjusted_movement = movement;
  let original_raw_movement_magnitude = raw_movement.magnitude();

  let mut desired_position = current_player_position;
  desired_position.x += movement.x;
  desired_position.y += movement.y;

  for game_object in game_objects.clone() {
    if game_object.object_type == GameObjectType::Wall            ||
       game_object.object_type == GameObjectType::UnbreakableWall {
      let difference = Vector2::difference(desired_position, game_object.position);

      // X axis collision prediction
      if f32::abs(difference.x) <= TILE_SIZE && f32::abs(current_player_position.y - game_object.position.y) < TILE_SIZE {
        adjusted_movement.x = 0.0;
      }

      // Y axis
      if f32::abs(difference.y) <= TILE_SIZE && f32::abs(current_player_position.x - game_object.position.x) < TILE_SIZE {
        adjusted_movement.y = 0.0;
      }

      adjusted_raw_movement = adjusted_movement.normalize();
      adjusted_raw_movement.x *= original_raw_movement_magnitude;
      adjusted_raw_movement.y *= original_raw_movement_magnitude;
    }
  }
  return (adjusted_raw_movement, adjusted_movement);
}

impl Extras for f32 {
  /// Same as signum but returns 0 if the number is 0.
  fn sign(&self) -> f32 {
    let mut sign: f32 = 0.0;

    if self > &0.0 {
      sign = 1.0;
    }

    if self < &0.0 {
      sign = -1.0;
    }

    return sign;
  }

  /// If the number is NaN or infinite, set it to 0.
  fn clean(&mut self) {
    if self.is_nan() || self.is_infinite() {
      *self = 0.0;
    }
  }
}

pub trait Extras {
  fn sign(&self) -> f32;
  fn clean(&mut self);
}