//! PhotoHouse's native startup window. One clock drives the counter and the Housinho frames.
//! The editor waits for this short-lived child process; its own viewport is never resized.
//! Anything that goes wrong here (no GPU, tiny texture limit, panic, Esc/Alt+F4) only skips the
//! splash: the editor always opens afterwards.
use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::{Align2, Color32, FontFamily, FontId, Rect, Stroke, StrokeKind, Vec2};

const FLAG: &str = "--photohouse-startup-splash";
const DURATION: Duration = Duration::from_millis(1600);
const FRAME_COUNT: usize = 52;
const COLUMNS: usize = 8;
const ROWS: usize = 7;
/// One frame of the sheet (2880 × 2912, frames downscaled from the 720 × 832 design originals).
const CELL: [usize; 2] = [360, 416];
const GREEN: Color32 = Color32::from_rgb(191, 254, 0);
const SURFACE: Color32 = Color32::from_rgb(36, 37, 37);
const INK: Color32 = Color32::from_rgb(241, 242, 237);
const SHEET: &[u8] = include_bytes!("../../../assets/splash/housinho-frames.webp");

pub fn requested() -> bool {
    std::env::args_os().any(|arg| arg == FLAG)
}

/// Runs the splash as a child process and waits for it. Never blocks the editor from opening.
pub fn before_editor() {
    let status = std::env::current_exe().and_then(|exe| std::process::Command::new(exe).arg(FLAG).status());
    match status {
        Ok(status) if status.success() => log::info!("PhotoHouse startup animation closed; opening editor"),
        Ok(status) => log::warn!("PhotoHouse startup animation failed ({status}); opening editor without it"),
        Err(error) => log::warn!("PhotoHouse startup animation could not start ({error}); opening editor without it"),
    }
}

#[derive(Debug, PartialEq)]
struct Progress {
    percent: u32,
    frame: usize,
    close: bool,
}

fn progress(elapsed: Duration) -> Progress {
    let fraction = (elapsed.as_secs_f64() / DURATION.as_secs_f64()).clamp(0.0, 1.0);
    Progress {
        percent: if fraction >= 1.0 { 100 } else { (fraction * 100.0).floor() as u32 },
        frame: if fraction >= 1.0 { FRAME_COUNT - 1 } else { (fraction * (FRAME_COUNT - 1) as f64).floor() as usize },
        close: elapsed >= DURATION,
    }
}

/// Cuts the sheet into one small texture-sized image per frame, so no texture ever approaches
/// the GPU's `max_texture_side` (2048 on older cards).
fn frames(rgba: &[u8], width: usize, height: usize) -> Result<Vec<egui::ColorImage>, String> {
    if width != CELL[0] * COLUMNS || height != CELL[1] * ROWS || rgba.len() != width * height * 4 {
        return Err(format!("unexpected startup sheet {width}×{height}"));
    }
    Ok((0..FRAME_COUNT)
        .map(|frame| {
            let (x0, y0) = ((frame % COLUMNS) * CELL[0], (frame / COLUMNS) * CELL[1]);
            let mut cell = Vec::with_capacity(CELL[0] * CELL[1] * 4);
            for y in y0..y0 + CELL[1] {
                let row = (y * width + x0) * 4;
                cell.extend_from_slice(&rgba[row..row + CELL[0] * 4]);
            }
            egui::ColorImage::from_rgba_unmultiplied(CELL, &cell)
        })
        .collect())
}

fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PhotoHouse — iniciando")
            .with_app_id("ai.storyteller.photocraft.startup")
            .with_icon(crate::app_icon::window_icon())
            .with_inner_size([720.0, 480.0])
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(false),
        centered: true,
        persist_window: false,
        ..Default::default()
    }
}

