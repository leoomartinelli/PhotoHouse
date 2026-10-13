#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use eframe::egui::{self, Color32, FontId, Pos2, Rect, Vec2};
use image::AnimationDecoder;
use std::{io::Cursor, path::PathBuf, process::Command, sync::mpsc, time::{Duration, Instant}};
use std::os::windows::process::CommandExt;

const MSI: &[u8] = include_bytes!(env!("PHOTOHOUSE_MSI"));
const GIF: &[u8] = include_bytes!("../assets/mascot.gif");
const ICON: &[u8] = include_bytes!("../../photohouse.ico");
const NO_WINDOW: u32 = 0x08000000;
const GREEN: Color32 = Color32::from_rgb(191, 254, 0);
const BG: Color32 = Color32::from_rgb(22, 22, 22);
const TEXT: Color32 = Color32::from_rgb(244, 244, 241);
const DIM: Color32 = Color32::from_rgb(148, 148, 148);

type Result<T> = std::result::Result<T, String>;

fn local_dir() -> Result<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
        .filter(|p| p.is_absolute()).ok_or_else(|| "Não foi possível localizar a pasta do usuário.".into())
}

fn outcome(code: Option<i32>) -> Result<bool> {
    match code {
        Some(0) => Ok(false),
        Some(3010) => Ok(true),
        Some(1618) => Err("Outra instalação está em andamento. Aguarde e tente novamente.".into()),
        Some(1602) => Err("A instalação foi cancelada.".into()),
        Some(1638) => Err("Uma versão mais recente ou incompatível já está instalada.".into()),
        Some(n) => Err(format!("O Windows não concluiu a instalação (código {n}). Consulte o registro de instalação.")),
        None => Err("O processo de instalação terminou sem informar o resultado.".into()),
    }
}

fn install() -> Result<bool> {
    let temp = tempfile::Builder::new().prefix("PhotoHouse-Setup-").tempdir().map_err(|e| e.to_string())?;
    let msi = temp.path().join("PhotoHouse.msi");
    std::fs::write(&msi, MSI).map_err(|e| format!("Não foi possível preparar o instalador: {e}"))?;
    let logs = local_dir()?.join("PhotoHouse/Installer");
    std::fs::create_dir_all(&logs).map_err(|e| e.to_string())?;
    let system = std::env::var_os("SystemRoot").ok_or("Pasta do Windows não encontrada.")?;
    let status = Command::new(PathBuf::from(system).join("System32/msiexec.exe"))
        .arg("/i").arg(&msi).args(["/qn", "/norestart", "REBOOT=ReallySuppress", "MSIRESTARTMANAGERCONTROL=DisableShutdown", "/L*v"])
        .arg(logs.join("install.log")).creation_flags(NO_WINDOW)
        .status().map_err(|e| format!("Não foi possível iniciar o Windows Installer: {e}"))?;
    outcome(status.code())
}

struct Animation {
    frames: Vec<(egui::ColorImage, u64)>,
    duration: u64,
}

impl Animation {
    fn decode() -> Result<Self> {
        let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(GIF)).map_err(|e| e.to_string())?;
        let mut frames = Vec::new();
        let mut duration = 0u64;
        for frame in decoder.into_frames() {
            if frames.len() >= 600 { return Err("A animação excede o limite de quadros.".into()); }
            let frame = frame.map_err(|e| e.to_string())?;
            let (num, den) = frame.delay().numer_denom_ms();
            duration = duration.saturating_add(u64::from(num.checked_div(den).unwrap_or(100).max(20)));
            let rgba = image::imageops::resize(frame.buffer(), 384, 384, image::imageops::FilterType::Triangle);
            frames.push((egui::ColorImage::from_rgba_unmultiplied([384, 384], rgba.as_raw()), duration));
        }
        if frames.is_empty() { return Err("O GIF está vazio.".into()); }
        Ok(Self { frames, duration })
    }

    fn index(&self, elapsed_ms: u64) -> usize {
        let phase = elapsed_ms.checked_rem(self.duration).unwrap_or(0);
        self.frames.iter().position(|(_, end)| phase < *end).unwrap_or(0)
    }
}

enum State { Waiting, Installing, Done(bool), Failed(String) }

struct Setup {
    animation: Animation,
    texture: Option<egui::TextureHandle>,
    frame_index: Option<usize>,
    started: Instant,
    state: State,
    receiver: Option<mpsc::Receiver<Result<bool>>>,
    preview: bool,
    screenshot: Option<PathBuf>,
    shot_requested: bool,
}

