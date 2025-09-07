use vislum_math::Matrix4;
use vislum_system::System;

use crate::{
    mesh::RenderMesh,
    resource::{Handle, IntoResourceId, ResourceId, ResourceStorage},
};

pub struct Scene {
    pub objects: Vec<SceneObject>,
    pub sub_scenes: Vec<ResourceId<Scene>>,
}

impl Scene {
    /// Apply a single command to the scene.
    pub fn apply_command(&mut self, command: SceneCommand) {
        match command {
            SceneCommand::ClearObjects => {
                self.objects.clear();
            }
            SceneCommand::ClearSubScenes => {
                self.sub_scenes.clear();
            }
            SceneCommand::AddObject(object) => {
                self.objects.push(object);
            }
            SceneCommand::AddSubScene(sub_scene) => {
                self.sub_scenes.push(sub_scene);
            }
        }
    }

    /// Apply a list of commands to the scene.
    pub fn apply_commands(&mut self, commands: impl IntoIterator<Item = SceneCommand>) {
        for command in commands {
            self.apply_command(command);
        }
    }
}

pub enum SceneCommand {
    /// Clear all objects from the scene.
    ClearObjects,

    /// Clear all sub-scenes from the scene.
    ClearSubScenes,

    /// Add an object to the scene.
    AddObject(SceneObject),

    /// Add a sub-scene to the scene.
    AddSubScene(ResourceId<Scene>),
}

#[derive(Debug, Clone)]
pub struct SceneObject {
    pub mesh: Handle<RenderMesh>,
}

pub struct SceneManager {
    pub scene: ResourceStorage<Scene>,
}

#[derive(Debug, thiserror::Error)]
pub enum ApplyCommandError {
    #[error("Scene not found: {0:?}")]
    SceneNotFound(ResourceId<Scene>),
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            scene: ResourceStorage::new(),
        }
    }

    /// Create a new scene.
    ///
    /// Retuns a owned handle to the scene.
    pub fn create(&mut self) -> Handle<Scene> {
        let scene = Scene {
            objects: Vec::new(),
            sub_scenes: Vec::new(),
        };

        self.scene.insert(scene)
    }

    /// Create a new scene and apply a list of commands to it.
    ///
    /// Returns a owned handle to the scene.
    pub fn create_with_commands(&mut self, initial_commands: impl IntoIterator<Item = SceneCommand>) -> Handle<Scene> {
        let mut scene = Scene {
            objects: Vec::new(),
            sub_scenes: Vec::new(),
        };

        scene.apply_commands(initial_commands);

        self.scene.insert(scene)
    }

    /// Apply a list of commands to a scene.
    pub fn apply(
        &mut self,
        scene_id: impl IntoResourceId<Scene>,
        commands: impl IntoIterator<Item = SceneCommand>,
    ) -> Result<(), ApplyCommandError> {
        let scene_id = scene_id.into_resource_id();
        let scene = self
            .scene
            .get_mut(scene_id)
            .ok_or(ApplyCommandError::SceneNotFound(scene_id))?;

        for command in commands {
            match command {
                SceneCommand::ClearObjects => {
                    scene.objects.clear();
                }
                SceneCommand::AddObject(scene_object) => {
                    scene.objects.push(scene_object);
                }
                SceneCommand::AddSubScene(resource_id) => {
                    scene.sub_scenes.push(resource_id);
                }
                SceneCommand::ClearSubScenes => {
                    scene.sub_scenes.clear();
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SceneVisitorError {
    #[error("Scene not found: {0:?}")]
    SceneNotFound(ResourceId<Scene>),
}

/// A trait for visiting a scene.
///
/// This is used to traverse the scene graph.
pub trait SceneVisitor {
    fn visit(&mut self, id: ResourceId<Scene>, scene: &Scene) -> Result<(), SceneVisitorError>;

    /// Called when the visitor is done visiting a scene.
    ///
    /// Some implementations may want to do some cleanup when the visitor is done.
    fn done_visiting(
        &mut self,
        id: ResourceId<Scene>,
        scene: &Scene,
    ) -> Result<(), SceneVisitorError> {
        let _ = (id, scene);
        Ok(())
    }
}

pub struct SceneVisitorContext<'a> {
    scene_system: &'a SceneManager,
}

impl<'a> SceneVisitorContext<'a> {
    pub fn new(scene_system: &'a SceneManager) -> Self {
        Self { scene_system }
    }

    pub fn visit(
        &mut self,
        visitor: &mut dyn SceneVisitor,
        scene_id: ResourceId<Scene>,
    ) -> Result<(), SceneVisitorError> {
        let scene = self
            .scene_system
            .scene
            .get(scene_id)
            .ok_or(SceneVisitorError::SceneNotFound(scene_id))?;

        visitor.visit(scene_id, scene)?;

        for sub_scene_id in scene.sub_scenes.iter().copied() {
            self.visit(visitor, sub_scene_id)?;
        }

        visitor.done_visiting(scene_id, scene)?;

        Ok(())
    }
}

pub struct FlattenedScene {
    pub objects: Vec<SceneObject>,
}

impl FlattenedScene {
    /// Create a new flattened scene.
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }
}

impl SceneVisitor for FlattenedScene {
    fn visit(&mut self, _id: ResourceId<Scene>, scene: &Scene) -> Result<(), SceneVisitorError> {
        self.objects.extend(scene.objects.iter().cloned());
        Ok(())
    }
}
