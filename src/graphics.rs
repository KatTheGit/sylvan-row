use macroquad::prelude::*;
use std::collections::HashMap;
use crate::gamedata::*;
use strum::IntoEnumIterator;
use crate::maths::*;
use crate::common;

#[macro_export]
macro_rules! load {
  ($file:expr $(,)?) => {
    Texture2D::from_file_with_format(include_bytes!(concat!("../assets/", $file)), None)
  };
}
pub fn load_game_object_textures() -> HashMap<GameObjectType, Texture2D>  {
  let mut game_object_tetures: HashMap<GameObjectType, Texture2D> = HashMap::new();
  for game_object_type in GameObjectType::iter() {
    game_object_tetures.insert(
      game_object_type,
      match game_object_type {
        GameObjectType::Wall                              => load!("gameobjects/wall.png"),
        GameObjectType::HernaniWall                       => load!("characters/hernani/textures/wall.png"),
        GameObjectType::RaphaelleAura                     => load!("characters/raphaelle/textures/secondary.png"),
        GameObjectType::UnbreakableWall                   => load!("gameobjects/unbreakable_wall.png"),
        GameObjectType::HernaniBullet                     => load!("characters/hernani/textures/bullet.png"),
        GameObjectType::RaphaelleBullet                   => load!("characters/raphaelle/textures/bullet.png"),
        GameObjectType::RaphaelleBulletEmpowered          => load!("characters/raphaelle/textures/bullet-empowered.png"),
        GameObjectType::CynewynnSword                     => load!("characters/cynewynn/textures/bullet.png"),
        GameObjectType::HernaniLandmine                   => load!("characters/hernani/textures/trap.png"),
        GameObjectType::ElizabethProjectileRicochet       => load!("characters/hernani/textures/bullet.png"),
        GameObjectType::ElizabethProjectileGround         => load!("characters/hernani/textures/trap.png"),
        GameObjectType::ElizabethProjectileGroundRecalled => load!("characters/hernani/textures/trap.png"),
        GameObjectType::ElizabethTurret                   => load!("ui/temp_ability_1.png"),
        GameObjectType::ElizabethTurretProjectile         => load!("characters/hernani/textures/bullet.png"),
        GameObjectType::Grass1                            => load!("gameobjects/grass-1.png"),
        GameObjectType::Grass2                            => load!("gameobjects/grass-2.png"),
        GameObjectType::Grass3                            => load!("gameobjects/grass-3.png"),
        GameObjectType::Grass4                            => load!("gameobjects/grass-4.png"),
        GameObjectType::Grass5                            => load!("gameobjects/grass-5.png"),
        GameObjectType::Grass6                            => load!("gameobjects/grass-6.png"),
        GameObjectType::Grass7                            => load!("gameobjects/grass-7.png"),
        GameObjectType::Grass1Bright                      => load!("gameobjects/grass-1-b.png"),
        GameObjectType::Grass2Bright                      => load!("gameobjects/grass-2-b.png"),
        GameObjectType::Grass3Bright                      => load!("gameobjects/grass-3-b.png"),
        GameObjectType::Grass4Bright                      => load!("gameobjects/grass-4-b.png"),
        GameObjectType::Grass5Bright                      => load!("gameobjects/grass-5-b.png"),
        GameObjectType::Grass6Bright                      => load!("gameobjects/grass-6-b.png"),
        GameObjectType::Grass7Bright                      => load!("gameobjects/grass-7-b.png"),
        GameObjectType::Water1                            => load!("gameobjects/water-edge.png"),
        GameObjectType::Water2                            => load!("gameobjects/water-full.png"),
        GameObjectType::CenterOrb                         => load!("gameobjects/orb.png"),
        GameObjectType::CenterOrbSpawnPoint               => load!("empty.png"),
        GameObjectType::WiroShield                        => load!("ui/temp_ability_1.png"),
        GameObjectType::WiroGunShot                       => load!("ui/temp_ability_1.png"),
        GameObjectType::WiroDashProjectile                => load!("empty.png"),
        GameObjectType::TemerityRocket                    => load!("ui/temp_ability_1.png"),
        GameObjectType::TemerityRocketSecondary           => load!("ui/temp_ability_1.png"),
      }
    );
  }
  return game_object_tetures;
}
pub fn load_character_textures() -> HashMap<Character, Texture2D> {
  let mut player_textures = HashMap::new();
  for character in Character::iter() {
    player_textures.insert(
      character,
      match character {
        Character::Cynewynn  => load!("characters/cynewynn/textures/main.png"),
        Character::Raphaelle => load!("characters/raphaelle/textures/main.png"),
        Character::Hernani => load!("characters/hernani/textures/main.png"),
        Character::Elizabeth => load!("characters/dummy/textures/template.png"),
        Character::Wiro      => load!("characters/dummy/textures/template.png"),
        Character::Dummy      => load!("characters/dummy/textures/template.png"),
        Character::Temerity      => load!("characters/dummy/textures/template.png"),
      }
    );
  }
  return player_textures;
}

// MARK: Draw
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
  macroquad::prelude::draw_rectangle(relative_position_x1*vh, relative_position_y1*vh, w*vh, h*vh, color);
}
pub fn draw_rectangle(position: Vector2, size: Vector2, color: Color) {
  macroquad::prelude::draw_rectangle(position.x, position.y, size.x, size.y, color);
}

pub fn draw_text_relative(text: &str, x: f32, y:f32, font: &Font, font_size: u16, vh: f32, center_position: Vector2, color: Color) -> () {
  let relative_position_x = x - center_position.x + (50.0 * (16.0/9.0)); //+ ((vh * (16.0/9.0)) * 100.0 )/ 2.0;
  let relative_position_y = y - center_position.y + 50.0; //+ (vh * 100.0) / 2.0;
  draw_text_ex(text, relative_position_x * vh, relative_position_y * vh, TextParams { font: Some(font), font_size: (font_size as f32 * vh) as u16, font_scale: 1.0, font_scale_aspect: 1.0, rotation: 0.0, color });
}

pub fn draw_lines(positions: Vec<Vector2>, camera: Vector2, vh: f32, team: common::Team, y_offset: f32, alpha: f32) -> () {
  if positions.len() < 2 { return; }
  for position_index in 0..positions.len()-1 {
    draw_line_relative(positions[position_index].x, positions[position_index].y + y_offset, positions[position_index+1].x, positions[position_index+1].y + y_offset, 0.4, match team {common::Team::Blue => Color { r: 0.2, g: 1.0-(position_index as f32 / positions.len() as f32), b: 0.8, a: alpha }, common::Team::Red => Color { r: 0.8, g: 0.7-0.3*(position_index as f32 / positions.len() as f32), b: 0.2, a: alpha }}, camera, vh);
  }
  // let texture = Texture2D::from_file_with_format(include_bytes!("../../assets/gameobjects/tq-flashback.png"), None  );
  // draw_image_relative(&texture, positions[0].x - TILE_SIZE/2.0, positions[0].y - (TILE_SIZE*1.5)/2.0, TILE_SIZE, TILE_SIZE * 1.5, vh, camera);
}