use thiserror::Error;

use crate::{
    ErasedSlot, InputIndex, InputInfo, InputSlots, NodeId, OperatorTypeId, OutputIndex, OutputInfo,
    SValueTypeInfo, SlotIndex, TaggedValue,
};

/// Re-export the `Reflect` macro.
pub use vislum_op_macros::Reflect;

#[derive(Debug, Error)]
pub enum ReflectError {
    #[error("The slot is not compatible with the operation.")]
    IncompatibleSlot,

    #[error("The slot index is out of bounds: {0}")]
    SlotOutOfBounds(SlotIndex),

    #[error("The input is out of bounds: {0}")]
    InputOutOfBounds(InputIndex),

    #[error("The output is out of bounds: {0}")]
    OutputOutOfBounds(OutputIndex),
}

/// Reflection on an operator.
pub trait Reflect {
    /// The type id of the operator.
    fn type_id(&self) -> OperatorTypeId;

    /// The number of inputs.
    fn num_inputs(&self) -> usize;

    /// Gets a reference to the input at the given index.
    fn get_input(&self, index: InputIndex) -> Option<&dyn InputReflect>;

    /// Gets a mutable reference to the input at the given index.
    fn get_input_mut(&mut self, index: InputIndex) -> Option<&mut dyn InputReflect>;

    /// The number of outputs.
    fn num_outputs(&self) -> usize;

    /// Gets a reference to the output at the given index.
    fn get_output(&self, index: OutputIndex) -> Option<&dyn OutputReflect>;
}

/// A placement for an [`Slot`] in an input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Placement {
    /// Place the slot at the given slot index.
    At(SlotIndex),
    /// Place the slot after the given slot index.
    After(SlotIndex),
    /// Place the slot at the end of the input.
    End,
}

impl Placement {
    /// The placement for a single slot.
    pub const SINGLE: Self = Self::At(0);
}

impl std::fmt::Display for Placement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Placement::At(index) => write!(f, "At({})", index),
            Placement::After(index) => write!(f, "After({})", index),
            Placement::End => write!(f, "End"),
        }
    }
}

#[derive(Debug, Error)]
#[error("The slot is not compatible with the operation.")]
pub struct SlotError;

pub trait InputReflect {
    /// Get the type info for the input.
    fn type_info(&self) -> SValueTypeInfo;

    /// Get the input info.
    fn info(&self) -> &InputInfo;

    /// Get the number of slots.
    fn num_slots(&self) -> usize;

    /// Set the slot at the given index.
    fn set_slot(&mut self, placement: Placement, slot: ErasedSlot) -> Result<(), SlotError>;

    /// Get the slot at the given index.
    fn get_slot(&self, index: SlotIndex) -> Option<ErasedSlot>;

    /// Clear all connections to a given [`NodeId`]
    fn invalidate_connection(&mut self, node_id: NodeId);
}

impl dyn InputReflect {
    /// Get the input info.
    pub fn slots<'a>(&'a self) -> InputSlots<'a> {
        InputSlots::new(self)
    }
}

pub trait OutputReflect {
    /// Get the type info for the output.
    fn type_info(&self) -> SValueTypeInfo;

    /// Get the output info.
    fn info(&self) -> &OutputInfo;

    /// Get the value of the output.
    ///
    /// Returns `None` if the output has not been set.
    fn get_value(&self) -> Option<TaggedValue>;
}
