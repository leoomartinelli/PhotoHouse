//! Liquid Glass optical simulation based on Apple WWDC25 / iOS 26 Liquid Glass principles.
//!
//! Provides screen-space bevel refraction, chromatic dispersion, Fresnel rim lighting,
//! directional highlights, frosted blur diffusion, and shape-aware lens
//! geometry using Signed Distance Fields (SDFs).

use photocraft_geom::Rect;
use serde::{Deserialize, Serialize};

use crate::Ctx;
use crate::image::{Edge, Image};

/// Shape geometry for the Liquid Glass lens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum LiquidGlassShape {
    /// Automatically derives the lens geometry from the layer's alpha silhouette / contour,
    /// or falls back to a rounded rectangle if the layer is fully opaque across its bounds.
    #[default]
    Auto,
    /// Analytic rounded rectangle (or square/rectangle when corner_radius is 0).
    RoundedRect,
    /// Analytic circular lens centered in the reference bounds.
    Circle,
    /// Analytic capsule / pill lens.
    Capsule,
}

/// 1D separable distance transform pass along a slice (Felzenszwalb-Huttenlocher).
fn edt_1d(src: &[f32], dst: &mut [f32], n: usize) {
    if n == 0 {
        return;
    }
    let mut v = vec![0usize; n];
    let mut z = vec![0.0f32; n + 1];
    let mut k = 0usize;
    v[0] = 0;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;

    for q in 1..n {
        let mut s = ((src[q] + (q * q) as f32) - (src[v[k]] + (v[k] * v[k]) as f32)) / (2.0 * (q - v[k]) as f32);
        while s <= z[k] {
            k = k.saturating_sub(1);
            s = ((src[q] + (q * q) as f32) - (src[v[k]] + (v[k] * v[k]) as f32)) / (2.0 * (q - v[k]) as f32);
        }
        k += 1;
        v[k] = q;
        z[k] = s;
        z[k + 1] = f32::INFINITY;
    }

    k = 0;
    for (q, value) in dst.iter_mut().enumerate().take(n) {
        while z[k + 1] < q as f32 {
            k += 1;
        }
        let dx = q as f32 - v[k] as f32;
        *value = dx * dx + src[v[k]];
    }
}

/// 2D Euclidean Distance Transform of a squared-distance float grid.
fn edt_2d(grid: &mut [f32], w: usize, h: usize) {
    if w == 0 || h == 0 {
        return;
    }
    // Column pass
    let mut col_in = vec![0.0f32; h];
    let mut col_out = vec![0.0f32; h];
    for x in 0..w {
        for y in 0..h {
            col_in[y] = grid[y * w + x];
        }
        edt_1d(&col_in, &mut col_out, h);
        for y in 0..h {
            grid[y * w + x] = col_out[y];
        }
    }

    // Row pass
    let mut row_out = vec![0.0f32; w];
    for y in 0..h {
        let row_start = y * w;
        let row_slice = &grid[row_start..row_start + w];
        edt_1d(row_slice, &mut row_out, w);
        grid[row_start..row_start + w].copy_from_slice(&row_out);
    }
}

/// Computes a Signed Distance Field from an alpha image: negative inside, positive outside.
fn compute_alpha_sdf(src: &Image, bounds: Rect) -> (Vec<f32>, usize, usize) {
    let w = bounds.width().max(1) as usize;
    let h = bounds.height().max(1) as usize;
    let inf = 1e8f32;

    let mut inside_grid = vec![inf; w * h];
    let mut outside_grid = vec![inf; w * h];
    let alpha_ch = src.ch.saturating_sub(1);

    for y in 0..h {
        let py = bounds.y0 + y as i32;
        for x in 0..w {
            let px = bounds.x0 + x as i32;
            let a = src.get(px, py, alpha_ch);
            let idx = y * w + x;
            if a >= 0.5 {
                // Inside
                inside_grid[idx] = 0.0;
            } else {
                // Outside
                outside_grid[idx] = 0.0;
            }
        }
    }

    edt_2d(&mut inside_grid, w, h);
    edt_2d(&mut outside_grid, w, h);

    let mut sdf = vec![0.0f32; w * h];
    for i in 0..w * h {
        let d_out = outside_grid[i].sqrt();
        let d_in = inside_grid[i].sqrt();
        // Negative inside, positive outside
        sdf[i] = d_in - d_out;
    }

    (sdf, w, h)
}

