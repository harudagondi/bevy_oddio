use bevy::asset::{AssetLoader, BoxedFuture, Error, LoadContext, LoadedAsset};
use minimp3::Decoder;

use crate::{frames::Stereo, AudioSource};

#[derive(Default)]
pub struct Mp3Loader;

impl AssetLoader for Mp3Loader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let mut decoder = Decoder::new(bytes);

            let mut samples = Vec::new();

            let mut current_sample_rate = 44100;

            loop {
                match decoder.next_frame() {
                    Ok(minimp3::Frame {
                        data, sample_rate, ..
                    }) => {
                        samples.push([data[0], data[1]]);
                        current_sample_rate = sample_rate;
                    }
                    Err(minimp3::Error::Eof) => break,
                    Err(err) => return Err(err.into()),
                }
            }

            let convert_i32_to_f32 = |int| {
                if int < 0 {
                    (f32::from(int)) / -(f32::from(i16::MIN))
                } else {
                    (f32::from(int)) / f32::from(i16::MAX)
                }
            };

            let samples: Vec<Stereo> = samples
                .into_iter()
                .map(|[l, r]| [convert_i32_to_f32(l), convert_i32_to_f32(r)])
                .map(Stereo::from)
                .collect();

            #[allow(clippy::cast_sign_loss)]
            let frames = oddio::Frames::from_iter(current_sample_rate as u32, samples);

            let audio_source = AudioSource { frames };

            load_context.set_default_asset(LoadedAsset::new(audio_source));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mp3"]
    }
}
