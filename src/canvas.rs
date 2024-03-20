use crate::proto::DrawingCanvas;

const WIDTH: usize = 50;
const HEIGHT: usize = 50;
const SIZE: usize = WIDTH * HEIGHT;

pub fn blank() -> DrawingCanvas {
    let contents = std::iter::repeat(0).take(SIZE).collect();
    DrawingCanvas {
        contents,
    }
}

pub fn merge_into(target: &mut DrawingCanvas, source: &DrawingCanvas) {
    target
        .contents
        .iter_mut()
        .zip(&source.contents)
        .for_each(|(t, s)| {
            *t = match *s {
                -1 => 0,
                0 => *t,
                o => o,
            }
        })
}
