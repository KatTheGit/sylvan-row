/// Common functions and structs used by both client and server.
/// Utility functions too.

use macroquad::prelude::*;
use rusty_pkl::*;
use std::collections::HashMap;
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use std::time::SystemTime;

pub const TILE_SIZE: f32 = 8.0;
pub const ORB_HEALING: u8 = 20;

pub const WALL_TYPES: [GameObjectType; 3] = [GameObjectType::Wall, GameObjectType::UnbreakableWall, GameObjectType::HernaniWall];
pub const WALL_TYPES_ALL: [GameObjectType; 5] = [GameObjectType::Wall, GameObjectType::UnbreakableWall, GameObjectType::HernaniWall, GameObjectType::Water1, GameObjectType::Water2];

/// Disable this for release builds.
pub const DEBUG: bool = false;

// this is bs
/// Any client sending packets faster than this will be ignored, as this could be a cheating attempt.
pub const MAX_PACKET_INTERVAL: f64 = 1.0 / 30.0;
/// A client sending packets slower than this will be ignored, as this could be a cheating attempt.
pub const MIN_PACKET_INTERVAL: f64 = 1.0 / 9.0;
pub const PACKET_INTERVAL_ERROR_MARGIN: f64 = 0.01;

/// how many packets are averaged when calculating legality of player position.
pub const PACKET_AVERAGE_SAMPLES: u8 = 5;
/// Port the server is hosted on. Used by server, and by the client to set the
/// default address of the server.
pub const SERVER_PORT:        u16 = 25569;
/// Default IP to be used when there's an issue with the moba_ip.txt file.
pub const DEFAULT_SERVER_IP: &str = "13.38.240.14"; // my AWS instance address

// MARK: Gamemodes

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Camera {
  pub position: Vector2,
}
impl Camera {
  pub fn new() -> Camera {
    return Camera { position: Vector2::new() };
  }
}

pub enum GameMode {
  /// Round-based fight
  Arena,
  /// Instant respawns, no rounds, chaos.
  DeathMatch,
}
/// This struct contains information related to the current match.
/// It is sent over network.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct GameModeInfo {
  /// Length of the game in seconds
  pub time: u16,
  /// How many rounds were won by the red team
  pub rounds_won_red: u8,
  /// How many rounds were won by the blue team
  pub rounds_won_blue: u8,
  /// Amount of players left on the red team
  pub alive_red: u8,
  /// Total amount of players on the team
  pub total_red: u8,
  /// Amount of players left on the blue team
  pub alive_blue: u8,
  /// Total amount of players on the team
  pub total_blue: u8,
  /// How long to wait until a respawn after death
  pub death_timeout: f32,
}
impl GameModeInfo {
  pub fn new() -> GameModeInfo {
    return GameModeInfo {
      time: 0,
      rounds_won_blue: 0,
      rounds_won_red: 0,
      total_red: 0,
      total_blue: 0,
      alive_red: 0,
      alive_blue: 0,
      death_timeout: 3.0,
    }
  }
}

