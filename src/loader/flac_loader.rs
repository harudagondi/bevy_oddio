use bevy::asset::{AssetLoader, BoxedFuture, Error, LoadContext, LoadedAsset};
use claxon::FlacReader;

use crate::{frames::Stereo, AudioSource};

#[derive(Default)]
pub struct FlacLoader;

impl AssetLoader for FlacLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        #[allow(clippy::cast_precision_loss)]
        Box::pin(async move {
            let mut reader = FlacReader::new(bytes)?;

            let sample_rate = reader.streaminfo().sample_rate;

            let samples: Vec<i32> = reader.samples().collect::<Result<_, _>>()?;

            let convert_i32_to_f32 = |int| {
                if int < 0 {
                    (int as f32) / -(i32::MIN as f32)
                } else {
                    (int as f32) / (i32::MAX as f32)
                }
            };

            let mut samples: Vec<f32> = samples.into_iter().map(convert_i32_to_f32).collect();

            let frames = oddio::Frames::from_iter(
                sample_rate,
                oddio::frame_stereo(&mut samples)
                    .iter_mut()
                    .map(|frame| Stereo::from(*frame)),
            );

            let audio_source = AudioSource { frames };

            load_context.set_default_asset(LoadedAsset::new(audio_source));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["flac"]
    }
}
