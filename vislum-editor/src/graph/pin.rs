use eframe::{
    egui::{Response, Sense, Stroke, Ui, vec2},
    epaint::CircleShape,
};
use vislum_op::value::ValueType;

use crate::util::derive_value_type_color;

pub fn pin_ui(ui: &mut Ui, value_type: &ValueType) -> Response {
    const DIAMETER: f32 = 8.0;

    let (rect, response) =
        ui.allocate_exact_size(vec2(DIAMETER, DIAMETER), Sense::click_and_drag());

    let fill = derive_value_type_color(value_type);

    ui.painter().add(CircleShape {
        center: rect.center(),
        radius: DIAMETER / 2.0,
        fill,
        stroke: Stroke::NONE,
    });

    response
}
