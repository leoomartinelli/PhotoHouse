//! The PhotoHouse mascot (the paint roller with eyes) as a live egui widget. It floats, blinks and
//! follows the pointer with its eyes, as `assets/mascote/photohouse-mascote-parado.svg` does in a
//! browser; that SVG's CSS and script can't run in egui, so this module redoes them in Rust.
//!
//! The drawing is three layers cut from that SVG (`assets/mascote/camadas/`): the body and each
//! eye, in the SVG's own units. Each is rasterized once for the on-screen pixel size (crisp at any
//! DPI, never past `max_texture_side`) and kept in the context; a frame only moves textured quads.

use std::f32::consts::TAU;

use egui::{Color32, Context, Id, Mesh, Pos2, Rect, Rot2, Sense, Shape, TextureHandle, TextureOptions, Ui, Vec2, pos2, vec2};

/// The layers and where each sits in the SVG's units (their `viewBox`es; a test keeps them in sync).
const LAYERS: [(&str, &[u8], Rect); 3] = [
    ("corpo", include_bytes!("../../../assets/mascote/camadas/corpo.svg"), rect(-2.0, 86.0, 2269.0, 1860.0)),
    ("olho-esquerdo", include_bytes!("../../../assets/mascote/camadas/olho-esquerdo.svg"), rect(834.0, 417.0, 289.0, 469.0)),
    ("olho-direito", include_bytes!("../../../assets/mascote/camadas/olho-direito.svg"), rect(1237.0, 338.0, 288.0, 468.0)),
];
const BODY: Rect = LAYERS[0].2;

/// The shadow under the roller (an ellipse in the SVG).
const SHADOW_CENTER: Pos2 = pos2(1300.0, 2075.0);
const SHADOW_RADIUS: Vec2 = vec2(420.0, 55.0);
/// Everything the animation can reach: the body at the top of its float, and the shadow.
const FRAME: Rect = Rect { min: pos2(-10.0, 86.0 - FLOAT_HEIGHT - 10.0), max: pos2(2275.0, 2140.0) };
/// Height over width of the drawing.
pub const ASPECT: f32 = (FRAME.max.y - FRAME.min.y) / (FRAME.max.x - FRAME.min.x);

/// `flutuar`: a 3.2 s bob up to 120 units with a ±1.5° sway, pivoting on the bottom of the body.
const FLOAT_PERIOD: f64 = 3.2;
const FLOAT_HEIGHT: f32 = 120.0;
const SWAY: f32 = 1.5 * TAU / 360.0;
/// `piscar`: eye height (1 open) over a 5.3 s cycle, a blink then a double blink.
const BLINK_PERIOD: f64 = 5.3;
const BLINK: [(f32, f32); 10] =
    [(0.0, 1.0), (0.38, 1.0), (0.40, 0.06), (0.42, 1.0), (0.79, 1.0), (0.805, 0.06), (0.815, 1.0), (0.83, 0.06), (0.84, 1.0), (1.0, 1.0)];
/// `olhar`: where the eyes wander over 9 s while nobody moves the pointer.
const LOOK_PERIOD: f64 = 9.0;
const LOOK: [(f32, Vec2); 12] = [
    (0.0, Vec2::ZERO),
    (0.14, Vec2::ZERO),
    (0.17, vec2(-70.0, 15.0)),
    (0.33, vec2(-70.0, 15.0)),
    (0.36, Vec2::ZERO),
    (0.50, Vec2::ZERO),
    (0.53, vec2(70.0, -25.0)),
    (0.70, vec2(70.0, -25.0)),
    (0.73, vec2(30.0, 30.0)),
    (0.84, vec2(30.0, 30.0)),
    (0.87, Vec2::ZERO),
    (1.0, Vec2::ZERO),
];
/// The script's pointer following: the eyes lean toward the pointer, at most 90 units (less
/// vertically), reaching that 900 units from the point between them.
const EYES_CENTER: Pos2 = pos2(1180.0, 610.0);
const EYES_REACH: f32 = 90.0;
const EYES_FULL_AT: f32 = 900.0;
/// After the pointer stops this long the eyes go back to wandering, as in the SVG.
const FOLLOW_FOR: f32 = 2.5;
/// Time constant of the eyes' easing toward their target (the SVG's 0.15 s ease-out).
const EASE: f32 = 0.06;

const fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { min: pos2(x, y), max: pos2(x + w, y + h) }
}

/// The animation at time `t` (seconds), in SVG units.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pose {
    /// How far the body is lifted (0..=FLOAT_HEIGHT).
    lift: f32,
    /// Sway in radians.
    tilt: f32,
    /// Eye height, 1 open, 0.06 shut.
    blink: f32,
}

fn pose(t: f64) -> Pose {
    let float = ((t / FLOAT_PERIOD).fract() as f32) * TAU;
    let blink = ((t / BLINK_PERIOD).fract()) as f32;
    Pose { lift: FLOAT_HEIGHT * (1.0 - float.cos()) / 2.0, tilt: -SWAY * float.sin(), blink: keyframes(&BLINK, blink, |a, b, f| a + (b - a) * f) }
}

