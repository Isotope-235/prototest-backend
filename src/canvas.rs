use crate::drawing::{drawing_canvas::Row, DrawingCanvas};

pub fn blank() -> DrawingCanvas {
    DrawingCanvas {
        rows: std::iter::repeat_with(|| Row { cols: vec![0; 50] }).take(50).collect(),
    }
}

pub fn merge(a: &DrawingCanvas, b: &DrawingCanvas) -> DrawingCanvas {
    let mut rows = Vec::new();
    for (a, b) in a.rows.iter().zip(b.rows.iter()) {
        let mut cols = Vec::new();
        for (a, b) in a.cols.iter().zip(b.cols.iter()) {
            cols.push(if *a == 0 { *b } else { *a });
        }
        rows.push(Row { cols });
    }
    DrawingCanvas { rows }
}