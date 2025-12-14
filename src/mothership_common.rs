use opaque_ke::{generic_array::GenericArray, CredentialFinalization, CredentialRequest, CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload};
use chacha20poly1305::{
  aead::{Aead, KeyInit},
  ChaCha20Poly1305, Nonce
};
use crate::{common, const_params::DefaultCipherSuite, database::FriendShipStatus, gamedata::Character};

// MARK: CLIENT to server
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct ClientToServerPacket {
  /// Actual packet contents. Can be a match request, a chat message, anything.
  pub information: ClientToServer,
}
impl ClientToServerPacket {
  /// Creates a data structure containing the nonce in its first 4 bytes
  /// and the encrypted packet in the remainder.
  pub fn cipher(&self, nonce: u32, key: Vec<u8>) -> Vec<u8> {
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[8..].copy_from_slice(&nonce.to_be_bytes());
    let formatted_nonce = Nonce::from_slice(&nonce_bytes);
    
    let key = GenericArray::from_slice(&key);
    let cipher = ChaCha20Poly1305::new(&key);

    let serialized_nonce = bincode::serialize::<u32>(&nonce).expect("oops");
    let serialized_packet = bincode::serialize::<ClientToServerPacket>(&self).expect("oops");
    let ciphered = cipher.encrypt(&formatted_nonce, serialized_packet.as_ref()).expect("shit");
    return [&serialized_nonce[..], &ciphered[..]].concat();
  }
}
impl ServerToClientPacket {
  /// Creates a data structure containing the nonce in its first 4 bytes
  /// and the encrypted packet in the remainder.
  pub fn cipher(&self, nonce: u32, key: Vec<u8>) -> Vec<u8> {
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[8..].copy_from_slice(&nonce.to_be_bytes());
    let formatted_nonce = Nonce::from_slice(&nonce_bytes);
    
    let key = GenericArray::from_slice(&key);
    let cipher = ChaCha20Poly1305::new(&key);

    let serialized_nonce = bincode::serialize::<u32>(&nonce).expect("oops");
    let serialized_packet = bincode::serialize::<ServerToClientPacket>(&self).expect("oops");
    let ciphered = cipher.encrypt(&formatted_nonce, serialized_packet.as_ref()).expect("shit");
    return [&serialized_nonce[..], &ciphered[..]].concat();
  }
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub enum ClientToServer {
  MatchRequest(MatchRequestData),
  MatchRequestCancel,
  /// User requests their statistics
  PlayerDataRequest,
  /// User requests a list of friends/pending/blocked players
  GetFriendList,
  /// User wants to send a friend request to the user in the `String`.
  SendFriendRequest(String),
  ///// User wants to accept the friend request of the user in the `String`.
  //AcceptFriendRequest(String),
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
  /// An error occured and the server refused to comply, for
  /// the reason described by the `RefusalReason`
  /// 
  /// This can be user error or a server error.
  InteractionRefused(RefusalReason),
  PlayerDataResponse(PlayerStatistics),
  /// Contains a list of the user's friends/pending/blocked, as requested
  /// by the user.
  /// - `String` for the username
  /// - `FriendShipStatus` for the... friendship status
  /// - `bool` for whether the player is online
  FriendListResponse(Vec<(String, FriendShipStatus, bool)>),
  FriendRequestSuccessful,
  FriendshipSuccessful,
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct PlayerStatistics {
  /// Victory count in standard gamemodes.
  pub wins: u16,
}
impl PlayerStatistics {
  pub fn new() -> PlayerStatistics {
    return PlayerStatistics {
      wins: 0,
    };
  }
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub enum RefusalReason {
  /// Someone already owns this username.
  UsernameTaken,
  /// Attempted username does not exist.
  UsernameInexistent,
  /// Any error that is entirely (or mostly) the server's fault.
  InternalError,
  /// Contains inappropriate words, symbols, etc...
  InvalidUsername,
  /// This friend request was already made.
  FriendRequestAlreadySent,
  /// This friend request was useless since users are already friends.
  AlreadyFriends,
  /// This request failed because the users are blocked.
  UsersBlocked,
  /// That's you, dummy!
  ThatsYouDummy,
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
  pub session_key: Vec<u8>,
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
  pub assigned_team: common::Team,
}

/// Possible messages between player threads.
#[derive(PartialEq, Clone, Debug)]
pub enum PlayerMessage {
  /// This thread must stop now.
  ForceDisconnect,
  SendPacket(ServerToClientPacket),
}

// filters out shit
pub fn profanity_filter() {

}