// MARK: Characters
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, EnumIter, PartialEq, Eq, Hash)]
pub enum Character {
  /// Used for testing.
  Dummy,
  Hernani,
  Raphaelle,
  Cynewynn,
  Elizabeth,
  Wiro,
  Temerity,
}
/// Struct that contains the properties for each character. These are stored
/// in the respective characters' `properties.pkl` files. This data structure
/// can be as large as we want it to be, since we never send it over network.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct CharacterProperties {
  /// Maximum (default) health. Standard is 100, but can be more or less.
  pub health: u8,

  /// Movement speed in units per second
  pub speed: f32,

  pub primary_damage:     u8,
  /// Amount allies are healed when hit by this ability.
  pub primary_heal:       u8,
  /// Used for empowered/alternate attacks
  pub primary_damage_2: u8,
  /// Amount allies are healed when hit by this ability.
  /// Used for empowered/alternate attacks.
  pub primary_heal_2: u8,
  /// Amount of healing after a hit. Currently only used by the bunny, who sends the health to allies.
  pub primary_lifesteal:  u8,
  pub primary_cooldown:   f32,
  pub primary_cooldown_2: f32,
  pub primary_range:      f32,
  pub primary_range_2:    f32,
  pub primary_range_3:    f32,
  pub primary_shot_speed: f32,
  pub primary_shot_speed_2: f32,
  /// Girth of the bullet
  pub primary_hit_radius: f32,
  pub primary_wall_hit_radius: f32,
  /// This value is applied to small integers. Be wary of rounding. Only applies to primary attack, in theory.
  pub wall_damage_multiplier: f32,


  pub secondary_damage:         u8,
  pub secondary_cooldown:       f32,
  pub secondary_heal:           u8,
  pub secondary_hit_charge:     u8,
  pub secondary_heal_charge:    u8,
  pub secondary_passive_charge: u8,
  pub secondary_range:          f32,
  pub secondary_charge_use:     u8,

  pub dash_distance:                f32,
  pub dash_cooldown:                f32,
  pub dash_damage_multiplier:       f32,
  pub dash_speed:                   f32,

  pub passive_range:                f32,
}

pub fn load_characters() -> HashMap<Character, CharacterProperties> {
  let mut characters: HashMap<Character, CharacterProperties> = HashMap::new();
  for character in Character::iter() {
    let character_properties: CharacterProperties = match character {
      Character::Dummy      => CharacterProperties::from_pkl(include_str!("../assets/characters/dummy/properties.pkl")),
      Character::Hernani => CharacterProperties::from_pkl(include_str!("../assets/characters/hernani/properties.pkl")),
      Character::Raphaelle => CharacterProperties::from_pkl(include_str!("../assets/characters/raphaelle/properties.pkl")),
      Character::Cynewynn =>  CharacterProperties::from_pkl(include_str!("../assets/characters/cynewynn/properties.pkl")),
      Character::Elizabeth =>  CharacterProperties::from_pkl(include_str!("../assets/characters/elizabeth/properties.pkl")),
      Character::Wiro =>  CharacterProperties::from_pkl(include_str!("../assets/characters/wiro/properties.pkl")),
      Character::Temerity =>  CharacterProperties::from_pkl(include_str!("../assets/characters/temerity/properties.pkl")),
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
      health:                   pkl_u8( find_parameter(&pkl, "health"                   ).unwrap()),
      speed:                    pkl_f32(find_parameter(&pkl, "speed"                    ).unwrap())*TILE_SIZE,
      primary_damage:           pkl_u8( find_parameter(&pkl, "primary_damage"           ).unwrap()),
      primary_damage_2:         pkl_u8( find_parameter(&pkl, "primary_damage_2"         ).unwrap()),
      primary_heal:             pkl_u8( find_parameter(&pkl, "primary_heal"             ).unwrap()),
      primary_heal_2:           pkl_u8( find_parameter(&pkl, "primary_heal_2"           ).unwrap()),
      primary_lifesteal:        pkl_u8( find_parameter(&pkl, "primary_lifesteal"        ).unwrap()),
      primary_cooldown:         pkl_f32(find_parameter(&pkl, "primary_cooldown"         ).unwrap()),
      primary_cooldown_2:       pkl_f32(find_parameter(&pkl, "primary_cooldown_2"       ).unwrap()),
      primary_range:            pkl_f32(find_parameter(&pkl, "primary_range"            ).unwrap())*TILE_SIZE,
      primary_range_2:          pkl_f32(find_parameter(&pkl, "primary_range_2"          ).unwrap())*TILE_SIZE,
      primary_range_3:          pkl_f32(find_parameter(&pkl, "primary_range_3"          ).unwrap())*TILE_SIZE,
      primary_shot_speed:       pkl_f32(find_parameter(&pkl, "primary_shot_speed"       ).unwrap())*TILE_SIZE,
      primary_shot_speed_2:     pkl_f32(find_parameter(&pkl, "primary_shot_speed_2"       ).unwrap())*TILE_SIZE,
      primary_hit_radius:       pkl_f32(find_parameter(&pkl, "primary_hit_radius"       ).unwrap())*TILE_SIZE,
      primary_wall_hit_radius:  pkl_f32(find_parameter(&pkl, "primary_wall_hit_radius"  ).unwrap())*TILE_SIZE,
      wall_damage_multiplier:   pkl_f32(find_parameter(&pkl, "wall_damage_multiplier"   ).unwrap()),
      secondary_damage:         pkl_u8( find_parameter(&pkl, "secondary_damage"         ).unwrap()),
      secondary_heal:           pkl_u8( find_parameter(&pkl, "secondary_heal"           ).unwrap()),
      secondary_hit_charge:     pkl_u8( find_parameter(&pkl, "secondary_hit_charge"     ).unwrap()),
      secondary_heal_charge:    pkl_u8( find_parameter(&pkl, "secondary_heal_charge"    ).unwrap()),
      secondary_passive_charge: pkl_u8( find_parameter(&pkl, "secondary_passive_charge" ).unwrap()),
      secondary_cooldown:       pkl_f32(find_parameter(&pkl, "secondary_cooldown"       ).unwrap()),
      secondary_range:          pkl_f32(find_parameter(&pkl, "secondary_range"          ).unwrap())*TILE_SIZE,
      secondary_charge_use:     pkl_u8( find_parameter(&pkl, "secondary_charge_use"     ).unwrap()),
      dash_distance:            pkl_f32(find_parameter(&pkl, "dash_distance"            ).unwrap())*TILE_SIZE,
      dash_cooldown:            pkl_f32(find_parameter(&pkl, "dash_cooldown"            ).unwrap()),
      dash_damage_multiplier:   pkl_f32(find_parameter(&pkl, "dash_damage_multiplier"   ).unwrap()),
      dash_speed:               pkl_f32(find_parameter(&pkl, "dash_speed"               ).unwrap())*TILE_SIZE,
      passive_range:            pkl_f32(find_parameter(&pkl, "passive_range"            ).unwrap())*TILE_SIZE,
    }
  }
}

