# blit
A Rust library for blitting 2D sprites

<a href="https://github.com/tversteeg/const-tweaker/actions"><img src="https://github.com/tversteeg/const-tweaker/workflows/CI/badge.svg" alt="CI"/></a>
[![Cargo](https://img.shields.io/crates/v/blit.svg)](https://crates.io/crates/blit) [![License: GPL-3.0](https://img.shields.io/crates/l/blit.svg)](#license) [![Downloads](https://img.shields.io/crates/d/blit.svg)](#downloads)

### [Documentation](https://docs.rs/blit/)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
blit = "0.7"
```

## Demos

### [Smiley](https://tversteeg.nl/blit/smiley/)

Web: https://tversteeg.nl/blit/smiley/

Uses the `["image"]` feature flag.

Displays the use of loading an image, blitting it fully and also blitting a subrectangle from it.

#### Local

```console
cargo run --example smiley
```

### [Aseprite Animation](https://tversteeg.nl/blit/aseprite-animation/)

Web: https://tversteeg.nl/blit/aseprite-animation/

Uses the `["image", "aseprite"]` feature flags.

Displays the use of using the `AnimationBuffer` and `Animation` structs to create a simple time based animation.

#### Local

```console
cargo run --example aseprite-animation
```
 
### [Aseprite 9 Slice](https://tversteeg.nl/blit/aseprite-9slice/)

Web: https://tversteeg.nl/blit/aseprite-9slice/

Uses the `["image", "aseprite"]` feature flags.

#### Local

```console
cargo run --example aseprite-9slice
```

![Example](img/example.png?raw=true)

## Example Code

Uses the `["image"]` feature flag:

```toml
[dependencies]
blit = { version = "0.7", features = ["image"] }
```

```rust
use blit::BlitExt;

const WIDTH: usize = 180;
const HEIGHT: usize = 180;
const MASK_COLOR: u32 = 0xFF00FF;

// The target buffer the image will be blit to
let mut buffer: Vec<u32> = vec![0xFFFFFFFF; WIDTH * HEIGHT];

// Open the example image using the image crate
let img = image::open("examples/smiley.png").unwrap().into_rgb8();

// Convert the image to a blit buffer
let blit_buffer = img_rgb.to_blit_buffer(Color::from_u32(MASK_COLOR));

// Blit the image twice to the buffer
let pos = (10, 10);
blit_buffer.blit(&mut buffer, WIDTH, pos);
let pos = (20, 20);
blit_buffer.blit(&mut buffer, WIDTH, pos);
```
