use std::ops::Deref;
use std::{borrow::Borrow, rc::Rc};
use std::hash::Hash;
use std::collections::HashMap;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::value::SValueTypeInfo;
use crate::compile::CompileNodeFn;
use crate::node::{InputBlueprint, NodeBlueprint};

/// A unique identifier for a node type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeTypeId(String);

impl NodeTypeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Deref for NodeTypeId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Borrow<str> for NodeTypeId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// A node type.
pub struct NodeType {
    pub id: NodeTypeId,
    pub inputs: Vec<InputDefinition>,
    pub outputs: Vec<OutputDefinition>,
    pub evaluation: EvaluationStrategy,
}

impl NodeType {
    /// Creates a new node schema.
    pub fn new(
        id: NodeTypeId, 
        inputs: Vec<InputDefinition>, 
        outputs: Vec<OutputDefinition>,
        compile_node_fn: CompileNodeFn,
    ) -> Self {
        Self { id, inputs, outputs, evaluation: EvaluationStrategy::Compile(compile_node_fn) }
    }

    /// Instantiates a new [`NodeBlueprint`] from this node type.
    pub fn instantiate(self: &Rc<Self>) -> NodeBlueprint {
        let inputs = self.inputs.iter()
            .map(|input| input.instantiate())
            .collect();

        NodeBlueprint {
            node_type: self.clone(),
            inputs,
            position: Default::default(),
        }
    }

    /// Gets an input definition by name.
    pub fn get_input(&self, index: usize) -> Option<&InputDefinition> {
        self.inputs.get(index)
    }

    /// Gets an output definition by name.
    pub fn get_output(&self, index: usize) -> Option<&OutputDefinition> {
        self.outputs.get(index)
    }

    pub fn get_input_by_name(&self, name: &str) -> Option<(usize, &InputDefinition)> {
        self.inputs.iter()
            .enumerate()
            .find(|(_, input)| input.name == name)
    }

    pub fn get_output_by_name(&self, name: &str) -> Option<(usize, &OutputDefinition)> {
        self.outputs.iter()
            .enumerate()
            .find(|(_, output)| output.name == name)
    }
}

/// Defines how a node type is compiled.
pub enum EvaluationStrategy {
    /// The node should be compiled using a native function.
    Compile(CompileNodeFn),
}

/// A schema for an input to a node.
pub struct InputDefinition {
    pub name: String,
    pub value_type: SValueTypeInfo,
    pub cardinality: InputCardinality,
    pub flags: AssignmentTypes,
}

impl InputDefinition {
    /// Creates a new input schema.
    pub fn new(
        name: impl Into<String>, 
        value_type: SValueTypeInfo,
        cardinality: InputCardinality,
        flags: AssignmentTypes,
    ) -> Self {
        Self { 
            name: name.into(), 
            value_type,
            cardinality,
            flags,
        }
    }

    /// Sets the capabilities of the input.
    #[inline]
    pub fn with_flags(mut self, capabilities: AssignmentTypes) -> Self {
        self.flags |= capabilities;
        self
    }

    /// Gets the capabilities of the input.
    #[inline]
    pub fn flags(&self) -> AssignmentTypes {
        self.flags
    }

    /// Instantiates an input.
    /// 
    /// If the type accepts a default value, it will be used.
    /// Otherwise, the input will be left unset.
    pub fn instantiate(&self) -> InputBlueprint {
        // If the input accepts constants, use the default value if it exists.
        if self.flags.contains(AssignmentTypes::CONSTANT) {
            if let Some(default) = self.value_type.default() {
                return InputBlueprint::Constant(default)
            } 
        }

        // Fallback to an unset input.
        InputBlueprint::Unset
    }
}

/// The cardinality of an input.
pub enum InputCardinality {
    /// The input accepts a single connection.
    Single,

    /// The input accepts multiple connections.
    Multiple,
}

bitflags! {
    #[derive(Debug, Default, Copy, Clone)]
    pub struct AssignmentTypes: u8 {
        /// The input accepts a constant value.
        const CONSTANT = 1 << 0;

        /// The input accepts an animation.
        const ANIMATION = 1 << 1;

        /// The input accepts connections.
        const CONNECTION = 1 << 2;

        /// All assignment types.
        const ALL = Self::CONSTANT.bits() | Self::ANIMATION.bits() | Self::CONNECTION.bits();
    }
}

impl AssignmentTypes {
    #[inline]
    pub fn accepts_connections(&self) -> bool {
        self.intersects(AssignmentTypes::CONNECTION)
    }

    #[inline]
    pub fn accepts_constants(&self) -> bool {
        self.contains(AssignmentTypes::CONSTANT)
    }

    #[inline]
    pub fn accepts_animations(&self) -> bool {
        self.contains(AssignmentTypes::ANIMATION)
    }
}

pub struct OutputDefinition {
    pub name: String,
    pub value_type: SValueTypeInfo,
}

impl OutputDefinition {
    /// Creates a new output schema.
    pub fn new(name: impl Into<String>, value_type: SValueTypeInfo) -> Self {
        Self { name: name.into(), value_type }
    }
}

pub trait RegisterNodeType {
    fn register_node_type(registry: &mut NodeTypeRegistry);
}

/// Bundles a group of node types registerers together.
#[macro_export]
macro_rules! bundle {
    {
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $($registerer:path),* $(,)?
        }
    } => {
        $(#[$attr])*
        $vis struct $name;

        #[automatically_derived]
        impl vislum_op::node_type::RegisterNodeType for $name {
            #[inline(always)]
            fn register_node_type(registry: &mut vislum_op::node_type::NodeTypeRegistry) {
                $(
                    <$registerer as vislum_op::node_type::RegisterNodeType>::register_node_type(registry);
                )*
            }
        }
    }
}

macro_rules! impl_bundle_tuple {
    ($($ty:ident),* $(,)?) => {
        #[automatically_derived]
        impl<$($ty),*> RegisterNodeType for ($($ty),*,)
        where 
            $(
                $ty: RegisterNodeType,
            )*
        {
            #[inline(always)]
            fn register_node_type(registry: &mut vislum_op::node_type::NodeTypeRegistry) {
                $(
                    <$ty as RegisterNodeType>::register_node_type(registry);
                )*
            }
        }
    };
}

impl_bundle_tuple!(T1);
impl_bundle_tuple!(T1, T2);
impl_bundle_tuple!(T1, T2, T3);
impl_bundle_tuple!(T1, T2, T3, T4);
impl_bundle_tuple!(T1, T2, T3, T4, T5);
impl_bundle_tuple!(T1, T2, T3, T4, T5, T6);
impl_bundle_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_bundle_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
// this is just a convenience for testing, so 8 is plenty.


#[derive(Default)]
pub struct NodeTypeRegistry {
    node_types: HashMap<NodeTypeId, Rc<NodeType>>,
}

impl NodeTypeRegistry {
    /// Creates a new node type registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node type to the registry.
    pub fn add(&mut self, node_type: NodeType) {
        self.node_types.insert(node_type.id.clone(), Rc::new(node_type));
    }

    #[inline(always)]
    pub fn register<T: RegisterNodeType>(&mut self) {
        T::register_node_type(self);
    }

    /// Gets a node type from the registry.
    pub fn get<T>(&self, id: &T) -> Option<Rc<NodeType>> 
    where 
        NodeTypeId: Borrow<T>,
        T: Hash + Eq + ?Sized,
    {
        self.node_types.get(id).cloned()
    }

    /// Iterates over the node types in the registry.
    pub fn iter(&self) -> impl Iterator<Item = &Rc<NodeType>> {
        self.node_types.values()
    }
}