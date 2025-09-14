use std::any::Any;

use thiserror::Error;

use crate::{compile::{InputDefinition, OutputDefinition}, node::{InputId, OutputId}, value::TaggedValue};

/// An erased slot.
pub enum ErasedSlot {
    Constant(TaggedValue),
}

pub enum Placement {
    /// The slot is placed at the end.
    End,
}


#[derive(Debug, Error)]
pub enum SetSlotError {
    #[error("Output '{0}' not found")]
    InvalidOutput(OutputId),
}

pub trait IntrospectInput {
    /// Retrieve the number of slots.
    /// 
    /// For a [`Single`] input, this is always 1.
    /// For a [`Multi`] input, this is the number of current slots.
    fn slots_len(&self) -> usize;

    /// Get the slot at the given placement.
    fn get_slot(&self, placement: Placement) -> Option<&ErasedSlot>;

    /// Sets the slot at the given placement.
    /// 
    /// ## Safety
    /// Some validations are performed on the slot, but the caller is responsible for ensuring that
    /// the change is compatible with the input definition.
    fn set_slot(&mut self, placement: Placement, slot: ErasedSlot) -> Result<(), SetSlotError>;
}

pub struct OutputValue {
    value: TaggedValue,
}

impl OutputValue {
    #[inline]
    pub fn value(&self) -> &TaggedValue {
        &self.value
    }
}

pub trait Introspect {
    /// Retrieve the number of inputs.
    fn inputs_len(&self) -> usize;

    /// Retrieve the input of a node.
    /// 
    /// Returns `None` if the input is not found.
    fn get_input(&self, input_id: InputId) -> Option<&dyn IntrospectInput>;

    /// Get the input as mutable.
    fn get_input_mut(&mut self, input_id: InputId) -> Option<&mut dyn IntrospectInput>;

    /// Retrieve the number of outputs.
    fn outputs_len(&self) -> usize;

    /// Retrieve the output of a node.
    /// 
    /// Returns `None` if the output is not found.
    fn get_output_value(&self, output_id: OutputId) -> Option<OutputValue>;

    /// Retrieve the input definition for an input.
    /// 
    /// Returns `None` if the input is not found.
    fn get_input_definition(&self, input_id: InputId) -> Option<&InputDefinition>;
    
    /// Retrieve the output definition for an output.
    /// 
    /// Returns `None` if the output is not found.
    fn get_output_definition(&self, output_id: OutputId) -> Option<&OutputDefinition>;

    /// Get the node as any.
    fn as_any(&self) -> &dyn Any;

    /// Get the node as mutable any.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// An untyped input.
pub struct DynamicInput {
    slots: Vec<ErasedSlot>,
    definition: InputDefinition,
}

impl IntrospectInput for DynamicInput {
    fn slots_len(&self) -> usize {
        self.slots.len()
    }

    fn get_slot(&self, placement: Placement) -> Option<&ErasedSlot> {
        match placement {
            Placement::End => self.slots.last(),
        }
    }

    fn set_slot(&mut self, placement: Placement, slot: ErasedSlot) -> Result<(), SetSlotError> {
        match placement {
            Placement::End => {
                self.slots.push(slot);
            }
        }
        
        Ok(())
    }
}