pub fn run() -> Result<(), eframe::Error> {
    let options = native_options();
    eframe::run_native(
        "PhotoHouse — iniciando",
        options,
        Box::new(move |cc| {
            let mut fonts = photocraft_ui_egui::egui_fonts::definitions();
            let mut regular = FontFamily::Proportional;
            let mut bold = FontFamily::Proportional;
            // Optional local font sidecars preserve Roboto without committing font binaries.
            if let Some(dir) = std::env::current_exe().ok().and_then(|exe| exe.parent().map(|p| p.join("fonts"))) {
                for (file, name, family) in [("Roboto-Regular.ttf", "splash-regular", &mut regular), ("Roboto-Bold.ttf", "splash-bold", &mut bold)] {
                    if let Ok(bytes) = std::fs::read(dir.join(file))
                        && !bytes.is_empty()
                        && bytes.len() <= 2 * 1024 * 1024
                    {
                        fonts.font_data.insert(name.into(), Arc::new(egui::FontData::from_owned(bytes)));
                        let mut stack = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
                        stack.insert(0, name.into());
                        *family = FontFamily::Name(name.into());
                        fonts.families.insert(family.clone(), stack);
                    }
                }
            }
            cc.egui_ctx.set_fonts(fonts);
            let max_side = cc.egui_ctx.input(|i| i.max_texture_side);
            if CELL[0].max(CELL[1]) > max_side {
                return Err(std::io::Error::other(format!("GPU max texture side {max_side} is too small for the startup animation")).into());
            }
            let image = photocraft_codecs::decode(SHEET).map_err(|error| std::io::Error::other(error.to_string()))?;
            let frames = frames(&image.to_rgba8(), image.width() as usize, image.height() as usize).map_err(std::io::Error::other)?;
            let frames = frames
                .into_iter()
                .enumerate()
                .map(|(i, color)| cc.egui_ctx.load_texture(format!("photohouse-startup-housinho-{i}"), color, egui::TextureOptions::LINEAR))
                .collect();
            Ok(Box::new(Splash { frames, regular, bold, start: None, capture_requested: [false; 3] }))
        }),
    )
}

struct Splash {
    frames: Vec<egui::TextureHandle>,
    regular: FontFamily,
    bold: FontFamily,
    start: Option<Instant>,
    capture_requested: [bool; 3],
}

impl eframe::App for Splash {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [36.0 / 255.0, 37.0 / 255.0, 37.0 / 255.0, 1.0]
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.collect_captures(&ctx);
        // Esc skips the animation; the editor still opens (as with Alt+F4 / window close).
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let elapsed = self.start.get_or_insert_with(Instant::now).elapsed();
        let state = progress(elapsed);
        let rect = ui.max_rect();
        self.paint(ui, rect, &state);
        if ui.interact(rect, egui::Id::new("splash-drag"), egui::Sense::drag()).drag_started() {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        self.request_capture(&ctx, state.percent);
        if state.close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}

impl Splash {
    fn paint(&self, ui: &mut egui::Ui, rect: Rect, state: &Progress) {
        let painter = ui.painter();
        let scale = rect.width() / 720.0;
        let split = rect.left() + rect.width() * 0.4237;
        painter.rect_filled(rect, 0.0, SURFACE);
        let art = Rect::from_min_max(egui::pos2(split, rect.top()), rect.max);
        if let Some(texture) = self.frames.get(state.frame).or(self.frames.last()) {
            egui::Image::new((texture.id(), texture.size_vec2())).paint_at(ui, art);
        }
        // Keep the stroke clear of Windows' rounded outer window clip.
        painter.rect_stroke(rect.shrink(2.0), 14.0, Stroke::new(1.5, GREEN), StrokeKind::Inside);
        let left = rect.left() + 32.0 * scale;
        let right = split - 32.0 * scale;
        let brand = egui::pos2(left, rect.top() + 28.0 * scale);
        let photo = painter.layout_no_wrap("photo".into(), FontId::new(24.0 * scale, self.bold.clone()), INK);
        let house_x = brand.x + photo.size().x;
        painter.galley(brand, photo, INK);
        painter.text(egui::pos2(house_x, brand.y), Align2::LEFT_TOP, "house", FontId::new(24.0 * scale, self.bold.clone()), GREEN);
        let pct = painter.layout_no_wrap("%".into(), FontId::new(35.0 * scale, self.regular.clone()), GREEN);
        let base = painter.layout_no_wrap("100".into(), FontId::new(100.0, self.bold.clone()), INK);
        let size = 100.0 * (right - left - pct.size().x - 9.0 * scale) / base.size().x.max(1.0);
        let number = painter.layout_no_wrap(format!("{:03}", state.percent), FontId::new(size, self.bold.clone()), INK);
        let top = rect.center().y - number.size().y / 2.0;
        let number_bottom = top + number.size().y;
        painter.galley(egui::pos2(left, top), number, INK);
        painter.galley(egui::pos2(right - pct.size().x, top + 3.0 * scale), pct, GREEN);
        let track = Rect::from_min_size(egui::pos2(left, number_bottom + 22.0 * scale), Vec2::new(right - left, 2.0));
        painter.rect_filled(track, 0.0, Color32::from_rgb(66, 68, 66));
        painter.rect_filled(Rect::from_min_size(track.min, Vec2::new(track.width() * state.percent as f32 / 100.0, track.height())), 0.0, GREEN);
        let footer = egui::pos2(left, rect.bottom() - 53.0 * scale);
        let name = painter.layout_no_wrap("The House".into(), FontId::new(13.0 * scale, self.bold.clone()), INK);
        let studio_x = footer.x + name.size().x;
        painter.galley(footer, name, INK);
        painter.text(egui::pos2(studio_x, footer.y), Align2::LEFT_TOP, " Studio", FontId::new(13.0 * scale, self.regular.clone()), INK);
        painter.text(
            footer + Vec2::new(0.0, 14.3 * scale),
            Align2::LEFT_TOP,
            concat!("Versão ", env!("CARGO_PKG_VERSION")),
            FontId::new(11.0 * scale, self.regular.clone()),
            INK.gamma_multiply(0.65),
        );
    }

