use vislum_asset::AssetSystem;
use vislum_op::OperatorSystem;

/// The runtime for the vislum engine.
pub struct Runtime {
    // pub assets: AssetSystem,
    pub operators: OperatorSystem,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            operators: OperatorSystem::new(),
        }
    }

    pub fn get_operator_system(&self) -> &OperatorSystem {
        &self.operators
    }

    pub fn get_operator_system_mut(&mut self) -> &mut OperatorSystem {
        &mut self.operators
    }
}
