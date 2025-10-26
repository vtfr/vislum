use vislum_op::{
    eval::{EvalContext, EvalError, Output, Single},
    prelude::*,
};

/// Adds two floats.
#[derive(Node)]
#[node(name("vislum.std.math.AddFloats"))]
pub struct AddFloats {
    #[input]
    a: Single<f32>,

    #[input]
    b: Single<f32>,

    #[output]
    add: Output<f32>,
}

impl Eval for AddFloats {
    fn eval(&mut self, context: &EvalContext) -> Result<(), EvalError> {
        let a = self.a.eval(context)?;
        let b = self.b.eval(context)?;
        self.add.set(a + b);
        Ok(())
    }
}

#[derive(Node)]
#[node(name("vislum.std.math.ConstantFloat"))]
pub struct ConstantFloat {
    #[input(assignment(CONSTANT))]
    value: Single<f32>,

    #[output]
    constant: Output<f32>,
}

impl Eval for ConstantFloat {
    fn eval(&mut self, context: &EvalContext) -> Result<(), EvalError> {
        let value = self.value.eval(context)?;
        self.constant.set(value);
        Ok(())
    }
}

#[derive(Node)]
#[node(name("vislum.std.math.MultiplyFloats"))]
pub struct MultiplyFloats {
    #[input]
    a: Single<f32>,

    #[input]
    b: Single<f32>,

    #[output]
    multiplied: Output<f32>,
}

impl Eval for MultiplyFloats {
    fn eval(&mut self, context: &EvalContext) -> Result<(), EvalError> {
        let a = self.a.eval(context)?;
        let b = self.b.eval(context)?;
        self.multiplied.set(a * b);
        Ok(())
    }
}

#[derive(Node)]
#[node(name("vislum.std.math.SinFloat"))]
pub struct SinFloat {
    #[input]
    value: Single<f32>,
    #[input]
    phase: Single<f32>,
    #[input]
    amplitude: Single<f32>,

    #[output]
    sin: Output<f32>,
}

impl Eval for SinFloat {
    fn eval(&mut self, context: &EvalContext) -> Result<(), EvalError> {
        let value = self.value.eval(context)?;
        let phase = self.phase.eval(context)?;
        let amplitude = self.amplitude.eval(context)?;

        self.sin.set((value + phase).sin() * amplitude);
        Ok(())
    }
}

#[derive(Node)]
#[node(name("vislum.std.math.CosFloat"))]
pub struct CosFloat {
    #[input]
    value: Single<f32>,
    #[input]
    phase: Single<f32>,
    #[input]
    amplitude: Single<f32>,
    #[output]
    cos: Output<f32>,
}

impl Eval for CosFloat {
    fn eval(&mut self, context: &EvalContext) -> Result<(), EvalError> {
        let value = self.value.eval(context)?;
        let phase = self.phase.eval(context)?;
        let amplitude = self.amplitude.eval(context)?;

        self.cos.set((value + phase).cos() * amplitude);
        Ok(())
    }
}

#[derive(Node)]
#[node(name("vislum.std.math.SinCosFloat"))]
pub struct SinCosFloat {
    #[input]
    value: Single<f32>,
    #[input]
    phase: Single<f32>,
    #[input]
    amplitude: Single<f32>,

    #[output]
    sin: Output<f32>,
    #[output]
    cos: Output<f32>,
}

impl Eval for SinCosFloat {
    fn eval(&mut self, context: &EvalContext) -> Result<(), EvalError> {
        let value = self.value.eval(context)?;
        let phase = self.phase.eval(context)?;
        let amplitude = self.amplitude.eval(context)?;

        let (sin, cos) = (value + phase).sin_cos();

        self.sin.set(sin * amplitude);
        self.cos.set(cos * amplitude);
        Ok(())
    }
}

bundle! {
    /// A bundle of all math operators.
    pub struct MathStd {
        AddFloats,
        ConstantFloat,
        MultiplyFloats,
        SinFloat,
        CosFloat,
        SinCosFloat,
    }
}
