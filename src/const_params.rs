use crate::gamedata::GameObjectType;
use opaque_ke::CipherSuite;

pub const ORB_HEALING: u8 = 20;

pub const WALL_HP: u8 = 30;

pub const WALL_TYPES: [GameObjectType; 3] = [GameObjectType::Wall, GameObjectType::UnbreakableWall, GameObjectType::HernaniWall];
pub const WALL_TYPES_ALL: [GameObjectType; 5] = [GameObjectType::Wall, GameObjectType::UnbreakableWall, GameObjectType::HernaniWall, GameObjectType::Water1, GameObjectType::Water2];

// debug constants. disable ALL for prod
/// Whether to spawn one dummy in the game
pub const SPAWN_DUMMY: bool = false;
/// Whether the server allows to start a game with only 1 player in queue
pub const MATCHMAKE_ALONE: bool = false;

pub const ROUNDS_TO_WIN: u8 = 2; // 2 = best of 3

/// The amount of time the game server waits for players to connect.
pub const MATCH_WAIT_TIME: f32 = 3.0;
/// The point in the match where players are incentivised to hurry up.
pub const MATCH_HURRY_UP_TIME: f32 = 90.0;
/// The maximum time a match can last. At this time, terminate the lobby after deciding a winner.
pub const MATCH_FORCE_END_TIME: f32 = 180.0;

pub const RATE_LIMIT_THRESHOLD: u8 = 50;

/// Time interval representing our network rate.
pub const PACKET_INTERVAL: f32 = 1.0 / 30.0;

/// how many packets are averaged when calculating legality of player position.
pub const PACKET_AVERAGE_SAMPLES: u8 = 5;
/// Port the server is hosted on. Used by server, and by the client to set the
/// default address of the server.
pub const SERVER_PORT: u16 = 25569;
/// Default IP to be used when there's an issue with the moba_ip.txt file.
pub const DEFAULT_SERVER_IP: &str = "13.38.240.14"; // my AWS instance's address

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
pub struct DefaultCipherSuite;

impl CipherSuite for DefaultCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = opaque_ke::argon2::Argon2<'static>; // argon2 is safer
}