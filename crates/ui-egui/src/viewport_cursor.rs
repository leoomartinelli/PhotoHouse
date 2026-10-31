//! PhotoHouse viewport pointers. A white Difference mask inverts the final framebuffer,
//! including guides and handles; the OS cursor cannot blend with application pixels.
use eframe::{
    egui_wgpu::{self, CallbackResources, CallbackTrait, RenderState},
    wgpu,
};
use egui::{CursorIcon, Painter, Pos2, Rect, vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Default,
    Move,
    Copy,
}

impl Kind {
    fn index(self) -> usize {
        match self {
            Self::Default => 0,
            Self::Move => 1,
            Self::Copy => 2,
        }
    }
    fn asset(self) -> (&'static [u8], [f32; 2], [f32; 2]) {
        match self {
            Self::Default => (include_bytes!("../../../assets/cursors/cursor_padrão.svg"), [512.0, 512.0], [95.66, 25.92]),
            Self::Move => (include_bytes!("../../../assets/cursors/cursor_move.svg"), [604.58, 646.8], [169.38, 152.24]),
            Self::Copy => (include_bytes!("../../../assets/cursors/cursor_copy_alt.svg"), [700.58, 537.58], [111.09, 29.35]),
        }
    }
}

fn mask(kind: Kind) -> Result<egui::ColorImage, String> {
    let (svg, size, _) = kind.asset();
    egui_extras::image::load_svg_bytes_with_size(
        svg,
        egui::load::SizeHint::Size { width: (size[0] / 8.0).ceil() as u32, height: (size[1] / 8.0).ceil() as u32, maintain_aspect_ratio: true },
        &Default::default(),
    )
}

struct Resources {
    pipeline: wgpu::RenderPipeline,
    masks: Vec<wgpu::BindGroup>,
}

pub(crate) fn install(rs: &RenderState) -> bool {
    let device = &rs.device;
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("viewport cursor mask"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let sampler =
        device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
    let mut masks = Vec::new();
    for kind in [Kind::Default, Kind::Move, Kind::Copy] {
        let image = match mask(kind) {
            Ok(image) => image,
            Err(error) => {
                log::error!("cursor SVG: {error}");
                return false;
            }
        };
        let size = wgpu::Extent3d { width: image.width() as u32, height: image.height() as u32, depth_or_array_layers: 1 };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("viewport cursor"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let pixels: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
        rs.queue.write_texture(
            texture.as_image_copy(),
            &pixels,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(size.width * 4), rows_per_image: Some(size.height) },
            size,
        );
        let view = texture.create_view(&Default::default());
        masks.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewport cursor"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        }));
    }
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("cursor difference"),
        source: wgpu::ShaderSource::Wgsl(include_str!("viewport_cursor.wgsl").into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("cursor difference"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });
    // coverage * (1 - destination) + destination * (1 - coverage).
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("cursor difference"),
        layout: Some(&layout),
        vertex: wgpu::VertexState { module: &module, entry_point: Some("vs"), buffers: &[], compilation_options: Default::default() },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: rs.target_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::OneMinusDst,
                        dst_factor: wgpu::BlendFactor::OneMinusSrc,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                }),
                write_mask: wgpu::ColorWrites::RED | wgpu::ColorWrites::GREEN | wgpu::ColorWrites::BLUE,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    rs.renderer.write().callback_resources.insert(Resources { pipeline, masks });
    true
}

