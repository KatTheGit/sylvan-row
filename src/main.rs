// hides the console window.
//#![windows_subsystem = "windows"]
pub fn main() {
  #[cfg(not(target_os="android"))]
  {
    #[cfg(target_os="windows")]
    std::env::set_var("WGPU_BACKEND", "dx12");
    sylvan_row::main();
  }
}