extern crate self as vislum_op;

pub mod compile;
pub mod eval;
pub mod introspect;
pub mod node;
pub mod node_type;
pub mod system;
pub mod types;
pub mod value;

pub mod prelude {
    pub use crate::bundle;

    pub use crate::node::{
        Connection, ConnectionPlacement, GraphBlueprint, GraphError, InputBlueprint, NodeBlueprint,
        NodeError, NodeId, OutputId,
    };

    pub use crate::eval::{Eval, Node};

    pub use crate::compile::{
        CompilationContext, CompileInput, CompileNode, GetInputDefinition, GetOutputDefinition,
    };

    pub use crate::node_type::{
        InputDefinition, NodeType, NodeTypeId, NodeTypeRegistry, OutputDefinition,
    };

    pub use crate::value::{SValueTypeInfo, TaggedValue, Value};

    pub use vislum_op_macros::Node;
}
