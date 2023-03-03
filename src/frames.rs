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