/// The idle eye offset at time `t`, eased between the `olhar` keyframes.
fn wander(t: f64) -> Vec2 {
    let at = (t / LOOK_PERIOD).fract() as f32;
    keyframes(&LOOK, at, |a, b, f| a + (b - a) * (f * f * (3.0 - 2.0 * f)))
}

/// Where the eyes look for a pointer at `p` (SVG units).
fn follow(p: Pos2) -> Vec2 {
    let d = p - EYES_CENTER;
    let len = d.length().max(1.0);
    let k = (len / EYES_FULL_AT).min(1.0) * EYES_REACH;
    vec2(d.x / len * k, d.y / len * k * 0.7)
}

fn keyframes<T: Copy>(keys: &[(f32, T)], at: f32, mix: impl Fn(T, T, f32) -> T) -> T {
    let i = keys.iter().rposition(|k| k.0 <= at).unwrap_or(0).min(keys.len() - 2);
    let ((t0, a), (t1, b)) = (keys[i], keys[i + 1]);
    mix(a, b, ((at - t0) / (t1 - t0)).clamp(0.0, 1.0))
}

/// Pixel size to rasterize a `units`-sized layer at `px_per_unit`, within the GPU's texture limit.
fn raster_size(units: Vec2, px_per_unit: f32, max_side: usize) -> [u32; 2] {
    let px = units * px_per_unit;
    let max = max_side as f32;
    let fit = (max / px.max_elem()).min(1.0);
    [(px.x * fit).ceil().clamp(1.0, max) as u32, (px.y * fit).ceil().clamp(1.0, max) as u32]
}

#[derive(Clone)]
struct Textures {
    px_per_unit: f32,
    layers: Vec<TextureHandle>,
}

/// The layer textures for `px_per_unit`, rounded up to a step of 2^(1/8) so resizing the window
/// rasterizes again only every ~9 %.
fn textures(ctx: &Context, px_per_unit: f32) -> Option<Vec<TextureHandle>> {
    let step = 2f32.powf((px_per_unit.max(1e-4).log2() * 8.0).ceil() / 8.0);
    let id = Id::new("photohouse-mascote-texturas");
    if let Some(t) = ctx.data(|d| d.get_temp::<Textures>(id))
        && t.px_per_unit == step
    {
        return Some(t.layers);
    }
    let max_side = ctx.input(|i| i.max_texture_side);
    let options = TextureOptions { mipmap_mode: Some(egui::TextureFilter::Linear), ..TextureOptions::LINEAR };
    let mut layers = Vec::with_capacity(LAYERS.len());
    for (name, svg, units) in LAYERS {
        let [width, height] = raster_size(units.size(), step, max_side);
        let hint = egui::load::SizeHint::Size { width, height, maintain_aspect_ratio: false };
        match egui_extras::image::load_svg_bytes_with_size(svg, hint, &Default::default()) {
            Ok(image) => layers.push(ctx.load_texture(format!("photohouse-mascote-{name}"), image, options)),
            Err(e) => {
                log::warn!("mascote: {name}: {e}");
                return None;
            }
        }
    }
    ctx.data_mut(|d| d.insert_temp(id, Textures { px_per_unit: step, layers: layers.clone() }));
    Some(layers)
}

/// Where the eyes are looking (eased), kept per widget.
#[derive(Clone, Copy, Default)]
struct Eyes(Vec2);

/// A mascot `width` points wide; its height follows the drawing.
pub fn mascote(ui: &mut Ui, width: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, width * ASPECT), Sense::hover());
    paint(ui, rect, response.id);
    response
}

