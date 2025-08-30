use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use serde::{Deserialize, Serialize};
use smallbox::SmallBox;
use smallbox::space::S2;
use thiserror::Error;

/// Re-export the `Value` macro from the `vislum-graph-macros` crate.
// pub use vislum_graph_macros::Value;

#[derive(Debug, Error)]
#[error("invalid tagged value conversion")]
pub struct IncompatibleValueTypeError;

/// A identifier for a value type.
///
/// E.g. "vislum.core.types.Float"
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ValueTypeId<'a>(&'a str);

impl Borrow<str> for ValueTypeId<'_> {
    fn borrow(&self) -> &str {
        &*self.0
    }
}

/// A static reference to a value type, for convenience.
pub type SValueTypeInfo = &'static ValueType;

#[rustfmt::skip]
pub trait Value: Clone
    + Debug
    + Default
    + TryFrom<TaggedValue, Error = IncompatibleValueTypeError> 
    + Into<TaggedValue> 
{
    /// Get the type info for the value
    const INFO: ValueType;
}

pub trait DynValue: Debug {
    /// Get a reference to the value as a `dyn Any`
    fn as_any(&self) -> &dyn Any;

    /// Clone the value into a `CustomValue`
    fn clone_custom_value(&self) -> CustomValue;

    /// Get the index of the variant for an enum value.
    ///
    /// Returns `None` if the value is not an enum.
    fn variant_index(&self) -> Option<usize>;

    /// Get the type info for the value
    fn type_info(&self) -> &'static ValueType;
}

#[derive(Debug)]
pub struct CustomValue(SmallBox<dyn DynValue, S2>);

impl Clone for CustomValue {
    fn clone(&self) -> Self {
        self.0.clone_custom_value()
    }
}

impl CustomValue {
    pub fn new<T: DynValue + Sized + 'static>(value: T) -> Self {
        let small_boxed: SmallBox<dyn DynValue, S2> = smallbox::smallbox!(value);
        CustomValue(small_boxed)
    }

    pub fn as_any(&self) -> &dyn Any {
        self.0.as_any()
    }
}

impl std::ops::Deref for CustomValue {
    type Target = dyn DynValue;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

/// The variant info for an enum value.
pub struct Variant {
    pub name: &'static str,
    pub constructor: fn() -> CustomValue,
}

impl Variant {
    #[inline(always)]
    pub fn construct(&self) -> CustomValue {
        (self.constructor)()
    }
}

impl Debug for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Variant").field("name", &self.name).finish()
    }
}

/// The serialization functions for the value.
pub struct ValueTypeSerializationInfo {
    serialize: fn(TaggedValue) -> Result<serde_json::Value, IncompatibleValueTypeError>,
    deserialize: fn(serde_json::Value) -> Result<TaggedValue, IncompatibleValueTypeError>,
}

impl ValueTypeSerializationInfo {
    pub const fn new<T>() -> Self
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Value,
    {
        fn serialize_impl<T>(
            value: TaggedValue,
        ) -> Result<serde_json::Value, IncompatibleValueTypeError>
        where
            T: serde::Serialize + TryFrom<TaggedValue, Error = IncompatibleValueTypeError>,
        {
            let enum_value: T = T::try_from(value)?;
            match serde_json::to_value(&enum_value) {
                Ok(json_value) => Ok(json_value),
                Err(_) => Err(IncompatibleValueTypeError),
            }
        }

        fn deserialize_impl<T>(
            value: serde_json::Value,
        ) -> Result<TaggedValue, IncompatibleValueTypeError>
        where
            T: serde::de::DeserializeOwned + Into<TaggedValue>,
        {
            match serde_json::from_value::<T>(value) {
                Ok(enum_value) => Ok(enum_value.into()),
                Err(_) => Err(IncompatibleValueTypeError),
            }
        }

        Self {
            serialize: serialize_impl::<T>,
            deserialize: deserialize_impl::<T>,
        }
    }

