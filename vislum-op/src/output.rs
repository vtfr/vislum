use crate::{OutputReflect, SValueTypeInfo, TaggedValue, Value};

#[derive(Debug)]
pub struct OutputInfo {
    pub name: &'static str,
    pub description: Option<&'static str>,
}

pub trait ConstructOutput {
    /// Constructs an output with the provided info.
    fn construct_output(info: OutputInfo) -> Self
    where
        Self: Sized;
}

/// An output that can be set to a value.
pub struct Output<T> {
    pub info: OutputInfo,
    pub value: Option<T>,
}

impl<T> Output<T>
where
    T: Value,
{
    pub fn set(&mut self, value: T) {
        self.value = Some(value);
    }
}

impl<T> ConstructOutput for Output<T>
where
    T: Value,
{
    fn construct_output(info: OutputInfo) -> Self {
        Self { info, value: None }
    }
}

impl<T> OutputReflect for Output<T>
where
    T: Value,
{
    fn type_info(&self) -> SValueTypeInfo {
        &T::INFO
    }

    fn info(&self) -> &OutputInfo {
        &self.info
    }

    fn get_value(&self) -> Option<TaggedValue> {
        match self.value {
            Some(ref value) => Some(value.clone().into()),
            None => None,
        }
    }
}
