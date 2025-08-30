use std::hash::Hash;

use eframe::egui::{self, Color32, Stroke, StrokeKind, Ui, epaint::RectShape};
use vislum_math::{Vector2I, vec2i};
use vislum_op::value::{SValueTypeInfo, ValueType};

pub fn paint_a_fucking_huge_red_rect(ui: &mut Ui) {
    ui.painter().add(RectShape::new(
        ui.available_rect_before_wrap(), 
        0.0, 
        Color32::RED, 
        Stroke::new(0.0, Color32::RED), 
        StrokeKind::Inside));
}

pub trait IntoVector2I {
    fn into_vector2i(self) -> Vector2I;
}

impl IntoVector2I for egui::Pos2 {
    fn into_vector2i(self) -> Vector2I {
        vec2i(self.x as i32, self.y as i32)
    }
}

/// Derives a color from a value type.
///
/// The color is derived from the value type's id.
/// The color is guaranteed to be different for different value types.
/// The color is guaranteed to be consistent for the same value type.
pub fn derive_value_type_color(data_type: &ValueType) -> Color32 {
    pub use std::hash::{DefaultHasher, Hasher};

    let mut state = DefaultHasher::new();
    data_type.id.hash(&mut state);
    let hash = state.finish();
    
    // Convert hash to HSL values
    let hue = (hash % 360) as f32;
    let saturation = 0.6f32; // Fixed saturation for consistent vibrancy
    let lightness = 0.6f32;  // Fixed lightness to ensure visibility
    
    // Convert HSL to RGB
    let c = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let h = hue / 60.0;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = lightness - c/2.0;

    let (r1, g1, b1) = if h < 1.0 {
        (c, x, 0.0)
    } else if h < 2.0 {
        (x, c, 0.0)
    } else if h < 3.0 {
        (0.0, c, x)
    } else if h < 4.0 {
        (0.0, x, c)
    } else if h < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r1 + m) * 255.0) as u8;
    let g = ((g1 + m) * 255.0) as u8;
    let b = ((b1 + m) * 255.0) as u8;

    Color32::from_rgb(r, g, b)
}