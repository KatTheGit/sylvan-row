use rustrict::*;

/// Conditions for validity:
/// - 8 characters or more
pub fn valid_password(text: &str) -> bool {
  if text.len() < 8 {
    return false;
  }
  return true;
}
/// Conditions for validity:
/// - 3-20 characters
/// - Does not contain `:`
/// - Does not contain swears and slurs.
pub fn valid_username(text: &str) -> bool {
  if text.len() < 3 {
    return false;
  }
  if text.len() > 20 {
    return false;
  }
  if text.contains(":") {
    return false;
  }
  return true;
}


pub enum ProfanityLevel {
  Swears,
  Slurs,
  SlursAndSwears,
  None,
}

pub fn contains_profanity(text: &str, level: ProfanityLevel) -> bool {

  match level {
    ProfanityLevel::Swears => {
      return text.is(Type::SEVERE)
    }
    ProfanityLevel::SlursAndSwears => {
      return text.is(Type::ANY)
    }
    ProfanityLevel::Slurs => {
      return text.is(Type::OFFENSIVE)
    }
    ProfanityLevel::None => {
      return text.is(Type::NONE)
    }
  }
}

pub fn censor_profanity(text: &str, level: ProfanityLevel) -> String {
  match level {
    ProfanityLevel::Slurs => {
      return Censor::from_str(text)
        .with_censor_threshold(Type::OFFENSIVE)
        .censor();
    }
    ProfanityLevel::SlursAndSwears => {
      return Censor::from_str(text)
        .with_censor_threshold(Type::ANY)
        .censor();
    }
    ProfanityLevel::Swears => {
      return Censor::from_str(text)
        .with_censor_threshold(Type::SEVERE)
        .censor();
    }
    ProfanityLevel::None => {
      return text.to_string();
    }
  }
}