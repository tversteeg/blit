use blit::BlitExt;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SIZE: usize = 1000;
const ITERATIONS: i32 = 10;

fn criterion_benchmark(c: &mut Criterion) {
    let rgb = image::load_from_memory(include_bytes!("../examples/smiley/smiley_rgb.png"))
        .unwrap()
        .into_rgb8();
    let rgba = image::load_from_memory(include_bytes!("../examples/smiley/smiley_rgba.png"))
        .unwrap()
        .into_rgba8();

    let blit = rgb.to_blit_buffer_with_mask_color(0xFF_00_FF);
    let size = blit.size();

    c.bench_function("blit", |b| {
        let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];

        b.iter(|| {
            for x in 0..ITERATIONS {
                blit.blit(&mut buffer, SIZE, black_box((x * 100, 0)));
            }
        });
    });

    c.bench_function("blit sub rect", |b| {
        let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];

        b.iter(|| {
            for x in 0..ITERATIONS {
                blit.blit_rect(
                    &mut buffer,
                    SIZE,
                    black_box((x * 100, 0)),
                    black_box((0, 0, size.0, size.1)),
                );
            }
        });
    });

    c.bench_function("blit exact fit", |b| {
        let mut buffer: Vec<u32> = vec![0; (size.0 * size.1) as usize];

        b.iter(|| {
            blit.blit(&mut buffer, size.0 as usize, (0, 0));
        });
    });

    c.bench_function("load img with mask", |b| {
        b.iter(|| {
            rgb.to_blit_buffer_with_mask_color(0xFF_00_FF);
        });
    });

    c.bench_function("load img with alpha", |b| {
        b.iter(|| {
            rgba.to_blit_buffer_with_alpha(127);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
