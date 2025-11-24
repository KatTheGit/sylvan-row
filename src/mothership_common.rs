use opaque_ke::{CredentialFinalization, CredentialRequest, CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload};

use crate::{const_params::DefaultCipherSuite, gamedata::Character};

// MARK: CLIENT to server
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct ClientToServerPacket {
  /// Actual packet contents. Can be a match request, a chat message, anything.
  pub information: ClientToServer,
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub enum ClientToServer {
  MatchRequest(MatchRequestData),
  MatchRequestCancel,
  RegisterRequestStep1(String, RegistrationRequest<DefaultCipherSuite>),
  RegisterRequestStep2(RegistrationUpload<DefaultCipherSuite>),
  LoginRequestStep1(String, CredentialRequest<DefaultCipherSuite>),
  LoginRequestStep2(CredentialFinalization<DefaultCipherSuite>),
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct MatchRequestData {
  /// List of requested gamemodes.
  pub gamemodes: Vec<GameMode>,
  pub character: Character,
}

// MARK: SERVER to client
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub enum ServerToClient {
  MatchAssignment(MatchAssignmentData),
  RegisterResponse1(RegistrationResponse<DefaultCipherSuite>),
  RegisterSuccessful,
  LoginResponse1(CredentialResponse<DefaultCipherSuite>),
  LoginResponse2,
  /// String contains reason for refusal, to be displayed
  AuthenticationRefused(RefusalReason),
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub enum RefusalReason {
  /// Someone already owns this username.
  UsernameTaken,
  /// Any error that is entirely the server's fault.
  InternalError,
  /// Contains inappropriate words, symbols, etc...
  InvalidUsername,
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct MatchAssignmentData {
  pub port: u16,
}
//#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
//pub struct MatchMakingData {
//  pub queue_size: u8,
//}
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

/// contains the channel and identifier of a player thread.
#[derive(Clone, Debug)]
pub struct PlayerInfo {
  pub username: String,
  pub session_key: String,
  /// The channel other threads can use to communicate with this player's
  /// associated thread.
  pub channel: tokio::sync::mpsc::Sender<PlayerMessage>,
  /// Whether the player is in a queue.
  /// 
  /// When returned by the game server, this flag actually
  /// represends whether the player won (true) or lost (false)
  pub queued: bool,
  /// Will be truncated if longer than the total amount of gamemodes.
  pub queued_gamemodes: Vec<GameMode>,
  pub selected_character: Character,
}

/// Possible messages between player threads.
#[derive(PartialEq, Clone, Debug)]
pub enum PlayerMessage {
  GameAssigned(MatchAssignmentData),
}

// filters out
pub fn profanity_filter() {

}