#![feature(test)]

extern crate blit;
extern crate image;
extern crate test;

use test::Bencher;

use blit::*;

const SIZE: usize = 1000;
const ITERATIONS: usize = 100;

#[bench]
fn blit_buffer(b: &mut Bencher) {
    let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];

    let img = image::open("examples/smiley.png").unwrap();
    let rgb = img.as_rgb8().unwrap();
    let blit = rgb.as_blit_buffer(0xFF00FF);

    b.iter(|| {
        for x in 0..ITERATIONS {
            blit.blit(&mut buffer, (SIZE, SIZE), (x as i32, 0));
        }
    });
}

#[bench]
fn blit_buffer_rect(b: &mut Bencher) {
    let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];

    let img = image::open("examples/smiley.png").unwrap();
    let rgb = img.as_rgb8().unwrap();
    let blit = rgb.as_blit_buffer(0xFF00FF);
    let size = blit.size();

    b.iter(|| {
        for x in 0..ITERATIONS {
            blit.blit_rect(&mut buffer, (SIZE, SIZE), (x as i32, 0), size, (0, 0));
        }
    });
}

#[bench]
fn blit_buffer_exact_fit(b: &mut Bencher) {
    let img = image::open("examples/smiley.png").unwrap();
    let rgb = img.as_rgb8().unwrap();
    let blit = rgb.as_blit_buffer(0xFF00FF);

    let size = rgb.dimensions();
    let size = (size.0 as usize, size.1 as usize);
    let mut buffer: Vec<u32> = vec![0; size.0 * size.1];

    b.iter(|| {
        for _ in 0..ITERATIONS {
            blit.blit(&mut buffer, size, (0, 0));
        }
    });
}

#[bench]
fn blit_img(b: &mut Bencher) {
    let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];

    let img = image::open("examples/smiley.png").unwrap();
    let rgb = img.as_rgb8().unwrap();

    b.iter(|| {
        for x in 0..ITERATIONS {
            rgb.blit_with_mask_color(&mut buffer, (SIZE, SIZE), (x as i32, 0), 0xFF00FF);
        }
    });
}
