use std::fs;
use opaque_ke::{ServerRegistration, ServerSetup};
use crate::const_params::DefaultCipherSuite;
use redb::{Database, Error, TableDefinition};
use rand_core::OsRng;

const DATABASE_LOCATION: &str = "./database";
const SERVER_SETUP_LOCATION: &str = "./server_setup";
const USER_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("User Data");
/// The u8 stores the FriendShipStatus enum.
const FRIENDS_TABLE: TableDefinition<&str, u8> = TableDefinition::new("Friendship Data"); // friendship is magic

/// Returns the PlayerData for the given username in the given database.
pub fn get_player(database: &Database, username: &str) -> Result<PlayerData, Error> {
  let read_txn = database.begin_read()?;
  let table = read_txn.open_table(USER_TABLE)?;
  let data = table.get(username)?;
  if let Some(data) = data {
    let deserialized = bincode::deserialize::<PlayerData>(data.value());
    match deserialized {
      Ok(data) => {
        return Ok(data);
      }
      Err(err) => {
        println!("{:?}", err);
        return Err(redb::Error::PreviousIo);
      }
    }
  } else {
    return Err(redb::Error::PreviousIo);
  }
}
/// Creates a new entry in the database for this player.
/// 
/// If player already exists, replaces data.
pub fn create_player(database: &mut Database, username: &str, player_data: PlayerData) -> Result<(), Error>{
  let write_txn = database.begin_write()?;
  {
    let mut table = write_txn.open_table(USER_TABLE)?;
    let serialized = match bincode::serialize(&player_data) {
      Ok(data) => {
        data
      },
      Err(err) => {
        // idk ngl
        println!("{:?}", err);
        return Err(redb::Error::PreviousIo); // tf
      }
    };
    table.insert(username, serialized.as_slice())?;
  }
  write_txn.commit()?;
  return Ok(());
}
/// Checks if an username is already in use in the database.
pub fn username_taken(database: &Database, username: &str) -> Result<bool, Error> {
  let read_txn = database.begin_read()?;
  let table = match read_txn.open_table(USER_TABLE) {
    Ok(table) => {table},
    Err(_err) => {
      return Ok(false);
    }
  };
  let taken: bool = table.get(username)?.is_some();
  return Ok(taken);
}
/// Loads the database struct. If the database doesn't exist, create a file
/// at DATABASE_LOCATION.
/// 
/// Also compacts the database.
pub fn load() -> Result<Database, Error> {
  let mut database = Database::create(DATABASE_LOCATION)?;
  let _ = database.compact();
  return Ok(database);
}

pub fn load_server_setup() -> Result<ServerSetup<DefaultCipherSuite>, Error> {
  let file_exists: bool;
  match fs::exists(SERVER_SETUP_LOCATION) {
    Ok(exists) => {
      file_exists = exists;
    }
    Err(_err) => {
      file_exists = false;
    }
  }
  // load file
  if file_exists {
    let serialized = fs::read(SERVER_SETUP_LOCATION)?;
    let server_setup = match ServerSetup::<DefaultCipherSuite>::deserialize(&serialized) {
      Ok(data) => { data },
      Err(err) => {
        println!("{:?}", err);
        return Err(redb::Error::PreviousIo);
      }
    };
    return Ok(server_setup);
  }
  // create new and store
  else {
    let mut rng = OsRng;
    let server_setup = ServerSetup::<DefaultCipherSuite>::new(&mut rng);
    let serialized = server_setup.serialize().to_vec();
    fs::write(SERVER_SETUP_LOCATION, serialized)?;
    return Ok(server_setup);
  }

}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerData {
  /// Sensitive password hash used by PAKE protocol.
  pub password_hash: ServerRegistration<DefaultCipherSuite>,
  /// Win count
  pub wins: usize,
}
impl PlayerData {
  pub fn new(password_hash: ServerRegistration<DefaultCipherSuite>) -> PlayerData {
    return PlayerData {
      password_hash,
      wins: 0,
    }
  }
}

// MARK: Friends

/// Deterministically generates a key with two usernames.
/// 
/// - "Alice", "bob" => "Alice:bob"
/// - "Zack", "Kat" => "Kat:Zack"
pub fn generate_friendship_key(username_a: &str, username_b: &str) -> String {
  if username_a > username_b {
    return format!("{}:{}", username_b, username_a);
  } else {
    return format!("{}:{}", username_a, username_b);
  }
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum FriendShipStatus {
  /// These users are friends :p
  Friends     = 0,
  /// UserA needs to answer to UserB's friend request
  PendingForA = 1,
  /// UserB needs to answer to UserA's friend request.
  PendingForB = 2,
  /// These users are enemies but maybe one day they'll be lovers.
  Blocked     = 3,
}
impl FriendShipStatus {
  pub fn from_u8(num: u8) -> FriendShipStatus {
    return match num {
      0 => FriendShipStatus::Friends,
      1 => FriendShipStatus::PendingForA,
      2 => FriendShipStatus::PendingForB,
      _ => FriendShipStatus::Blocked,
    }
  }
}
pub fn get_friend_request_type(requesting_user: &str, answering_user: &str) -> FriendShipStatus {
  if requesting_user > answering_user {
    return FriendShipStatus::PendingForA;
  } else {
    return FriendShipStatus:: PendingForB;
  }
}
/// Sets the friend status for a username pair.
pub fn set_friend_status(database: &Database, username_a: &str, username_b: &str, friendship_status: FriendShipStatus) -> Result<(), Error> {
  let write_txn = database.begin_write()?;
  {
    let mut table = write_txn.open_table(FRIENDS_TABLE)?;
    let key = generate_friendship_key(username_a, username_b);
    let friendship_status: u8 = friendship_status as u8;
    table.insert(key.as_str(), friendship_status)?;
  }
  write_txn.commit()?;
  return Ok(());
}
/// Returns the friendship status for the given username pair
pub fn get_friend_status(database: &Database, username_a: &str, username_b: &str) -> Result<FriendShipStatus, Error>  {
  let read_txn = database.begin_read()?;
  let table = read_txn.open_table(FRIENDS_TABLE)?;
  let key = generate_friendship_key(username_a, username_b);
  let data = table.get(key.as_str())?;
  if let Some(status) = data {
    let status: u8 = status.value();
    let status: FriendShipStatus = FriendShipStatus::from_u8(status);
    return Ok(status);
  }
  // If there is no friendship data
  return Err(Error::Corrupted(String::from("norelation")));
}
/// Returns all stored friendship statuses for the username given.
pub fn get_status_list(database: &Database, username: &str) -> Result<Vec<(String, FriendShipStatus)>, Error> {
  let read_txn = database.begin_read()?;
  let table = read_txn.open_table(FRIENDS_TABLE)?;
  let prefix = format!("{}:", username);
  let suffix = format!(":{}", username);
  let mut prefix_values = table.range(prefix.as_str()..)?;
  let mut suffix_values = table.range(..suffix.as_str())?;
  let mut values = Vec::new();
  while let Some(value) = prefix_values.next() {
    let value = value?;
    if !value.0.value().starts_with(&prefix) {
      break;
    }
    values.push(value);
  }
  while let Some(value) = suffix_values.next() {
    let value = value?;
    if value.0.value().ends_with(&suffix) {
      values.push(value);
    }
  }
  let mut friendship_statuses: Vec<(String, FriendShipStatus)> = Vec::new();
  for value in values {
    let friendship_status: (String, FriendShipStatus) = (value.0.value().to_string(), FriendShipStatus::from_u8(value.1.value()));
    friendship_statuses.push(friendship_status);
  }
  return Ok(friendship_statuses);
}