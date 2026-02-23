use std::io::{Read, Write};
use std::fs::File;
use crate::const_params::*;
use opaque_ke::generic_array::GenericArray;
use chacha20poly1305::{
  aead::{Aead, KeyInit},
  ChaCha20Poly1305, Nonce
};

/// Encodes data to be sent over TCP
/// ```
///   u32     T
/// [length][data]
/// ```
/// 
/// note: `length` does not include the u32 header.
pub fn tcp_encode<T: serde::Serialize>(data: T) -> Result<Vec<u8>, bincode::Error> {
  // serialize
  let serialized_data = bincode::serialize::<T>(&data)?;
  // set up header
  let data_len: u32 = serialized_data.len() as u32;
  let data_len_serialized = bincode::serialize::<u32>(&data_len).expect("a");
  // create the packet
  let mut encoded_packet = data_len_serialized;
  encoded_packet.extend(serialized_data.iter());

  return Ok(encoded_packet);
}

/// Encodes and encrypts data to be sent over TCP
/// ```
///   u32     u32    T
/// [length][nonce][data]
/// ```
/// 
/// note: `length` does not include the u32 header and the u32 nonce
pub fn tcp_encode_encrypt<T: serde::Serialize>(data: T, key: Vec<u8>, nonce: u32) -> Result<Vec<u8>, bincode::Error> {
  // get nonce
  let mut nonce_bytes = [0u8; 12];
  nonce_bytes[8..].copy_from_slice(&nonce.to_be_bytes());
  let formatted_nonce = Nonce::from_slice(&nonce_bytes);
  // set up cipher
  let key = GenericArray::from_slice(&key);
  let cipher = ChaCha20Poly1305::new(&key);
  // encrypt
  let serialized_packet = bincode::serialize::<T>(&data)?;
  let ciphered = cipher.encrypt(&formatted_nonce, serialized_packet.as_ref()).expect("shit");
  // set up header
  let data_len: u32 = ciphered.len() as u32;
  let data_len_serialized = bincode::serialize::<u32>(&data_len).expect("a");
  let serialized_nonce = bincode::serialize::<u32>(&nonce).expect("oops");
  // create the packet
  let mut encoded_packet = data_len_serialized;
  encoded_packet.extend(serialized_nonce.iter());
  encoded_packet.extend(ciphered.iter());

  return Ok(encoded_packet);
}

/// is given the whole buffer, decodes every packet it can find inside,
/// and returns a list of decoded packets.
/// 
/// counterpart to `tcp_encode`
/// 
/// If any packet is erroneous, it will ignore the rest of the buffer.
pub fn tcp_decode<T: serde::de::DeserializeOwned>(mut data: Vec<u8>) -> Result<Vec<T>, bincode::Error> {
  let mut decoded_packets: Vec<T> = Vec::new();

  while data.len() >= 4 {
    let len = bincode::deserialize::<u32>(&data[..4])? as usize;
    if len > 2048 {
      break;
    }
    if len < 1 {
      break;
    }
    if let Some(data_to_decode) = data.get(4..len+4) {
      let packet = bincode::deserialize::<T>(data_to_decode)?;
      decoded_packets.push(packet);
      data.drain(0..len+4);
    } else {
      // the length prefix given was erroneous. Ignore everything.
      break;
    }
  }
  return Ok(decoded_packets);
}
/// is given the whole buffer, decodes and decrypts every packet
/// it can find inside, and returns a list of decoded packets.
/// 
/// counterpart to `tcp_encode_encrypt`
/// 
/// If any packet is erroneous, it will ignore the rest of the buffer.
pub fn tcp_decode_decrypt<T: serde::de::DeserializeOwned>(mut data: Vec<u8>, key: Vec<u8>, last_nonce: &mut u32) -> Result<Vec<T>, bincode::Error> {
  let mut decoded_packets: Vec<T> = Vec::new();

  while data.len() >= 8 {
    // length check
    let len = bincode::deserialize::<u32>(&data[..4])? as usize;
    if len > 2048 {
      break;
    }
    if len < 1 {
      break;
    }
    // nonce
    let recv_nonce = &data[4..8];
    let recv_nonce = match bincode::deserialize::<u32>(&recv_nonce){
      Ok(nonce) => nonce,
      Err(_) => {
        break;
      }
    };
    if recv_nonce <= *last_nonce {
      break;
    }
    let nonce_num = recv_nonce;
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[8..].copy_from_slice(&recv_nonce.to_be_bytes());
    let nonce_formatted = Nonce::from_slice(&nonce_bytes);
    // set up cipher
    let key = GenericArray::from_slice(key.as_slice());
    let cipher = ChaCha20Poly1305::new(key);

    if let Some(data_to_decode) = data.get(8..len+8) {
      
      let deciphered = match cipher.decrypt(&nonce_formatted, data_to_decode.as_ref()) {
        Ok(decrypted) => {
          // this is a valid packet, update last_nonce
          *last_nonce = nonce_num;
          decrypted
        },
        Err(_err) => {
          break;
        },
      };
      let packet = bincode::deserialize::<T>(&deciphered)?;
      decoded_packets.push(packet);
      data.drain(0..len+8);
    } else {
      // the length prefix given was erroneous. Ignore everything.
      break;
    }
  }
  return Ok(decoded_packets);
}

//pub fn udp_encode_encrypt<T>(data: T) {
//
//}
//pub fn udp_decode_decrypt<T>(data: Vec<u8>) {
//  
//}

pub fn get_ip() -> String {
  let mut server_ip: String;
  let ip_file_name = "moba_ip.txt";
  let ip_file = File::open(ip_file_name);
  let default_ip: String = format!("{}:{}", DEFAULT_SERVER_IP, SERVER_PORT);
  match ip_file {
    // file exists
    Ok(mut file) => {
      let mut data = vec![];
      match file.read_to_end(&mut data) {
        // could read file
        Ok(_) => {
          server_ip = String::from_utf8(data).expect("Couldn't read IP in file.");
          server_ip.retain(|c| !c.is_whitespace());
          // if smaller than smallest possible length: we have a problem (file might be empty)
          if server_ip.len() < String::from("0.0.0.0:0").len() {
            println!("IP address was invalid (are you using X.X.X.X:X format?). Defaulting to {}", default_ip);
            server_ip = default_ip;
          }
        }
        // couldnt read file
        Err(_) => {
          println!("Couldn't read IP. defaulting to {}.", default_ip);
          server_ip = default_ip;
        }
      }
    }
    // file doesn't exist
    Err(error) => {
      println!("Config file not found, attempting to creating one... Error: {}", error);
      match File::create(ip_file_name) {
        // Could create file
        Ok(mut file) => {
          let _ = file.write_all(default_ip.as_bytes());
          println!("Config file created with default ip {}", default_ip);
          server_ip = default_ip;
        }
        // Couldn't create file
        Err(error) => {
          println!("Could not create config file. Defaulting to {}\nReason:\n{}", default_ip, error);
          server_ip = default_ip;
        }
      }
    }
  }
  println!("{:?}", server_ip);
  return server_ip
}