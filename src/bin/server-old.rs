use sylvan_row::{common, const_params::*, gamedata::Character, gameserver, mothership_common::*} ;
use std::{sync::{Arc, Mutex}, thread::{JoinHandle}};
// https://tokio.rs/tokio/tutorial/ this documentation is so peak
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::mpsc, net::{TcpListener}};

#[tokio::main]
async fn main() {


  let mut identifier_counter: usize = 0;
  let listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", SERVER_PORT)).await.unwrap();

  let players: Vec<PlayerInfo> = Vec::new();
  // Arc allows for shared access, and Mutex makes it mutually exclusive.
  let players = Arc::new(Mutex::new(players));

  // Contains all threads running game servers
  let fleet: Vec<JoinHandle<()>> = Vec::new();
  let fleet = Arc::new(Mutex::new(fleet));

  let main_players = Arc::clone(&players);
  loop {

    // Accept a new peer.
    let (mut socket, _addr) = listener.accept().await.unwrap();
    // Create the channels to communicate to this thread.
    let (tx, mut rx): (mpsc::Sender<PlayerMessage>, mpsc::Receiver<PlayerMessage>)
      = mpsc::channel(32);
    // Create the identifier.
    let identifier = identifier_counter; identifier_counter += 1;
    
    // Store the information.
    {
      let mut players = main_players.lock().unwrap();
      players.push(PlayerInfo { username: String::from(identifier.to_string()), channel: tx, queued: false, queued_gamemodes: Vec::new(), selected_character: Character::Hernani, session_key: String::from("") });
    }
  
    
    // for simplicity's sake these will be referred to as threads
    // in code and comments.
    let local_players = Arc::clone(&players);
    let local_fleet = Arc::clone(&fleet);
    tokio::spawn(async move {
      let self_identifier = identifier;
      let mut buffer = [0; 2048];
      loop {
        // this thing is really cool and handles whichever branch is ready first
        tokio::select! {
          // wait until we recieve packet, and write it to buffer.
          socket_read = socket.read(&mut buffer) => {
            match socket_read {
              Ok(0) => {
                {
                  // The player disconnected. remove them from the list.
                  let mut players = local_players.lock().unwrap();
                  for t_index in 0..players.len() {
                    //if players[t_index].username == self_identifier {
                    //  players.remove(t_index);
                    //  break;
                    //}
                  }
                }
                return
              }
              Ok(_len) => { }
              Err(err) => {
                println!("ERROR: {:?}", err);
                return; // An error happened. We should probably inform the client later, and log this.
              }
            };
            // handle the packet
            let packet = bincode::deserialize::<ClientToServerPacket>(&buffer);
            match packet {
              Ok(packet) => {
                match packet.information {
                  // MARK: Match Request
                  ClientToServer::MatchRequest(data) => {
                    println!("Match request");
                    let players_copy: Vec<PlayerInfo>;
                    let mut players_to_match: Vec<usize> = Vec::new();

                    {
                      // add the player to the queue
                      let mut players = local_players.lock().unwrap();
                      // this player's index
                      let p_index = from_id(self_identifier, players.clone());
                      players[p_index].queued = true;
                      if data.gamemodes.len() <= 2{ players[p_index].queued_gamemodes = data.gamemodes; }
                      players[p_index].selected_character = data.character;

                      // finally
                      players_copy = players.clone();
                      // do matchmaking checks
                      let mut queued_1v1: Vec<usize> = Vec::new();
                      let mut queued_2v2: Vec<usize> = Vec::new();
                      for player_index in 0..players_copy.len() {
                        if !players_copy[player_index].queued { continue; }
                        if players_copy[player_index].queued_gamemodes.contains(&GameMode::Standard2V2) {
                          queued_2v2.push(player_index);
                        }
                        if players_copy[player_index].queued_gamemodes.contains(&GameMode::Standard1V1) {
                          queued_1v1.push(player_index);
                        }
                      }
                      if queued_2v2.len() >= 4 {
                        queued_2v2.truncate(4);
                      players_to_match = queued_2v2;
                      }
                      else if queued_1v1.len() >= 2 {
                        queued_1v1.truncate(2);
                        players_to_match = queued_1v1;
                      }
                      for matched_player in players_to_match.clone() {
                        players[matched_player].queued = false;
                      }
                    }
                    // Create a game
                    if !players_to_match.is_empty() {
                      let port = common::get_random_port();
                      {
                        let mut fleet = local_fleet.lock().unwrap();
                        let mut player_info = Vec::new();
                        for player_index in 0..players_copy.len() {
                          if players_to_match.contains(&player_index) {
                            player_info.push(players_copy[player_index].clone())
                          }
                        }
                        fleet.push(
                          std::thread::spawn(move || {
                            gameserver::game_server(player_info.len(), port, player_info);
                          }
                        ));
                      }
                      for pm_index in players_to_match {
                        players_copy[pm_index].channel.send(PlayerMessage::GameAssigned(
                          MatchAssignmentData {
                            port: port,
                          }
                        )).await.unwrap();
                      }
                    }
                  }
                  // MARK: Match Cancel
                  ClientToServer::MatchRequestCancel => {
                    {
                      let mut players = local_players.lock().unwrap();
                      // player's index
                      let p_index = from_id(self_identifier, players.clone());
                      players[p_index].queued = false;
                    }
                  }
                }
              }
              Err(err) => {
                println!("ERROR: {:?}", err)
              }
            }
          }

          thread_message = rx.recv() => {
            if let Some(message) = thread_message {
              match message {
                PlayerMessage::GameAssigned(data) => {
                  socket.write_all(
                    &bincode::serialize::<ServerToClientPacket>(
                      &ServerToClientPacket {
                        information: ServerToClient::MatchAssignment(
                          MatchAssignmentData { port: data.port }
                        )
                      }
                    ).expect("oops")
                  ).await.unwrap();
                },
              }
            }
          }
        }
      }
    });
  }
}


/// returns the index of a player in a list of players by its ID.
fn from_id(id: usize, players: Vec<PlayerInfo>) -> usize {
  for p_index in 0..players.len() {
    if players[p_index].username == String::from(id.to_string()) {
      return p_index;
    }
  }
  // this should never happen.
  println!("ERROR: from_id could not find player");
  return 0;
}