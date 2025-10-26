use std::collections::HashMap;

use eframe::egui::{
    self, Align, Color32, Frame, Id, Label, Layout, Margin, Rect, RichText, Sense, Stroke,
    UiBuilder, Widget, WidgetText, vec2,
};
use vislum_op::{node::InputId, prelude::*, system::NodeGraphSystem};

use crate::graph::{GraphElementPositioning, pin::pin_ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeOutputKey {
    node_id: NodeId,
    index: OutputId,
}

impl NodeOutputKey {
    pub const fn new(node_id: NodeId, index: OutputId) -> Self {
        Self { node_id, index }
    }
}

/// Uniquely identifies a node input virtual or real slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeInputVirtualSlotKey {
    node_id: NodeId,
    index: InputId,
    placement: ConnectionPlacement,
}

impl NodeInputVirtualSlotKey {
    pub const fn new(node_id: NodeId, index: InputId, placement: ConnectionPlacement) -> Self {
        Self {
            node_id,
            index,
            placement,
        }
    }

    /// Returns true if the slot is virtual.
    #[inline]
    pub fn is_virtual(&self) -> bool {
        false
        //matches!(self.placement, ConnectionPlacement::End)
    }

    /// Returns true if the slot is real.
    #[inline]
    pub fn is_real(&self) -> bool {
        !self.is_virtual()
    }

    #[inline]
    pub fn index(&self) -> InputId {
        self.index
    }

    #[inline]
    pub fn connection_placement(&self) -> ConnectionPlacement {
        self.placement
    }
}

pub struct NodeView<'a> {
    pub node_id: NodeId,
    pub node: &'a NodeBlueprint,
    pub element_positioning: &'a mut GraphElementPositioning,
    pub actions: Vec<NodeAction>,
}

impl<'a> NodeView<'a> {
    pub fn new(
        node_id: NodeId,
        node: &'a NodeBlueprint,
        element_positioning: &'a mut GraphElementPositioning,
    ) -> Self {
        Self {
            node_id,
            node,
            element_positioning,
            actions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NodeAction {
    TitleClicked,
    TitleDoubleClicked,
    TitleDragStarted,
    TitleDragged(egui::Pos2),
    TitleDragStopped,
    BeginDraggingInput(NodeInputVirtualSlotKey),
    DraggedInput(NodeInputVirtualSlotKey, egui::Pos2),
    EndDraggingInput(NodeInputVirtualSlotKey),
    Delete,
}

pub struct NodeResponse {
    pub node_id: NodeId,
    pub actions: Vec<NodeAction>,
}

impl<'a> NodeView<'a> {
    pub fn ui(mut self, ui: &mut egui::Ui) -> NodeResponse {
        let position = self.node.position;

        let ui_builder = UiBuilder::new()
            .id_salt(("node", self.node_id))
            .max_rect(Rect::from_min_size(
                egui::pos2(position.x() as f32, position.y() as f32),
                vec2(200.0, 100.0),
            ))
            .sense(Sense::hover());

        let response = ui.scope_builder(ui_builder, |ui| {
            egui::Frame::new()
                .inner_margin(Margin::symmetric(8, 6))
                .fill(Color32::DARK_GRAY)
                .stroke(Stroke::new(1., Color32::GRAY))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        self.inputs_ui(ui);
                        self.title_ui(ui);
                        self.outputs_ui(ui);
                    })
                });
        });

        // Track the rect of the node.
        self.element_positioning
            .node_rects
            .insert(self.node_id, response.response.rect);

        NodeResponse {
            node_id: self.node_id,
            actions: self.actions,
        }
    }

    fn title_ui(&mut self, ui: &mut egui::Ui) {
        Frame::new()
            .inner_margin(Margin::symmetric(8, 6))
            .show(ui, |ui| {
                let title = Label::new("op")
                    .selectable(false)
                    .sense(Sense::click_and_drag());

                let title_response = ui.add(title);
                title_response.context_menu(|ui| {
                    if ui.button("Delete").clicked() {
                        self.actions.push(NodeAction::Delete);
                    }
                });
                if title_response.double_clicked() {
                    self.actions.push(NodeAction::TitleDoubleClicked);
                }
                if title_response.clicked() {
                    self.actions.push(NodeAction::TitleClicked);
                }
                if title_response.drag_delta().length_sq() > 0.0 {
                    self.actions.push(NodeAction::TitleDragged(
                        title_response.drag_delta().to_pos2(),
                    ));
                }
                if title_response.drag_started() {
                    self.actions.push(NodeAction::TitleDragStarted);
                }
                if title_response.drag_stopped() {
                    self.actions.push(NodeAction::TitleDragStopped);
                }
            });
    }

    fn inputs_ui(&mut self, ui: &mut egui::Ui) {
        let ui_builder = UiBuilder::new()
            .id_salt((self.node_id, "inputs"))
            .layout(Layout::top_down(Align::Min));

        ui.scope_builder(ui_builder, |ui| {
            for (input_index, (input, definition)) in self.node.inputs().enumerate() {
                // If the input does not accept connections, skip it.
                if !definition.flags.accepts_connections() {
                    continue;
                }

                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        let response = pin_ui(ui, &definition.value_type);

                        // Track the rect of the pin.
                        self.element_positioning
                            .node_input_virtual_slot_rects
                            .insert(
                                NodeInputVirtualSlotKey::new(
                                    self.node_id,
                                    input_index,
                                    ConnectionPlacement::End,
                                ),
                                response.rect,
                            );

                        let _ = Label::new(&definition.name)
                            .selectable(false)
                            .sense(Sense::click_and_drag())
                            .ui(ui);
                    });
                });
            }
        });
    }

    fn outputs_ui(&mut self, ui: &mut egui::Ui) {
        let ui_builder = UiBuilder::new()
            .id_salt((self.node_id, "outputs"))
            .layout(Layout::top_down(Align::Min).with_cross_align(Align::Min));

        ui.scope_builder(ui_builder, |ui| {
            for (output_index, output) in self.node.outputs().enumerate() {
                ui.horizontal(|ui| {
                    let _ = Label::new(&output.name)
                        .selectable(false)
                        .sense(Sense::click_and_drag())
                        .ui(ui);

                    let response = pin_ui(ui, &output.value_type);

                    // Track the rect of the pin.
                    self.element_positioning.node_output_rects.insert(
                        NodeOutputKey::new(self.node_id, output_index),
                        response.rect,
                    );
                });
            }
        });
    }
}
