use crate::gamedata::Character;

// CLIENT to server
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct ClientToServerPacket {
  /// Actual packet contents. Can be a match request, a chat message, anything.
  pub information: ClientToServer,
  /// User's auth token. Not in use right now.
  pub identifier: u64,
  /// The port the client is listening on.
  pub port: u16,
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub enum ClientToServer {
  MatchRequest(MatchRequestData),
  MatchRequestCancel,
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct MatchRequestData {
  /// List of requested gamemodes.
  pub gamemode: Vec<GameMode>,
  pub character: Character,
}

// SERVER to client
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub enum ServerToClient {
  MatchAssignment(MatchAssignmentData),
  MatchMakingInformation(MatchMakingData),
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct MatchAssignmentData {
  pub port: u16,
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct MatchMakingData {
  pub queue_size: u8,
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct ServerToClientPacket {
  pub information: ServerToClient,
}

// OTHER
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub enum GameMode {
  Standard1V1,
  Standard2V2,
}
/// Data stored for each player in queue.
#[derive(PartialEq, Clone, Debug)]
pub struct QueuedPlayer {
  pub ip: String,
  pub id: u64,
  pub requested_gamemode: Vec<GameMode>,
  pub character: Character,
}