use vislum_op::{bundle, EvalError, EvaluateContext, Operator, Output, Reflect, Single};

/// Adds two floats.
#[derive(Reflect)]
#[reflect(name("vislum.std.math.AddFloats"))]
pub struct AddFloats {
    #[input]
    a: Single<f32>,
    #[input]
    b: Single<f32>,

    #[output]
    add: Output<f32>,
}

impl Operator for AddFloats {
    fn evaluate(&mut self, context: EvaluateContext) -> Result<(), EvalError> {
        let a = self.a.evaluate(context)?;
        let b = self.b.evaluate(context)?;
        self.add.set(a + b);
        Ok(())
    }
}

#[derive(Reflect)]
#[reflect(name("vislum.std.math.ConstantFloat"))]
pub struct ConstantFloat {
    #[input]
    value: Single<f32>,

    #[output]
    constant: Output<f32>,
}

impl Operator for ConstantFloat {
    fn evaluate(&mut self, context: EvaluateContext) -> Result<(), EvalError> {
        let value = self.value.evaluate(context)?;
        self.constant.set(value);
        Ok(())
    }
}

#[derive(Reflect)]
#[reflect(name("vislum.std.math.MultiplyFloats"))]
pub struct MultiplyFloats {
    #[input]
    a: Single<f32>,
    #[input]
    b: Single<f32>,

    #[output]
    multiplied: Output<f32>,
}

impl Operator for MultiplyFloats {
    fn evaluate(&mut self, context: EvaluateContext) -> Result<(), EvalError> {
        let a = self.a.evaluate(context)?;
        let b = self.b.evaluate(context)?;
        self.multiplied.set(a * b);
        Ok(())
    }
}

#[derive(Reflect)]
#[reflect(name("vislum.std.math.SinFloat"))]
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

impl Operator for SinFloat {
    fn evaluate(&mut self, context: EvaluateContext) -> Result<(), EvalError> {
        let value = self.value.evaluate(context)?;
        let phase = self.phase.evaluate(context)?;
        let amplitude = self.amplitude.evaluate(context)?;

        self.sin.set((value + phase).sin() * amplitude);
        Ok(())
    }
}

#[derive(Reflect)]
#[reflect(name("vislum.std.math.CosFloat"))]
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

impl Operator for CosFloat {
    fn evaluate(&mut self, context: EvaluateContext) -> Result<(), EvalError> {
        let value = self.value.evaluate(context)?;
        let phase = self.phase.evaluate(context)?;
        let amplitude = self.amplitude.evaluate(context)?;

        self.cos.set((value + phase).cos() * amplitude);
        Ok(())
    }
}

#[derive(Reflect)]
#[reflect(name("vislum.std.math.SinCosFloat"))]
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

impl Operator for SinCosFloat {
    fn evaluate(&mut self, context: EvaluateContext) -> Result<(), EvalError> {
        let value = self.value.evaluate(context)?;
        let phase = self.phase.evaluate(context)?;
        let amplitude = self.amplitude.evaluate(context)?;

        let (sin, cos) = (value + phase).sin_cos();

        self.sin.set(sin * amplitude);
        self.cos.set(cos * amplitude);
        Ok(())
    }
}

bundle! {
    /// A bundle of all math operators.
    pub struct Math {
        AddFloats,
        ConstantFloat,
        MultiplyFloats,
        SinFloat,
        CosFloat,
        SinCosFloat,
    }
}