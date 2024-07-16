use top_down_shooter::common::*;
use macroquad::prelude::*;
use gilrs::*;
use std::net::UdpSocket;
use std::time::*;
use bincode;

/// stores all game objects. Recieved from server, rendered by client.
static mut GAME_STATE: Vec<GameObject> = Vec::new();
static mut PLAYERS: Vec<ClientPlayer> = Vec::new();
static mut SELF: ClientPlayer = ClientPlayer {
  health: 100,
  position: Vec2 { x: 0.0, y: 0.0 },
  aim_direction: Vec2 { x: 0.0, y: 0.0 },
  character: Character::SniperGirl,
  secondary_charge: 0,
};

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
  let server_ip = "0.0.0.0";
  let server_ip: String = format!("{}:{}", server_ip, SERVER_LISTEN_PORT);
  let listening_ip: String = format!("0.0.0.0:{}", CLIENT_LISTEN_PORT);
  let sending_ip: String = format!("0.0.0.0:{}", CLIENT_SEND_PORT);
  let socket = UdpSocket::bind(sending_ip).expect("Could not bind client socket");
  let listening_socket = UdpSocket::bind(listening_ip).expect("Could not bind server socket");


  let mut gilrs = Gilrs::new().unwrap();
  let mut active_gamepad = None;

  // Iterate over all connected gamepads
  for (_id, gamepad) in gilrs.gamepads() {
    println!("GAME: {} is {:?}", gamepad.name(), gamepad.power_info());
  }

  // MARK: Network thread
  // listens to server packets
  std::thread::spawn(move || {
    let mut buffer = [0; 1024];
    loop {
      // recieve packet
      let (amt, _src) = listening_socket.recv_from(&mut buffer).expect(":(");
      let data = &buffer[..amt];
      let recieved_server_info: ServerPacket = bincode::deserialize(data).expect("awwww");
      // println!("CLIENT: Received from {}: {:?}", src, recieved_server_info);

      // if we sent an illegal position, and server does a position override:
      if recieved_server_info.player_packet_is_sent_to.override_position {
        // then correct our position
        unsafe {
          SELF.position = recieved_server_info.player_packet_is_sent_to.position_override.as_vec2();
        }
      }

      unsafe {
        GAME_STATE = recieved_server_info.game_objects;
        PLAYERS = Vec::new();
        for player in recieved_server_info.players {
          PLAYERS.push(ClientPlayer {
            health: player.health,
            position: player.position.as_vec2(),
            aim_direction: player.aim_direction.as_vec2(),
            character: Character::SniperGirl, // temporary
            secondary_charge: 255, // temporary
          });
        }
      }
    }
  });

  // used to only send information every once in a while instead of each frame
  let mut networking_counter = Instant::now();

  // MARK: Game Loop
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
    while let Some(Event { id, event: _, time: _ }) = gilrs.next_event() {
      // println!("CLIENT: {:?} New event from {}: {:?}", time, id, event);
      active_gamepad = Some(id);
    }
    
    let mut movement_vector: Vec2 = Vec2::new(0.0, 0.0);

    unsafe {
    if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
      match gamepad.axis_data(Axis::RightStickX)  {
        Some(axis_data) => {
          SELF.aim_direction.x = axis_data.value();
        } _ => {}
      }
      match gamepad.axis_data(Axis::RightStickY)  {
        Some(axis_data) => {
          SELF.aim_direction.y = -axis_data.value();
        } _ => {}
      }
      match gamepad.axis_data(Axis::LeftStickX)  {
        Some(axis_data) => {
          movement_vector.x = (((axis_data.value() * 5.0) as i32) as f32 / 5.0);
        } _ => {}
      }
      match gamepad.axis_data(Axis::LeftStickY)  {
        Some(axis_data) => {
          // crazy rounding shenanigans to round to closest multiple of 0.2
          movement_vector.y = (((-axis_data.value() * 5.0) as i32) as f32 / 5.0);
          println!("{}", axis_data.value());
        } _ => {}
      }
    }}
    println!("raw: {}", movement_vector);
    if movement_vector.length() > 0.0 {
      movement_vector = movement_vector.normalize();
    }
    println!("normal: {}", movement_vector);
    movement_vector *= movement_speed * get_frame_time();
    println!("multiplied: {}", movement_vector);
    
    unsafe {
      SELF.position.x += movement_vector.x;
      SELF.position.y += movement_vector.y;
      if SELF.aim_direction.length() < controller_deadzone {
        SELF.aim_direction = Vec2 {x: 0.0, y: 0.0};
      }
    }

    clear_background(BLACK);
    unsafe {
      // the real background
      draw_rectangle(0.0, 0.0, 100.0 * VW, 100.0 * VH, WHITE)          
    }

    unsafe {
      SELF.draw();
      SELF.draw_crosshair();

      for player in PLAYERS.clone() {
        player.draw();
      }
      // for game_object in GAME_STATE.clone() {
      //   match game_object.object_type {
      //     GameObjectType::SniperGirlBullet => {
      // 
      //     }
      //     GameObjectType::UnbreakableWall => {
      // 
      //     }
      //     GameObjectType::Wall => {
      // 
      //     }
      //   }
      // }
    }

    // unsafe {println!("{:?}", GAME_STATE);}

    // everything under this block only happens at 100Hz
    if networking_counter.elapsed().as_secs_f64() > MAX_PACKET_INTERVAL {
      // reset counter
      networking_counter = Instant::now();

      // do all networking logic
      unsafe {
        let client_packet: ClientPacket = ClientPacket {
          position:      Vector2 {x: SELF.position.x, y: SELF.position.y },
          aim_direction: Vector2 { x: SELF.aim_direction.x, y: SELF.aim_direction.y },
          shooting: false,
          shooting_secondary: false,
        };
        
        let serialized: Vec<u8> = bincode::serialize(&client_packet).expect("Failed to serialize message");
        socket.send_to(&serialized, server_ip.clone()).expect("Failed to send packet to server.");
      }
    }


    // show fps and await next frame
    draw_text(format!("{} fps", 1.0/get_frame_time()).as_str(), 20.0, 20.0, 20.0, DARKGRAY);
    next_frame().await;
  }
}