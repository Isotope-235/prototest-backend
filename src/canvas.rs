use crate::proto::DrawingCanvas;

pub fn blank(width: usize, height: usize) -> DrawingCanvas {
    let contents = std::iter::repeat(0).take(width * height).collect();
    DrawingCanvas {
        contents,
        width: width as i32,
        height: height as i32
    }
}

pub fn try_merge_into(target: &mut DrawingCanvas, source: &DrawingCanvas) -> Result<(), String>{

    if source.width != target.width {
        return Err(format!("Source and target dimensions do not match! Source canvas width is {0} but width required by the target canvas is {1}.", source.width, target.width));
    }

    if source.height != target.height {
        return Err(format!("Source and target dimensions do not match! Source canvas height is {0} but height required by the target canvas is {1}.", source.height, target.height));
    }

    let source_len = source.contents.len();
    let target_size = target.width * target.height;
    if source_len != target_size.max(0) as usize {
        return Err(format!("Source and target dimensions do not match! The number of pixels in the source canvas is {source_len}, but the number of pixels required by the target canvas is {target_size} ({0} * {1}.)", target.width, target.height));
    }

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
        });

    Ok(())
}

pub fn check_invariants(canvas: &DrawingCanvas) -> CheckInvariantsResult {
    use CheckInvariantsResult::*;
    let DrawingCanvas { contents, width, height } = canvas;

    if *width < 1 {
        return NegativeWidth;
    }

    if *height < 1 {
        return NegativeHeight;
    }

    let Some(checked_size) = width.checked_mul(*height) else {
        return OverflowingSize;
    };

    if checked_size as usize != contents.len() {
        return MismatchedSize;
    }

    Fine
}

pub enum CheckInvariantsResult {
    Fine,
    NegativeWidth,
    NegativeHeight,
    OverflowingSize,
    MismatchedSize
}