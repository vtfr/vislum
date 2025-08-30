use std::error::Error;

use vislum_op::{compile::CompilationContext, eval::{Eval, EvalContext, EvalError, Multiple, Output, Single}, prelude::*};

#[derive(Node)]
struct SumAll {
    #[input]
    b: Multiple<f32>,
    #[output]
    c: Output<f32>,
}

impl Eval for SumAll {
    fn eval(&mut self, ctx: &EvalContext) -> Result<(), EvalError> {
        let values = self.b.eval(ctx)?.into_iter().sum::<f32>();
        self.c.set(values as f32);
        Ok(())
    }
}

#[derive(Node)]
struct Constant {
    #[input]
    value: Single<f32>,
    #[output]
    constant: Output<f32>,
}

impl Eval for Constant {
    fn eval(&mut self, ctx: &EvalContext) -> Result<(), EvalError> {
        let value = self.value.eval(ctx)?;
        self.constant.set(value);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>>{
    let mut registry = NodeTypeRegistry::new();
    <SumAll as vislum_op::node_type::RegisterNodeType>::register_node_type(&mut registry);
    <Constant as vislum_op::node_type::RegisterNodeType>::register_node_type(&mut registry);

    let constant_type = registry.get("Constant").unwrap();
    let sum_all_type = registry.get("SumAll").unwrap();

    let mut graph = GraphBlueprint::new();

    let constant_node_id = graph.add_node_of_type(constant_type.clone());
    graph.assign_constant(constant_node_id, 0, TaggedValue::Float(1.0))?;

    let sum_all_node_id = graph.add_node_of_type(sum_all_type.clone());
    graph.connect(sum_all_node_id, 0, ConnectionPlacement::End, Connection::new(constant_node_id, 0))?;
    graph.connect(sum_all_node_id, 0, ConnectionPlacement::End, Connection::new(constant_node_id, 0))?;
    graph.connect(sum_all_node_id, 0, ConnectionPlacement::End, Connection::new(constant_node_id, 0))?;
    graph.connect(sum_all_node_id, 0, ConnectionPlacement::End, Connection::new(constant_node_id, 0))?;
    graph.connect(sum_all_node_id, 0, ConnectionPlacement::End, Connection::new(constant_node_id, 0))?;
    graph.connect(sum_all_node_id, 0, ConnectionPlacement::End, Connection::new(constant_node_id, 0))?;
    graph.connect(sum_all_node_id, 0, ConnectionPlacement::End, Connection::new(constant_node_id, 0))?;
    graph.connect(sum_all_node_id, 0, ConnectionPlacement::End, Connection::new(constant_node_id, 0))?;

    let mut ctx = CompilationContext::new(&graph);
    let sum_all_node = ctx.compile_node(sum_all_node_id).unwrap();

    sum_all_node.eval(&EvalContext)?;

    println!("{:?}", sum_all_node.get_output(0));

    Ok(())
}