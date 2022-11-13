# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2022-11-13

### Added

- New spatial audio feature:
  - Two versions: spatial audio and spatial buffered audio.
  - `SpatialAudioOutput`, `SpatialAudioSink`, `SpatialAudioSinks`, and `play_queued_spatial_audio` are added for spatial audio.
  - `SpatialBufferedAudioOutput`, `SpatialBufferedAudioSink`, `SpatialBufferedAudioSinks`, and `play_queued_spatial_buffered_audio` are added for spatial buffered audio.
  - `Audio` now has `play_spatial` and `play_spatial_buffered` methods.
  - `AudioApp` now has a `add_spatial_audio_source` and `add_spatial_buffered_audio_source` methods implemented for `App`.

### Changed

- Changed the `ToSignal::Signal` type of `AudioSource` to `Gain<Speed<FramesSignal<F>>>`

## [0.2.0] - 2022-09-04

### Added

- A bunch of newtypes frames that implement `TypeUuid` and `Frame`:
  - `Mono`, for mono output.
  - `Stereo` for stereo output.
- `FromFrame` trait
  - Has a `from_frame` method, which converts predefined `oddio::Frame` to the newtypes.
- A bunch of builtin `oddio` types
  - `Constant`
  - `Cycle`
  - `Stream`
- `Gain` example, which showcases controlling signals.

### Changed

- `Audio::play` now only returns `Handle<AudioSink<Source>>`.
- `AudioApp::add_audio_source` now requires a `const N: usize` and `F` generics.
  - `N` is the number of channels.
  - `F` is a type that implements `oddio::Frame` and `FromFrame<[Sample; N]>`.
  - `F` can be implied.
- `play_queued_audio` now requires a `const N: usize` and `F` generics, similar to previous.
- `Audio` requires `F` generic that implements `oddio::Frame`.
- `AudioSource` requires `F` generic that implements `oddio::Frame`.
  - `AudioSource` now accepts `Arc<Frames<F>>` instead of `Arc<Frames<Stereo>>`.
  - `AudioSource::Signal` now returns `FramesSignal<F>`.
- `AudioOutput` requires `const N: usize` and `F: Frame + FromFrame<[Sample; N]>` generics.
- Type alias for `Stereo` is now a newtype struct.
- `AudioSink` now derefs to `ManuallyDrop<Handle<Stop<<Source as ToSignal>::Signal>>>`.
  - There is now no `SplitSignal` in between `Stop` and `Source`.

### Removed

- `AudioHandle`. Use `AudioSink` to control the playing audio.
- `AudioHandles`. Use `AudioSinks`.
- `SpatialScene` settings.

## [0.1.0] - 2022-07-01

- Released `bevy_oddio` 0.1 🎉

[Unreleased]: https://github.com/harudagondi/bevy_oddio/compare/v0.2.0..HEAD
[0.3.0]: https://github.com/harudagondi/bevy_oddio/compare/v0.2.0..v0.3.0
[0.2.0]: https://github.com/harudagondi/bevy_oddio/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/harudagondi/bevy_oddio/releases/tag/v0.1.0
