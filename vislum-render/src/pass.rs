use std::{borrow::Cow, collections::HashMap};

use smallvec::SmallVec;

use crate::{mesh::Mesh, resource::ResourceId};

#[derive(Default)]
pub struct PassAttachment {
    meshes: SmallVec<[ResourceId<Mesh>; 16]>,
}

pub struct Pass {
    /// The name of the pass.
    name: Cow<'static, str>,
    
    /// The type of pass.
    pass_type: PassType,
    
    /// The attachments that are written by this pass.
    write: PassAttachment,

    /// The attachments that are read by this pass.
    read: PassAttachment,
    
    /// A function that evaluates the pass.
    evaluate: Box<dyn Fn(&Pass) + 'static>,

    /// Whether this pass is the final pass in the pipeline.
    /// 
    /// Final passes are used to present the final image to the screen, 
    /// and are not reordered.
    /// 
    /// Final passes are expected to have a single output attachment,
    /// which will be blit to the screen.
    final_pass: bool,
}

pub enum PassType {
    Render,
    Compute,
}

pub struct PassBuilder {
    name: Cow<'static, str>,
    pass_type: PassType,
    write: PassAttachment,
    read: PassAttachment,
    evaluate: Option<Box<dyn Fn(&Pass) + 'static>>,
}

impl PassBuilder {
    pub fn render(name: impl Into<Cow<'static, str>>) -> Self {
        Self { 
            name: name.into(),
            pass_type: PassType::Render,
            write: PassAttachment::default(), 
            read: PassAttachment::default(), 
            evaluate: None 
        }
    }

    pub fn build(self) -> Pass {
        Pass {
            name: self.name,
            pass_type: self.pass_type,
            write: self.write,
            read: self.read,
            evaluate: self.evaluate.unwrap_or(Box::new(|_| {})),
            final_pass: false,
        }
    }
}

/// A pass that clears the screen.
pub fn clear_screen_pass() -> Pass {
    PassBuilder::render("Clear Screen Pass")
        .build()
}

/// Renders a list of passes.
pub fn render_passes(_passes: Vec<Pass>) {
    todo!()
}