pub fn pkl_u8(pkl_value: PklValue) -> u8 {
  return match pkl_value {
    PklValue::Integer(value) => value as u8,
    _ => panic!("Pkl value parser could not parse that {:?}", pkl_value)
  }
}
pub fn pkl_f32(pkl_value: PklValue) -> f32 {
  return match pkl_value {
    PklValue::Float(value) => value as f32,
    _ => panic!("Pkl value parser could not parse that {:?}", pkl_value)
  }
}
pub fn parse_pkl_string(pkl_string: &str) -> Result<PklValue, String> {
  let content = pkl_string;
  let mut lines = content.lines();
  let root_object = parse_object(&mut lines)?;
  Ok(root_object)
}

// MARK: Client

/// Information held by client about self and other players.
#[derive(Debug, Clone)]
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
  pub time_since_last_dash: f32,
  pub is_dead: bool,
  pub camera: Camera,
  pub buffs: Vec<Buff>,
  pub previous_positions: Vec<Vector2>,
  pub ping: u16,
  pub last_shot_time: f32,
  pub last_secondary_time: f32,
  /// wants to dash
  pub dashing: bool,
  /// is currently dashing
  pub is_dashing: bool,
  pub dashed_distance: f32,
}
/// Information sent by server to client about other players.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct OtherPlayer {
  pub health: u8,
  pub position: Vector2,
  pub aim_direction: Vector2,
  pub character: Character,
  pub secondary_charge: u8,
  pub movement_direction: Vector2,
  pub shooting_primary: bool,
  pub shooting_secondary: bool,
  pub team: Team,
  pub time_since_last_dash: f32,
  pub is_dead: bool,
  pub camera: Camera,
  pub buffs: Vec<Buff>,
  pub previous_positions: Vec<Vector2>,
}
impl ClientPlayer {
  pub fn from_otherplayer(other_player: OtherPlayer) -> ClientPlayer {
    return ClientPlayer { health: other_player.health,
      position: other_player.position,
      aim_direction: other_player.aim_direction,
      character: other_player.character,
      secondary_charge: other_player.secondary_charge,
      movement_direction: other_player.movement_direction,
      shooting_primary: other_player.shooting_primary,
      shooting_secondary: other_player.shooting_secondary,
      team: other_player.team,
      time_since_last_dash: other_player.time_since_last_dash,
      is_dead: other_player.is_dead,
      camera: other_player.camera,
      buffs: other_player.buffs,
      previous_positions: other_player.previous_positions,
      ping: 0,
      last_shot_time: 0.0,
      last_secondary_time: 0.0,
      dashing: false,
      is_dashing: false,
      dashed_distance: 0.0,
    }
  }
  pub fn draw(&self, texture: &Texture2D, vh: f32, camera_position: Vector2, font: &Font, character: CharacterProperties) {
    // TODO: animations
    let bg_offset: Vector2 = Vector2 { x: -12.0, y: -16.5 };
    let bg_size: Vector2 = Vector2 {x: bg_offset.x*-2.0, y: 7.0};
    let bg_opacity: f32 = 0.4;
    let color = match self.team {
      Team::Blue => Color { r: 0.3, g: 0.5, b: 0.7, a: bg_opacity },
      Team::Red => Color { r: 0.7, g: 0.5, b: 0.3, a: bg_opacity },
    };
    draw_rectangle_relative(bg_offset.x + self.position.x, bg_offset.y + self.position.y, bg_size.x, bg_size.y, color, camera_position, vh);

    let size: f32 = 10.0;
    draw_image_relative(&texture, self.position.x -(size/2.0), self.position.y - ((size/2.0)* (8.0/5.0)), size, size * (8.0/5.0), vh, camera_position, Vector2::new(), WHITE);
    let health_bar_offset: Vector2 = Vector2 { x: -5.0, y: -11.0 };
    let secondary_bar_offset: Vector2 = Vector2 { x: -5.0, y: -13.0 };
    let dash_bar_offset: Vector2 = Vector2 { x: -5.0, y: -15.0 };
    let mut dash_ratio = self.time_since_last_dash / character.dash_cooldown;
    if dash_ratio > 1.0 {dash_ratio = 1.0};
    draw_line_relative(
      self.position.x + dash_bar_offset.x,
      self.position.y + dash_bar_offset.y,
      dash_ratio * 10.0 + self.position.x + dash_bar_offset.x,
      self.position.y + dash_bar_offset.y,
      1.5,
      BLUE,
      camera_position, vh);
    draw_line_relative(
      self.position.x + secondary_bar_offset.x,
      self.position.y + secondary_bar_offset.y,
      self.secondary_charge as f32 / 10.0 + self.position.x + secondary_bar_offset.x,
      self.position.y + secondary_bar_offset.y,
      1.5,
      ORANGE,
      camera_position, vh);
    // let health_counter_offset: Vector2 = Vector2 { x: -3.9, y: -10.0 };
    // let health_counter_with_leading_zeros = format!("{:0>3}", self.health.to_string());
    // let mut font = load_ttf_font_from_bytes(include_bytes!("./../assets/fonts/Action_Man.ttf")).expect("Could not load font.");
    // font.set_filter(FilterMode::Nearest);
    // draw_text_relative(health_counter_with_leading_zeros.as_str(), self.position.x + health_counter_offset.x, self.position.y + health_counter_offset.y, &font, 16, vh, camera_position, GREEN);
    draw_line_relative(
      self.position.x + health_bar_offset.x,
      self.position.y + health_bar_offset.y,
      self.health as f32 / 10.0 + self.position.x + health_bar_offset.x,
      self.position.y + health_bar_offset.y,
      1.5,
      GREEN,
      camera_position, vh);
    let health_counter_offset: Vector2 = Vector2 { x: -11.5, y: -10.6 };
    let health_counter_with_leading_zeros = format!("{:0>3}", self.health.to_string());
    let font_size: u16 = 4;
    draw_text_relative(health_counter_with_leading_zeros.as_str(), self.position.x + health_counter_offset.x, self.position.y + health_counter_offset.y, &font, font_size, vh, camera_position, GREEN);
    let secondary_counter_offset: Vector2 = Vector2 { x: 5.9, y: -10.6 };
    let secondary_counter_with_leading_zeros = format!("{:0>3}", self.secondary_charge.to_string());
    draw_text_relative(secondary_counter_with_leading_zeros.as_str(), self.position.x + secondary_counter_offset.x, self.position.y + secondary_counter_offset.y, &font, font_size, vh, camera_position, ORANGE);

    let mut buff_offset: Vector2 = Vector2 { x: -11.5, y: -17.0 };
    for buff in self.buffs.clone() {
      draw_text_relative(match buff.buff_type { BuffType::FireRate => "+ fire rate", BuffType::RaphaelleFireRate => "+ fire rate", BuffType::Speed => if buff.value > 0.0 { "+ speed"} else {"- speed"}, BuffType::WiroSpeed => "+ speed", BuffType::Impulse => "+ impulse"}, self.position.x + buff_offset.x, self.position.y + buff_offset.y, &font, font_size, vh, camera_position, SKYBLUE);
      buff_offset.y -= 3.0;
    }
  }
  pub fn new() -> ClientPlayer {
    return ClientPlayer {
      health: 100,
      position: Vector2::new(),
      aim_direction: Vector2::new(),
      character: Character::Hernani,
      secondary_charge: 100,
      movement_direction: Vector2::new(),
      shooting_primary: false,
      shooting_secondary: false,
      team: Team::Blue,
      time_since_last_dash: 0.0,
      is_dead: false,
      camera: Camera::new(),
      buffs: Vec::new(),
      previous_positions: Vec::new(),
      ping: 0,
      last_shot_time: 0.0,
      last_secondary_time: 0.0,
      dashing: false,
      is_dashing: false,
      dashed_distance: 0.0,
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
  pub dashing:         bool,
  /// TEMPORARY
  pub character: Character,
  pub port: u16,
  pub timestamp: SystemTime,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq)]
pub enum Team {
  Red = 0,
  Blue = 1,
}
// MARK: Server
/// Information sent by srever to client about themself
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ServerRecievingPlayerPacket {
  pub health: u8,
  pub override_position:  bool,
  pub position_override:  Vector2,
  pub shooting_primary:   bool,
  pub shooting_secondary: bool,
  pub secondary_charge:   u8,
  pub character:          Character,
  pub is_dead:            bool,
  pub buffs:              Vec<Buff>,
  pub previous_positions: Vec<Vector2>,
  pub team:               Team,
  pub time_since_last_primary: f32,
  pub time_since_last_dash: f32,
  pub time_since_last_secondary: f32,
}

/// information sent by server to client
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ServerPacket {
  pub player_packet_is_sent_to: ServerRecievingPlayerPacket,
  pub players:       Vec<OtherPlayer>,
  pub game_objects:  Vec<GameObject>,
  pub gamemode_info: GameModeInfo,
  pub timestamp:     SystemTime,
}
// MARK: Gameobject
/// defines any non-player gameplay element
/// Contains fields that can describe all necessary information for most game objects.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct GameObject {
  pub object_type: GameObjectType,
  pub size: Vector2,
  pub position: Vector2,
  pub direction: Vector2,
  pub to_be_deleted: bool,
  /// pikmin
  pub owner_port: u16,
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
  HernaniWall,
  RaphaelleAura,
  UnbreakableWall,
  HernaniBullet,
  RaphaelleBullet,
  RaphaelleBulletEmpowered,
  CynewynnSword,
  HernaniLandmine,
  /// Elizabeth's projectile, as it has just been fired and is still flying.
  ElizabethProjectileRicochet,
  /// Elizabeth's projectile, once it's on the ground.
  ElizabethProjectileGround,
  /// Elizabeth's projectile, once it's returning to her.
  ElizabethProjectileGroundRecalled,
  ElizabethTurret,
  ElizabethTurretProjectile,
  Grass1,
  Grass2,
  Grass3,
  Grass4,
  Grass5,
  Grass6,
  Grass7,
  Grass1Bright,
  Grass2Bright,
  Grass3Bright,
  Grass4Bright,
  Grass5Bright,
  Grass6Bright,
  Grass7Bright,
  /// Currently, an edge water tile
  Water1,
  /// Currently, a full water tile
  Water2,
  /// The orb in the middle of the map
  CenterOrb,
  CenterOrbSpawnPoint,
  WiroShield,
  /// Wiro's damaging projectile. Size is constant, but hit_radius is proportional to speed.
  WiroGunShot,
  /// When wiro dashes, this projectile copies his position and is used for
  /// the healing/damaging logic
  WiroDashProjectile,
}
// MARK: Vectors
// utility
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq)]
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
  /// vec2 - vec1
  /// TO DO: Phase out this function
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
use std::ops;
// Add Vector2 to Vector2
impl ops::Add<Vector2> for Vector2 {
  type Output = Vector2;
  fn add(self, other: Vector2) -> Vector2 {
    let result: Vector2 = Vector2 {
      x: self.x + other.x,
      y: self.y + other.y,
    };
    result
  }
}
// Add Vector2 to Vector2
impl ops::AddAssign<Vector2> for Vector2 {
  fn add_assign(&mut self, other: Vector2) -> () {
    *self = Vector2 {
      x: self.x + other.x,
      y: self.y + other.y,
    };
  }
}
// Add Vector2 to Vector2
impl ops::SubAssign<Vector2> for Vector2 {
  fn sub_assign(&mut self, other: Vector2) -> () {
    *self = Vector2 {
      x: self.x - other.x,
      y: self.y - other.y,
    };
  }
}
// Substract Vector2 to Vector2
impl ops::Sub<Vector2> for Vector2 {
  type Output = Vector2;
  fn sub(self, other: Vector2) -> Vector2 {
    let result: Vector2 = Vector2 {
      x: self.x - other.x,
      y: self.y - other.y,
    };
    result
  }
}
// Multiply Vector2 by f32
impl ops::Mul<f32> for Vector2 {
  type Output = Vector2;
  fn mul(self, other: f32) -> Vector2 {
    let result: Vector2 = Vector2 {
      x: self.x * other,
      y: self.y * other,
    };
    result
  }
}
// Divide Vector2 by f32
impl ops::Div<f32> for Vector2 {
  type Output = Vector2;
  fn div(self, other: f32) -> Vector2 {
    let result: Vector2 = Vector2 {
      x: self.x / other,
      y: self.y / other,
    };
    result
  }
}

