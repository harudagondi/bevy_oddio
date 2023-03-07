use oddio::{Frame, Sample};

/// Stores the length of an array in an associated constant.
/// FIXME: Seal this trait with the Restrictions RFC.
pub trait ArrayLength {
    /// The length of the array.
    const LENGTH: usize;
}

impl<const N: usize, T> ArrayLength for [T; N] {
    const LENGTH: usize = N;
}

/// Representation of a Frame as an array of samples.
pub trait AsArray: Frame {
    /// The representation.
    type Array: ArrayLength;
}

impl AsArray for Sample {
    type Array = [Sample; 1];
}

impl<const N: usize> AsArray for [Sample; N] {
    type Array = [Sample; N];
}

/// Convert a mutable reference of a list of samples to the corresponding newtyped frame.
///
/// # Safety
///
/// This function must uphold the following invariants:
///
/// 1. `F` must have equivalent memory representation to `[Sample; F::Array::LENGTH]`.
/// 2. `F::Array::LENGTH` must be a number where `input.len()` mod `F::Array::LENGTH` == 0.
pub unsafe fn frame_n<F: Frame + AsArray>(input: &mut [Sample]) -> &mut [F] {
    let slice: &mut [F::Array] =
        core::slice::from_raw_parts_mut(input.as_mut_ptr().cast(), input.len() / F::Array::LENGTH);
    &mut *(slice as *mut [F::Array] as *mut [F])
}

#[cfg(test)]
mod tests {
    use {
        crate::frame_n,
        oddio::{Frame, Sample},
    };

    #[test]
    fn normal() {
        let mut original = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];
        // SAFETY: original.len() % 1 == 0
        let mono: &mut [[Sample; 1]] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *mono,
            [
                [1.0],
                [2.0],
                [3.0],
                [4.0],
                [5.0],
                [6.0],
                [7.0],
                [8.0],
                [9.0],
                [10.0],
                [11.0],
                [12.0],
            ]
        );
        // SAFETY: original.len() % 2 == 0
        let stereo: &mut [[Sample; 2]] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *stereo,
            [
                [1.0, 2.0],
                [3.0, 4.0],
                [5.0, 6.0],
                [7.0, 8.0],
                [9.0, 10.0],
                [11.0, 12.0],
            ]
        );
        // SAFETY: original.len() % 3 == 0
        let trio: &mut [[Sample; 3]] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *trio,
            [
                [1.0, 2.0, 3.0],
                [4.0, 5.0, 6.0],
                [7.0, 8.0, 9.0],
                [10.0, 11.0, 12.0],
            ]
        );
        // SAFETY: original.len() % 4 == 0
        let tetrio: &mut [[Sample; 4]] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *tetrio,
            [
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
                [9.0, 10.0, 11.0, 12.0],
            ]
        );
        // SAFETY: original.len() % 6 == 0
        let senario: &mut [[Sample; 6]] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *senario,
            [
                [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                [7.0, 8.0, 9.0, 10.0, 11.0, 12.0],
            ]
        );
        // SAFETY: original.len() % 12 == 0
        let duodecario: &mut [[Sample; 12]] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *duodecario,
            [[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0],]
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn custom_frames() {
        #[repr(transparent)]
        #[derive(Debug, PartialEq, Clone, Copy)]
        struct Mono([Sample; 1]);
        #[repr(transparent)]
        #[derive(Debug, PartialEq, Clone, Copy)]
        struct Stereo([Sample; 2]);
        #[repr(transparent)]
        #[derive(Debug, PartialEq, Clone, Copy)]
        struct Trio([Sample; 3]);
        #[repr(transparent)]
        #[derive(Debug, PartialEq, Clone, Copy)]
        struct Tetrio([Sample; 4]);
        #[repr(transparent)]
        #[derive(Debug, PartialEq, Clone, Copy)]
        struct Senario([Sample; 6]);
        #[repr(transparent)]
        #[derive(Debug, PartialEq, Clone, Copy)]
        struct Duodecario([Sample; 12]);

        macro_rules! impl_custom_frame {
            ($name:ident, $length:literal) => {
                impl Frame for $name {
                    const ZERO: Self = $name([0.0; $length]);

                    fn channels(&self) -> &[Sample] {
                        &self.0
                    }

                    fn channels_mut(&mut self) -> &mut [Sample] {
                        &mut self.0
                    }
                }

                impl super::AsArray for $name {
                    type Array = [Sample; $length];
                }
            };
        }

        impl_custom_frame!(Mono, 1);
        impl_custom_frame!(Stereo, 2);
        impl_custom_frame!(Trio, 3);
        impl_custom_frame!(Tetrio, 4);
        impl_custom_frame!(Senario, 6);
        impl_custom_frame!(Duodecario, 12);

        let mut original = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];
        // SAFETY: original.len() % 1 == 0
        let mono: &mut [Mono] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *mono,
            [
                Mono([1.0]),
                Mono([2.0]),
                Mono([3.0]),
                Mono([4.0]),
                Mono([5.0]),
                Mono([6.0]),
                Mono([7.0]),
                Mono([8.0]),
                Mono([9.0]),
                Mono([10.0]),
                Mono([11.0]),
                Mono([12.0]),
            ]
        );
        // SAFETY: original.len() % 2 == 0
        let stereo: &mut [Stereo] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *stereo,
            [
                Stereo([1.0, 2.0]),
                Stereo([3.0, 4.0]),
                Stereo([5.0, 6.0]),
                Stereo([7.0, 8.0]),
                Stereo([9.0, 10.0]),
                Stereo([11.0, 12.0]),
            ]
        );
        // SAFETY: original.len() % 3 == 0
        let trio: &mut [Trio] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *trio,
            [
                Trio([1.0, 2.0, 3.0]),
                Trio([4.0, 5.0, 6.0]),
                Trio([7.0, 8.0, 9.0]),
                Trio([10.0, 11.0, 12.0]),
            ]
        );
        // SAFETY: original.len() % 4 == 0
        let tetrio: &mut [Tetrio] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *tetrio,
            [
                Tetrio([1.0, 2.0, 3.0, 4.0]),
                Tetrio([5.0, 6.0, 7.0, 8.0]),
                Tetrio([9.0, 10.0, 11.0, 12.0]),
            ]
        );
        // SAFETY: original.len() % 6 == 0
        let senario: &mut [Senario] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *senario,
            [
                Senario([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
                Senario([7.0, 8.0, 9.0, 10.0, 11.0, 12.0]),
            ]
        );
        // SAFETY: original.len() % 12 == 0
        let duodecario: &mut [Duodecario] = unsafe { frame_n(&mut original) };
        assert_eq!(
            *duodecario,
            [Duodecario([
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0
            ]),]
        );
    }
}
