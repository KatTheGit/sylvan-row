use bevy::color::palettes::css::*;
use bevy::color::Srgba;
use bevy::ecs::system::Commands;
use bevy::window::Window;
use bevy::prelude::*;
use rusty_pkl::*;
use core::f32;
use std::collections::HashMap;
use std::sync::MutexGuard;
use std::vec;
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use crate::bevy_immediate::*;
use crate::const_params::*;
use crate::maths::*;
use std::time::SystemTime;
use crate::bevy_graphics::*;
use std::time::Instant;
// MARK: Gamemodes

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct Camera {
  pub position: Vector2,
  pub zoom: f32,
}
impl Camera {
  pub fn new() -> Camera {
    return Camera { position: Vector2::new(), zoom: 1.0 };
  }
}
// MARK: Client

/// Information held by client about self and other players.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientPlayer {
  pub username: String,
  pub health: u8,
  pub position: Vector2,
  pub aim_direction: Vector2,
  pub character: Character,
  pub secondary_charge: u8,
  pub movement_direction: Vector2,
  pub shooting_primary: bool,
  pub shooting_secondary: bool,
  pub team: Team,
  pub is_dead: bool,
  pub camera: Camera,
  pub buffs: Vec<Buff>,
  pub previous_positions: Vec<Vector2>,
  pub ping: u16,
  pub last_shot_time: f32,
  pub last_secondary_time: f32,
  pub time_since_last_dash: f32,
  /// Used for audio
  pub used_primary: bool,
  /// Used for audio
  pub used_secondary: bool,
  /// Used for audio
  pub used_dash: bool,
  /// wants to dash
  pub dashing: bool,
  /// is currently dashing
  pub is_dashing: bool,
  pub dashed_distance: f32,
  pub stacks: u8,
  /// Used by client player. If true, lock movement and interpolate position
  pub interpolating: bool,
  pub interpol_prev: Vector2,
  pub interpol_next: Vector2,
  pub passive_elapsed: f32,
  pub current_animation: AnimationState,
}
/// Information sent by server to client about other players.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct OtherPlayer {
  pub username: String,
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
  pub stacks: u8,
}
impl ClientPlayer {
  pub fn from_otherplayer(other_player: OtherPlayer) -> ClientPlayer {
    return ClientPlayer {
      username: other_player.username,
      health: other_player.health,
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
      stacks: other_player.stacks,
      interpolating: false,
      interpol_next: Vector2::new(),
      interpol_prev: Vector2::new(),
      used_primary: false,
      used_secondary: false,
      used_dash: false,
      passive_elapsed: 0.0,
      current_animation: AnimationState::new(vec![], 0.0, 0),
    }
  }
  pub fn draw(&self, texture: &Texture, vh: f32, camera: Camera, font: &Handle<Font>, character: CharacterProperties, settings: Settings, z: i8, commands: &mut Commands, window: &Window) {
    // TODO: animations
    let bg_offset: Vector2 = Vector2 { x: -12.0, y: -16.5 };
    let bg_size: Vector2 = Vector2 {x: bg_offset.x*-2.0, y: 7.0};
    let bg_opacity: f32 = 0.4;
    let color = match self.team {
      Team::Blue => Srgba { red: 0.3, green: 0.5, blue: 0.7, alpha: bg_opacity },
      Team::Red => Srgba { red: 0.7, green: 0.5, blue: 0.3, alpha: bg_opacity },
    };
    draw_rectangle_relative(bg_offset.x + self.position.x, bg_offset.y + self.position.y, bg_size.x, bg_size.y, color, camera.clone(), vh, z, window, commands);

    let size: f32 = 10.0;
    draw_image_relative(texture, self.position.x -(size/2.0), self.position.y - ((size/2.0)* (8.0/5.0)), size, size * (8.0/5.0), vh, camera.clone(), z, window, commands);
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
      camera.clone(), vh, z, window, commands);
    draw_line_relative(
      self.position.x + secondary_bar_offset.x,
      self.position.y + secondary_bar_offset.y,
      self.secondary_charge as f32 / 10.0 + self.position.x + secondary_bar_offset.x,
      self.position.y + secondary_bar_offset.y,
      1.5,
      ORANGE,
      camera.clone(), vh, z, window, commands);
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
      camera.clone(), vh, z, window, commands);
    let health_counter_offset: Vector2 = Vector2 { x: -11.5, y: -10.6 };
    let health_counter_with_leading_zeros = format!("{:0>3}", self.health.to_string());
    let font_size: f32 = 4.0;
    draw_text_relative(health_counter_with_leading_zeros.as_str(), self.position.x + health_counter_offset.x, self.position.y + health_counter_offset.y, &font, font_size, vh, camera.clone(), z, window, commands);
    let secondary_counter_offset: Vector2 = Vector2 { x: 5.9, y: -10.6 };
    let secondary_counter_with_leading_zeros = format!("{:0>3}", self.secondary_charge.to_string());
    draw_text_relative(secondary_counter_with_leading_zeros.as_str(), self.position.x + secondary_counter_offset.x, self.position.y + secondary_counter_offset.y, &font, font_size, vh, camera.clone(), z, window, commands);
    
    let displayed_name =
      if settings.display_char_name_instead {
        self.character.name()
      }
      else {
        self.username.clone()
      };

    let username_offset: Vector2 = Vector2 { x: -11.5, y: -17.0 };
    draw_text_relative(&displayed_name, self.position.x + username_offset.x, self.position.y + username_offset.y, font, font_size, vh, camera.clone(), z, window, commands);
    let mut buff_offset: Vector2 = Vector2 { x: -11.5, y: -21.0 };
    for buff in self.buffs.clone() {
      if !vec![BuffType::Impulse].contains(&buff.buff_type) {
        draw_text_relative(match buff.buff_type { BuffType::FireRate => "+ fire rate", BuffType::RaphaelleFireRate => "+ fire rate", BuffType::Speed => if buff.value > 0.0 { "+ speed"} else {"- speed"}, BuffType::WiroSpeed => "+ speed", BuffType::Impulse => "+ impulse"}, self.position.x + buff_offset.x, self.position.y + buff_offset.y, &font, font_size, vh, camera.clone(), z, window, commands);
      }
      buff_offset.y -= 3.0;
    }
  }
  pub fn new() -> ClientPlayer {
    return ClientPlayer {
      username: String::from("New User"),
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
      stacks: 0,
      interpolating: false,
      interpol_next: Vector2::new(),
      interpol_prev: Vector2::new(),
      used_dash: true,
      used_primary: true,
      used_secondary: true,
      passive_elapsed: 0.0,
      current_animation: AnimationState::new(vec![], 0.0, 0),
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
  pub stacks:             u8,
  pub is_dashing:         bool,
  pub passive_elapsed:    f32,
}

/// information sent by server to client
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ServerPacket {
  pub player_packet_is_sent_to: ServerRecievingPlayerPacket,
  pub players:       Vec<OtherPlayer>,
  pub game_objects:  Vec<GameObject>,
  pub gamemode_info: GameModeInfo,
  pub timestamp:     SystemTime,
  pub events: Vec<GameEvent>,
}

pub fn get_random_port() -> u16 {
  // Find a random free port and use it
  let min_port: u16 = 30000;
  let max_port: u16 = 30300;
  let mut port: u16;
  loop {
    let y = crappy_random();
    port = min_port + (y * (max_port - min_port) as f64) as u16;
    print!("Attempted port: {}. Status: ", port);
    let dummy_ip: String = format!("0.0.0.0:{}", port);
    match std::net::UdpSocket::bind(dummy_ip) {
      Ok(_) => {
        // PORT IS GOOD
        println!("Good");
        break;
      }
      Err(_) => {
        // PORT IS BAD
        // try again
        println!("Bad. Trying again.");
      }
    }
  }
  return port;
}

/// Information held by server about players.
/// 
/// This struct can be as hefty as we want, it doesn't get sent over network.
#[derive(Debug, Clone)]
pub struct ServerPlayer {
  pub username:             String,
  pub cipher_key:           Vec<u8>,
  pub last_nonce:           u32,
  pub ip:                   String,
  pub port:                 u16,
  pub team:                 Team,
  pub character:            Character,
  pub health:               u8,
  pub position:             Vector2,
  pub shooting:             bool,
  /// To calculate cooldowns
  pub shooting_secondary:   bool,
  pub secondary_charge:     u8,
  pub aim_direction:        Vector2,
  pub move_direction:       Vector2,
  pub had_illegal_position: bool,
  pub is_dashing:           bool,
  pub dash_direction:       Vector2,
  pub dashed_distance:      f32,
  pub previous_positions:   Vec<Vector2>,
  /// bro forgor to live
  pub is_dead:              bool,
  pub last_shot_time:       Instant,
  pub secondary_cast_time:  Instant,
  pub last_dash_time:       Instant,
  pub death_timer_start:    Instant,  
  pub passive_timer:        Instant,
  /// Remember to apply appropriate logic after check.
  /// 
  /// General counter to keep track of ability stacks. Helps determine things
  /// like whether the next shot is empowered, or how powerful an ability
  /// should be after being charged up.
  pub stacks:  u8,
  /// list of buffs
  pub buffs:                Vec<Buff>,
  pub last_packet_time:     Instant,
  pub events: Vec<GameEvent>,
}
impl ServerPlayer {
  pub fn damage(&mut self, mut dmg: u8, characters: HashMap<Character, CharacterProperties>) -> () {
    if self.is_dead {
      return;
    }
    // Special per-character handling
    match self.character {
      Character::Raphaelle => {
        self.buffs.push(
          Buff { value: 6.0, duration: 0.5, buff_type: BuffType::Speed, direction: Vector2::new() }
        );
      }
      _ => {}
    }

    // apply dashing damage reduction or increase.
    if self.is_dashing {
      dmg = (dmg as f32 * characters[&self.character].dash_damage_multiplier) as u8;
    }

    if self.health < dmg {
      self.health = 0
    } else {
      self.health -= dmg;
    }
  }
  pub fn heal(&mut self, heal: u8, characters: HashMap<Character, CharacterProperties>) -> () {
    if self.is_dead {
      return;
    }

    // this edge case crashes the server
    if self.health as i16 + heal as i16 > characters[&self.character].health as i16 {
      self.health = characters[&self.character].health;
    } else {
      self.health += heal;
    }
  }
  pub fn add_charge(&mut self, charge: u8) -> () {
    if self.is_dead {
      return;
    }

    if self.secondary_charge + charge > 100 {
      self.secondary_charge = 100;
    } else {
      self.secondary_charge += charge;
    }
  }
  pub fn kill(&mut self, red_spawn: Vector2, blue_spawn: Vector2) {
    //println!("killing");
    // remove all previous positions
    self.previous_positions = Vec::new();
    self.secondary_charge = 0;
    // set them back to 100
    self.health = 100;
    // set dead flag for other handling
    self.is_dead = true;
    // mark when they died so we know when to respawn them
    self.death_timer_start = Instant::now();
    // send them to their respective spawn
    if self.team == Team::Blue {
      self.position = blue_spawn;
      // println!("Sending {} to blue spawn", self.ip);
    } 
    else {
      self.position = red_spawn;
      // println!("Sending {} to red team spawn", self.ip);
    }
  }
}

pub fn index_by_username(username: &str, players: Vec<ServerPlayer>) -> usize{
  for p_index in 0..players.len() {
    if players[p_index].username == username {
      return p_index;
    }
  }
  println!("index_by_port function error - data race condition, mayhaps?\nAlternatively, there's just no players at all");
  return 0;
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum GameEvent {
  /// Informs that an attack of type `GameObjectType` (owned by `String` #1) has hit a player (`String` #2)
  AttackHit(GameObjectType, String, String),
  /// Attack of type `GameObjectType` was fired by its owner `String`.
  AttackFired(GameObjectType, String),
  /// Wall was hit by object `GameObjectType`, owned by `String`.
  WallHit(GameObjectType, String),
}
// (vscode) MARK: Gamemode

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
  /// Whether the game has started and is in aciton (true), or
  /// we're waiting for a round;
  pub game_active: bool
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
      game_active: false,
    }
  }
}

