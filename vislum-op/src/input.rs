use crate::{
    EvalError, EvaluateContext, InputReflect, NodeConnection, NodeId, Operator, OutputReflect,
    Placement, SValueTypeInfo, SlotError, TaggedValue, Value,
};

pub type SlotIndex = usize;
pub type InputIndex = usize;
pub type OutputIndex = usize;

/// Information on an input.
#[derive(Debug)]
pub struct InputInfo {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub default_value: Option<TaggedValue>,
    pub minimum_slots: usize,
    pub maximum_slots: usize,
}

impl Default for InputInfo {
    fn default() -> Self {
        Self {
            name: "Unnamed",
            description: None,
            default_value: None,
            minimum_slots: 0,
            maximum_slots: 1,
        }
    }
}

/// Constructs a new input.
///
/// Used by the [`Operator`] trait to construct runtime
/// information on an input.
pub trait ConstructInput {
    fn type_info() -> SValueTypeInfo;

    /// Constructs a parameter with the provided info.
    fn construct_input(info: InputInfo) -> Self
    where
        Self: Sized;
}

/// A storage for either a value, an animation or a connection with
/// another node.
pub enum Slot<T> {
    Constant(T),
    Connection(NodeConnection),
    Dangling,
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self::Dangling
    }
}

impl<T> Slot<T> {
    /// Invalidates a connection to a given [`NodeId`].
    ///
    /// Returns true if after the connection is invalidated the slot is now dangling.
    pub fn invalidate_connection(&mut self, node_id: NodeId) -> bool {
        if let Slot::Connection(node_connection) = self {
            if node_connection.node_id == node_id {
                *self = Slot::Dangling;
                return true;
            }
        }

        false
    }

    /// Returns true if the slot is dangling.
    pub fn is_dangling(&self) -> bool {
        matches!(self, Self::Dangling)
    }
}

/// A type-erased slot.
#[derive(Debug)]
pub enum ErasedSlot {
    Constant(TaggedValue),
    Connection(NodeConnection),
    Dangling,
}

/// A input that accepts a single slot.
///
/// Parameters can be assigned constant values or animated.
pub struct Single<T> {
    pub info: InputInfo,
    pub value: Slot<T>,
}

impl<T> Single<T>
where
    T: Value,
{
    pub fn evaluate(&self, context: EvaluateContext) -> Result<T, EvalError> {
        match &self.value {
            Slot::Constant(value) => Ok(value.clone()),
            Slot::Connection(node_connection) => {
                let value = context
                    .evaluator
                    .get_node_output(node_connection.node_id, node_connection.output_index)?;

                T::try_from(value).map_err(|_| EvalError)
            }
            Slot::Dangling => Err(EvalError),
        }
    }
}

impl<T> ConstructInput for Single<T>
where
    T: Value,
{
    fn type_info() -> SValueTypeInfo {
        &T::INFO
    }

    fn construct_input(info: InputInfo) -> Self
    where
        Self: Sized,
    {
        Self {
            info,
            value: Slot::Dangling,
        }
    }
}

impl<T> InputReflect for Single<T>
where
    T: Value,
{
    fn type_info(&self) -> SValueTypeInfo {
        &T::INFO
    }

    fn info(&self) -> &InputInfo {
        &self.info
    }

    fn num_slots(&self) -> usize {
        1
    }

    fn set_slot(&mut self, placement: Placement, slot: ErasedSlot) -> Result<(), crate::SlotError> {
        // Only allow updating the first slot.
        // match placement {
        //     Placement::SINGLE => {}
        //     _ => return Err(SlotError),
        // }

        self.value = match slot {
            ErasedSlot::Constant(value) => T::try_from(value)
                .map(Slot::Constant)
                .map_err(|_| SlotError)?,
            ErasedSlot::Dangling => Slot::Dangling,
            ErasedSlot::Connection(node_connection) => Slot::Connection(node_connection),
        };

        Ok(())
    }

    fn get_slot(&self, index: SlotIndex) -> Option<ErasedSlot> {
        match index {
            0 => Some(match self.value {
                Slot::Constant(ref value) => ErasedSlot::Constant(value.clone().into()),
                Slot::Dangling => ErasedSlot::Dangling,
                Slot::Connection(node_connection) => ErasedSlot::Connection(node_connection),
            }),
            _ => None,
        }
    }

    fn invalidate_connection(&mut self, node_id: NodeId) {
        self.value.invalidate_connection(node_id);
    }
}

pub struct Multi<T> {
    pub info: InputInfo,
    pub values: Vec<Slot<T>>,
}

impl<T> Multi<T> {
    pub fn iter<'a>(&'a self, ctx: EvaluateContext<'a>) -> MultiIter<'a, T>
    where
        T: Value,
    {
        MultiIter {
            multi: self,
            index: 0,
            ctx,
        }
    }
}

pub struct MultiIter<'a, T> {
    multi: &'a Multi<T>,
    index: usize,
    ctx: EvaluateContext<'a>,
}