pub(crate) fn show(painter: &Painter, at: Pos2, kind: Kind, gpu: bool) -> CursorIcon {
    let (_, size, hotspot) = kind.asset();
    let scale = 26.0 / 512.0;
    let rect = Rect::from_min_size(at - vec2(hotspot[0], hotspot[1]) * scale, vec2(size[0], size[1]) * scale);
    if !gpu {
        // Non-wgpu hosts retain the supplied artwork and a visible pointer.
        let id = egui::Id::new(("viewport-cursor-fallback", kind.index()));
        let cached = painter.ctx().data(|d| d.get_temp::<egui::TextureHandle>(id));
        let texture = if let Some(texture) = cached {
            texture
        } else {
            let Ok(image) = mask(kind) else { return CursorIcon::Default };
            // Texture creation also reads the context: never call it under data_mut's write lock.
            let texture = painter.ctx().load_texture("viewport cursor fallback", image, egui::TextureOptions::LINEAR);
            painter.ctx().data_mut(|d| d.insert_temp(id, texture.clone()));
            texture
        };
        painter.image(texture.id(), rect, Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
        return CursorIcon::None;
    }
    painter.add(egui_wgpu::Callback::new_paint_callback(rect, Pointer(kind)));
    CursorIcon::None
}

struct Pointer(Kind);
impl CallbackTrait for Pointer {
    fn paint(&self, _info: egui::PaintCallbackInfo, pass: &mut wgpu::RenderPass<'static>, resources: &CallbackResources) {
        let Some(res) = resources.get::<Resources>() else { return };
        let Some(mask) = res.masks.get(self.0.index()) else { return };
        pass.set_pipeline(&res.pipeline);
        pass.set_bind_group(0, mask, &[]);
        pass.draw(0..6, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Preview {
        pointers: bool,
    }
    impl eframe::App for Preview {
        fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
            egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
                let painter = ui.painter();
                for (col, color) in [egui::Color32::BLACK, egui::Color32::WHITE, egui::Color32::from_rgb(40, 170, 210)].into_iter().enumerate() {
                    let x = col as f32 * 160.0;
                    painter.rect_filled(Rect::from_min_size(egui::pos2(x, 0.0), vec2(160.0, 180.0)), 0.0, color);
                    // A UI overlay must be inverted too, not only document pixels.
                    painter.rect_filled(Rect::from_min_size(egui::pos2(x + 75.0, 0.0), vec2(2.0, 180.0)), 0.0, egui::Color32::RED);
                    if self.pointers {
                        for (row, kind) in [Kind::Default, Kind::Move, Kind::Copy].into_iter().enumerate() {
                            show(painter, egui::pos2(x + 70.0, 25.0 + row as f32 * 50.0), kind, true);
                        }
                    }
                }
            });
        }
    }

    #[test]
    fn gpu_difference_inverts_background_and_overlays() {
        let _gpu = photocraft_testkit::gpu_lock();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(480.0, 180.0)).with_pixels_per_point(2.0).wgpu().build_eframe(|cc| {
            assert!(install(cc.wgpu_render_state.as_ref().unwrap()));
            Preview { pointers: false }
        });
        h.run_steps(2);
        let before = h.render().unwrap();
        h.state_mut().pointers = true;
        h.run_steps(2);
        let after = h.render().unwrap();
        let mut changed = [0; 3];
        let mut overlay_changed = 0;
        for (a, b) in before.pixels().zip(after.pixels()) {
            if a != b {
                for channel in 0..3 {
                    if a[channel].abs_diff(b[channel]) > 30 {
                        changed[channel] += 1;
                    }
                }
                if a[0] > 250 && a[1] < 5 && a[2] < 5 && b[0] < 10 && b[1] > 245 && b[2] > 245 {
                    overlay_changed += 1;
                }
            }
        }
        assert!(changed.into_iter().all(|n| n > 500));
        assert!(overlay_changed > 20, "the red overlay must become cyan beneath the cursor");
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/design/viewport-cursors");
        std::fs::create_dir_all(&path).unwrap();
        after.save(path.join("difference-preview.png")).unwrap();
    }
    #[test]
    fn supplied_svg_masks_have_transparency_and_content() {
        for kind in [Kind::Default, Kind::Move, Kind::Copy] {
            let image = mask(kind).unwrap();
            assert!(image.pixels.iter().any(|p| p.a() == 0));
            assert!(image.pixels.iter().any(|p| p.a() == 255));
            let (_, size, hot) = kind.asset();
            assert!(hot[0] > 0.0 && hot[0] < size[0] && hot[1] > 0.0 && hot[1] < size[1]);
        }
    }

    #[test]
    fn non_gpu_cursor_creates_and_reuses_its_texture_without_locking_context() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            let painter = ui.painter();
            for kind in [Kind::Default, Kind::Move, Kind::Copy] {
                assert_eq!(show(painter, egui::pos2(20.0, 20.0), kind, false), CursorIcon::None);
                assert_eq!(show(painter, egui::pos2(30.0, 20.0), kind, false), CursorIcon::None);
            }
        });
        output.textures_delta.clear();
    }

    #[test]
    fn move_alt_restores_cursor_and_leaving_canvas_restores_os_pointer() {
        let _gpu = photocraft_testkit::gpu_lock();
        use egui::{Event, Modifiers};
        let mut h = egui_kittest::Harness::builder().with_size(vec2(800.0, 600.0)).with_max_steps(64).wgpu().build_eframe(|cc| {
            crate::PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
            let mut app = crate::PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
            app.set_wgpu(cc.wgpu_render_state.clone().unwrap());
            app.run("file.new", serde_json::json!({"width": 200, "height": 200})).unwrap();
            app.session.prefs.edit(|p| p.cursors.other = photocraft_engine::prefs::OtherCursor::Standard);
            app.ui.tool = crate::Tool::Move;
            app
        });
        h.run_steps(4);
        let at = h.state().last_canvas_rect.center();
        h.event(Event::PointerMoved(at));
        h.run_steps(4);
        assert_eq!(h.output().platform_output.cursor_icon, CursorIcon::None);
        let normal = h.render().unwrap();
        h.event(Event::ModifiersChanged(Modifiers::ALT));
        h.run_steps(4);
        let copy = h.render().unwrap();
        // Compare only the cursor region, excluding tooltips and modifier-dependent panels.
        assert!((at.y as u32..at.y as u32 + 35).any(|y| (at.x as u32..at.x as u32 + 35).any(|x| normal.get_pixel(x, y) != copy.get_pixel(x, y))));
        h.event(Event::ModifiersChanged(Modifiers::NONE));
        h.run_steps(4);
        let restored = h.render().unwrap();
        for y in at.y as u32..at.y as u32 + 35 {
            for x in at.x as u32..at.x as u32 + 35 {
                assert_eq!(normal.get_pixel(x, y), restored.get_pixel(x, y));
            }
        }
        h.event(Event::PointerMoved(egui::pos2(10.0, 10.0)));
        h.run_steps(4);
        assert_ne!(h.output().platform_output.cursor_icon, CursorIcon::None);
        assert!(h.output().platform_output.cursor_image.is_none());
    }
}
