use cgmath::{Vector3, Vector4};

#[derive(Debug, Clone)]
pub enum MaterialBinding {
    Vector3(String, Vector3<f32>),
    Vector4(String, Vector4<f32>),
    Scalar(String, f32),
}

pub struct MaterialDefinition {
    pub base_color: Vector4<f32>,
    pub roughness: f32,
    pub metallic: f32,
    pub bindings: Vec<MaterialBinding>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MaterialBindingArchetype {
    Vector3,
    Vector4,
    Scalar,
}

/// A high-level representation of a material archetype.
///
/// Used to batch objects together when rendering them.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MaterialArchetypeCacheKey {
    pub bindings: Vec<MaterialBindingArchetype>,
}

pub struct MaterialManager {}
