use bevy::{
    prelude::{Deref, DerefMut},
    reflect::TypeUuid,
};
use oddio::{Frame, Sample};

/// Internal trait to convert frames.
pub trait FromFrame<F: Frame> {
    /// The number of channels of the given frame.
    const CHANNELS: usize;
    /// Convert given frame to the implementing type.
    fn from_frame(f: F) -> Self;
}

impl<const N: usize> FromFrame<[Sample; N]> for [Sample; N] {
    const CHANNELS: usize = N;

    fn from_frame(f: [Sample; N]) -> Self {
        f
    }
}

impl FromFrame<[Sample; 1]> for Sample {
    const CHANNELS: usize = 1;

    fn from_frame(f: [Sample; 1]) -> Self {
        f[0]
    }
}

macro_rules! impl_frame {
    ($(#[$meta:meta])* $name:ident, $n:literal, $uuid:expr) => {
        #[repr(transparent)]
        #[derive(TypeUuid, Deref, DerefMut, Clone, Copy)]
        #[uuid = $uuid]
        $(#[$meta])*
        pub struct $name([Sample; $n]);

        impl From<$name> for [Sample; $n] {
            fn from(x: $name) -> Self {
                x.0
            }
        }

        impl From<[Sample; $n]> for $name {
            fn from(x: [Sample; $n]) -> Self {
                $name(x)
            }
        }

        impl FromFrame<[Sample; $n]> for $name {
            const CHANNELS: usize = $n;

            fn from_frame(f: [Sample; $n]) -> Self {
                f.into()
            }
        }

        impl Frame for $name {
            const ZERO: Self = $name(Frame::ZERO);

            fn channels(&self) -> &[Sample] {
                self.0.channels()
            }

            fn channels_mut(&mut self) -> &mut [Sample] {
                self.0.channels_mut()
            }
        }
    };
}

impl_frame!(
    /// A frame with a single channel.
    Mono,
    1,
    "43915b3a-332c-4104-81ff-3b68bdb192c3"
);
impl_frame!(
    /// A frame with two channels.
    Stereo,
    2,
    "94ca7739-0a77-4142-91de-b7150fecc689"
);

/// Convert a mutable reference of a list of samples to the corresponding newtyped frame.
///
/// # Safety
///
/// This function must uphold the following invariants:
///
/// 1. `F` must have equivalent memory representation to `[Sample; N]`.
/// 2. `N` must be a number where `input.len()` mod `N` == 0.
pub unsafe fn frame_n<F: Frame + FromFrame<[Sample; N]>, const N: usize>(
    input: &mut [Sample],
) -> &mut [F] {
    let slice: &mut [[Sample; N]] =
        core::slice::from_raw_parts_mut(input.as_mut_ptr().cast(), input.len() / N);
    &mut *(slice as *mut [[Sample; N]] as *mut [F])
}