    #[inline(always)]
    pub fn serialize(
        &self,
        value: TaggedValue,
    ) -> Result<serde_json::Value, IncompatibleValueTypeError> {
        (self.serialize)(value)
    }

    #[inline(always)]
    pub fn deserialize(
        &self,
        value: serde_json::Value,
    ) -> Result<TaggedValue, IncompatibleValueTypeError> {
        (self.deserialize)(value)
    }
}

/// The type info for a value.
pub struct ValueType {
    /// The unique identifier for this value type.
    pub id: ValueTypeId<'static>,

    /// Stores the variants for an enum value.
    ///
    /// When this field is `None`, the value is not an enum.
    pub variants: Option<&'static [Variant]>,

    /// Stores the serialization info for the value.
    pub serialization: Option<ValueTypeSerializationInfo>,

    /// Constructs a default value for this type.
    pub default_fn: Option<fn() -> TaggedValue>,
}

impl ValueType {
    pub fn default(&self) -> Option<TaggedValue> {
        self.default_fn.map(|f| f())
    }
}

impl std::fmt::Debug for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValueTypeInfo")
            .field("id", &self.id)
            // .field("variants", &self.variants)
            // .field("serializable", &self.serialization.is_some())
            .finish()
    }
}

macro_rules! impl_value {
    ($($name:ident($type:ty)),* $(,)?) => {
        #[derive(Debug, Clone)]
        pub enum TaggedValue {
            $($name($type),)*
            // /// Stores a custom value. This is used for values
            // /// that don't fit into known primitive types, such
            // /// as enums and user-defined types.
            CustomValue(CustomValue),
        }

        impl TaggedValue {
            pub fn type_info(&self) -> SValueTypeInfo {
                match self {
                    $(
                        Self::$name(_) => &<$type as Value>::INFO,
                    )*
                    Self::CustomValue(custom_value) => custom_value.type_info(),
                }
            }

            #[inline(always)]
            pub fn type_id(&self) -> &ValueTypeId {
                &self.type_info().id
            }
        }

        $(
            impl Value for $type {
                const INFO: ValueType = ValueType {
                    id: ValueTypeId(concat!("vislum.core.types.", stringify!($name))),
                    variants: None,
                    serialization: Some(ValueTypeSerializationInfo::new::<$type>()),
                    default_fn: Some(|| <$type as Default>::default().into()),
                };
            }

            impl TryFrom<TaggedValue> for $type {
                type Error = IncompatibleValueTypeError;

                fn try_from(value: TaggedValue) -> Result<Self, Self::Error> {
                    match value {
                        TaggedValue::$name(value) => Ok(value),
                        _ => Err(IncompatibleValueTypeError),
                    }
                }
            }

            impl Into<TaggedValue> for $type {
                #[inline(always)]
                fn into(self) -> TaggedValue {
                    TaggedValue::$name(self)
                }
            }
        )*
    };
}

impl_value! {
    Float(f32),
    Int(i32),
    Bool(bool),
}

pub struct ValueTypeRegistry {
    pub values: HashMap<ValueTypeId<'static>, SValueTypeInfo>,
}

impl Default for ValueTypeRegistry {
    fn default() -> Self {
        let mut registry = Self {
            values: HashMap::new(),
        };

        // Register the primitive types
        registry.register::<f32>();
        registry.register::<i32>();
        registry.register::<bool>();
        registry
    }
}

impl ValueTypeRegistry {
    #[inline(always)]
    pub fn register<T: Value>(&mut self) {
        self.add(&T::INFO);
    }

    /// Adds a value type to the registry.
    pub fn add(&mut self, info: SValueTypeInfo) {
        self.values.insert(info.id, info);
    }

    /// Gets a value type from the registry.
    pub fn get<'a>(&'a self, key: &ValueTypeId<'a>) -> Option<SValueTypeInfo> {
        self.values.get(key).copied()
    }
}
