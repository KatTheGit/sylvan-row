// hides the console window.
#![windows_subsystem = "windows"]
pub fn main() {
  #[cfg(not(target_os="android"))]
  sylvan_row::main();
}