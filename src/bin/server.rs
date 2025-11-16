use sylvan_row::common;
use sylvan_row::gameserver::game_server;
use sylvan_row::mothership_common::*;

use std::fmt::format;
use std::io::Write;
use std::{io::Read, net::{TcpListener, TcpStream}};

/// This is the mothership server code (aka matchmaking server).
/// 
/// Its job is to perform all necessary duties of a usual live-service game's
/// server (chat, queue, etc), and create instances of game servers.
/// 
/// The client and server communicate through TCP. Almost every interaction is
/// recieve-reply.
fn main() {

  let mut matchmaking_queue: Vec<QueuedPlayer> = Vec::new();

  // for now the "fleet" will be one single machine.
  let mut fleet: Vec<std::thread::JoinHandle<()>> = Vec::new();

  let tcp_listener = TcpListener::bind("127.0.0.1:25569")
    .expect("Coulnd't bind TcpListener.");

  loop {
    // If we've recieved packets, handle them.
    for stream in tcp_listener.incoming() {
      match stream {
        Ok(mut stream) => {
          // buffer to hold info
          let mut buffer: [u8; 1024] = [0; 1024];
          // read what the client sent
          stream.read(&mut buffer).expect("lol");
          match bincode::deserialize::<ClientToServerPacket>(&buffer) {
            // if it's valid
            Ok(data) => {
              // save the address
              let addr = stream.peer_addr().expect("Couldn't read peer address...");
              // and handle the packet.
              match data.information {
                // MARK: Matchmaking
                // matchmaking request. Add to the queue and check if a game can start.
                ClientToServer::MatchRequest(request) => {
                  // add to queue

                  // check that they aren't already in queue
                  for queued in matchmaking_queue.clone() {
                    if queued.ip == format!("{}:{}", addr.ip(), data.port)
                    && queued.id == data.identifier {
                      // if yes, ignore them.
                      continue;
                    }
                  }
                  // if valid, add to queue.
                  matchmaking_queue.push(
                    QueuedPlayer {
                      ip: format!("{}:{}", addr.ip(), data.port),
                      id: data.identifier,
                      requested_gamemode: request.gamemode,
                      character: request.character,
                    }
                  );
                  // do matchmaking checks.
                  // for now this is just a very basic algorithm. If a game can be formed, do it.
                  let mut queuers_1v1: Vec<usize> = Vec::new();
                  let mut queuers_2v2: Vec<usize> = Vec::new();
                  for player_index in 0..matchmaking_queue.len() {
                    if matchmaking_queue[player_index].requested_gamemode.contains(&GameMode::Standard1V1) {
                      queuers_1v1.push(player_index);
                    }
                    if matchmaking_queue[player_index].requested_gamemode.contains(&GameMode::Standard2V2) {
                      queuers_2v2.push(player_index);
                    }
                  }
                  let mut queuers: Vec<usize> = Vec::new();
                  if queuers_2v2.len() >= 4 {
                    queuers = queuers_2v2;
                  }
                  // give 2v2 priority
                  else if queuers_1v1.len() >= 2 {
                    queuers = queuers_1v1;
                  }
                  // if there are enough people in queue
                  if !queuers.is_empty() {
                    let player_count = queuers.len();
                    let port: u16 = common::get_random_port();
                    // add an instance of game server
                    let mut players = Vec::new();
                    for queuer_index in queuers.clone() {
                      players.push(matchmaking_queue[queuer_index].clone());
                    }
                    fleet.push(
                      std::thread::spawn(move || {
                        // run game server
                        // in the future pass this thread more information so we can like assign victories for
                        // stats and ranked
                        game_server(player_count, port, players);
                      })
                    );
                    
                    // send the first 4 players to the match
                    for index in 0..player_count {
                      let addr = matchmaking_queue[queuers[index]].ip.clone();
                      println!("{}", addr);
                      // send a match assignment packet.
                      let stream = TcpStream::connect(addr);
                      match stream {
                        Ok(mut stream) => {
                          let packet = ServerToClientPacket {
                            information: ServerToClient::MatchAssignment(
                              MatchAssignmentData {
                                port: port,
                              }
                            ),
                          };
                          match stream.write(&bincode::serialize(&packet).expect("idk")) {
                            Ok(_) => {},
                            Err(_) => {},
                          };
                        },
                        Err(_) => {
                          // Don't worry about it.
                          // In the future, this should probably cancel the match.
                        }
                      
                      }
                    }
                    // remove the players from the queue
                    let mut updated_queue: Vec<QueuedPlayer> = Vec::new();
                    for index in 0..matchmaking_queue.len() {
                      //matchmaking_queue.remove(queuers[index]);
                      if !queuers.contains(&index) {
                        updated_queue.push(matchmaking_queue[index].clone());
                      }
                    }
                    matchmaking_queue = updated_queue;
                  }
                },
                // QUEUE CANCELED!
                ClientToServer::MatchRequestCancel => {
                  for p_index in 0..matchmaking_queue.len() {
                    println!("{:?}  {:?}", matchmaking_queue[p_index].ip, addr.to_string());
                    if matchmaking_queue[p_index].ip == format!("{}:{}", addr.ip(), data.port)
                    && matchmaking_queue[p_index].id == data.identifier {
                      println!("Canceling request...");
                      matchmaking_queue.remove(p_index);
                      break;
                    }
                  }
                }
              }
            }
            Err(_) => {
              // If the packet is erroneous, ignore it.
              // In the future, this should probably inform the
              // client they're on the wrong game version.
            }
          };
        },

        Err(_error) => {
          // ignore
        }
      }
    println!("{:?}", matchmaking_queue);

    }
  }
}

