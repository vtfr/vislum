use eframe::egui;
use vislum_op::ConstructOperator;
use vislum_runtime::Runtime;

use crate::graph::{GraphView, GraphViewContext};

pub struct Editor {
    graph_view: GraphView,
    runtime: Runtime,
}

impl Editor {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let mut runtime = Runtime::new();
        runtime
            .get_operator_system_mut()
            .register_operator_type::<vislum_op::Add>();
        runtime
            .get_operator_system_mut()
            .get_graph_mut()
            .add_node(<vislum_op::Add as ConstructOperator>::construct_operator());

        Self {
            runtime,
            graph_view: Default::default(),
        }
    }

    pub fn run() {
        let native_options = eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        };

        eframe::run_native(
            "Vislum Editor",
            native_options,
            Box::new(|cc| {
                let app = Self::new(cc);

                Ok(Box::new(app))
            }),
        )
        .unwrap();
    }
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.graph_view.ui(
                ui,
                GraphViewContext {
                    op_system: self.runtime.get_operator_system(),
                },
            );
        });
    }
}
