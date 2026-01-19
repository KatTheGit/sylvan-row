use macroquad::prelude::*;
use std::time::SystemTime;
use crate::maths::*;
use crate::gamedata::*;
use crate::graphics::*;
use crate::ui::Settings;
use std::time::Instant;
use std::collections::HashMap;
// MARK: Gamemodes

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct Camera {
  pub position: Vector2,
}
impl Camera {
  pub fn new() -> Camera {
    return Camera { position: Vector2::new() };
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
  pub stacks: u8,
  /// Used by client player. If true, lock movement and interpolate position
  pub interpolating: bool,
  pub interpol_prev: Vector2,
  pub interpol_next: Vector2,
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
    }
  }
  pub fn draw(&self, texture: &Texture2D, vh: f32, camera_position: Vector2, font: &Font, character: CharacterProperties, settings: Settings) {
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
    
    let displayed_name =
      if settings.display_char_name_instead {
        self.character.name()
      }
      else {
        self.username.clone()
      };

    let username_offset: Vector2 = Vector2 { x: -11.5, y: -17.0 };
    draw_text_relative(&displayed_name, self.position.x + username_offset.x, self.position.y + username_offset.y, font, font_size, vh, camera_position, Color { r: color.r, g: color.g, b: color.b, a: 1.0 });
    let mut buff_offset: Vector2 = Vector2 { x: -11.5, y: -21.0 };
    for buff in self.buffs.clone() {
      draw_text_relative(match buff.buff_type { BuffType::FireRate => "+ fire rate", BuffType::RaphaelleFireRate => "+ fire rate", BuffType::Speed => if buff.value > 0.0 { "+ speed"} else {"- speed"}, BuffType::WiroSpeed => "+ speed", BuffType::Impulse => "+ impulse"}, self.position.x + buff_offset.x, self.position.y + buff_offset.y, &font, font_size, vh, camera_position, SKYBLUE);
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
  pub stacks: u8,
  pub is_dashing: bool,
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

pub fn get_random_port() -> u16 {
  // Find a random free port and use it
  let min_port: u16 = 49152; // start of dynamic port range
  let max_port: u16 = 65535;
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
  pub last_shot_time:       Instant,
  pub shooting_secondary:   bool,
  pub secondary_cast_time:  Instant,
  pub secondary_charge:     u8,
  pub aim_direction:        Vector2,
  pub move_direction:       Vector2,
  pub had_illegal_position: bool,
  pub is_dashing:           bool,
  pub dash_direction:       Vector2,
  pub dashed_distance:      f32,
  pub last_dash_time:       Instant,
  pub previous_positions:   Vec<Vector2>,
  /// bro forgor to live
  pub is_dead:              bool,
  pub death_timer_start:    Instant,  
  /// Remember to apply appropriate logic after check.
  /// 
  /// General counter to keep track of ability stacks. Helps determine things
  /// like whether the next shot is empowered, or how powerful an ability
  /// should be after being charged up.
  pub stacks:  u8,
  /// list of buffs
  pub buffs:                Vec<Buff>,
  pub last_packet_time:     Instant,
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