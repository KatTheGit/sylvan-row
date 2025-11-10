// MARK: Vectors
// utility
use macroquad::math::Vec2;
use std::time::SystemTime;
use crate::const_params::*;
use crate::gamedata::*;

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

// (vscode) MARK: Other

/// A no dependency crappy random function that returns a random value between 1 and -1.
pub fn crappy_random() -> f64 {
  return f64::sin(SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .expect("idk clock error").as_millis() as f64
  );
}