use std::error::Error;

use vislum_op::{compile::CompilationContext, eval::{Eval, EvalContext, EvalError, Multiple, Output, Single}, prelude::*, system::NodeGraphSystem};

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
    let mut system = NodeGraphSystem::default();
    system.register_node_types::<(SumAll, Constant)>();

    let constant_node_id = system.add_node("Constant");
    system.assign_constant(constant_node_id, 0, TaggedValue::Float(1.0))?;

    let sum_all_node_id = system.add_node("SumAll");
    system.connect(sum_all_node_id, 0, ConnectionPlacement::End, Connection::new(constant_node_id, 0))?;

    let result = system.eval(&EvalContext, sum_all_node_id)?;
    println!("{:?}", result.get_output(0));
    
    system.assign_constant(constant_node_id, 0, TaggedValue::Float(6.0))?;
    let result = system.eval(&EvalContext, sum_all_node_id)?;
    println!("{:?}", result.get_output(0));

    Ok(())
}