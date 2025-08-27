extern crate self as vislum_op;

pub mod animation;
pub mod data;
pub mod eval;
pub mod graph;
pub mod input;
pub mod op;
pub mod output;
pub mod reflect;
pub mod system;
pub mod value;

pub use animation::*;
pub use data::*;
pub use eval::*;
pub use graph::*;
pub use input::*;
pub use op::*;
pub use output::*;
pub use reflect::*;
pub use system::*;
pub use value::*;

#[derive(Reflect)]
pub struct Add {
    #[input]
    a: Single<f32>,
    #[input]
    b: Single<f32>,
    #[output]
    c: Output<f32>,
}

impl Operator for Add {
    fn evaluate(&mut self, context: EvaluateContext) -> Result<(), EvalError> {
        let a = self.a.evaluate(context)?;
        let b = self.b.evaluate(context)?;
        let c = a + b;
        self.c.set(c);
        Ok(())
    }
}
