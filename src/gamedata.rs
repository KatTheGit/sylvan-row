use rusty_pkl::*;
use core::f32;
use std::collections::HashMap;
use std::vec;
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use crate::const_params::*;
use crate::maths::*;

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
  Dummy,
  Hernani,
  Raphaelle,
  Cynewynn,
  Elizabeth,
  Wiro,
  Temerity,
}
impl Character {
  pub fn name(self) -> String {
    return match self {
      Character::Cynewynn => String::from("Cynewynn"),
      Character::Elizabeth => String::from("Josey"),
      Character::Temerity => String::from("Temerity"),
      Character::Wiro => String::from("Wiro"),
      Character::Hernani => String::from("Hernani"),
      Character::Raphaelle => String::from("Raphaelle"),
      Character::Dummy => String::from("Dummy"),
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
      passive_value:            pkl_u8( find_parameter(&pkl, "passive_value"            ).unwrap()),
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
  /// ROCKET LAUNCHA
  TemerityRocket,
  TemerityRocketSecondary,
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
            passive:   AbilityDescription { description: String::from("Primary cooldown is lower the higher her secondary charge,\nreduced by up to {0}s."), values: vec![character_properties[&character].primary_cooldown_2] },
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
            primary:   AbilityDescription { description: String::from("Casts a piecring shot, dealing {0} damage to opponents.\nDamaging someone heals nearby allies by {1}, and yourself by {2}.\nIf empowered, does not heal, but deals {3} damage instead."), values: vec![character_properties[&character].primary_damage as f32, character_properties[&character].primary_heal_2 as f32, character_properties[&character].primary_lifesteal as f32, character_properties[&character].primary_damage_2 as f32] },
            secondary: AbilityDescription { description: String::from("Places down a healpool that lasts {0}s, healing allies\nby {1} every second, and providing a fire rate buff."), values: vec![character_properties[&character].secondary_cooldown as f32, character_properties[&character].secondary_heal as f32] },
            dash:      AbilityDescription { description: String::from("Dashes, empowering her PRIMARY. If she lands an empowered\nPRIMARY, reduces this cooldown by {0}s."), values: vec![character_properties[&character].primary_cooldown_2] },
            passive:   AbilityDescription { description: String::from("Gains a small speed buff upon taking damage."), values: vec![character_properties[&character].primary_damage as f32] },
          }
        }
        Character::Temerity => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("A three-hit combo dealing {0} damage, with each attack\ngaining in range ( {1} | {2} | {3} )."), values: vec![character_properties[&character].primary_damage as f32, character_properties[&character].primary_range, character_properties[&character].primary_range_2, character_properties[&character].primary_range_3] },
            secondary: AbilityDescription { description: String::from("Launches a rocket under herself, dealing {0} damage and\nboosting herself backwads."), values: vec![character_properties[&character].secondary_damage as f32] },
            dash:      AbilityDescription { description: String::from("Can hold DASH near walls to initiate a wallride."), values: vec![] },
            passive:   AbilityDescription { description: String::from("Heals nearby walls by {0} every second."), values: vec![character_properties[&character].passive_value as f32] },
          }
        }
        Character::Elizabeth => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("Throws a knife which can bounce off walls, that deals\n{0} damage and stays on the ground."), values: vec![character_properties[&character].primary_damage as f32] },
            secondary: AbilityDescription { description: String::from("Creates a turret that pinballs around, shooting nearby\nopponents every {0}s, dealing {1} damage."), values: vec![character_properties[&character].primary_cooldown_2, character_properties[&character].secondary_damage as f32] },
            dash:      AbilityDescription { description: String::from("Dashes and recalls all grounded knives, which slow\nopponents caught in their path and deal {0} damage."), values: vec![character_properties[&character].primary_damage_2 as f32] },
            passive:   AbilityDescription { description: String::from("No passive ability."), values: vec![] },
          }
        }
        Character::Wiro => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("An attack dealing {0} damage up-close and {1} damage at range.\nLanding this ability empowers DASH"), values: vec![character_properties[&character].primary_damage as f32, character_properties[&character].primary_damage_2 as f32] },
            secondary: AbilityDescription { description: String::from("Holds up a shield, damage taken with it costs secondary charge.\nIf active, PASSIVE no longer applies."), values: vec![] },
            dash:      AbilityDescription { description: String::from("A long dash. If empowered, heals allies in his path by {0} and\ndamages opponents by {1}"), values: vec![character_properties[&character].secondary_heal as f32, character_properties[&character].secondary_damage as f32] },
            passive:   AbilityDescription { description: String::from("Nearby allies gain a small speed buff. Disabled if SECONDARY\nis active."), values: vec![] },
          }
        }
        Character::Dummy => {
          CharacterDescription {
            primary:   AbilityDescription { description: String::from("dummy."), values: vec![character_properties[&character].primary_damage as f32] },
            secondary: AbilityDescription { description: String::from("dummy."), values: vec![character_properties[&character].primary_damage as f32] },
            dash:      AbilityDescription { description: String::from("dummy."), values: vec![character_properties[&character].primary_damage as f32] },
            passive:   AbilityDescription { description: String::from("dummy."), values: vec![character_properties[&character].primary_damage as f32] },
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