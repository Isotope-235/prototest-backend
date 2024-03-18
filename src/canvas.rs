use crate::drawing::{drawing_canvas::Row, DrawingCanvas};

pub fn blank() -> DrawingCanvas {
    DrawingCanvas {
        rows: std::iter::repeat_with(|| Row { cols: vec![0; 50] }).take(50).collect(),
    }
}

pub fn clamp(canvas: &DrawingCanvas) -> DrawingCanvas {
    let mut out = Vec::with_capacity(50);

    for row in &canvas.rows {
        out.push(Row { cols: Vec::with_capacity(50) } );
        for px in &row.cols {
            out.last_mut().unwrap().cols.push(if *px == 0 {-1} else {1});
        }
    }

    DrawingCanvas { rows: out }
}

pub fn merge(a: &DrawingCanvas, b: &DrawingCanvas) -> DrawingCanvas {
    let mut rows = Vec::with_capacity(50);

    for (a, b) in a.rows.iter().zip(&b.rows) {
        rows.push(Row { cols: Vec::with_capacity(50) });
        for (a, b) in a.cols.iter().zip(&b.cols) {
            rows.last_mut().unwrap().cols.push((a + b).clamp(-1, 1));
        }
    }

    DrawingCanvas { rows }
}