use opaque_ke::ServerRegistration;
use crate::const_params::DefaultCipherSuite;
use redb::{Database, Error, ReadableTable, TableDefinition};

const DATABASE_LOCATION: &str = "./database";
const TABLEDEF: TableDefinition<&str, &[u8]> = TableDefinition::new("my_data");

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
pub fn username_taken(database: &mut Database, username: &str) -> Result<bool, Error> {
  let read_txn = database.begin_read()?;
  let table = read_txn.open_table(TABLEDEF)?;
  let taken: bool = table.get(username)?.is_some();
  return Ok(taken);
}
/// Loads the database struct. If the database doesn't exist, create a file
/// at DATABASE_LOCATION.
pub fn load() -> Result<Database, Error> {
  let database = Database::create(DATABASE_LOCATION)?;
  return Ok(database);
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