/// Analytic Signed Distance Function for a rounded rectangle.
#[inline]
fn sd_rounded_rect(px: f32, py: f32, bx: f32, by: f32, r: f32) -> f32 {
    let qx = px.abs() - (bx - r);
    let qy = py.abs() - (by - r);
    let ox = qx.max(0.0);
    let oy = qy.max(0.0);
    (ox * ox + oy * oy).sqrt() + qx.max(qy).min(0.0) - r
}

/// Analytic Signed Distance Function for a circle.
#[inline]
fn sd_circle(px: f32, py: f32, r: f32) -> f32 {
    (px * px + py * py).sqrt() - r
}

/// Analytic Signed Distance Function for a capsule / pill.
#[inline]
fn sd_capsule(px: f32, py: f32, bx: f32, by: f32) -> f32 {
    if bx >= by {
        let r = by;
        let l = (bx - r).max(0.0);
        let qx = px - px.clamp(-l, l);
        (qx * qx + py * py).sqrt() - r
    } else {
        let r = bx;
        let l = (by - r).max(0.0);
        let qy = py - py.clamp(-l, l);
        (px * px + qy * qy).sqrt() - r
    }
}

/// Evaluates distance and 2D normal for analytic shapes.
fn eval_analytic_sdf(shape: LiquidGlassShape, px: f32, py: f32, bx: f32, by: f32, r: f32) -> (f32, f32, f32) {
    let dist_fn = |x: f32, y: f32| match shape {
        LiquidGlassShape::Circle => sd_circle(x, y, bx.min(by)),
        LiquidGlassShape::Capsule => sd_capsule(x, y, bx, by),
        _ => sd_rounded_rect(x, y, bx, by, r),
    };

    let d = dist_fn(px, py);
    let eps = 0.5f32;
    let dx = dist_fn(px + eps, py) - dist_fn(px - eps, py);
    let dy = dist_fn(px, py + eps) - dist_fn(px, py - eps);
    let len = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = if len > 1e-5 { (dx / len, dy / len) } else { (0.0, 0.0) };
    (d, nx, ny)
}

/// Evaluates distance and normal from a precomputed 2D SDF grid.
fn eval_grid_sdf(sdf: &[f32], w: usize, h: usize, gx: f32, gy: f32) -> (f32, f32, f32) {
    let ix = gx.clamp(0.0, (w - 1) as f32);
    let iy = gy.clamp(0.0, (h - 1) as f32);
    let x0 = (ix.floor() as usize).min(w - 1);
    let y0 = (iy.floor() as usize).min(h - 1);
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let fx = ix - x0 as f32;
    let fy = iy - y0 as f32;

    let d00 = sdf[y0 * w + x0];
    let d10 = sdf[y0 * w + x1];
    let d01 = sdf[y1 * w + x0];
    let d11 = sdf[y1 * w + x1];

    let d0 = d00 + (d10 - d00) * fx;
    let d1 = d01 + (d11 - d01) * fx;
    let d = d0 + (d1 - d0) * fy;

    // Finite differences
    let xm = x0.saturating_sub(1);
    let xp = (x1 + 1).min(w - 1);
    let ym = y0.saturating_sub(1);
    let yp = (y1 + 1).min(h - 1);

    let dx = sdf[y0 * w + xp] - sdf[y0 * w + xm];
    let dy = sdf[yp * w + x0] - sdf[ym * w + x0];
    let len = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = if len > 1e-5 { (dx / len, dy / len) } else { (0.0, 0.0) };

    (d, nx, ny)
}

/// Diffusion and silhouette are computed once and shared by all output tiles.
pub(crate) struct Prepared {
    sdf: Option<(Vec<f32>, usize, usize)>,
    scene: Image,
}

