use vislum_math::Vector3;

use crate::{material::{RenderMaterial, MaterialSystem}, mesh::{RenderMesh, MeshManager}, resource::ResourceId, types::RenderDevice};

#[derive(Default, Clone, Copy)]
pub struct RenderState {}

pub struct Camera {
    pub position: Vector3,
}

/// A renderable object.
pub struct RenderObject {
    /// The mesh to render.
    pub mesh: ResourceId<RenderMesh>,

    /// The material to render with.
    pub material: ResourceId<RenderMaterial>,

    /// The state of the render object.
    pub state: RenderState,
}

pub struct RenderScene {
    /// The objects to render.
    pub objects: Vec<RenderObject>,
}

pub struct Renderer<'a> {
    device: &'a RenderDevice,
    queue: &'a wgpu::Queue,
    scene: &'a RenderScene,
    mesh_manager: &'a MeshManager,
    material_manager: &'a MaterialSystem,
}

struct PreparedRenderObject<'a> {
    mesh: &'a RenderMesh,
    material: &'a RenderMaterial,
}

impl<'a> Renderer<'a> {
    pub fn render(&self) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Default::default(),
            timestamp_writes: Default::default(),
            occlusion_query_set: Default::default(),
        });

        // pass.set_pipeline(pipeline);
        // pass.set_vertex_buffer(slot, buffer_slice);
        // pass.set_bind_group(index, bind_group, offsets);

        // for object in &self.scene.objects {
        //     let Some(resolved_object) = self.resolve_render_object(&object) else {
        //         continue;
        //     };
        // }
    }

    fn resolve_render_object(&self, object: &RenderObject) -> Option<PreparedRenderObject> {
        let mesh = self.mesh_manager.get(object.mesh)?;
        let material = self.material_manager.get(object.material)?;

        Some(PreparedRenderObject { 
            mesh, 
            material,
        })
    }
}