/// wrapper function for `draw_texture_ex`, simplifies it.
pub fn draw_image(texture: &Texture2D, x: f32, y: f32, w: f32, h: f32, vh: f32, rotation: Vector2, color: Color) -> () {
  let mut rotation_rad: f32 = 0.0;
  if rotation.magnitude() != 0.0 {
    rotation_rad = rotation.y.atan2(rotation.x);
  }
  draw_texture_ex(texture, x * vh, y * vh, color, DrawTextureParams {
    dest_size: Some(Vec2 { x: w * vh, y: h * vh}),
    source: None,
    rotation: rotation_rad,
    flip_x: false,
    flip_y: false,
    pivot: Some(Vec2 { x: (x+w/2.0)*vh, y: (y+h/2.0)*vh })
  });
}
// MARK: Draw
/// same as draw_image but draws relative to a ceratain position and centers it.
/// The x and y parameters are still world coordinates.
pub fn draw_image_relative(texture: &Texture2D, x: f32, y: f32, w: f32, h: f32, vh: f32, center_position: Vector2, rotation: Vector2, color: Color) -> () {

  // draw relative to position and centered.
  let relative_position_x = x - center_position.x + (50.0 * (16.0/9.0)); //+ ((vh * (16.0/9.0)) * 100.0 )/ 2.0;
  let relative_position_y = y - center_position.y + 50.0; //+ (vh * 100.0) / 2.0;

  draw_image(texture, relative_position_x, relative_position_y, w, h, vh, rotation, color);
}

