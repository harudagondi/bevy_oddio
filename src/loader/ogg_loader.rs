use {
    crate::AudioSource,
    bevy::asset::{AssetLoader, BoxedFuture, Error, LoadContext, LoadedAsset},
    lewton::inside_ogg::OggStreamReader,
    std::io::Cursor,
};

#[derive(Default)]
pub struct OggLoader;

impl AssetLoader for OggLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let mut ogg_stream_reader = OggStreamReader::new(Cursor::new(bytes))?;

            let mut samples: Vec<Vec<f32>> = Vec::new();

            while let Some(packets) =
                ogg_stream_reader.read_dec_packet_generic::<Vec<Vec<f32>>>()?
            {
                samples.extend(packets);
            }

            let channels = ogg_stream_reader.ident_hdr.audio_channels;

            match channels {
                1 => {
                    let frames = oddio::Frames::from_iter(
                        ogg_stream_reader.ident_hdr.audio_sample_rate,
                        samples.iter().map(|packet| [packet[0]]),
                    );

                    let audio_source = AudioSource { frames };

                    load_context.set_default_asset(LoadedAsset::new(audio_source));
                }
                2 => {
                    let frames = oddio::Frames::from_iter(
                        ogg_stream_reader.ident_hdr.audio_sample_rate,
                        samples.iter().map(|packet| [packet[0], packet[1]]),
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
        &["ogg", "oga", "spx"]
    }
}