impl Prepared {
    pub(crate) fn new(src: &Image, ctx: &Ctx, shape: LiquidGlassShape, blur: f32, backdrop: Option<&Image>) -> Self {
        let b = ctx.bounds;
        let transparent = ctx.alpha && shape == LiquidGlassShape::Auto && (b.y0..b.y1).any(|y| (b.x0..b.x1).any(|x| src.get(x, y, src.ch - 1) < 0.5));
        let sdf = transparent.then(|| compute_alpha_sdf(src, b.inflate(1)));
        let scene = backdrop.unwrap_or(src);
        let scene = if blur > 0.05 {
            let radius = blur.clamp(0.0, 50.0);
            let pad = (radius * 3.0).ceil() as i32 + 3;
            let rect = scene.rect;
            let expanded = rect.inflate(pad);
            let mut padded = Image::new(expanded, scene.ch);
            for y in expanded.y0..expanded.y1 {
                for x in expanded.x0..expanded.x1 {
                    let offset = ((y - expanded.y0) as usize * expanded.width() as usize + (x - expanded.x0) as usize) * scene.ch;
                    for c in 0..scene.ch {
                        padded.data[offset + c] = scene.get_edge(x, y, c, Edge::Repeat, rect);
                    }
                }
            }
            let scene_ctx = Ctx { bounds: rect, alpha: backdrop.is_some() || ctx.alpha, ..*ctx };
            Image { rect, ch: scene.ch, data: crate::blur::gaussian(&padded, rect, &scene_ctx, radius) }
        } else {
            scene.clone()
        };
        Self { sdf, scene }
    }
}