pub fn draw_line_relative(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color, center_position: Vector2, vh:f32) -> () {
  let relative_position_x1 = x1 - center_position.x + (50.0 * (16.0/9.0));
  let relative_position_y1 = y1 - center_position.y + 50.0;
  let relative_position_x2 = x2 - center_position.x + (50.0 * (16.0/9.0));
  let relative_position_y2 = y2 - center_position.y + 50.0;
  draw_line(relative_position_x1 * vh, relative_position_y1 * vh, relative_position_x2 * vh, relative_position_y2 * vh, thickness * vh, color);
}
pub fn draw_rectangle_relative(x1: f32, y1: f32, w: f32, h: f32, color: Color, center_position: Vector2, vh:f32) -> () {
  let relative_position_x1 = x1 - center_position.x + (50.0 * (16.0/9.0));
  let relative_position_y1 = y1 - center_position.y + 50.0;
  draw_rectangle(relative_position_x1*vh, relative_position_y1*vh, w*vh, h*vh, color);
}

pub fn draw_text_relative(text: &str, x: f32, y:f32, font: &Font, font_size: u16, vh: f32, center_position: Vector2, color: Color) -> () {
  let relative_position_x = x - center_position.x + (50.0 * (16.0/9.0)); //+ ((vh * (16.0/9.0)) * 100.0 )/ 2.0;
  let relative_position_y = y - center_position.y + 50.0; //+ (vh * 100.0) / 2.0;
  draw_text_ex(text, relative_position_x * vh, relative_position_y * vh, TextParams { font: Some(font), font_size: (font_size as f32 * vh) as u16, font_scale: 1.0, font_scale_aspect: 1.0, rotation: 0.0, color });
}

