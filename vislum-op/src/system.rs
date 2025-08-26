use vislum_system::System;

use crate::{Graph, OperatorTypeRegistry, RegisterOperator, ValueTypeRegistry};

#[derive(Default, System)]
pub struct OperatorSystem {
    operator_type_registry: OperatorTypeRegistry,
    value_type_registry: ValueTypeRegistry,
    graph: Graph,
}

impl OperatorSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_graph(&self) -> &Graph {
        &self.graph
    }

    pub fn get_graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    pub fn register_operator_type<T: RegisterOperator>(&mut self) {
        self.operator_type_registry.register::<T>();

        for value_type in T::collect_value_types() {
            self.value_type_registry.add(value_type);
        }
    }
}
