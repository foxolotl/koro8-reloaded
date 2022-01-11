use rand::{RngCore, Rng};
use sdl2::{Sdl, mixer::{DEFAULT_FORMAT, DEFAULT_CHANNELS, Chunk, Sdl2MixerContext}, AudioSubsystem};

pub struct Buzzer {
    _audio: AudioSubsystem,
    _mixer: Sdl2MixerContext,
    chunks: Vec<(Chunk, u32)>,
    rng: Box<dyn RngCore>
}

impl Buzzer {
    pub fn new(sdl: &Sdl, sound_dir_path: &str, channels: i32, rng: Box<dyn RngCore>) -> Option<Buzzer> {
        let audio = sdl.audio().ok()?;
        sdl2::mixer::open_audio(44100, DEFAULT_FORMAT, DEFAULT_CHANNELS, 1024).ok()?;
        let mixer = sdl2::mixer::init(sdl2::mixer::InitFlag::MP3).ok()?;
        sdl2::mixer::allocate_channels(channels);

        let sound_dir = std::fs::read_dir(sound_dir_path).ok()?;
        let chunks: Vec<_> = sound_dir.filter_map(|file| {
            let path = file.ok()?.path();
            sdl2::mixer::Chunk::from_file(path).ok().map(|c| (c, 0))
        }).collect();

        let buzzer = Buzzer {
            _audio: audio,
            _mixer: mixer,
            chunks,
            rng
        };
        Some(buzzer)
    }
}

impl crate::arch::Buzzer for Buzzer {
    fn start(&mut self) {
        // Ensure that we get a good mix of sounds
        let min_play_count = self.chunks.iter().map(|(_, play_count)| *play_count).min_by(core::cmp::Ord::cmp).unwrap();
        let mut eligible_chunks: Vec<_> = self.chunks.iter_mut().filter(|(_, play_count)| *play_count == min_play_count).collect();

        let chunk_index = self.rng.gen::<usize>() % eligible_chunks.len();
        let chunk = &mut eligible_chunks[chunk_index];
        let _ = sdl2::mixer::Channel::all().play(&chunk.0, 0);
        chunk.1 += 1;
    }

    fn stop(&mut self) {
        // can't stop the korone
    }

    fn reset(&mut self) {
        // nothing to do here
    }
}