// MARK: Characters
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, EnumIter, PartialEq, Eq, Hash)]
pub enum Character {
  /// Used for testing.
  Cynewynn,
  Dummy,
  Fedya,
  Hernani,
  Koldo,
  Raphaelle,
  Temerity,
  Wiro,
}
impl Character {
  pub fn name(self) -> String {
    return match self {
      Character::Cynewynn => String::from("Cynewynn"),
      Character::Dummy => String::from("Dummy"),
      Character::Fedya => String::from("Fedya"),
      Character::Hernani => String::from("Hernani"),
      Character::Koldo => String::from("Koldo"),
      Character::Raphaelle => String::from("Raphaelle"),
      Character::Temerity => String::from("Temerity"),
      Character::Wiro => String::from("Wiro"),
    }
  }
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
  pub passive_value:                u8,
  pub passive_cooldown:             f32,
}

pub fn load_characters() -> HashMap<Character, CharacterProperties> {
  let mut characters: HashMap<Character, CharacterProperties> = HashMap::new();
  for character in Character::iter() {
    let character_properties: CharacterProperties = match character {
      Character::Dummy      => CharacterProperties::from_pkl(include_str!("../assets/characters/dummy/properties.pkl")),
      Character::Hernani => CharacterProperties::from_pkl(include_str!("../assets/characters/hernani/properties.pkl")),
      Character::Raphaelle => CharacterProperties::from_pkl(include_str!("../assets/characters/raphaelle/properties.pkl")),
      Character::Cynewynn =>  CharacterProperties::from_pkl(include_str!("../assets/characters/cynewynn/properties.pkl")),
      Character::Fedya =>  CharacterProperties::from_pkl(include_str!("../assets/characters/fedya/properties.pkl")),
      Character::Wiro =>  CharacterProperties::from_pkl(include_str!("../assets/characters/wiro/properties.pkl")),
      Character::Koldo =>  CharacterProperties::from_pkl(include_str!("../assets/characters/koldo/properties.pkl")),
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
      passive_value:            pkl_u8( find_parameter(&pkl, "passive_value"            ).unwrap()),
      passive_cooldown:         pkl_f32(find_parameter(&pkl, "passive_cooldown"         ).unwrap()),
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

// MARK: Gameobject
/// defines any non-player gameplay element
/// Contains fields that can describe all necessary information for most game objects.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct GameObject {
  /// A numerical identifier used to track objects, mostly used by the client but
  /// generated by the server.
  pub id: u16,
  pub object_type: GameObjectType,
  /// Contains additional data like data for bullets and walls.
  pub extra_data: ObjectData,
  pub position: Vector2,
  pub to_be_deleted: bool,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]

pub enum ObjectData {
  BulletData(BulletData),
  WallData(WallData),
  NoData,
}
/// enumerates all possible gameobjects. Their effects are then handled by the server.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Hash, Eq)]
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
  /// Fedya's projectile, as it has just been fired and is still flying.
  FedyaProjectileRicochet,
  /// Fedya's projectile, once it's on the ground.
  FedyaProjectileGround,
  /// Fedya's projectile, once it's returning to her.
  FedyaProjectileGroundRecalled,
  FedyaTurret,
  FedyaTurretProjectile,
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
  /// ROCKET LAUNCHA
  TemerityRocket,
  TemerityRocketSecondary,
  /// Normal primary.
  KoldoCannonBall,
  /// Normal primary boosted by passive and second primary in the ultimate.
  KoldoCannonBallEmpowered,
  /// The first primary in the ultimate.
  KoldoCannonBallEmpoweredUltimate,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct BulletData {
  pub direction: Vector2,
  pub owner_username: String,
  pub hitpoints: u8,
  pub lifetime: f32,
  pub hit_players: Vec<usize>,
  pub traveled_distance: f32,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct WallData {
  pub lifetime: f32,
  pub hitpoints: u8,
}
impl GameObject {
  pub fn get_bullet_data(&self) -> BulletData {
    match self.extra_data.clone() {
      ObjectData::BulletData(data) => {
        return data;
      }
      _ => {
        panic!("get_bullet_data error. Does not have this type of data.")
      }
    }
  }
  pub fn get_bullet_data_safe(&self) -> Result<BulletData, ()> {
    match self.extra_data.clone() {
      ObjectData::BulletData(data) => {
        return Ok(data);
      }
      _ => {
        return Err(())
      }
    }
  }
  pub fn get_wall_data(&self) -> WallData {
    match self.extra_data.clone() {
      ObjectData::WallData(data) => {
        return data;
      }
      _ => {
        panic!("get_wall_data error. Does not have this type of data.")
      }
    }
  }
  pub fn get_wall_data_safe(&self) -> Result<WallData, ()> {
    match self.extra_data.clone() {
      ObjectData::WallData(data) => {
        return Ok(data);
      }
      _ => {
        return Err(())
      }
    }
  }
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
pub fn load_map_from_file(map: &str, id: &mut u16) -> Vec<GameObject> {
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
      id: id.clone(),
      object_type: match gameobject_type {
        "wall"            => {GameObjectType::Wall},
        "unbreakablewall" => {GameObjectType::UnbreakableWall},
        "water1" => {GameObjectType::Water1},
        "water2" => {GameObjectType::Water2},
        "orb"    => {GameObjectType::CenterOrbSpawnPoint},
        _                 => {panic!("Unexpected ojbect in map file.")},
      },
      position: Vector2 { x: pos_x, y: pos_y },
      to_be_deleted: false,
      extra_data: ObjectData::WallData(
        WallData {
          lifetime: f32::INFINITY,
          hitpoints: WALL_HP,
        }
      ),
    });
    *id += 1;
  }
  return map_to_return;
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
  /// Increases the player's speed by `value`'s amount.
  /// 
  /// Use negative value for slow.
  Speed,
}
#[derive(Debug, Clone)]
pub struct CharacterDescription {
  pub primary: AbilityDescription,
  pub secondary: AbilityDescription,
  pub dash: AbilityDescription,
  pub passive: AbilityDescription,
}
#[derive(Debug, Clone)]
pub struct AbilityDescription {
  pub description: String,
  pub values: Vec<f32>,
}
impl CharacterDescription {
  pub fn create_all_descriptions(character_properties: HashMap<Character, CharacterProperties>) -> HashMap<Character, CharacterDescription> {
    let mut character_descriptions: HashMap<Character, CharacterDescription> = HashMap::new();
    for character in Character::iter() {
      character_descriptions.insert( character, match character {
        Character::Cynewynn => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("Swings her sword, dealing {0} damage."), values: vec![character_properties[&character].primary_damage as f32] },
            secondary: AbilityDescription { description: String::from("Teleports {0} seconds in the past, and gains {1} health."), values: vec![character_properties[&character].secondary_cooldown, character_properties[&character].secondary_heal as f32] },
            dash:      AbilityDescription { description: String::from("Dashes, taking {0}% less damage."), values: vec![(1.0 - character_properties[&character].dash_damage_multiplier) * 100.0] },
            passive:   AbilityDescription { description: String::from("Primary cooldown is lower the higher her secondary charge,reduced by up to {0}s."), values: vec![character_properties[&character].primary_cooldown_2] },
          }
        }
        Character::Hernani => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("Fires a musket, dealing {0} damage."), values: vec![character_properties[&character].primary_damage as f32] },
            secondary: AbilityDescription { description: String::from("Places a wall in front of himself."), values: vec![] },
            dash:      AbilityDescription { description: String::from("Dashes, placing a bear trap that deals {0} damage when triggered."), values: vec![character_properties[&character].primary_damage_2 as f32] },
            passive:   AbilityDescription { description: String::from("Deals {0}% damage to walls."), values: vec![character_properties[&character].wall_damage_multiplier * 100.0] },
          }
        }
        Character::Raphaelle => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("Casts a {0} damage piercing shot. Landing it heals nearby allies by {1}, and herself by {2}. If empowered, deals {3} damage and doesn't heal."), values: vec![character_properties[&character].primary_damage as f32, character_properties[&character].primary_heal_2 as f32, character_properties[&character].primary_lifesteal as f32, character_properties[&character].primary_damage_2 as f32] },
            secondary: AbilityDescription { description: String::from("Places down a healpool that lasts {0}s, healing allies by {1} every second, and providing a fire rate buff."), values: vec![character_properties[&character].secondary_cooldown as f32, character_properties[&character].secondary_heal as f32] },
            dash:      AbilityDescription { description: String::from("Dashes, empowering her PRIMARY. If she lands an empowered PRIMARY, reduces this cooldown by {0}s."), values: vec![character_properties[&character].primary_cooldown_2] },
            passive:   AbilityDescription { description: String::from("Gains a small speed buff upon taking damage."), values: vec![character_properties[&character].primary_damage as f32] },
          }
        }
        Character::Temerity => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("A three-hit combo dealing {0} damage, with each attack gaining in range ( {1}m | {2}m | {3}m )."), values: vec![character_properties[&character].primary_damage as f32, character_properties[&character].primary_range / TILE_SIZE, character_properties[&character].primary_range_2 / TILE_SIZE, character_properties[&character].primary_range_3 / TILE_SIZE] },
            secondary: AbilityDescription { description: String::from("Launches a rocket under herself, dealing {0} damage and boosting herself backwads."), values: vec![character_properties[&character].secondary_damage as f32] },
            dash:      AbilityDescription { description: String::from("Can hold DASH near walls to initiate a wallride."), values: vec![] },
            passive:   AbilityDescription { description: String::from("Heals nearby walls by {0} every second."), values: vec![character_properties[&character].passive_value as f32] },
          }
        }
        Character::Fedya => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("Throws a knife which can bounce off walls, that deals {0} damage and stays on the ground."), values: vec![character_properties[&character].primary_damage as f32] },
            secondary: AbilityDescription { description: String::from("Creates a turret that pinballs around, shooting nearby opponents every {0}s, dealing {1} damage."), values: vec![character_properties[&character].primary_cooldown_2, character_properties[&character].secondary_damage as f32] },
            dash:      AbilityDescription { description: String::from("Dashes and recalls all grounded knives, which slow opponents caught in their path and deal {0} damage."), values: vec![character_properties[&character].primary_damage_2 as f32] },
            passive:   AbilityDescription { description: String::from("No passive ability."), values: vec![] },
          }
        }
        Character::Wiro => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("An attack dealing {0} damage up-close and {1} damage at range. Landing this ability empowers DASH"), values: vec![character_properties[&character].primary_damage as f32, character_properties[&character].primary_damage_2 as f32] },
            secondary: AbilityDescription { description: String::from("Holds up a shield, damage taken with it costs secondary charge. If active, PASSIVE's speed no longer applies."), values: vec![] },
            dash:      AbilityDescription { description: String::from("A long dash. If empowered, heals allies in his path by {0} and damages opponents by {1}"), values: vec![character_properties[&character].secondary_heal as f32, character_properties[&character].secondary_damage as f32] },
            passive:   AbilityDescription { description: String::from("Nearby allies gain a small speed buff if SECONDARY is inactive. SECONDARY can only charge passively {0} seconds after use."), values: vec![character_properties[&character].passive_cooldown] },
          }
        }
        Character::Koldo => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("Fire a cannonball, dealing {0} damage. Firing close ({1}m) to a wall boosts you backwards."), values: vec![character_properties[&character].primary_damage as f32,character_properties[&character].primary_range_3/TILE_SIZE ] },
            secondary: AbilityDescription { description: String::from("Reset PRIMARY's cooldown and empower the next two PRIMARIES with PASSIVE's effects, with the first one additionally slowing enemies and piercing."), values: vec![] },
            dash:      AbilityDescription { description: String::from("A short dash that resets the cooldown of PRIMARY."), values: vec![] },
            passive:   AbilityDescription { description: String::from("Standing still for {0}s gives your PRIMARY recoil, increased range ({1}m) and increased damage ({2})."), values: vec![character_properties[&character].passive_cooldown as f32, character_properties[&character].primary_range_2/TILE_SIZE, character_properties[&character].primary_damage_2 as f32] },
          }
        }
        Character::Dummy => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("dummy."), values: vec![] },
            secondary: AbilityDescription { description: String::from("dummy."), values: vec![] },
            dash:      AbilityDescription { description: String::from("dummy."), values: vec![] },
            passive:   AbilityDescription { description: String::from("dummy."), values: vec![] },
          }
        }
      });
    }
    return character_descriptions;
  }
}
impl AbilityDescription {
  pub fn to_text(&self) -> String {
    let mut text = String::new();
    let split: Vec<&str> = self.description.split(['{', '}']).collect();
    for (index, string) in split.iter().enumerate() {
      // we always alternate between actual text and a property to parse, so use the even-ness
      // of the index to know which operation to perform.
      if index % 2 == 0 {
        text.push_str(string);
      } else {
        let property_index = str::parse::<usize>(string).expect("oops");
        let property_value = self.values[property_index];
        let mut property_value_string = property_value.to_string();
        // if it's an integer, remove the trailing ".0" to make it cleaner.
        if property_value_string.ends_with(".0") {
          property_value_string.truncate(property_value_string.len() - 2);
        }
        text.push_str(&property_value_string);
      }
    }
    return text;
  }
}

pub fn add_event_all(event: GameEvent, players: &mut MutexGuard<Vec<ServerPlayer>>) {
  for p_index in 0..players.len() {
    players[p_index].events.push(event.clone());
  }
}