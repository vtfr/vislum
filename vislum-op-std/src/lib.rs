pub mod math;

vislum_op::bundle! {
    /// A bundle of all standard operators.
    pub struct Std {
        math::Math,
    }
}