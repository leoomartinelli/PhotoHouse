//! Visual and 36 MP performance probe for issue #25.
use photocraft_color::PixelFormat;
use photocraft_geom::Rect;
use photocraft_paint::{BrushSettings, StrokePoint, StrokeRenderer};
use photocraft_raster::Surface;
use std::{io::Write, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let points: Vec<_> = (0..100)
        .map(|i| {
            let t = f64::from(i) / 99.0;
            StrokePoint::new(200.0 + t * 5000.0, 600.0 + t * 4000.0, 1.0)
        })
        .collect();
    for spacing in [0.100001, 0.1] {
        let b = BrushSettings { size: 300.0, spacing, pressure_size: false, ..Default::default() };
        let mut target = Surface::new(PixelFormat::RGBA8);
        target.fill_rect(Rect::new(0, 0, 6000, 6000), &[1.0; 4]);
        let pre = target.clone();
        let start = Instant::now();
        let mut r = StrokeRenderer::new(&b, Some(PixelFormat::RGBA8), 1.0);
        for chunk in points.chunks(8) {
            r.push(chunk);
            r.composite(&pre, &mut target, None, false, false);
        }
        r.finish();
        r.composite(&pre, &mut target, None, false, false);
        println!("spacing={spacing}: {:.1} ms", start.elapsed().as_secs_f64() * 1000.0);
    }
    let b = BrushSettings { size: 150.0, pressure_size: false, ..Default::default() };
    let mut r = StrokeRenderer::new(&b, None, 1.0);
    r.push(&[StrokePoint::new(100.0, 350.0, 1.0), StrokePoint::new(450.0, 180.0, 1.0)]);
    r.finish();
    let mut thin = StrokeRenderer::new(&BrushSettings { size: 20.0, ..b }, None, 1.0);
    thin.push(&[StrokePoint::new(100.0, 60.0, 1.0), StrokePoint::new(450.0, 60.0, 1.0)]);
    thin.finish();
    let mut out = std::fs::File::create("issue-25-brush.ppm")?;
    write!(out, "P6\n600 450\n255\n")?;
    for y in 0..450 {
        for x in 0..600 {
            let c = r.coverage_at(x, y).max(thin.coverage_at(x, y));
            out.write_all(&[((1.0 - c) * 255.0) as u8, 255, ((1.0 - c) * 255.0) as u8])?;
        }
    }
    Ok(())
}
