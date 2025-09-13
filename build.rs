// Thank you https://stackoverflow.com/questions/30291757/attaching-an-icon-resource-to-a-rust-application
use std::{ env, io, };
fn main() -> io::Result<()> {
  if env::var_os("CARGO_CFG_WINDOWS").is_some() {
    winresource::WindowsResource::new()
      // This path can be absolute, or relative to your crate root.
      .set_icon("assets/icon/icon.ico")
      .compile()?;
  }
  Ok(())
}