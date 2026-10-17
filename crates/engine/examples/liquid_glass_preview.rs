//! Synthetic visual fixture: cargo run -p photocraft-engine --example liquid_glass_preview -- output.png
use photocraft_algo::{Ctx, Image, LiquidGlassShape, liquid_glass};
use photocraft_color::ColorMode;
use photocraft_geom::Rect;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap_or_else(|| "liquid-glass-preview.png".into());
    let bounds = Rect::new(0, 0, 800, 600);
    let mut scene = Image::new(bounds, 4);
    for y in 0..600 {
        for x in 0..800 {
            let u = x as f32 / 800.0;
            let v = y as f32 / 600.0;
            let ribbon = ((u * 9.0 + v * 5.0).sin() * 0.5 + 0.5).powi(8);
            let grid = if x % 64 < 3 || y % 64 < 3 { 0.15 } else { 0.0 };
            let i = (y * 800 + x) * 4;
            scene.data[i..i + 4].copy_from_slice(&[
                (0.08 + ribbon * 0.85 + grid).min(1.0),
                (0.28 + u * 0.25 + grid).min(1.0),
                (0.65 + v * 0.3 - ribbon * 0.5 + grid).min(1.0),
                1.0,
            ]);
        }
    }
    let mut output = scene.clone();
    for (rect, shape, radius) in [
        (Rect::new(100, 85, 310, 295), LiquidGlassShape::Circle, 100.0),
        (Rect::new(370, 85, 700, 295), LiquidGlassShape::RoundedRect, 42.0),
        (Rect::new(150, 365, 650, 530), LiquidGlassShape::Capsule, 80.0),
    ] {
        let mut mask = Image::new(rect, 4);
        for p in mask.data.as_chunks_mut::<4>().0 {
            p.copy_from_slice(&[1.0, 0.0, 0.0, 1.0]);
        }
        let ctx = Ctx { bounds: rect, mode: ColorMode::Rgb, alpha: true };
        let glass = liquid_glass(&mask, rect, &ctx, shape, radius, 28.0, 65.0, 12.0, 85.0, 70.0, -135.0, 4.0, 4.0, [1.0; 4], Some(&scene));
        for y in rect.y0..rect.y1 {
            for x in rect.x0..rect.x1 {
                let i = ((y - rect.y0) as usize * rect.width() as usize + (x - rect.x0) as usize) * 4;
                let o = (y as usize * 800 + x as usize) * 4;
                for c in 0..3 {
                    output.data[o + c] = glass[i + c] * glass[i + 3] + output.data[o + c] * (1.0 - glass[i + 3]);
                }
            }
        }
    }
    let mut ppm = b"P6\n800 600\n255\n".to_vec();
    for p in output.data.as_chunks::<4>().0 {
        for v in &p[..3] {
            ppm.push((v.clamp(0.0, 1.0) * 255.0).round() as u8);
        }
    }
    let img = photocraft_codecs::decode(&ppm)?;
    let png = photocraft_codecs::encode(&img, photocraft_codecs::Format::Png, &Default::default())?;
    std::fs::write(path, png)?;
    Ok(())
}
