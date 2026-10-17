//! Canvas placement while the Liquid Glass editor is open. Only dialog state changes
//! during a drag; OK commits one smart-filter edit and Cancel leaves the document intact.
use egui::{CursorIcon, Stroke};
use serde_json::{Map, Value, json};

use crate::{PhotocraftApp, canvas::ViewXform, theme::Tokens};

pub const COMMAND: &str = "filter.distort.liquidGlass";

pub fn owns(fields: &Map<String, Value>) -> bool {
    fields.get("__command").and_then(Value::as_str) == Some(COMMAND)
}

fn offset(fields: &Map<String, Value>, key: &str) -> f64 {
    fields.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

pub fn init(app: &PhotocraftApp, fields: &mut Map<String, Value>) {
    let Some(st) = app.session.active() else { return };
    let target = fields.get("__smartFilter").and_then(|f| f.get("layer")).and_then(Value::as_u64).map(photocraft_doc::LayerId).or(st.active_layer);
    let Some(layer) = target.and_then(|id| st.doc.layer(id)) else { return };
    let bounds =
        layer.surface().map(|s| s.content_bounds()).filter(|b| !b.is_empty()).unwrap_or_else(|| photocraft_compose::layer_bounds(layer, st.doc.bounds()));
    let edited = fields.contains_key("__smartFilter");
    let (x, y) = if edited { (offset(fields, "offsetX"), offset(fields, "offsetY")) } else { (0.0, 0.0) };
    fields.insert("layer".into(), json!(layer.id.0));
    fields.insert("__glassDocument".into(), json!(st.doc.id.0));
    fields.insert("__glassBounds".into(), json!([f64::from(bounds.x0) - x, f64::from(bounds.y0) - y, f64::from(bounds.x1) - x, f64::from(bounds.y1) - y]));
    fields.remove("__glassDrag");
}

/// A deterministic document-space drag, also exercised by control-channel UI tests.
pub(crate) fn drag(fields: &mut Map<String, Value>, point: [f64; 2], pressed: bool, held: bool) {
    let Some(bounds) = fields.get("__glassBounds").cloned().and_then(|v| serde_json::from_value::<[f64; 4]>(v).ok()) else { return };
    let (x, y) = (offset(fields, "offsetX"), offset(fields, "offsetY"));
    if pressed && point[0] >= bounds[0] + x && point[0] < bounds[2] + x && point[1] >= bounds[1] + y && point[1] < bounds[3] + y {
        fields.insert("__glassDrag".into(), json!([point[0], point[1], x, y]));
    }
    if let Some(start) = fields.get("__glassDrag").cloned().and_then(|v| serde_json::from_value::<[f64; 4]>(v).ok()) {
        fields.insert("offsetX".into(), json!((start[2] + point[0] - start[0]).round().clamp(-100000.0, 100000.0)));
        fields.insert("offsetY".into(), json!((start[3] + point[1] - start[1]).round().clamp(-100000.0, 100000.0)));
    }
    if !held {
        fields.remove("__glassDrag");
    }
}

pub fn canvas(app: &mut PhotocraftApp, ctx: &egui::Context, painter: &egui::Painter, xf: &ViewXform, canvas: egui::Rect) {
    let Some(doc) = app.session.active().map(|st| st.doc.id.0) else { return };
    let Some(dialog) = app.ui.dialogs.last_mut().filter(|d| owns(&d.fields)) else { return };
    let f = &mut dialog.fields;
    if f.get("__glassDocument").and_then(Value::as_u64) != Some(doc) {
        return;
    }
    let free = crate::dialogs::free_pointer_over(ctx, canvas);
    let (pressed, held) = ctx.input(|i| (i.pointer.primary_pressed(), i.pointer.primary_down()));
    if let Some(p) = ctx.pointer_hover_pos() {
        drag(f, xf.to_doc(p), pressed && free.is_some(), held);
    } else if !held {
        f.remove("__glassDrag");
    }
    if let Some(b) = f.get("__glassBounds").cloned().and_then(|v| serde_json::from_value::<[f64; 4]>(v).ok()) {
        let (x, y) = (offset(f, "offsetX"), offset(f, "offsetY"));
        let points = [(b[0], b[1]), (b[2], b[1]), (b[2], b[3]), (b[0], b[3])].map(|(px, py)| xf.to_screen((px + x) as f32, (py + y) as f32));
        painter.add(egui::Shape::closed_line(points.to_vec(), Stroke::new(1.0, Tokens::get(ctx).accent)));
        if free.is_some_and(|p| {
            let d = xf.to_doc(p);
            d[0] >= b[0] + x && d[0] < b[2] + x && d[1] >= b[1] + y && d[1] < b[3] + y
        }) {
            ctx.set_cursor_icon(if f.contains_key("__glassDrag") { CursorIcon::Grabbing } else { CursorIcon::Grab });
        }
    }
    if held && f.contains_key("__glassDrag") {
        ctx.request_repaint();
    }
}

/// A completed position may be shown while the next drag position is rendering.
/// Everything except position must still match, including the smart-filter target.
pub(crate) fn accepts_preview(old: &crate::filter_dialog::FilterPreviewKey, new: &crate::filter_dialog::FilterPreviewKey) -> bool {
    if old == new {
        return true;
    }
    if old.command != COMMAND {
        return false;
    }
    let mut comparable = old.clone();
    if let (Some(a), Some(b)) = (comparable.params.as_object_mut(), new.params.as_object()) {
        for key in ["offsetX", "offsetY"] {
            a.remove(key);
            if let Some(value) = b.get(key) {
                a.insert(key.into(), value.clone());
            }
        }
    }
    comparable == *new
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intermediate_previews_only_allow_position_to_lag() {
        let old = crate::filter_dialog::FilterPreviewKey {
            doc: photocraft_doc::DocId(1),
            revision: 1,
            dialog: 1,
            active: Some(photocraft_doc::LayerId(2)),
            target: Default::default(),
            command: COMMAND.into(),
            k: 1,
            params: json!({"offsetX":0,"offsetY":0,"blur":2,"__smartFilter":{"layer":2,"index":0}}),
        };
        let mut new = old.clone();
        new.params["offsetX"] = json!(50);
        assert!(accepts_preview(&old, &new));
        new.params["blur"] = json!(10);
        assert!(!accepts_preview(&old, &new));
        new.params["blur"] = json!(2);
        new.params["__smartFilter"]["index"] = json!(1);
        assert!(!accepts_preview(&old, &new));
        new = old.clone();
        new.revision += 1;
        assert!(!accepts_preview(&old, &new));
    }
    #[test]
    fn drag_is_relative_to_press_and_can_resume_after_reopening() {
        let mut f = json!({"__glassBounds":[10,20,110,80],"offsetX":30,"offsetY":-5}).as_object().unwrap().clone();
        drag(&mut f, [50.0, 30.0], true, true);
        drag(&mut f, [80.0, 40.0], false, true);
        drag(&mut f, [90.0, 45.0], false, false);
        assert_eq!(f["offsetX"], 70.0);
        assert_eq!(f["offsetY"], 10.0);
        assert!(!f.contains_key("__glassDrag"));
        drag(&mut f, [100.0, 40.0], true, true);
        drag(&mut f, [90.0, 30.0], false, false);
        assert_eq!(f["offsetX"], 60.0);
        assert_eq!(f["offsetY"], 0.0);
        drag(&mut f, [0.0, 0.0], true, true);
        drag(&mut f, [500.0, 500.0], false, false);
        assert_eq!(f["offsetX"], 60.0, "presses outside the lens must not move it");
    }
    #[test]
    fn editing_previews_one_filter_and_cancel_preserves_the_document() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.run("file.new", json!({"width":96,"height":64,"background":"#3366cc"})).unwrap();
        app.run("shape.create", json!({"kind":"rect","rect":[10,10,20,20],"fill":"#ff0000"})).unwrap();
        app.run(COMMAND, json!({"blur":0,"cornerRadius":0})).unwrap();
        let st = app.session.active().unwrap();
        let id = st.active_layer.unwrap();
        let before = st.doc.clone();
        let photocraft_doc::LayerContent::Smart(sm) = &before.layer(id).unwrap().content else { panic!() };
        crate::smart_ui::open_editor(&mut app, &egui::Context::default(), id.0, sm, 0);
        let dialog = app.ui.dialogs.last().unwrap().id;
        let f = &mut app.ui.dialog_mut(dialog).unwrap().fields;
        assert_eq!(f["__preview"], true);
        let (request, _) = crate::control::ControlRequest::new(
            "ui.pointer",
            json!({"events":[
                {"kind":"down","x":15,"y":15}, {"kind":"move","x":45,"y":25}, {"kind":"up","x":45,"y":25}
            ]}),
        );
        crate::control::handle(&mut app, &egui::Context::default(), &request);
        let request = crate::filter_dialog::preview_params_of(&app.ui.dialog_mut(dialog).unwrap().fields);
        let shown = crate::filter_dialog::preview_document(&before, Some(id), COMMAND, &request, 1).unwrap();
        let photocraft_doc::LayerContent::Smart(preview) = &shown.layer(id).unwrap().content else { panic!() };
        assert_eq!(preview.smart_filters.len(), 1, "preview must edit rather than append");
        assert_eq!(preview.cache.as_ref().unwrap().pixel(15, 15)[3], 0.0);
        assert!(preview.cache.as_ref().unwrap().pixel(45, 25)[3] > 0.9);
        assert_eq!(
            app.session.active().unwrap().doc.layer(id).unwrap().surface().unwrap().pixel(15, 15),
            before.layer(id).unwrap().surface().unwrap().pixel(15, 15)
        );
        crate::dialogs::cancel(&mut app, dialog).unwrap();
        assert_eq!(
            app.session.active().unwrap().doc.layer(id).unwrap().surface().unwrap().pixel(15, 15),
            before.layer(id).unwrap().surface().unwrap().pixel(15, 15)
        );
        crate::smart_ui::open_editor(&mut app, &egui::Context::default(), id.0, sm, 0);
        let dialog = app.ui.dialogs.last().unwrap().id;
        app.ui.dialog_mut(dialog).unwrap().fields.insert("offsetX".into(), json!(30));
        crate::dialogs::confirm(&mut app, dialog).unwrap();
        let photocraft_doc::LayerContent::Smart(edited) = &app.session.active().unwrap().doc.layer(id).unwrap().content else { panic!() };
        assert_eq!(edited.smart_filters.len(), 1);
        assert_eq!(edited.source, sm.source);
        assert!(edited.cache.as_ref().unwrap().pixel(45, 15)[3] > 0.9);
    }
}
