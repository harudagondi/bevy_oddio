# Bevy Oddio

[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

A third party Bevy plugin that integrates [`oddio`] into [Bevy].

[`oddio`]: https://github.com/Ralith/oddio
[Bevy]: https://github.com/bevyengine/bevy

## Usage

```rust no_run
use bevy::prelude::*;
use bevy_oddio::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(play_background_audio)
        .run();
}

fn play_background_audio(asset_server: Res<AssetServer>, mut audio: ResMut<Audio>) {
    audio.play(asset_server.load("background_audio.wav"), 0.0);
}
```

## Compatibility

| `bevy_oddio`  | `bevy` |
| ------------- | ------ |
| main          | main   |
| 0.1.0         | 0.8    |

## License

`bevy_oddio` is primarily distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

## Acknowledgement

I'd like to say thanks to the authors of [`oddio`] and [Bevy] for making this plugin possible.