use opaque_ke::generic_array::GenericArray;
use opaque_ke::{errors::ProtocolError, CipherSuite};
use rand_core::RngCore;
use ::rand::rngs::OsRng;
use ring::hkdf;
use opaque_ke::{
    ClientLogin, ClientLoginFinishParameters, ClientRegistration,
    ClientRegistrationFinishParameters, CredentialFinalization, CredentialRequest,
    CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload, ServerLogin,
    ServerLoginParameters, ServerRegistration, ServerRegistrationLen, ServerSetup,
};
use sylvan_row::database::{self, PlayerData};
use sylvan_row::const_params::DefaultCipherSuite;
use chacha20poly1305::{
  aead::{Aead, AeadCore, KeyInit},
  ChaCha20Poly1305, Nonce
};

fn main() -> Result<(), ProtocolError> {
  // setup (server)
  let mut rng = OsRng;
  let server_setup = ServerSetup::<DefaultCipherSuite>::new(&mut rng);

  // REGISTER

  // client step 1
  let mut client_rng = OsRng;
  let client_registration_start_result =
    ClientRegistration::<DefaultCipherSuite>::start(&mut client_rng, b"toyota")?;

  // server step 2
  let server_registration_start_result = ServerRegistration::<DefaultCipherSuite>::start(
    &server_setup,
    client_registration_start_result.message,
    b"ornito",
  )?;

  // client step 3
  let client_registration_finish_result = client_registration_start_result.state.finish(
    &mut client_rng,
    b"toyota",
    server_registration_start_result.message,
    ClientRegistrationFinishParameters::default(),
  )?;

  // server step 4
  let password_file = ServerRegistration::<DefaultCipherSuite>::finish(
    client_registration_finish_result.message,
  );

  let password_file_bytes = ServerRegistration::<DefaultCipherSuite>::serialize(&password_file);

  let mut database = database::load().expect("oops");
  database::create_player(&mut database, "ornito", PlayerData { password_hash: password_file, wins: 0});
  let player = database::get_player(&mut database, "ornito").expect("oops");
  let password = player.password_hash;


  // LOGIN

  // client step 1
  let mut client_rng = OsRng;
  let client_login_start_result = ClientLogin::<DefaultCipherSuite>::start(&mut client_rng, b"toyota")?;

  // server step 2
  let password_file = password; //ServerRegistration::<DefaultCipherSuite>::deserialize(&password_file_bytes)?;
  let mut server_rng = OsRng;
  let server_login_start_result = ServerLogin::start(
      &mut server_rng,
      &server_setup,
      Some(password_file),
      client_login_start_result.message,
      b"ornito",
      ServerLoginParameters::default(),
  )?;
  
  // client step 3
  let client_login_finish_result = client_login_start_result.state.finish(
    &mut client_rng,
    b"toyota",
    server_login_start_result.message,
    ClientLoginFinishParameters::default(),
  )?;

  // server step 4
  let server_login_finish_result = server_login_start_result.state.finish(
    client_login_finish_result.message,
    ServerLoginParameters::default(),
  )?;
  let pake_key = server_login_finish_result.session_key;

  println!("{:?}\n{:?}", client_login_finish_result.session_key, server_login_finish_result.session_key);


  //let key = ChaCha20Poly1305::generate_key(&mut OsRng);
  
  

  // Shrink PAKE key
  let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, &[]);
  let prk = salt.extract(&pake_key);
  let okm = prk.expand(&[], hkdf::HKDF_SHA256).unwrap();
  let mut key_bytes = [0u8; 32];
  okm.fill(&mut key_bytes).unwrap();
  let key = GenericArray::from_slice(&key_bytes);

  println!("{:?}", key);
  // Create nonce
  let nonce_counter: u32 = 0;
  let mut nonce_bytes = [0u8; 12];
  nonce_bytes[8..].copy_from_slice(&nonce_counter.to_be_bytes());
  let nonce = Nonce::from_slice(&nonce_bytes);
  
  // Cipher
  let cipher = ChaCha20Poly1305::new(&key);
  let ciphertext = cipher.encrypt(&nonce, b"plaintext message".as_ref()).expect("shit");
  let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref()).expect("shit");
  
  // always 4 bytes
  println!("{:?}", bincode::serialize(&1));
  println!("{:?}", bincode::deserialize::<u32>(&[1, 0, 12, 41]));


  assert_eq!(&plaintext, b"plaintext message");


  return Ok(());
}