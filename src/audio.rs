use kira::*;
use kira::sound::static_sound::StaticSoundData;
use std::io::Cursor;

pub fn play_sound(sound: &[u8], track: &mut track::TrackHandle) {
  let cursor = Cursor::new(sound.to_owned());
  let sound_data = StaticSoundData::from_cursor(cursor.clone()).expect("oops");
  track.play(sound_data).expect("oops");
}
/// volume is a value between 0.0-100.0
pub fn set_volume(volume: f32, track: &mut track::TrackHandle) {
  let decibels_min = -25.0;
  let decibels_max = 0.0;
  let decibel_volume = if volume == 0.0 {-1000.0} else {decibels_min + (volume / 100.0) * (decibels_max - decibels_min)};
  track.set_volume(Decibels(decibel_volume), Tween::default());
}