impl Setup {
    fn new(cc: &eframe::CreationContext<'_>, animation: Animation, preview: bool, screenshot: Option<PathBuf>) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        let mut style = (*cc.egui_ctx.style_of(egui::Theme::Dark)).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = BG;
        style.visuals.override_text_color = Some(TEXT);
        style.visuals.selection.bg_fill = GREEN;
        cc.egui_ctx.set_style_of(egui::Theme::Dark, style);
        // Use the installed system font; do not redistribute Microsoft's font files.
        if let Some(windows) = std::env::var_os("SystemRoot") {
            if let Ok(font) = std::fs::read(PathBuf::from(windows).join("Fonts/segoeui.ttf")) {
                let mut fonts = egui::FontDefinitions::default();
                fonts.font_data.insert("Segoe UI".into(), egui::FontData::from_owned(font).into());
                fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "Segoe UI".into());
                cc.egui_ctx.set_fonts(fonts);
            }
        }
        Self { animation, texture: None, frame_index: None, started: Instant::now(), state: State::Waiting,
            receiver: None, preview, screenshot, shot_requested: false }
    }
}

impl eframe::App for Setup {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let elapsed = self.started.elapsed();
        if !self.preview && matches!(self.state, State::Waiting) && elapsed > Duration::from_millis(500) {
            let (tx, rx) = mpsc::channel();
            self.receiver = Some(rx);
            self.state = State::Installing;
            let wake = ctx.clone();
            match std::thread::Builder::new().name("windows-installer".into()).spawn(move || {
                let _ = tx.send(install());
                wake.request_repaint();
            }) {
                Ok(_) => {},
                Err(e) => self.state = State::Failed(format!("Não foi possível iniciar: {e}")),
            }
        }
        if let Some(rx) = &self.receiver {
            match rx.try_recv() {
                Ok(result) => { self.state = match result { Ok(reboot) => State::Done(reboot), Err(e) => State::Failed(e) }; self.receiver = None; },
                Err(mpsc::TryRecvError::Disconnected) => { self.state = State::Failed("A instalação foi interrompida. Consulte o registro de instalação.".into()); self.receiver = None; },
                Err(mpsc::TryRecvError::Empty) => {},
            }
        }
        let busy = !self.preview && matches!(self.state, State::Installing);
        if busy && ctx.input(|i| i.viewport().close_requested()) { ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose); }

        let index = self.animation.index(elapsed.as_millis() as u64);
        if self.frame_index != Some(index) {
            if let Some((frame, _)) = self.animation.frames.get(index) {
                if let Some(texture) = &mut self.texture { texture.set(frame.clone(), egui::TextureOptions::LINEAR); }
                else { self.texture = Some(ctx.load_texture("mascot", frame.clone(), egui::TextureOptions::LINEAR)); }
                self.frame_index = Some(index);
            }
        }

        let area = ui.max_rect();
        let center = area.center().x;
        let painter = ui.painter();
        painter.rect_filled(area, 0, BG);
        painter.text(Pos2::new(28.0, 28.0), egui::Align2::LEFT_CENTER, "PhotoHouse", FontId::proportional(17.0), TEXT);
        let close = Rect::from_min_size(Pos2::new(area.right()-46.0, 12.0), Vec2::splat(32.0));
        painter.text(close.center(), egui::Align2::CENTER_CENTER, "×", FontId::proportional(24.0), if busy { Color32::from_gray(65) } else { DIM });
        if ui.interact(close, egui::Id::new("close"), egui::Sense::click()).clicked() && !busy { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
        let drag = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(area.right()-50.0, 50.0));
        if ui.interact(drag, egui::Id::new("drag"), egui::Sense::drag()).drag_started() { ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag); }

        let (title, subtitle) = match &self.state {
            State::Done(true) => ("Tudo pronto. O PhotoHouse é seu!", "Reinicie o Windows antes de abrir o app."),
            State::Done(false) => ("Tudo pronto. O PhotoHouse é seu!", "Sua próxima criação começa aqui."),
            State::Failed(_) => ("Não foi possível concluir", "Você pode tentar novamente com segurança."),
            _ => ("Quase lá, estou instalando o seu app!", "Um novo espaço para suas ideias."),
        };
        ui.painter().text(Pos2::new(center, 94.0), egui::Align2::CENTER_CENTER, title, FontId::proportional(25.0), TEXT);
        ui.painter().text(Pos2::new(center, 131.0), egui::Align2::CENTER_CENTER, subtitle, FontId::proportional(14.0), DIM);
        if let Some(texture) = &self.texture {
            ui.painter().image(texture.id(), Rect::from_center_size(Pos2::new(center, 342.0), Vec2::splat(360.0)),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        }
        let bar = Rect::from_center_size(Pos2::new(center, 554.0), Vec2::new(392.0, 5.0));
        ui.painter().rect_filled(bar, 3, Color32::from_rgb(47, 47, 47));
        match &self.state {
            State::Done(_) => { ui.painter().rect_filled(bar, 3, GREEN); },
            State::Failed(_) => {},
            _ => {
                // Indeterminate progress: no invented percentage while MSI performs its transaction.
                let phase = (elapsed.as_secs_f32() * 0.7).sin() * 0.5 + 0.5;
                let start = bar.left() + phase * (bar.width() - 110.0);
                ui.painter().rect_filled(Rect::from_min_size(Pos2::new(start, bar.top()), Vec2::new(110.0, 5.0)), 3, GREEN);
            }
        }
        match &self.state {
            State::Done(reboot) => {
                if !reboot && ui.put(Rect::from_center_size(Pos2::new(center, 597.0), Vec2::new(200.0, 38.0)),
                    egui::Button::new(egui::RichText::new("Abrir PhotoHouse").color(BG).size(15.0)).fill(GREEN).corner_radius(8)).clicked() {
                    let launched = local_dir().and_then(|p| Command::new(p.join("Programs/PhotoHouse/PhotoHouse.exe")).spawn().map(|_| ()).map_err(|e| e.to_string()));
                    match launched { Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close), Err(e) => self.state = State::Failed(format!("Instalado, mas não foi possível abrir: {e}")) }
                }
            },
            State::Failed(error) => {
                let error = error.clone();
                ui.put(Rect::from_center_size(Pos2::new(center, 598.0), Vec2::new(510.0, 72.0)), egui::Label::new(egui::RichText::new(error).size(13.0).color(DIM)).wrap());
            },
            _ => { ui.painter().text(Pos2::new(center, 584.0), egui::Align2::CENTER_CENTER, "Instalando o PhotoHouse no seu computador…", FontId::proportional(13.0), DIM); },
        }
        ui.painter().text(Pos2::new(center, 652.0), egui::Align2::CENTER_CENTER, "PHOTOHOUSE  /  0.7.0", FontId::proportional(10.0), Color32::from_gray(90));

        if self.preview && self.screenshot.is_some() && elapsed > Duration::from_secs(2) && !self.shot_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default())); self.shot_requested = true;
        }
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                if let Some(path) = &self.screenshot {
                    let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    if let Err(e) = image::save_buffer(path, &bytes, image.width() as u32, image.height() as u32, image::ColorType::Rgba8) { eprintln!("{e}"); }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        ctx.request_repaint_after(Duration::from_millis(20));
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|s| s == "--extract-msi") {
        let path = args.get(1).ok_or("Informe o caminho de saída do MSI.")?;
        std::fs::write(path, MSI)?; return Ok(());
    }
    let preview = args.iter().any(|s| s == "--preview");
    let screenshot = args.iter().position(|s| s == "--screenshot").and_then(|i| args.get(i+1)).map(PathBuf::from);
    let animation = Animation::decode()?;
    let icon = image::load_from_memory_with_format(ICON, image::ImageFormat::Ico)?.into_rgba8();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 680.0]).with_resizable(false)
            .with_decorations(false).with_icon(egui::IconData { width: icon.width(), height: icon.height(), rgba: icon.into_raw() }),
        centered: true,
        ..Default::default()
    };
    eframe::run_native("PhotoHouse — Instalação", options, Box::new(move |cc| Ok(Box::new(Setup::new(cc, animation, preview, screenshot)))))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn msi_exit_codes_do_not_report_failure_as_success() {
        assert_eq!(outcome(Some(0)), Ok(false));
        assert_eq!(outcome(Some(3010)), Ok(true));
        for code in [None, Some(1602), Some(1603), Some(1618), Some(1638)] { assert!(outcome(code).is_err()); }
    }
    #[test]
    fn supplied_gif_is_animated_and_loops() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let animation = Animation::decode()?;
        assert!(animation.frames.len() > 1);
        assert_eq!(animation.index(0), 0);
        assert_eq!(animation.index(animation.duration), 0);
        assert_ne!(animation.index(animation.duration / 2), 0);
        Ok(())
    }
    #[test]
    fn payload_is_an_msi_compound_document() { assert!(MSI.starts_with(&[0xd0,0xcf,0x11,0xe0,0xa1,0xb1,0x1a,0xe1])); }
}