/// Shape supplies coverage; the layers below supply the optical scene.
#[allow(clippy::too_many_arguments)]
pub fn liquid_glass(
    src: &Image,
    out: Rect,
    ctx: &Ctx,
    shape: LiquidGlassShape,
    corner_radius: f32,
    thickness: f32,
    refraction: f32,
    dispersion: f32,
    rim_light: f32,
    specular: f32,
    light_angle: f32,
    blur: f32,
    tint_opacity: f32,
    tint_color: [f32; 4],
    backdrop: Option<&Image>,
) -> Vec<f32> {
    let prepared = Prepared::new(src, ctx, shape, blur, backdrop);
    render(
        src,
        out,
        ctx,
        shape,
        corner_radius,
        thickness,
        refraction,
        dispersion,
        rim_light,
        specular,
        light_angle,
        tint_opacity,
        tint_color,
        backdrop,
        &prepared,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render(
    src: &Image,
    out: Rect,
    ctx: &Ctx,
    shape: LiquidGlassShape,
    corner_radius: f32,
    thickness: f32,
    refraction: f32,
    dispersion: f32,
    rim_light: f32,
    specular: f32,
    light_angle: f32,
    tint_opacity: f32,
    tint_color: [f32; 4],
    backdrop: Option<&Image>,
    prepared: &Prepared,
) -> Vec<f32> {
    let b = ctx.bounds;
    let (hw, hh) = (b.width() as f32 * 0.5, b.height() as f32 * 0.5);
    let (cx, cy) = (b.x0 as f32 + hw, b.y0 as f32 + hh);
    let radius = corner_radius.clamp(0.0, hw.min(hh));
    let thick = thickness.clamp(1.0, hw.min(hh).max(1.0));
    let scene = &prepared.scene;
    let fmt = photocraft_color::PixelFormat { mode: ctx.mode, sample: photocraft_color::SampleType::F32, alpha: ctx.alpha };
    let mut result = Vec::with_capacity(out.width() as usize * out.height() as usize * src.ch);
    let mut red = vec![0.0; scene.ch];
    let mut green = vec![0.0; scene.ch];
    let mut blue = vec![0.0; scene.ch];
    let mut pixel = vec![0.0; src.ch];
    let chroma = dispersion.clamp(0.0, 100.0) * 0.0012;
    let tint = (tint_opacity * 0.01).clamp(0.0, 1.0) * tint_color[3];
    let (lx, ly) = (light_angle.to_radians().cos(), light_angle.to_radians().sin());
    let rgba = |v: &[f32]| {
        if backdrop.is_some() { [v[0], v[1], v[2], v[3]] } else { photocraft_raster::to_rgba(&fmt, v) }
    };
    for y in out.y0..out.y1 {
        for x in out.x0..out.x1 {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let (d, nx, ny) = if let Some((sdf, w, h)) = &prepared.sdf {
                eval_grid_sdf(sdf, *w, *h, (x - b.x0 + 1) as f32, (y - b.y0 + 1) as f32)
            } else {
                eval_analytic_sdf(shape, px - cx, py - cy, hw, hh, radius)
            };
            let coverage = if prepared.sdf.is_some() { src.get(x, y, src.ch - 1) } else { (0.5 - d).clamp(0.0, 1.0) };
            if coverage <= 0.0 {
                if backdrop.is_some() {
                    pixel.fill(0.0);
                } else {
                    src.sample(px, py, Edge::Transparent, src.rect, ctx.alpha, &mut pixel);
                }
                result.extend_from_slice(&pixel);
                continue;
            }
            // Flat interior, curved bevel. Refraction bends inward, staying continuous
            // at the bevel's inner boundary, with dispersion confined to the rim.
            let bevel = (1.0 - (-d).max(0.0) / thick).clamp(0.0, 1.0);
            let displacement = -refraction * 0.5 * bevel * bevel;
            let (dx, dy) = (nx * displacement, ny * displacement);
            let sample = |k: f32, dst: &mut [f32]| scene.sample(px + dx * k, py + dy * k, Edge::Repeat, scene.rect, backdrop.is_some() || ctx.alpha, dst);
            sample(1.0, &mut green);
            if chroma > 0.0001 && bevel > 0.0 {
                sample(1.0 - chroma, &mut red);
                sample(1.0 + chroma, &mut blue);
            } else {
                red.copy_from_slice(&green);
                blue.copy_from_slice(&green);
            }
            let (r, g, bl) = (rgba(&red), rgba(&green), rgba(&blue));
            let directional = 0.3 + 0.7 * (nx * lx + ny * ly).abs().powi(3);
            let glint = ((-(-d).max(0.0) / 1.15).exp() * 0.65 + bevel.powi(3) * 0.12) * directional * rim_light * 0.01;
            let reflection = bevel.powi(2) * (nx * lx + ny * ly).max(0.0).powi(6) * specular * 0.003;
            let light = (glint + reflection).clamp(0.0, 1.0);
            let mut color = [r[0], g[1], bl[2], g[3] * coverage];
            for c in 0..3 {
                let tinted = color[c] * (1.0 - tint) + tint_color[c] * tint;
                color[c] = tinted * (1.0 - light) + light;
            }
            photocraft_raster::from_rgba_into(&fmt, color, &mut pixel);
            if backdrop.is_none() && coverage < 1.0 {
                for (c, v) in pixel.iter_mut().enumerate() {
                    *v = src.get(x, y, c) * (1.0 - coverage) + *v * coverage;
                }
                if ctx.alpha
                    && let Some(a) = pixel.last_mut()
                {
                    *a = src.get(x, y, src.ch - 1);
                }
            }
            result.extend_from_slice(&pixel);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analytic_sdf_evaluates_exact_shapes() {
        // Circle of radius 50
        assert_eq!(sd_circle(0.0, 0.0, 50.0), -50.0);
        assert_eq!(sd_circle(50.0, 0.0, 50.0), 0.0);
        assert_eq!(sd_circle(60.0, 0.0, 50.0), 10.0);

        // Square of half-size 50, corner radius 0
        assert_eq!(sd_rounded_rect(0.0, 0.0, 50.0, 50.0, 0.0), -50.0);
        assert_eq!(sd_rounded_rect(50.0, 0.0, 50.0, 50.0, 0.0), 0.0);
        assert_eq!(sd_rounded_rect(60.0, 0.0, 50.0, 50.0, 0.0), 10.0);

        // Rounded rect corner
        let d = sd_rounded_rect(45.0, 45.0, 50.0, 50.0, 10.0);
        assert!(d < 0.0);
    }

    #[test]
    fn liquid_glass_refracts_and_adds_rim_light() {
        let b = Rect::new(0, 0, 64, 64);
        let mut img = Image::new(b, 4);
        // Paint grid pattern
        for y in 0..64 {
            for x in 0..64 {
                let v = if (x / 8 + y / 8) % 2 == 0 { 0.8 } else { 0.2 };
                let i = (y * 64 + x) * 4;
                img.data[i] = v;
                img.data[i + 1] = v;
                img.data[i + 2] = v;
                img.data[i + 3] = 1.0;
            }
        }

        let ctx = Ctx { bounds: b, mode: photocraft_color::ColorMode::Rgb, alpha: true };

        let result =
            liquid_glass(&img, b, &ctx, LiquidGlassShape::RoundedRect, 12.0, 16.0, 25.0, 5.0, 80.0, 50.0, 135.0, 0.0, 10.0, [1.0, 1.0, 1.0, 1.0], None);

        assert_eq!(result.len(), 64 * 64 * 4);
        // Center should remain clear and readable
        let center_idx = (32 * 64 + 32) * 4;
        assert!(result[center_idx + 3] > 0.9);
        // Edge should show rim lighting increase
        let edge_idx = (32 * 64 + 4) * 4;
        assert!(result[edge_idx] > 0.0);
    }
    fn glass(src: &Image, bg: &Image, shape: LiquidGlassShape, blur: f32) -> Vec<f32> {
        let ctx = Ctx { bounds: src.rect, mode: photocraft_color::ColorMode::Rgb, alpha: true };
        liquid_glass(src, src.rect, &ctx, shape, 8.0, 8.0, 20.0, 0.0, 0.0, 0.0, 135.0, blur, 0.0, [1.0; 4], Some(bg))
    }

    #[test]
    fn silhouette_is_negative_inside_and_does_not_invert_alpha() {
        let mut src = Image::new(Rect::new(0, 0, 32, 32), 4);
        for y in 8..24 {
            for x in 8..24 {
                let i = (y * 32 + x) * 4;
                src.data[i..i + 4].copy_from_slice(&[1.0, 0.0, 0.0, 1.0]);
            }
        }
        let (sdf, _, _) = compute_alpha_sdf(&src, src.rect);
        assert!(sdf[16 * 32 + 16] < 0.0);
        assert!(sdf[0] > 0.0);
        let mut bg = Image::new(src.rect, 4);
        for p in bg.data.as_chunks_mut::<4>().0 {
            p.copy_from_slice(&[0.1, 0.4, 0.8, 1.0]);
        }
        let out = glass(&src, &bg, LiquidGlassShape::Auto, 0.0);
        assert_eq!(out[3], 0.0);
        assert_eq!(&out[(16 * 32 + 16) * 4..(16 * 32 + 16) * 4 + 4], &[0.1, 0.4, 0.8, 1.0]);
    }

    #[test]
    fn frosting_is_achromatic_and_shape_color_does_not_leak() {
        let mut src = Image::new(Rect::new(0, 0, 32, 32), 4);
        let mut bg = src.clone();
        for (i, p) in bg.data.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let v = if i % 2 == 0 { 1.0 } else { 0.0 };
            p.copy_from_slice(&[v, v, v, 1.0]);
        }
        for p in src.data.as_chunks_mut::<4>().0 {
            p.copy_from_slice(&[1.0, 0.0, 0.0, 1.0]);
        }
        let out = glass(&src, &bg, LiquidGlassShape::RoundedRect, 3.0);
        let center = &out[(16 * 32 + 16) * 4..(16 * 32 + 16) * 4 + 4];
        assert!((center[0] - 0.5).abs() < 0.03);
        assert!((center[0] - center[1]).abs() < 1e-5);
        assert!((center[1] - center[2]).abs() < 1e-5);
        assert_eq!(out[3], 0.0);
    }

    #[test]
    fn rgba_backdrop_supports_gray_and_cmyk_without_channel_mismatch() {
        use photocraft_color::{ColorMode, PixelFormat, SampleType};
        for mode in [ColorMode::Grayscale, ColorMode::Cmyk, ColorMode::Rgb] {
            for sample in [SampleType::U8, SampleType::U16, SampleType::F32] {
                let fmt = PixelFormat { mode, sample, alpha: true };
                let b = Rect::new(0, 0, 32, 32);
                let mut src = Image::new(b, fmt.channels());
                for p in src.data.chunks_exact_mut(fmt.channels()) {
                    *p.last_mut().unwrap() = 1.0;
                }
                let mut bg = Image::new(b, 4);
                for p in bg.data.as_chunks_mut::<4>().0 {
                    p.copy_from_slice(&[0.5, 0.5, 0.5, 1.0]);
                }
                let ctx = Ctx { bounds: b, mode, alpha: true };
                let out = liquid_glass(&src, b, &ctx, LiquidGlassShape::Circle, 8.0, 8.0, 20.0, 5.0, 70.0, 60.0, 135.0, 2.0, 0.0, [1.0; 4], Some(&bg));
                assert_eq!(out.len(), src.data.len());
                assert!(out.iter().all(|v| v.is_finite()));
                assert_eq!(*out.last().unwrap(), 0.0);
            }
        }
    }
}