    /// Explicit, opt-in native-render captures for local visual verification.
    fn request_capture(&mut self, ctx: &egui::Context, percent: u32) {
        if std::env::var_os("PHOTOHOUSE_SPLASH_CAPTURE_DIR").is_none() {
            return;
        }
        for (index, threshold) in [0, 50, 100].into_iter().enumerate() {
            if percent >= threshold
                && let Some(requested) = self.capture_requested.get_mut(index)
                && !*requested
            {
                *requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(threshold)));
            }
        }
    }

    fn collect_captures(&self, ctx: &egui::Context) {
        let Some(dir) = std::env::var_os("PHOTOHOUSE_SPLASH_CAPTURE_DIR").map(std::path::PathBuf::from) else { return };
        let captures: Vec<_> = ctx.input(|i| {
            i.raw
                .events
                .iter()
                .filter_map(|event| {
                    if let egui::Event::Screenshot { user_data, image, .. } = event {
                        Some((*user_data.data.as_ref()?.downcast_ref::<u32>()?, image.clone()))
                    } else {
                        None
                    }
                })
                .collect()
        });
        for (percent, image) in captures {
            let result = (|| -> Result<(), String> {
                let rgba: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                let raw = photocraft_codecs::Image::from_u8(image.width() as u32, image.height() as u32, photocraft_codecs::ChannelLayout::Rgba, rgba)
                    .map_err(|e| e.to_string())?;
                let bytes = photocraft_codecs::encode(&raw, photocraft_codecs::Format::Png, &Default::default()).map_err(|e| e.to_string())?;
                std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                std::fs::write(dir.join(format!("splash-{percent:03}.png")), bytes).map_err(|e| e.to_string())
            })();
            if let Err(error) = result {
                log::warn!("PhotoHouse startup capture: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_is_opaque_and_has_no_outer_margin() {
        let options = native_options();
        assert_eq!(options.viewport.transparent, Some(false));
        assert_eq!(options.viewport.decorations, Some(false));
        assert_eq!(options.viewport.inner_size, Some(Vec2::new(720.0, 480.0)));
    }

    #[test]
    fn animation_and_counter_share_endpoints_and_close_on_the_last_frame() {
        assert_eq!(progress(Duration::ZERO), Progress { percent: 0, frame: 0, close: false });
        assert_eq!(progress(Duration::from_millis(800)), Progress { percent: 50, frame: 25, close: false });
        assert_eq!(progress(Duration::from_millis(1599)), Progress { percent: 99, frame: 50, close: false });
        assert_eq!(progress(DURATION), Progress { percent: 100, frame: 51, close: true });
        assert_eq!(progress(Duration::from_secs(u64::MAX)).frame, 51);
        assert!(DURATION <= Duration::from_secs(2));
    }

    #[test]
    fn embedded_sheet_splits_into_small_frame_textures() {
        let image = photocraft_codecs::decode(SHEET).expect("startup asset decodes");
        let frames = frames(&image.to_rgba8(), image.width() as usize, image.height() as usize).expect("sheet splits");
        assert_eq!(frames.len(), FRAME_COUNT);
        // Fits the smallest real-world limit (GeForce G210: 2048) with room to spare.
        assert!(frames.iter().all(|f| f.size == CELL && f.size[0].max(f.size[1]) <= 2048));
        assert!(super::frames(&[0; 16], 2, 2).is_err());
    }
}
