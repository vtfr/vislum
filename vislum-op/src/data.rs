use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::OperatorTypeId;

#[derive(Serialize, Deserialize)]
struct NodeData {
    operator_type_id: OperatorTypeId<'static>,
    inputs: HashMap<String, InputData>,
}

#[derive(Serialize, Deserialize)]
enum InputData {
    Todo,
}