/// Loads any map from a properly formatted string: `<object> [posX] [posY]`
/// 
/// example:
/// ```rust
/// let game_objects: Vec<GameObject> = load_map_from_file(include_str!("../../assets/maps/map1.map"));
/// ```
/// map1.map:
/// ```
/// wall 10.0 10.0
/// wall 20.0 10.0
/// wall 30.0 10.0
/// wall 40.0 10.0
/// unbreakablewall 50.0 10.0
/// ```
pub fn load_map_from_file(map: &str) -> Vec<GameObject> {
  let mut map_to_return: Vec<GameObject> = Vec::new();
  for line in map.lines() {
    let opcodes: Vec<&str> = line.split(" ").collect();
    let gameobject_type = opcodes[0].to_lowercase();
    let gameobject_type = gameobject_type.as_str();
    let pos_x: f32 = opcodes[1].parse().unwrap();
    let pos_y: f32 = opcodes[2].parse().unwrap();
    let pos_x = pos_x * TILE_SIZE;
    let pos_y = pos_y * TILE_SIZE;

    map_to_return.push(GameObject {
      object_type: match gameobject_type {
        "wall"            => {GameObjectType::Wall},
        "unbreakablewall" => {GameObjectType::UnbreakableWall},
        "water1" => {GameObjectType::Water1},
        "water2" => {GameObjectType::Water2},
        "orb"    => {GameObjectType::CenterOrbSpawnPoint},
        _                 => {panic!("Unexpected ojbect in map file.")},
      },
      size: match gameobject_type {
        "wall" => Vector2 { x: TILE_SIZE, y: TILE_SIZE*2.0 },
        "unbreakablewall" => Vector2 { x: TILE_SIZE, y: TILE_SIZE*2.0 },
        "water1" => Vector2 { x: TILE_SIZE, y: TILE_SIZE*2.0 },
        "water2" => Vector2 { x: TILE_SIZE, y: TILE_SIZE*2.0 },
        "orb"    => Vector2 { x: TILE_SIZE*1.0, y: TILE_SIZE*1.0 },
        _        => {panic!("Unexpected ojbect in map file.")},
      },
      position: Vector2 { x: pos_x, y: pos_y },
      direction: Vector2::new(),
      to_be_deleted: false,
      owner_port: 200,
      hitpoints: 30,
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

  let mut desired_position = current_player_position;
  desired_position.x += movement.x;
  desired_position.y += movement.y;

  let player_size = TILE_SIZE/4.0;
  let tile_size = TILE_SIZE/2.0;
  let collision_size = tile_size + player_size;

  for game_object in game_objects.clone() {
    if game_object.object_type == GameObjectType::Wall           ||
       game_object.object_type == GameObjectType::HernaniWall    ||
       game_object.object_type == GameObjectType::Water1         ||
       game_object.object_type == GameObjectType::Water2         ||
       game_object.object_type == GameObjectType::UnbreakableWall {
      let difference = Vector2::difference(desired_position, game_object.position);

      // X axis collision prediction
      if f32::abs(difference.x) <= collision_size && f32::abs(current_player_position.y - game_object.position.y) < collision_size {
        adjusted_movement.x = 0.0;
        adjusted_raw_movement.x = 0.0;
      }

      // Y axis
      if f32::abs(difference.y) <= collision_size && f32::abs(current_player_position.x - game_object.position.x) < collision_size {
        adjusted_movement.y = 0.0;
        adjusted_raw_movement.y = 0.0;
      }
    }
  }
  return (adjusted_raw_movement, adjusted_movement);
}

// MARK: Extras (f32)
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

/// Stores information about any buff.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Buff {
  /// Numerical value associated with buff, like speed gained, or fire rate increase.
  pub value: f32,
  /// Time left in seconds
  pub duration: f32,
  /// Type of buff. Speed, Fire rate, etc...
  pub buff_type: BuffType,
  /// Direction
  pub direction: Vector2,
}
/// Every possible type of buff or nerf
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum BuffType {
  RaphaelleFireRate,
  WiroSpeed,
  FireRate,
  Impulse,
  Speed,
}

pub fn dashing_logic(mut is_dashing: bool, mut dashed_distance: f32, dash_direction: Vector2, delta_time: f64, char_dash_speed: f32, char_dash_distance: f32, game_objects: Vec<GameObject>, current_position: Vector2) -> (Vector2, f32, bool) {
  let mut new_position = Vector2::new();
  let player_dashing_speed: f32 = char_dash_speed;
  let player_max_dash_distance: f32 = char_dash_distance;
  
  let mut dash_movement = Vector2::new();
  dash_movement.x = dash_direction.x * player_dashing_speed * delta_time as f32;
  dash_movement.y = dash_direction.y * player_dashing_speed * delta_time as f32;
  
  // calculate current expected position based on input
  let (new_movement_raw, _): (Vector2, Vector2) = object_aware_movement(
    current_position,
    dash_direction,
    dash_movement,
    game_objects,
  );
  
  if dashed_distance < player_max_dash_distance {
    new_position.x = current_position.x + new_movement_raw.x * player_dashing_speed * delta_time as f32;
    new_position.y = current_position.y + new_movement_raw.y * player_dashing_speed * delta_time as f32;
    let dashed_distance_this_frame = Vector2::distance(new_position, current_position);
    if dashed_distance_this_frame > 0.0 {
      dashed_distance += Vector2::distance(new_position, current_position);
    }
    else {
      is_dashing = false;
    }
  }
  else {
    dashed_distance = 0.0;
    is_dashing = false;
    // The final frame of the dash new_position isn't updated, if not
    // for this line the player would get sent back to 0.0-0.0
    new_position = current_position; // idk why this is like this, whatever
  }
  return (new_position, dashed_distance, is_dashing);
}

pub fn apply_wallride_force(movement_vector: Vector2, game_objects: Vec<GameObject>, player_pos: Vector2) -> Vector2 {
  let mut new_mov_vector: Vector2 = movement_vector;

  let constant: f32 = 1.0;
  let cutoff: f32 = 2.0;

  for game_object in game_objects {
    // if the object is a wall
    if WALL_TYPES.contains(&game_object.object_type){
      let wall_pos = game_object.position;
      let difference = Vector2::difference(player_pos, wall_pos);
      if difference.magnitude() < TILE_SIZE * cutoff {
        new_mov_vector.y += (1.0/(difference.magnitude())) * constant * difference.normalize().y;
        new_mov_vector.x += (1.0/(difference.magnitude())) * constant * difference.normalize().x;
      }

    }
  }

  return new_mov_vector;
}