use std::{error::Error, rc::Rc};

use vislum_op::{eval::{CompilationContext, Eval, EvalContext, EvalError, Output, Single}, prelude::*};

#[derive(Node)]
struct Banana {
    #[input(assignment(CONSTANT))]
    a: Single<f32>,
    #[input(assignment(CONSTANT))]
    b: Single<f32>,
    #[output]
    c: Output<f32>,
}

impl Eval for Banana {
    fn eval(&mut self, ctx: &EvalContext) -> Result<(), EvalError> {
        let a = self.a.eval(ctx)?;
        let b = self.b.eval(ctx)?;
        self.c.set(a + b);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>>{
    let mut registry = NodeTypeRegistry::new();
    <Banana as vislum_op::node_type::RegisterNodeType>::register_node_type(&mut registry);

    let banana_type = registry.get("Banana").unwrap();

    let mut graph = GraphBlueprint::new();
    let node_id = graph.add_node_of_type(banana_type.clone());
    graph.assign_constant(node_id, 0, TaggedValue::Float(1.0))?;
    graph.assign_constant(node_id, 1, TaggedValue::Float(2.0))?;

    let mut ctx = CompilationContext::new(&graph);
    let node = ctx.compile_node(node_id).unwrap();

    node.eval(&EvalContext)?;

    println!("{:?}", node.get_output(0));

    Ok(())
}