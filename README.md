# signrs

Signrs is a digital signage player that is cross-platform, including low-power single-board computers like the Raspberry Pi. The player is a single executable that runs the developer's signage application.

This project is very heavily influenced by the awesome [info-beamer](https://info-beamer.com/) project. It deviates in the following ways:

* Applications are built in JavaScript rather than Lua.
* The player runs on Windows, Linux, (and theoretically) macOS. Applications can be developed and tested on a desktop before being deployed to the signage hardware, e.g., a Raspberry Pi.
* Each player runs a webserver that allows making simple changes to the running app, and provides the API to create a player management tool.

https://github.com/superlou/signrs/assets/709695/9efa7d1f-9efe-4683-b5d3-db6a524a9ad2

## Key Missing Features

* [ ] Choosing an open-source license
* [ ] Parallelize drawing and JS engine to allow more complex applications to run at 60 FPS on resource-limited computers.
* [ ] Improve the web frontend to allow syncrhonizing updates to multiple players.
* [ ] Video playback

## Running the Player

Build and run the player using `cargo run --release <path to application>`. Examples are in the [examples directory](https://github.com/superlou/signrs/tree/main/examples), e.g.,:

`cargo run --release examples/app2`

To see the avaialable command-line arguments run:

`cargo run --release -- --help`

To view more debugging information, set the logging level with the `RUST_LOG` environment variable:

```
RUST_LOG=DEBUG cargo run examples/app2
```

To profile on Linux:

```
sudo sysctl -w kernel.perf_event_paranoid=-1
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --flamechart --no-inline -- examples/app2
```

## Creating an Application

Every application needs to have a "main.js" file, and export an `init` and `draw` function. The minimum application looks like this:

```js
export function init() {
    set_resolution(960, 540);
}

export function draw(dt) {
    // Called once per frame with dt being the time between calls
    clear_screen(new Color(0, 0, 0));
}
```

### Signage Application API

#### Drawing Classes

##### `new Color(r, g, b, a=1)`
##### `new Font(fontPath)`
##### `new Image(imagePath)`

#### Drawing Functions

##### `clear_screen(color: Color)`

Clears the drawing area and fills it with `color`.

##### `draw_image(image: Image, x, y, width, height, alpha=1)`

Draws an image with optional transparency.

##### `draw_text(font: Font, text, x, y, size, color: Color)`

Draws text.

##### `size_text(font: Font, text, size)`

Returns `[width, height]` of the sized text.

##### `draw_rectangle(x, y, w, h, color: Color)`

Draws a rectangle.

##### `with_offset(x, y, callback)`

Runs the `callback` with the coordinates of all drawing calls offset by (x, y).

#### Initialization Helpers

##### `set_resolution(width, height)`

Sets the resolution of the drawing area.

#### File Helpers

##### `watch_json(jsonFilePath, callback(data), runFirst=true)`

Creates a file watcher that runs immediately (if runFirst is true) and then whenever the file is changed. `data` is the JSON decoded data.