/// Paint the mascot centred in `rect`, animated. `id` keeps its eyes' state between frames.
pub fn paint(ui: &Ui, rect: Rect, id: Id) {
    let ctx = ui.ctx();
    let scale = (rect.width() / FRAME.width()).min(rect.height() / FRAME.height());
    let origin = rect.center() - FRAME.center().to_vec2() * scale;
    let to_screen = |p: Pos2| origin + p.to_vec2() * scale;
    let to_units = |p: Pos2| ((p - origin) / scale).to_pos2();

    let (t, dt, pointer, still) = ctx.input(|i| (i.time, i.stable_dt.clamp(0.0, 0.1), i.pointer.hover_pos(), i.pointer.time_since_last_movement()));
    let pose = pose(t);
    let target = match pointer {
        Some(p) if still < FOLLOW_FOR => follow(to_units(p)),
        _ => wander(t),
    };
    let eyes = ctx.data(|d| d.get_temp::<Eyes>(id)).map_or(target, |e| e.0 + (target - e.0) * (1.0 - (-dt / EASE).exp()));
    ctx.data_mut(|d| d.insert_temp(id, Eyes(eyes)));
    ctx.request_repaint();

    let shadow = pose.lift / FLOAT_HEIGHT;
    let alpha = ((0.22 - 0.10 * shadow) * 255.0) as u8;
    let radius = vec2(SHADOW_RADIUS.x * (1.0 - 0.2 * shadow), SHADOW_RADIUS.y) * scale;
    ui.painter().add(Shape::ellipse_filled(to_screen(SHADOW_CENTER), radius, Color32::from_black_alpha(alpha)));

    let Some(textures) = textures(ctx, scale * ctx.pixels_per_point()) else { return };
    let lift = vec2(0.0, -pose.lift * scale);
    let pivot = to_screen(pos2(BODY.center().x, BODY.max.y)) + lift;
    let uv = Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0));
    for (i, ((_, _, units), texture)) in LAYERS.iter().zip(&textures).enumerate() {
        let mut r = Rect::from_min_max(to_screen(units.min), to_screen(units.max)).translate(lift);
        if i > 0 {
            r = Rect::from_center_size(r.center() + eyes * scale, vec2(r.width(), r.height() * pose.blink));
        }
        let mut mesh = Mesh::with_texture(texture.id());
        mesh.add_rect_with_uv(r, uv, Color32::WHITE);
        mesh.rotate(Rot2::from_angle(pose.tilt), pivot);
        ui.painter().add(Shape::mesh(mesh));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_boxes_match_their_view_boxes() {
        for (name, svg, units) in LAYERS {
            let svg = std::str::from_utf8(svg).unwrap();
            let vb: Vec<f32> = svg.split("viewBox=\"").nth(1).unwrap().split('"').next().unwrap().split(' ').map(|v| v.parse().unwrap()).collect();
            assert_eq!(rect(vb[0], vb[1], vb[2], vb[3]), units, "{name}");
            assert!(FRAME.contains_rect(units), "{name}");
        }
    }

    #[test]
    fn floats_and_blinks_like_the_svg() {
        assert_eq!(pose(0.0), Pose { lift: 0.0, tilt: 0.0, blink: 1.0 });
        let top = pose(FLOAT_PERIOD / 2.0);
        assert!((top.lift - FLOAT_HEIGHT).abs() < 1e-3 && top.tilt.abs() < 1e-4);
        assert!((pose(FLOAT_PERIOD / 4.0).tilt + SWAY).abs() < 1e-4, "leans left on the way up");
        for shut in [0.40, 0.805, 0.83] {
            assert!((pose(BLINK_PERIOD * shut).blink - 0.06).abs() < 1e-3, "{shut}");
        }
        assert_eq!(pose(BLINK_PERIOD * 0.6).blink, 1.0);
    }

    #[test]
    fn eyes_follow_the_pointer_within_natural_limits() {
        assert_eq!(follow(EYES_CENTER), Vec2::ZERO);
        let right = follow(EYES_CENTER + vec2(5000.0, 0.0));
        assert!((right.x - EYES_REACH).abs() < 1e-3 && right.y == 0.0);
        let down = follow(EYES_CENTER + vec2(0.0, 5000.0));
        assert!((down.y - EYES_REACH * 0.7).abs() < 1e-3, "less reach vertically");
        assert!((follow(EYES_CENTER + vec2(-450.0, 0.0)).x + EYES_REACH / 2.0).abs() < 1e-3, "closer, less lean");
        for t in 0..90 {
            assert!(wander(t as f64 / 10.0).length() <= 80.0);
        }
    }

    #[test]
    fn textures_fit_the_gpu_limit() {
        assert_eq!(raster_size(vec2(2269.0, 1860.0), 0.5, 2048), [1135, 930]);
        let [w, h] = raster_size(vec2(2269.0, 1860.0), 4.0, 2048);
        assert!(w == 2048 && h < 2048, "{w}x{h}");
    }

    #[test]
    fn rasterizes_once_then_only_moves_quads_and_the_eyes_turn_to_the_pointer() {
        let ctx = Context::default();
        let id = Id::new("mascote-test");
        let rect = Rect::from_min_size(Pos2::ZERO, vec2(200.0, 200.0 * ASPECT));
        let mut frame = |pointer: Option<Pos2>| {
            let mut input = egui::RawInput::default();
            input.events.extend(pointer.map(egui::Event::PointerMoved));
            let mut out = ctx.run_ui(input, |ui| paint(ui, rect, id));
            let uploads = out.textures_delta.set.iter().filter(|(t, d)| *t != egui::TextureId::default() && d.pos.is_none()).count();
            out.textures_delta.clear();
            uploads
        };
        assert_eq!(frame(None), LAYERS.len(), "body and both eyes");
        for _ in 0..30 {
            assert_eq!(frame(Some(pos2(400.0, 0.0))), 0, "no new texture while animating");
        }
        let eyes = ctx.data(|d| d.get_temp::<Eyes>(id)).unwrap().0;
        assert!(eyes.x > 0.8 * EYES_REACH && eyes.y < 0.0, "up and to the right: {eyes:?}");
        for _ in 0..30 {
            frame(Some(pos2(-400.0, 400.0)));
        }
        let eyes = ctx.data(|d| d.get_temp::<Eyes>(id)).unwrap().0;
        assert!(eyes.x < -0.5 * EYES_REACH && eyes.y > 0.0 && eyes.length() <= EYES_REACH, "down and to the left: {eyes:?}");
    }
}
