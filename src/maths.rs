// MARK: Vectors
// utility
use macroquad::math::Vec2;
use std::time::SystemTime;
use crate::const_params::*;
use crate::gamedata::*;
use crate::common::*;
use std::sync::MutexGuard;
use std::collections::HashMap;

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

// MARK: Extras (f32)
impl ExtrasF32 for f32 {
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
impl ExtrasU16 for u16 {
  /// Increments the number and returns its
  /// value (pre-incrementation).
  fn increment(&mut self) -> u16 {
      let current = self.clone();
      *self += 1;
      return current;
  }
}
pub trait ExtrasU16 {
  fn increment(&mut self) -> u16;
}

pub trait ExtrasF32 {
  fn sign(&self) -> f32;
  fn clean(&mut self);
}

// (vscode) MARK: Logic

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

/// ## Checks whether a projectile hits a shield
/// ### Math:
///- We have the direction the shield is facing as a vector (x, y),
///  i.e. (1, 2), a vector of slope 2/1
///  - To express it as a line in cartesian space, we can have ay = bx, where
///    in our example, a=1 and b=2, to have 2y = 1x, or 2y - 1x = 0
///    - Therefore, a vector (a, b) becomes the equation (by - ax = 0). 
///      - This is an equation of the form Ax + By + C = 0,
///        - A = -a
///        - B = b
///        - C = 0
///    - This line will always cross the origin (0, 0)
///- To get the distance of a point (c, d) from the line, we get its position
///  relative to the line's origin (and not the world origin),
///  and apply the equation:
///  - $dist = \frac{|Ac + Bd|}{\sqrt{b^2 + a^2}}
///          = \frac{|bd - ac|}{\sqrt{b^2 + a^2}}$
///  - **if $dist < hit\_radius$, we've got a hit on the shield.**
///- HOWEVER all of this logic assumes the shield is an infinite line. To apply
///  this to a finite line, we can add an additional distance check between the
///  origin of the shield and the projectile, once we've detected a collision:
///  - if dist(projectile_pos, shield_pos) < shield_width/2
///    - NOW we've got a hit.
///      - The only issue with this, is that, at the edge of the shield, it will
///        look like only the center of the projectile has a collision with the
///        shield, *which is good enough*.
/// - The direction vector is perpendicular to the shield's representative line.
///   - To obtain this perpendicular line, we can simply swap the coordinates.
pub fn hits_shield(shield_position: Vector2, shield_direction: Vector2, projectile_position: Vector2, shield_width: f32, projectile_hit_radius: f32) -> bool {
  // get the position of the projectile relative to the shield's origin
  let relative_projectile_pos: Vector2 = Vector2::difference(shield_position, projectile_position);
  // check that the relative distance is shorter than the shield's width/2.
  // if it is, we're in range to check for hits
  if relative_projectile_pos.magnitude() < shield_width/2.0 {
    // get the perpendicular line that represents our shield.
    // this was originally (x, y) = (-y, x), but that didn't work, for some reason.
    // This seems to work however. Why? Don't know, don't care.
    let shield_line: Vector2 = Vector2 { x: -shield_direction.x, y: shield_direction.y }.normalize();
    // calculate the perpendicular distance between the shield line and the projectile
    let perpendicular_distance =
      (f32::abs(shield_line.y * relative_projectile_pos.y - shield_line.x * relative_projectile_pos.x)) /
      (f32::sqrt(f32::powi(shield_line.x, 2) + f32::powi(shield_line.y, 2)));

    if perpendicular_distance < projectile_hit_radius {
      return true
    }
  }
  return false;
}

/// Applies modifications to players and game objects as a result of
/// bullet behaviour this frame, for a specific bullet. This dumbed
/// down version is only to be used for character primaries.
/// 
/// This is a wrapper for `apply_simple_bullet_logic_extra`, with just
/// the simplest parameters.
pub fn apply_simple_bullet_logic(
  players:     MutexGuard<Vec<ServerPlayer>>,
  characters:            HashMap<Character, CharacterProperties>,
  game_objects:          Vec<GameObject>,
  o_index:     usize,
  true_delta_time:       f64,
  pierceing_shot:        bool,
) -> (MutexGuard<Vec<ServerPlayer>>, Vec<GameObject>, bool) {
  return apply_simple_bullet_logic_extra(players, characters, game_objects, o_index, true_delta_time, pierceing_shot, 255, 255, false, f32::INFINITY, f32::INFINITY);
}

/// Applies modifications to players and game objects as a result of
/// bullet behaviour this frame, for a specific bullet.
/// 
/// Set `special_samage` to `255` to use default character damage number.
/// Same with `special_healing`. Setting it to 0 will nullify it.
/// Set `special_speed` to f32::INFINITY to use default.
pub fn apply_simple_bullet_logic_extra(
  mut players: MutexGuard<Vec<ServerPlayer>>,
  characters:            HashMap<Character, CharacterProperties>,
  mut game_objects:      Vec<GameObject>,
  o_index:     usize,
  true_delta_time:       f64,
  pierceing_shot:        bool,
  special_damage:        u8,
  special_healing:       u8,
  ricochet:              bool,
  special_speed:         f32,
  special_hit_radius:    f32,
) -> (MutexGuard<Vec<ServerPlayer>>, Vec<GameObject>, bool) {
  let game_object = game_objects[o_index].clone();
  let mut bullet_data = game_object.get_bullet_data();
  let owner_username = bullet_data.owner_username.clone();
  let player = players[index_by_username(&owner_username, players.clone())].clone();
  let character = player.character;
  let character_properties = characters[&character].clone();
  let wall_hit_radius: f32 = character_properties.primary_wall_hit_radius;
  
  let hit_radius: f32;

  if special_hit_radius == f32::INFINITY {
    hit_radius = character_properties.primary_hit_radius;
  } else {
    hit_radius = special_hit_radius;
  }

  let bullet_speed: f32;
  if special_speed == f32::INFINITY { bullet_speed  = character_properties.primary_shot_speed}
  else                    {bullet_speed = special_speed}
  
  // Set special values
  let damage: u8;
  if special_damage == 255 { damage = character_properties.primary_damage; }
  else                     { damage = special_damage; }
  
  let healing: u8;
  if special_damage == 255 { healing = character_properties.primary_heal; }
  else                     { healing = special_healing; }
  
  // Temporary. To be improved later.
  let wall_damage = (damage as f32 * character_properties.wall_damage_multiplier) as u8;
  
  // Calculate collisions with walls
  for victim_object_index in 0..game_objects.len() {
    // if it's a wall
    if WALL_TYPES.contains(&game_objects[victim_object_index].object_type) {
      // if it's colliding
      if !ricochet || (ricochet && bullet_data.hitpoints == 0){
        let distance = Vector2::distance(game_object.position, game_objects[victim_object_index].position);
        let buffer = 0.5 * 1.2;
        if distance < (TILE_SIZE*buffer + wall_hit_radius) {
          // delete the bullet
          game_objects[o_index].to_be_deleted = true;
          // damage the wall if it's not unbreakable
          if game_objects[victim_object_index].object_type != GameObjectType::UnbreakableWall {
            let mut wall_data = game_objects[victim_object_index].get_wall_data();
            if wall_data.hitpoints < wall_damage {
              game_objects[victim_object_index].to_be_deleted = true;
            } else {
              wall_data.hitpoints -= wall_damage;
            }
            game_objects[victim_object_index].extra_data = ObjectData::WallData(wall_data)
          }
          return (players, game_objects, false); // return early
        }
      }

      if ricochet && bullet_data.hitpoints > 0 {
        let pos = game_object.position;
        let direction = bullet_data.direction;
        let check_distance = TILE_SIZE * 0.1;
        let buffer = 0.5 * 1.6;
        let check_position: Vector2 = Vector2 {
          x: pos.x + direction.x * check_distance ,
          y: pos.y + direction.y * check_distance ,
        };
        let distance = Vector2::distance(check_position, game_objects[victim_object_index].position);
        if distance < (TILE_SIZE*buffer + wall_hit_radius) {
          
          if bullet_data.hitpoints == 0 {
            return (players, game_objects, false);
          }
          if bullet_data.hitpoints != 0 {
            bullet_data.hitpoints = 0;
            // we need to flip x direction or flip y direction
            // |distance.x| - |distance.y|
            // negative => flip horizontal
            let distance = Vector2::difference(game_object.position, game_objects[victim_object_index].position);
            if f32::abs(distance.x) - f32::abs(distance.y) > 0.0 {
              // flip horizontal
              // also check that we're going in opposing directions :)
              if bullet_data.direction.x * -distance.x < 0.0 {
                bullet_data.direction.x *= -1.0;
              }
            } else {
              // flip vertical
              // also check that we're going in opposing directions :)
              if bullet_data.direction.y * -distance.y < 0.0 {
                bullet_data.direction.y *= -1.0;
              }
            }
            //game_objects[o_index].lifetime = characters[&character].primary_range / characters[&character].primary_shot_speed;
            game_objects[o_index].position.x += bullet_data.direction.x * (TILE_SIZE*0.3); // this might break at very low freq like 26Hz
            game_objects[o_index].position.y += bullet_data.direction.y * (TILE_SIZE*0.3);

            // reset the lifetime (which in turn resets its range)
            let distance = characters[&character].primary_range;
            let speed = bullet_speed;
            bullet_data.lifetime = distance / speed;
            game_objects[o_index].to_be_deleted = false;
          }
        }
      }
    }
  }

  let mut hit: bool = false; // whether we've hit something

  // orb
  for victim_object_index in 0..game_objects.len() {
    if game_objects[victim_object_index].object_type == GameObjectType::CenterOrb {
      // if it's colliding
      let distance = Vector2::distance(game_object.position, game_objects[victim_object_index].position);
      if distance < (0.5 * TILE_SIZE + wall_hit_radius) {
        if bullet_data.hit_players.contains(&548) {
          continue;
        }
        let mut direction: Vector2 = bullet_data.direction;
        direction.x *= damage as f32 / 2.0;
        direction.y *= damage as f32 / 2.0;

        let mut orb_wall_data = game_objects[victim_object_index].get_wall_data();
        if orb_wall_data.hitpoints > damage {
          // hurt the orb :(
          orb_wall_data.hitpoints -= damage
        } else {
          // KILL THE ORB
          orb_wall_data.hitpoints = 0;
          game_objects[victim_object_index].to_be_deleted = true;
        }
        // apply knockback to the orb
        game_objects[victim_object_index].position.y += direction.y;
        game_objects[victim_object_index].position.x += direction.x;
        // apply orb healing
        if orb_wall_data.hitpoints == 0 {
          let team = player.team;
          for p_index in 0..players.len() {
            if players[p_index].team == team {
              players[p_index].health += ORB_HEALING;
              if players[p_index].health > 100 {
                players[p_index].health = 100;
              }
            }
          }
        }
        // 548 IS THE NUMBER OF THE ORB
        bullet_data.hit_players.push(548);
        if !pierceing_shot {
          game_objects[o_index].to_be_deleted = true;
        }
        hit = true;
        game_objects[victim_object_index].extra_data = ObjectData::WallData(orb_wall_data)
      }
    }
  }

  // Calculate collisions with players
  for p_index in 0..players.len() {
    if players[p_index].is_dead {
      continue; // skip dead player
    }
    // If we hit a bloke
    if Vector2::distance(game_object.position, players[p_index].position) < hit_radius &&
    owner_username != players[p_index].username {
      // And if we didn't hit this bloke before
      if !(bullet_data.hit_players.contains(&p_index)) {
        // Apply bullet damage
        if players[p_index].team != player.team {
          // Confirmed hit.
          hit = true;
          players[p_index].damage(damage, characters.clone());
          bullet_data.hit_players.push(p_index);
          // Destroy the bullet if it doesn't pierce.
          if !pierceing_shot {
            game_objects[o_index].to_be_deleted = true;
          }
        }
        // Apply bullet healing, only if in the same team
        if players[p_index].team == player.team && healing > 0 {
          players[p_index].heal(healing, characters.clone());
          bullet_data.hit_players.push(p_index);
          // Destroy the bullet if it doesn't pierce.
          if !pierceing_shot {
            game_objects[o_index].to_be_deleted = true;
          }
        }
        // Apply appropriate secondary charge
        let owner_index = index_by_username(&owner_username, players.clone());
        players[owner_index].add_charge(character_properties.secondary_hit_charge);
      }
    }
  }
  game_objects[o_index].position.x += bullet_data.direction.x * true_delta_time as f32 * bullet_speed;
  game_objects[o_index].position.y += bullet_data.direction.y * true_delta_time as f32 * bullet_speed;
  bullet_data.traveled_distance += true_delta_time as f32 * bullet_speed;
  // update the game object's bullet data.
  game_objects[o_index].extra_data = ObjectData::BulletData(bullet_data);
  return (players, game_objects, hit);
}


// (vscode) MARK: Other

/// A no dependency random function that returns a random value between 0.0 and 1.0
pub fn crappy_random() -> f64 {
  return (f64::sin(SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .expect("idk clock error").as_nanos() as f64
  )
  + 1.0)/2.0;
}