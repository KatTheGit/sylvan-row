use top_down_shooter::common::*;
use macroquad::prelude::*;
use gilrs::*;
use std::net::UdpSocket;
use std::time::*;
use bincode;

#[macroquad::main("Game")]
async fn main() {
  game().await;
}

async fn game() {

  //temporary
  let controller_deadzone: f32 = 0.2;

  // temporary
  let movement_speed: f32 = 100.0;

  // temporary
  let server_ip: &str = "0.0.0.0:25567";
  let socket = UdpSocket::bind("0.0.0.0:0").expect("CLIENT: could not bind socket idk");

  // all temporary
  let mut player: ClientPlayer = ClientPlayer {
    health: 100,
    position: Vec2 { x: 0.0, y: 0.0 },
    aim_direction: Vec2 { x: 0.0, y: 0.0 },
    textures: vec![Texture2D::from_file_with_format(include_bytes!("../../assets/player/player1.png"), None)],
    secondary_charge: 0,
  };

  let mut gilrs = Gilrs::new().unwrap();
  let mut active_gamepad = None;

  // Iterate over all connected gamepads
  for (_id, gamepad) in gilrs.gamepads() {
    println!("GAME: {} is {:?}", gamepad.name(), gamepad.power_info());
  }

  // used to only send information every once in a while instead of each frame
  let mut networking_counter = Instant::now();

  loop {
    unsafe {
      if screen_height() * (16.0/9.0) > screen_width() {
        VW = screen_width() / 100.0;
        VH = VW / (16.0/9.0);
      } else {
        VH = screen_height() / 100.0;
        VW = VH * (16.0/9.0);
      }
    }

    // Examine new events
    while let Some(Event { id, event, time }) = gilrs.next_event() {
      // println!("CLIENT: {:?} New event from {}: {:?}", time, id, event);
      active_gamepad = Some(id);
    }
    // You can also use cached gamepad state
    if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
      match gamepad.axis_data(Axis::RightStickX)  {
        Some(axis_data) => {
          if axis_data.value().abs() > controller_deadzone {
            player.aim_direction.x = axis_data.value();
          } else {
            player.aim_direction.x = 0.0;
          }
        } _ => {}
      }
      match gamepad.axis_data(Axis::RightStickY)  {
        Some(axis_data) => {
          if axis_data.value().abs() > controller_deadzone {
            player.aim_direction.y = -axis_data.value();
          } else {
            player.aim_direction.y = 0.0;
          }
        } _ => {}
      }
      match gamepad.axis_data(Axis::LeftStickX)  {
        Some(axis_data) => {
          player.position.x += (((axis_data.value() * 5.0) as i32) as f32 / 5.0) * movement_speed * get_frame_time();
        } _ => {}
      }
      match gamepad.axis_data(Axis::LeftStickY)  {
        Some(axis_data) => {
          // crazy rounding shenanigans to round to closest multiple of 0.2
          player.position.y += (((-axis_data.value() * 5.0) as i32) as f32 / 5.0) * movement_speed * get_frame_time();
        } _ => {}
      }
    }

    clear_background(BLACK);
    unsafe {
      // the real background
      draw_rectangle(0.0, 0.0, 100.0 * VW, 100.0 * VH, WHITE)          
    }

    player.draw();
    player.draw_crosshair();

    // everything under if block only happens 10 times per second
    if networking_counter.elapsed().as_millis() > 100 {
      // reset
      networking_counter = Instant::now();

      // do all networking logic
      let client_packet: ClientPacket = ClientPacket {
        position: Vector2 {x: player.position.x, y: player.position.y },
        aim_direction: Vector2 { x: player.aim_direction.x, y: player.aim_direction.y },
        shooting: false,
        shooting_secondary: false,
      };
  
      let serialized = bincode::serialize(&client_packet).expect("Failed to serialize message");
      socket.send_to(&serialized, server_ip).expect("idc");
    }


    // show fps and await next frame
    draw_text(format!("{} fps", 1.0/get_frame_time()).as_str(), 20.0, 20.0, 20.0, DARKGRAY);
    next_frame().await;
  }
}

fn draw_image(texture: &Texture2D, x: f32, y: f32, w: f32, h: f32) {
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