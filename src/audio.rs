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
  // 40 * log10(x) - 80
  let decibel_volume = if volume == 0.0 {-100000.0} else {40.0 * f32::log10(volume) - 80.0};
  track.set_volume(Decibels(decibel_volume), Tween::default());
}