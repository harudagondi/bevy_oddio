use bevy::asset::{AssetLoader, BoxedFuture, Error, LoadContext, LoadedAsset};
use minimp3::Decoder;

use crate::AudioSource;

#[derive(Default)]
pub struct Mp3Loader;

impl AssetLoader for Mp3Loader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let decoder = Decoder::new(bytes);

            let samples = Vec::new();

            let mut sample_rate;

            loop {
                match decoder.next_frame() {
                    Ok(minimp3::Frame {
                        data, sample_rate, ..
                    }) => samples.push([data[0], data[1]]),
                    Err(minimp3::Error::Eof) => break,
                    Err(err) => return Err(err.into()),
                }
            }

            let convert_i32_to_f32 = |int| {
                if int < 0 {
                    (int as f32) / -(i16::MIN as f32)
                } else {
                    (int as f32) / (i16::MAX as f32)
                }
            };

            let mut samples: Vec<[f32; 2]> = samples
                .into_iter()
                .map(|[l, r]| [convert_i32_to_f32(l), convert_i32_to_f32(r)])
                .collect();

            let frames = oddio::Frames::from_iter(sample_rate, samples);

            let audio_source = AudioSource { frames };

            load_context.set_default_asset(LoadedAsset::new(audio_source));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mp3"]
    }
}
