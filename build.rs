// Thank you https://stackoverflow.com/questions/30291757/attaching-an-icon-resource-to-a-rust-application
use std::{ env, io, };
use winresource::*;
fn main() -> io::Result<()> {
  if env::var_os("CARGO_CFG_WINDOWS").is_some() {
    winresource::WindowsResource::new()
      // This path can be absolute, or relative to your crate root.
      .set_icon("assets/icon/icon.ico")
      .set_language(0x0009)
      .set("FileDescription", "Sylvan Row game client")
      .set("LegalCopyright", "KatTheGit")
      .set("ProductName", "Sylvan Row")
      .set("ProductVersion", "hi :3")
      .compile()?;
  }
  Ok(())
}