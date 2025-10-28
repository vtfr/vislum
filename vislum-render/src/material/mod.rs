use smallvec::SmallVec;

/// The definition of a material property.
pub enum MaterialPropertiyDefinition {
    /// A float.
    Float(String),
}

/// The definition of a material.
pub struct MaterialDefinition {
    /// The properties of the material.
    /// 
    /// These are guaranteed to be in the order they were added.
    properties: SmallVec<[MaterialPropertiyDefinition; 16]>,
}

impl MaterialDefinition {
}

/// A manager for materials.
/// 
/// Stores the definitions of the materials and their instances.
pub struct MaterialManager {

}