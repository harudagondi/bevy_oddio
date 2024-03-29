use {
    crate::AudioSource,
    bevy::asset::{AssetLoader, BoxedFuture, Error, LoadContext, LoadedAsset},
    claxon::FlacReader,
};

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

            let channels = reader.streaminfo().channels;

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

            match channels {
                1 => {
                    let frames = oddio::Frames::from_iter(
                        sample_rate,
                        samples.into_iter().map(|frame| [frame]),
                    );

                    let audio_source = AudioSource { frames };

                    load_context.set_default_asset(LoadedAsset::new(audio_source));
                }
                2 => {
                    let frames = oddio::Frames::from_iter(
                        sample_rate,
                        oddio::frame_stereo(&mut samples).iter().copied(),
                    );

                    let audio_source = AudioSource { frames };

                    load_context.set_default_asset(LoadedAsset::new(audio_source));
                }
                _ => unimplemented!("bevy_oddio only have support for 1 or 2 channels only."),
            }

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["flac"]
    }
}
