use crate::drawing::{drawing_canvas::Row, DrawingCanvas};

pub fn blank() -> DrawingCanvas {
    DrawingCanvas {
        rows: std::iter::repeat_with(|| Row { cols: vec![0; 50] }).take(50).collect(),
    }
}

pub fn merge(a: &DrawingCanvas, b: &DrawingCanvas) -> DrawingCanvas {
    let mut rows = Vec::with_capacity(50);

    for (a, b) in a.rows.iter().zip(&b.rows) {
        rows.push(Row { cols: Vec::with_capacity(50) });
        for (a, b) in a.cols.iter().zip(&b.cols) {
            rows.last_mut().unwrap().cols.push((a + b).min(1).max(0));
        }
    }

    DrawingCanvas { rows }
}