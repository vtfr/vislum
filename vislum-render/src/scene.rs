use crate::resource::{Handle, ResourceStorage};

slotmap::new_key_type! {
    /// A unique identifier for an object in a scene.
    struct ObjectKey;
}

/// A opaque handle to a scene.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneHandle(Handle<Scene>);

/// A opaque handle to a scene object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectHandle(Handle<Object>);

/// A scene and its contents.
pub struct Scene {
    /// All sub-scenes.
    ///
    /// These have been guaranteed not to contain any cycles, as these are detected
    /// by the [`SceneManager`].
    sub_scenes: Vec<SceneHandle>,

    objects: Vec<ObjectHandle>,

    /// All lights in the scene.
    lights: Vec<()>,
}

/// A object in a scene.
pub struct Object {
    
}

pub trait SceneVisitor {
    /// Called when entering a scene.
    fn enter_scene(&mut self, scene: &Scene);

    /// Called when exiting a scene.
    fn exit_scene(&mut self, scene: &Scene);

    /// Called when entering an object.
    fn visit_object(&mut self, object: &Object);
}

/// A manager for scenes.
#[derive(Default)]
pub struct SceneManager {
    objects: ResourceStorage<Object>,
    scenes: ResourceStorage<Scene>,
}

impl SceneManager {
    pub fn visit_scene<'a>(&'a self, visitor: &mut impl SceneVisitor, handle: &SceneHandle) {
        let scene = self.get_scene(handle);

        visitor.enter_scene(scene);

        for sub_scene in scene.sub_scenes.iter() {
            self.visit_scene(visitor, sub_scene);
        }

        for object in scene.objects.iter() {
            visitor.visit_object(self.get_object(object));
        }

        visitor.exit_scene(scene);
    }

    /// Gets an object by its handle.
    ///
    /// The object is guaranteed to be valid for the lifetime of the scene manager.
    pub fn get_object(&self, handle: &ObjectHandle) -> &Object {
        match self.objects.get(&handle.0) {
            Some(object) => object,
            None => unreachable!(),
        }
    }

    /// Gets a scene by its handle.
    ///
    /// The scene is guaranteed to be valid for the lifetime of the scene manager.
    pub fn get_scene(&self, handle: &SceneHandle) -> &Scene {
        match self.scenes.get(&handle.0) {
            Some(scene) => scene,
            None => unreachable!(),
        }
    }
}
