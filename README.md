# blit

[![Build Status](https://github.com/tversteeg/blit/workflows/CI/badge.svg)](https://github.com/tversteeg/blit/actions?workflow=CI)
[![Crates.io](https://img.shields.io/crates/v/blit.svg)](https://crates.io/crates/blit)
[![Documentation](https://docs.rs/blit/badge.svg)](https://docs.rs/blit)
[![License: GPL-3.0](https://img.shields.io/crates/l/blit.svg)](#license)
[![Downloads](https://img.shields.io/crates/d/blit.svg)](#downloads)

### [Documentation](https://docs.rs/blit/)

<!-- cargo-rdme start -->

Draw sprites quickly using a masking color or an alpha treshold.

#### [Interactive Demo](https://tversteeg.nl/blit/showcase)

This crate works with RGBA `u32` buffers.
The alpha channel can only be read with a singular treshold, converting it to a binary transparent or opaque color.
The reason this limitation is in place is that it allows efficient rendering optimizations.

For ergonomic use of this crate without needing to type convert everything most functions accepting numbers are generic with the number types being [`num_traits::ToPrimitive`], this might seem confusing but any number can be passed to these functions immediately.

When using this crate the most important function to know about is [`Blit::blit`], which is implemented for [`BlitBuffer`].

#### Example

```rust
use blit::{Blit, ToBlitBuffer, BlitOptions, geom::Size};

const CANVAS_SIZE: Size = Size { width: 180, height: 180 };
const MASK_COLOR: u32 = 0xFF_00_FF;
// Create a buffer in which we'll draw our image
let mut canvas: Vec<u32> = vec![0xFF_FF_FF_FF; CANVAS_SIZE.pixels()];

// Load the image from disk using the `image` crate
let img = image::open("examples/smiley_rgb.png").unwrap().into_rgb8();

// Blit by creating a special blitting buffer first where the MASK_COLOR will be the color that will be made transparent
let blit_buffer = img.to_blit_buffer_with_mask_color(MASK_COLOR);

// Draw the image 2 times to the buffer
blit_buffer.blit(&mut canvas, CANVAS_SIZE, &BlitOptions::new_position(10, 10));
blit_buffer.blit(&mut canvas, CANVAS_SIZE, &BlitOptions::new_position(20, 20));
```

<!-- cargo-rdme end -->
