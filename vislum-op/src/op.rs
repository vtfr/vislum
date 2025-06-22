use std::collections::HashMap;

use atomicow::CowArc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{EvalError, EvaluateContext, Inputs, Outputs, Reflect, SValueTypeInfo};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperatorTypeId<'a>(CowArc<'a, str>);

impl Serialize for OperatorTypeId<'static> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for OperatorTypeId<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(OperatorTypeId::new_owned(&s))
    }
}

impl OperatorTypeId<'_> {
    /// Creates a new operator type id.
    pub const fn new(name: &'static str) -> Self {
        Self(CowArc::Static(name))
    }

    /// Creates a new operator type id from a string.
    pub fn new_owned(name: &str) -> Self {
        Self(CowArc::new_owned_from_arc(name))
    }

    /// Returns a string slice of the operator type id.
    pub fn as_str(&self) -> &str {
        &*self.0
    }

    /// Consumes the operator type id and returns a static operator type id.
    pub fn into_owned(&self) -> OperatorTypeId<'static> {
        OperatorTypeId(self.0.clone().into_owned())
    }
}

impl std::fmt::Display for OperatorTypeId<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An operator.
pub trait Operator: Reflect {
    /// Evaluates the operator.
    fn evaluate(&mut self, context: EvaluateContext) -> Result<(), EvalError>;
}

impl dyn Operator {
    pub fn inputs<'a>(&'a self) -> Inputs<'a> {
        Inputs::new(self)
    }

    pub fn outputs(&self) -> Outputs {
        Outputs::new(self)
    }
}

/// A trait for constructing an operator with
/// the default constructor.
///
/// Used for convenience, when the Operator type is known
/// at compile time.
pub trait ConstructOperator {
    /// Constructs an operator.
    fn construct_operator() -> Box<dyn Operator>;
}

/// A trait for registering operators.
pub trait RegisterOperator {
    /// Registers an operator.
    fn register_operator(registry: &mut OperatorTypeRegistry);

    /// Collects the value types used by this operator, so they
    /// can be serialized and deserialized.
    #[allow(unused_variables)]
    fn collect_value_types() -> &'static [SValueTypeInfo] {
        &[]
    }
}

/// Information about an operator type.
pub struct OperatorTypeInfo {
    pub id: OperatorTypeId<'static>,
    pub construct: fn() -> Box<dyn Operator>,
}

/// A registry of operator types.
///
/// This registry is used to register operator types, so they
/// can be serialized and deserialized, as well as operated
/// through the editor.
#[derive(Default)]
pub struct OperatorTypeRegistry {
    registry: HashMap<OperatorTypeId<'static>, OperatorTypeInfo>,
}

impl OperatorTypeRegistry {
    pub fn add(&mut self, info: OperatorTypeInfo) {
        self.registry.insert(info.id.into_owned(), info);
    }

    pub fn register<T: RegisterOperator>(&mut self) {
        T::register_operator(self);
    }

    pub fn iter(&self) -> impl Iterator<Item = &OperatorTypeInfo> {
        self.registry.values()
    }
}
