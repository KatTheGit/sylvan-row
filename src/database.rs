use std::{any::Any, fs};
use opaque_ke::{ServerRegistration, ServerSetup};
use crate::const_params::DefaultCipherSuite;
use redb::{Database, Error, TableDefinition, TableError};
use rand_core::OsRng;

const DATABASE_LOCATION: &str = "./database";
const SERVER_SETUP_LOCATION: &str = "./server_setup";
const TABLEDEF: TableDefinition<&str, &[u8]> = TableDefinition::new("my_data");
const TABLE_DEF: TableDefinition<&str, u32> = TableDefinition::new("cool and awesome database");

/// Returns the PlayerData for the given username in the given database.
pub fn get_player(database: &Database, username: &str) -> Result<PlayerData, Error> {
  let read_txn = database.begin_read()?;
  let table = read_txn.open_table(TABLEDEF)?;
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
    let mut table = write_txn.open_table(TABLEDEF)?;
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
  let table = match read_txn.open_table(TABLEDEF) {
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
    Err(err) => {
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