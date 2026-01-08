use opaque_ke::{CredentialFinalization, CredentialRequest, CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload};
use crate::{common, const_params::DefaultCipherSuite, database::FriendShipStatus, gamedata::Character};

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
  /// User requests their statistics
  PlayerDataRequest,
  /// User requests a list of friends/pending/blocked players
  GetFriendList,
  /// User wants to send a friend request to the user in the `String`.
  /// 
  /// Also used to accept friend requests.
  SendFriendRequest(String),
  /// User wants to send a chat message (String 2) to a recipient (String 1).
  SendChatMessage(String, String),
  LobbyInvite(String),
  LobbyInviteAccept(String),
  LobbyLeave,
  // LOGIN
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
  /// The user recieved a chat message (String 2) from a sender (String 1), of type ChatMessageType
  ChatMessage(String, String, ChatMessageType),
  /// Lobby invitation from a user (String)
  LobbyInvite(String),
  /// Update users about everyone else in the lobby.
  LobbyUpdate(Vec<LobbyPlayerInfo>)
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct LobbyPlayerInfo {
  pub username: String,
  // pub character: Character,   maybe later :)
  pub is_ready: bool,
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
pub enum ChatMessageType {
  /// Private message
  Private,
  /// Message broadcast to the whole group (lobby)
  Group,
  /// Message broadcast to the whole in-game team
  Team,
  /// Message broadcast to every played in the current game.
  All,
  /// Message sent by the server itself.
  Administrative,
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
  /// The user isn't currently online.
  UserNotOnline,
  /// The user's request is invalidated because the concerned peer
  /// is not a friend.
  NotFriends,
  /// If the invite was invalid in any way.
  InvalidInvite,
  /// You are already in a party and cannot join another party.
  AlreadyInPary,
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct MatchAssignmentData {
  pub port: u16,
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
  /// Whether this user is the party owner. True by default.
  pub is_party_leader: bool,
  /// All users that are in the party owned by this user.
  pub queued_with: Vec<String>,
  /// All users that have invited this user.
  pub invited_by: Vec<String>,
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