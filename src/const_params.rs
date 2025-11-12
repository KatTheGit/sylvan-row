use crate::gamedata::GameObjectType;

pub const TILE_SIZE: f32 = 8.0;
pub const ORB_HEALING: u8 = 20;

pub const WALL_HP: u8 = 30;

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

/// Time interval representing our network rate.
pub const PACKET_INTERVAL: f32 = 1.0 / 30.0;

/// how many packets are averaged when calculating legality of player position.
pub const PACKET_AVERAGE_SAMPLES: u8 = 5;
/// Port the server is hosted on. Used by server, and by the client to set the
/// default address of the server.
pub const SERVER_PORT:        u16 = 25569;
/// Default IP to be used when there's an issue with the moba_ip.txt file.
pub const DEFAULT_SERVER_IP: &str = "13.38.240.14"; // my AWS instance address
