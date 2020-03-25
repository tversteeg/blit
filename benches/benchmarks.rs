use blit::*;
use criterion::{criterion_group, criterion_main, Criterion};

const SIZE: usize = 1000;
const ITERATIONS: i32 = 10;

fn criterion_benchmark(c: &mut Criterion) {
    let img_rgb = image::open("examples/smiley_rgb.png").unwrap();
    let rgb = img_rgb.as_rgb8().unwrap();
    let img_rgba = image::open("examples/smiley_rgba.png").unwrap();
    let rgba = img_rgba.as_rgba8().unwrap();

    let blit = rgb.to_blit_buffer(Color::from_u32(0xFF_00_FF));
    let size = blit.size();

    c.bench_function("buffer", |b| {
        let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];

        b.iter(|| {
            for x in 0..ITERATIONS {
                blit.blit(&mut buffer, SIZE, (x * 100, 0));
            }
        });
    });

    c.bench_function("buffer rect", |b| {
        let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];

        b.iter(|| {
            for x in 0..ITERATIONS {
                blit.blit_rect(&mut buffer, SIZE, (x * 100, 0), (0, 0, size.0, size.1));
            }
        });
    });

    c.bench_function("exact fit", |b| {
        let mut buffer: Vec<u32> = vec![0; (size.0 * size.1) as usize];

        b.iter(|| {
            blit.blit(&mut buffer, size.0 as usize, (0, 0));
        });
    });

    c.bench_function("img rgb", |b| {
        let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];

        b.iter(|| {
            for x in 0..ITERATIONS {
                rgb.blit(&mut buffer, SIZE, (x * 100, 0), Color::from_u32(0xFF_00_FF));
            }
        });
    });

    c.bench_function("img rgba", |b| {
        let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];

        b.iter(|| {
            for x in 0..ITERATIONS {
                rgba.blit(&mut buffer, SIZE, (x * 100, 0), Color::from_u32(0xFF_00_FF));
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
