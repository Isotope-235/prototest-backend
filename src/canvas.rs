use std::ops::Deref;

use crate::proto::{drawing_canvas::Row, DrawingCanvas};

const SIZE: usize = 50;

fn sizevec<T>() -> Vec<T> {
    Vec::with_capacity(SIZE)
}

pub fn blank() -> DrawingCanvas {
    DrawingCanvas {
        rows: std::iter::repeat_with(|| Row {
            cols: vec![0; SIZE],
        })
        .take(SIZE)
        .collect(),
    }
}

pub fn clamp(canvas: impl Deref<Target = DrawingCanvas>) -> DrawingCanvas {
    let mut out = sizevec();

    for row in &canvas.rows {
        out.push(Row { cols: sizevec() });
        for px in &row.cols {
            out.last_mut().unwrap().cols.push(*px);
        }
    }

    DrawingCanvas { rows: out }
}

#[allow(dead_code)]
pub fn merge(a: &DrawingCanvas, b: &DrawingCanvas) -> (bool, DrawingCanvas) {
    let mut changed = false;
    let mut rows = sizevec();

    for (a, b) in a.rows.iter().zip(&b.rows) {
        rows.push(Row { cols: sizevec() });
        for (a, b) in a.cols.iter().zip(&b.cols) {
            if *a != *b {
                changed = true;
            }
            rows.last_mut().unwrap().cols.push((a + b).clamp(0, 1));
        }
    }

    (changed, DrawingCanvas { rows })
}

pub fn merge_into(target: &mut DrawingCanvas, source: &DrawingCanvas) -> bool {
    let mut changed = false;
    
    for (left, right) in target.rows.iter_mut().zip(&source.rows) {
        for (l, r) in left.cols.iter_mut().zip(&right.cols) {
            if *l != *r {
                changed = true;
            }
            *l = (*l + *r).clamp(0, 1);
        }
    }

    changed
}
