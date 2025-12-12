/// Conditions for validity:
/// - 8 characters or more
pub fn valid_password(text: String) -> bool {
  if text.len() < 8 {
    return false;
  }
  return true;
}
/// Conditions for validity:
/// - 3 characters or more
/// - Does not contain `:`
pub fn valid_username(text: String) -> bool {
  if text.len() < 3 {
    return false;
  }
  if text.contains(":") {
    return false;
  }
  return true;
}