use std::{error::Error, rc::Rc};

use vislum_op::{eval::{Node, EvalContext, Output, Single}, prelude::*};

#[derive(Node)]
struct Banana {
    #[input]
    a: Single<f32>,
    #[input]
    b: Single<f32>,
    #[output]
    c: Output<f32>,
}

impl Node for Banana {
    fn eval(&mut self, ctx: &EvalContext) -> Result<(), ()> {
        todo!()
    }
}

fn main() -> Result<(), Box<dyn Error>>{
    // let add_node_type = Rc::new(NodeType::new(
    //     NodeTypeId::new("Add"),
    //     vec![
    //         InputDefinition::new("a", &f32::INFO, InputCardinality::Single, AssignmentTypes::CONSTANT | AssignmentTypes::CONNECTION),
    //         InputDefinition::new("b", &f32::INFO, InputCardinality::Single, AssignmentTypes::CONSTANT),
    //     ],
    //     vec![
    //         OutputDefinition::new("c", &f32::INFO),
    //     ],
    // ));

    // let mut graph = GraphBlueprint::new();
    // let node_id1 = graph.add_node_of_type(add_node_type.clone());
    // let node_id2 = graph.add_node_of_type(add_node_type.clone());

    // dbg!(graph.can_connect(node_id1, 0, Connection::new(node_id2, 0)));

    // graph.connect(
    //     node_id1, 
    //     0, 
    //     ConnectionPlacement::End, 
    //     Connection::new(node_id2, 0),
    // )?;

    // graph.assign_constant(node_id1, 0, TaggedValue::Float(1.0))?;
    // let node = graph.get_node(node_id1).unwrap();
    // let (_, input_def) = node.get_input_with_def(0).unwrap();

    // println!("{:?}", input_def.flags());

    Ok(())
}