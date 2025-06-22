use std::collections::HashMap;

use eframe::egui::{
    self, Align, Color32, Label, Layout, Margin, Rect, RichText, Sense, Stroke, UiBuilder,
    WidgetText, vec2,
};
use vislum_op::{InputIndex, Node, NodeId, OutputIndex, Placement};

use crate::graph::GraphElementPositioning;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeOutputKey {
    node_id: NodeId,
    index: OutputIndex,
}

impl NodeOutputKey {
    pub const fn new(node_id: NodeId, index: OutputIndex) -> Self {
        Self { node_id, index }
    }
}

/// Uniquely identifies a node input virtual or real slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeInputVirtualSlotKey {
    node_id: NodeId,
    index: InputIndex,
    placement: Placement,
}

impl NodeInputVirtualSlotKey {
    pub const fn new(node_id: NodeId, index: InputIndex, placement: Placement) -> Self {
        Self {
            node_id,
            index,
            placement,
        }
    }

    /// Returns true if the slot is virtual.
    #[inline]
    pub fn is_virtual(&self) -> bool {
        matches!(self.placement, Placement::After(_))
    }

    /// Returns true if the slot is real.
    #[inline]
    pub fn is_real(&self) -> bool {
        !self.is_virtual()
    }

    #[inline]
    pub fn index(&self) -> InputIndex {
        self.index
    }

    #[inline]
    pub fn placement(&self) -> Placement {
        self.placement
    }
}

pub struct NodeView<'a> {
    pub node_id: NodeId,
    pub node: &'a Node,
    pub element_positioning: &'a mut GraphElementPositioning,
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
}

pub struct NodeResponse {
    pub actions: Vec<NodeAction>,
}

impl<'a> NodeView<'a> {
    /// Draws the node's inner UI.
    pub fn inner_ui(&mut self, ui: &mut egui::Ui) -> NodeResponse {
        let mut actions = Vec::new();

        let ui_builder = UiBuilder::new()
            .id_salt(self.node_id)
            .layout(Layout::left_to_right(Align::Min).with_cross_align(Align::Center))
            .sense(Sense::hover());

        ui.scope_builder(ui_builder, |ui| {
            egui::Frame::new()
                .inner_margin(Margin::symmetric(8, 6))
                .fill(Color32::DARK_GRAY)
                .stroke(Stroke::new(1., Color32::GRAY))
                .show(ui, |ui| {
                    ui.set_max_size(vec2(200.0, 100.0));

                    self.inputs_ui(ui, &mut actions);
                    self.title_ui(ui, &mut actions);
                    self.outputs_ui(ui, &mut actions);
                });
        });

        NodeResponse { actions }
    }

    fn title_ui(&mut self, ui: &mut egui::Ui, actions: &mut Vec<NodeAction>) {
        let title = Label::new("op").sense(Sense::click_and_drag());

        let title_response = ui.add(title);
        if title_response.double_clicked() {
            actions.push(NodeAction::TitleDoubleClicked);
        }
        if title_response.clicked() {
            actions.push(NodeAction::TitleClicked);
        }
        if title_response.drag_started() {
            actions.push(NodeAction::TitleDragStarted);
        }
        if title_response.drag_stopped() {
            actions.push(NodeAction::TitleDragStopped);
        }
    }

    fn inputs_ui(&mut self, ui: &mut egui::Ui, actions: &mut Vec<NodeAction>) {
        let ui_builder = UiBuilder::new()
            .id_salt((self.node_id, "inputs"))
            .layout(Layout::top_down(Align::Min).with_cross_align(Align::Min));

        ui.scope_builder(ui_builder, |ui| {
            for input in self.node.inputs() {
                let info = input.info();

                ui.horizontal(|ui| {
                    ui.label(info.name).on_hover_ui(|ui| {
                        ui.label(format!(
                            "{} slots of type {:?}",
                            input.num_slots(),
                            input.type_info().id
                        ));
                    });
                });
            }
        });
    }

    fn outputs_ui(&mut self, ui: &mut egui::Ui, actions: &mut Vec<NodeAction>) {
        let ui_builder = UiBuilder::new()
            .id_salt((self.node_id, "outputs"))
            .layout(Layout::top_down(Align::Min).with_cross_align(Align::Min));

        ui.scope_builder(ui_builder, |ui| {
            for output in self.node.outputs() {
                let info = output.info();

                ui.horizontal(|ui| {
                    ui.label(info.name);
                });
            }
        });
    }
}
