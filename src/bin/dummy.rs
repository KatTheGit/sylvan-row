use sylvan_row::ui::{load_password, save_password};

fn main() {
  save_password("shit", "bob", &mut Vec::new());
  let password = load_password("bob");
  println!("{}", password);
}