impl<'a, T> Iterator for MultiIter<'a, T>
where
    T: Value,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.multi.values.len() {
            let slot = self.multi.values.get(self.index)?;
            self.index += 1;

            match &slot {
                Slot::Constant(value) => return Some(value.clone()),
                Slot::Connection(node_connection) => {
                    let value = self
                        .ctx
                        .evaluator
                        .get_node_output(node_connection.node_id, node_connection.output_index)
                        .ok()?;

                    return T::try_from(value).ok();
                }
                Slot::Dangling => continue,
            }
        }

        None
    }
}

impl<T> ConstructInput for Multi<T>
where
    T: Value,
{
    fn type_info() -> SValueTypeInfo {
        &T::INFO
    }

    fn construct_input(info: InputInfo) -> Self {
        Self {
            info,
            values: Vec::new(),
        }
    }
}

impl<T> InputReflect for Multi<T>
where
    T: Value,
{
    fn type_info(&self) -> SValueTypeInfo {
        &T::INFO
    }

    fn info(&self) -> &InputInfo {
        &self.info
    }

    fn num_slots(&self) -> usize {
        self.values.len()
    }

    fn set_slot(&mut self, placement: Placement, slot: ErasedSlot) -> Result<(), SlotError> {
        let typed_slot = match slot {
            ErasedSlot::Constant(value) => {
                Slot::Constant(T::try_from(value).map_err(|_| SlotError)?)
            }
            ErasedSlot::Dangling => Slot::Dangling,
            ErasedSlot::Connection(node_connection) => Slot::Connection(node_connection),
        };

        match placement {
            Placement::At(index) => {
                let slot = self.values.get_mut(index).ok_or(SlotError)?;

                *slot = typed_slot;
            }
            Placement::After(index) => {
                self.values.insert(index, typed_slot);
            }
            _ => return Err(SlotError),
        }

        Ok(())
    }

    fn get_slot(&self, index: SlotIndex) -> Option<ErasedSlot> {
        let slot = self.values.get(index)?;

        Some(match slot {
            Slot::Constant(value) => ErasedSlot::Constant(value.clone().into()),
            Slot::Dangling => ErasedSlot::Dangling,
            Slot::Connection(node_connection) => ErasedSlot::Connection(*node_connection),
        })
    }

    fn invalidate_connection(&mut self, node_id: NodeId) {
        self.values.retain_mut(|slot| {
            slot.invalidate_connection(node_id);
            !slot.is_dangling()
        });
    }
}

/// An iterator over the inputs of an operator.
pub struct Inputs<'a> {
    pub operator: &'a dyn Operator,
    pub index: InputIndex,
}

impl Inputs<'_> {
    pub fn new<'a>(operator: &'a dyn Operator) -> Inputs<'a> {
        Inputs { operator, index: 0 }
    }
}

impl<'a> Iterator for Inputs<'a> {
    type Item = &'a dyn InputReflect;

    fn next(&mut self) -> Option<Self::Item> {
        let input = self.operator.get_input(self.index);
        self.index += 1;
        input
    }
}

pub struct InputsMut<'a> {
    operator: &'a mut dyn Operator,
    index: InputIndex,
}

impl<'a> InputsMut<'a> {
    pub fn new(operator: &'a mut dyn Operator) -> Self {
        Self { operator, index: 0 }
    }
}

impl<'a> Iterator for InputsMut<'a> {
    type Item = &'a mut dyn InputReflect;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.operator.num_inputs() {
            return None;
        }

        let input = self.operator.get_input_mut(self.index)?;

        // Transmute lifetimes.
        //
        // SAFETY: We know the operator lives for 'a and each input is distinct
        // The lifetime 'a is the lifetime of the operator reference
        let input = unsafe {
            std::mem::transmute::<&'_ mut dyn InputReflect, &'a mut dyn InputReflect>(input)
        };

        self.index += 1;
        Some(input)
    }
}

/// An iterator over the outputs of an operator.
pub struct Outputs<'a> {
    pub operator: &'a dyn Operator,
    pub index: OutputIndex,
}

impl<'a> Outputs<'a> {
    pub fn new(operator: &'a dyn Operator) -> Self {
        Self { operator, index: 0 }
    }
}

impl<'a> Iterator for Outputs<'a> {
    type Item = &'a dyn OutputReflect;

    fn next(&mut self) -> Option<Self::Item> {
        let output = self.operator.get_output(self.index);
        self.index += 1;
        output
    }
}

/// An iterator over the slots of an input.
pub struct InputSlots<'a> {
    pub input: &'a dyn InputReflect,
    pub index: SlotIndex,
}

impl InputSlots<'_> {
    pub fn new<'a>(input: &'a dyn InputReflect) -> InputSlots<'a> {
        InputSlots { input, index: 0 }
    }
}

impl<'a> Iterator for InputSlots<'a> {
    type Item = ErasedSlot;

    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.input.get_slot(self.index);
        self.index += 1;
        slot
    }
}
