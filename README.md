# blit
A Rust library for blitting 2D sprites

<a href="https://github.com/tversteeg/const-tweaker/actions"><img src="https://github.com/tversteeg/const-tweaker/workflows/CI/badge.svg" alt="CI"/></a>
[![Cargo](https://img.shields.io/crates/v/blit.svg)](https://crates.io/crates/blit) [![License: GPL-3.0](https://img.shields.io/crates/l/blit.svg)](#license) [![Downloads](https://img.shields.io/crates/d/blit.svg)](#downloads)

### [Documentation](https://docs.rs/blit/)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
blit = "0.6"
```

### Run the example

On Linux you need the `xorg-dev` package as required by `minifb`. `sudo apt install xorg-dev`

    cargo run --example smiley

This should produce the following window:

![Example](img/example.png?raw=true)

## Examples

```rust
extern crate image;

use blit::*;

const WIDTH: usize = 180;
const HEIGHT: usize = 180;
const MASK_COLOR: u32 = 0xFF00FF;

let mut buffer: Vec<u32> = vec![0xFFFFFFFF; WIDTH * HEIGHT];

let img = image::open("examples/smiley.png").unwrap();
let img_rgb = img.as_rgb8().unwrap();

// Blit directly to the buffer
let pos = (0, 0);
img_rgb.blit(&mut buffer, WIDTH, pos, Color::from_u32(MASK_COLOR));

// Blit by creating a special blitting buffer first, this has some initial
// overhead but is a lot faster after multiple calls
let blit_buffer = img_rgb.to_blit_buffer(Color::from_u32(MASK_COLOR));

let pos = (10, 10);
blit_buffer.blit(&mut buffer, WIDTH, pos);
let pos = (20, 20);
blit_buffer.blit(&mut buffer, WIDTH, pos);

// Save the blit buffer to a file
blit_buffer.save("smiley.blit");
```
