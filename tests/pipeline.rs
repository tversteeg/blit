use blit::{
    geom::{Coordinate, Rect, Size},
    Blit, BlitBuffer,
};

const SOURCE_SIZE: Size = Size {
    width: 20,
    height: 20,
};
const TARGET_SIZE: Size = Size {
    width: 100,
    height: 100,
};

#[test]
fn full() {
    let (source, mut target) = gen_buffers();

    // Draw without any changes, should be in the upper left
    source.blit(&mut target, TARGET_SIZE).draw();

    assert_source_drawn(&source, Rect::new(0, 0, SOURCE_SIZE), &mut target, (0, 0));
}

#[test]
fn position() {
    let (source, mut target) = gen_buffers();

    // Draw at offset of 10 pixels
    source
        .blit(&mut target, TARGET_SIZE)
        .position((10, 10))
        .draw();
    assert_source_drawn(&source, Rect::new(0, 0, SOURCE_SIZE), &mut target, (10, 10));
}

fn assert_source_drawn<R, C>(source: &BlitBuffer, source_view: R, target: &[u32], target_offset: C)
where
    R: Into<Rect>,
    C: Into<Coordinate>,
{
    let target_offset = target_offset.into();
    let source_view = source_view.into();

    // The target view is the source view but with an offset
    let mut target_view = source_view + target_offset;

    // Verify the pixels match
    source_view
        .parent_ranges_iter(SOURCE_SIZE)
        .zip(target_view.parent_ranges_iter(TARGET_SIZE))
        .for_each(|(source_slice, target_slice)| {
            if source.pixels()[source_slice.clone()] != target[target_slice.clone()] {
                panic!("bytes mismatch at source range {source_slice:?} and target range {target_slice:?}");
            }
        });
}

fn gen_buffers() -> (BlitBuffer, Vec<u32>) {
    (
        BlitBuffer::from_iter(0..SOURCE_SIZE.pixels() as u32, SOURCE_SIZE.width, 0),
        vec![0; TARGET_SIZE.pixels()],
    )
}
