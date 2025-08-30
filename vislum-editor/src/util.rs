use eframe::egui::{Color32, Stroke, StrokeKind, Ui, epaint::RectShape};

pub fn paint_a_fucking_huge_red_rect(ui: &mut Ui) {
    ui.painter().add(RectShape::new(
        ui.available_rect_before_wrap(), 
        0.0, 
        Color32::RED, 
        Stroke::new(0.0, Color32::RED), 
        StrokeKind::Inside));
}