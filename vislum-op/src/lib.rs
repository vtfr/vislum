pub mod value;
pub mod node_type;
pub mod node;
pub mod types;
pub mod compile;
pub mod eval;

pub mod prelude {
    pub use crate::bundle;

    pub use crate::node::{
        Connection, ConnectionPlacement, InputBlueprint, NodeBlueprint, NodeId, OutputId, GraphBlueprint, GraphError,
        NodeError,
    };

    pub use crate::eval::{Node, Eval};

    pub use crate::compile::{CompileInput, GetInputDefinition, GetOutputDefinition, CompileNode, CompilationContext};

    pub use crate::node_type::{
        InputDefinition, NodeType, NodeTypeRegistry, OutputDefinition, NodeTypeId,
    };

    pub use crate::value::{TaggedValue, SValueTypeInfo, Value};

    pub use vislum_op_macros::Node;
}