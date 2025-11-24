use opaque_ke::{errors::ProtocolError, CipherSuite};
use rand_core::RngCore;
use ::rand::rngs::OsRng;
use opaque_ke::{
    ClientLogin, ClientLoginFinishParameters, ClientRegistration,
    ClientRegistrationFinishParameters, CredentialFinalization, CredentialRequest,
    CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload, ServerLogin,
    ServerLoginParameters, ServerRegistration, ServerRegistrationLen, ServerSetup,
};
use sylvan_row::database::{self, PlayerData};
use sylvan_row::const_params::DefaultCipherSuite;


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

  println!("{:?}\n{:?}", client_login_finish_result.session_key, server_login_finish_result.session_key);

  return Ok(());
}