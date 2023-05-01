use blit::{geom::Size, Blit, BlitOptions, ToBlitBuffer};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use criterion_perf_events::Perf;
use perfcnt::linux::{HardwareEventType, PerfCounterBuilderLinux};

const SIZE: usize = 300;

fn criterion_benchmark(c: &mut Criterion<Perf>) {
    let rgb = image::load_from_memory(include_bytes!("../examples/showcase/smiley_rgb.png"))
        .unwrap()
        .into_rgb8();

    let blit = rgb.to_blit_buffer_with_mask_color(0xFF_00_FF);
    let size = blit.size();

    let mut group = c.benchmark_group("blit position");
    for x in [
        -(size.width as i32),
        -(size.width as i32) / 2,
        0,
        SIZE as i32 / 2,
        SIZE as i32 - size.width as i32 / 2,
        SIZE as i32,
    ] {
        let options = BlitOptions::new_position(x, 0);

        group.bench_with_input(BenchmarkId::from_parameter(x), &options, |b, options| {
            let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];

            b.iter(|| {
                blit.blit(
                    &mut buffer,
                    black_box(Size::new(SIZE, SIZE)),
                    black_box(options),
                )
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("blit sub rect");
    for x in [
        -(size.width as i32),
        -(size.width as i32) / 2,
        0,
        SIZE as i32 / 2,
        SIZE as i32 - size.width as i32 / 2,
        SIZE as i32,
    ] {
        let options =
            BlitOptions::new_position(x, 0).with_sub_rect((0, 0, size.width / 2, size.height / 2));

        group.bench_with_input(BenchmarkId::from_parameter(x), &options, |b, options| {
            let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];

            b.iter(|| {
                blit.blit(
                    &mut buffer,
                    black_box(Size::new(SIZE, SIZE)),
                    black_box(options),
                )
            });
        });
    }
    group.finish();

    c.bench_function("blit exact fit", |b| {
        let mut buffer: Vec<u32> = vec![0; size.pixels()];

        b.iter(|| {
            blit.blit(&mut buffer, black_box(size), black_box(&BlitOptions::new()));
        });
    });

    c.bench_function("load img with mask", |b| {
        b.iter(|| {
            rgb.to_blit_buffer_with_mask_color(0xFF_00_FF);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_measurement(Perf::new(PerfCounterBuilderLinux::from_hardware_event(HardwareEventType::Instructions)));
    targets = criterion_benchmark
);
criterion_main!(benches);
