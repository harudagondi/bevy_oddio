use bevy::asset::{AssetLoader, BoxedFuture, Error, LoadContext, LoadedAsset};

use crate::{frames::Stereo, AudioSource};

#[derive(Default)]
pub struct WavLoader;

// Adapted from https://github.com/Ralith/oddio/blob/main/examples/wav.rs
impl AssetLoader for WavLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        #[allow(clippy::cast_lossless, clippy::cast_precision_loss)]
        Box::pin(async move {
            let mut reader = hound::WavReader::new(bytes)?;
            let hound::WavSpec {
                sample_rate: source_sample_rate,
                sample_format,
                bits_per_sample,
                ..
            } = reader.spec();

            // convert the WAV data to floating point samples
            // e.g. i8 data is converted from [-128, 127] to [-1.0, 1.0]
            let samples_result: Result<Vec<f32>, _> = match sample_format {
                hound::SampleFormat::Int => {
                    let max_value = 2_u32.pow(bits_per_sample as u32 - 1) - 1;
                    reader
                        .samples::<i32>()
                        .map(|sample| sample.map(|sample| sample as f32 / max_value as f32))
                        .collect()
                }
                hound::SampleFormat::Float => reader.samples::<f32>().collect(),
            };
            let mut samples = samples_result?;

            // channels are interleaved, so we put them together in stereo
            let samples_stereo = oddio::frame_stereo(&mut samples)
                .iter_mut()
                .map(|frame| Stereo::from(*frame));
            let frames = oddio::Frames::from_iter(source_sample_rate, samples_stereo);

            let audio_source = AudioSource { frames };

            load_context.set_default_asset(LoadedAsset::new(audio_source));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wav"]
    }
}
