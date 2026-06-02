use bevy::{audio::Volume, prelude::*};

use crate::{AudioParams, AudioTrack};

pub fn play_sound(sound: String, commands: &mut Commands, asset_server: AssetServer, volume: f32, track: AudioTrack) {
  let audio = asset_server.load(sound);
  match track {
    AudioTrack::Music => {
      commands.spawn(
        (
          AudioPlayer::new(
          audio
          ),
          PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: Volume::Linear(volume),
            ..Default::default()
          },
          MusicChannel,
        )
      );
    }
    AudioTrack::SoundEffectSelf => {
      commands.spawn(
        (
          AudioPlayer::new(
          audio
          ),
          PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: Volume::Linear(volume),
            ..Default::default()
          },
          SfxChannelSelf,
        )
      );
    }
    AudioTrack::SoundEffectOther => {
      commands.spawn(
        (
          AudioPlayer::new(
          audio
          ),
          PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: Volume::Linear(volume),
            ..Default::default()
          },
          SfxChannelOther,
        )
      );
    }
  }
}

/// Converts a volume ranging from 0-100 (100 being loudest) into decibels.
pub fn to_decibels(volume: f32) -> f32 {
  //https://www.desmos.com/calculator/ppy0gu3gah
  let decibel_volume = if volume == 0.0 {f32::NEG_INFINITY} else {40.0 * f32::log10(volume) - 80.0};
  return decibel_volume;
}

#[derive(PartialEq)]
pub enum SoundChannel {
  Music,
  Sfx,
}

// Define your channel markers
#[derive(Component)]
pub struct MusicChannel;

#[derive(Component)]
pub struct SfxChannelSelf;
#[derive(Component)]
pub struct SfxChannelOther;

// Set volume for everything on a channel
pub fn set_track_volume(
  channels: &mut AudioParams,
  track: AudioTrack,
  volume: f32,
) {
  match track {
    AudioTrack::Music => {
      for mut sink in &mut channels.music {
        sink.set_volume(Volume::Linear(volume));
      }
    }
    AudioTrack::SoundEffectOther => {
      for mut sink in &mut channels.sfx_other {
        sink.set_volume(Volume::Linear(volume));
      }
    }
    AudioTrack::SoundEffectSelf => {
      for mut sink in &mut channels.sfx_self {
        sink.set_volume(Volume::Linear(volume));
      }
    }
  }
}
pub fn stop_sounds_in_track(
  channels: &mut AudioParams,
  track: AudioTrack,
  commands: &mut Commands
) {
  match track {
    AudioTrack::Music => {
      for sink in &channels.entity_music {
        commands.entity(sink).despawn();
      }
    }
    AudioTrack::SoundEffectOther => {
      for sink in &channels.entity_sfx_other {
        commands.entity(sink).despawn();
      }
    }
    AudioTrack::SoundEffectSelf => {
      for sink in &channels.entity_sfx_self {
        commands.entity(sink).despawn();
      }
    }
  }
}

//pub fn play_sound(sound: &[u8], track: &mut track::TrackHandle) {
//  let cursor = Cursor::new(sound.to_owned());
//  let sound_data = StaticSoundData::from_cursor(cursor.clone()).expect("oops");
//  track.play(sound_data).expect("oops");
//}
//// play a sound with a volume inversely proportional to `distance`.
//pub fn play_sound_distance(sound: &[u8], track: &mut track::TrackHandle, distance: f32) {
//  let cursor = Cursor::new(sound.to_owned());
//  let sound_data = StaticSoundData::from_cursor(cursor.clone()).expect("oops");
//  //https://www.desmos.com/calculator/ar7w5afy1s
//  // the range (in tiles) at which we hear the full sound
//  let full_sound_cutoff = 5.0;
//  // decibels the sound falls off per tile.
//  let sound_faloff_amplitude = 1.0;
//  let volume = if distance/TILE_SIZE < full_sound_cutoff {
//    0.0
//  } else {
//    - sound_faloff_amplitude *(distance/TILE_SIZE) + full_sound_cutoff * sound_faloff_amplitude
//  };
//  let sound_data = sound_data.volume(Decibels(volume));
//  track.play(sound_data).expect("oops");
//}
///// volume is a value between 0.0-100.0
//pub fn set_volume(volume: f32, track: &mut track::TrackHandle) {
//  // 40 * log10(x) - 80
//  let decibel_volume = if volume == 0.0 {-100000.0} else {40.0 * f32::log10(volume) - 80.0};
//  track.set_volume(Decibels(decibel_volume), Tween::default());
//}