# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Support for mono.
- Support for `oddio::Sine` and `oddio::SpatialScene`.

### Changed
- `Audio::play` now returns `AudioHandle` and `AudioSink`.
- All public facing structs now need `F` to be specified, where `F` is either `Mono` or `Stereo`. 

## [0.1.0] - 2022-07-01
- Released `bevy_oddio` 0.1 🎉

[Unreleased]: https://github.com/harudagondi/bevy_oddio/compare/v0.1.0..HEAD
[0.1.0]: https://github.com/harudagondi/bevy_oddio/releases/tag/v0.1.0