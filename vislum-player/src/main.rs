use std::ops::DerefMut;

use vislum_op::{
    ConstructOperator, ErasedSlot, EvaluationSystems, Evaluator, Graph, InputSlots, Multi,
    NodeConnection, NodeId, Operator, Output, Placement, Reflect, Single, TaggedValue,
};

#[derive(Reflect)]
struct Add {
    #[input]
    b: Multi<f32>,

    #[output]
    c: Output<f32>,
}

impl Operator for Add {
    fn evaluate(
        &mut self,
        context: vislum_op::EvaluateContext,
    ) -> Result<(), vislum_op::EvalError> {
        let b = self.b.iter(context).sum::<f32>();
        self.c.set(b);
        Ok(())
    }
}

fn main() {
    let mut graph = Graph::new();
    let node_id1 = graph.add_node(<Add as ConstructOperator>::construct_operator());
    let node_id2 = graph.add_node(<Add as ConstructOperator>::construct_operator());
    let node_id3 = graph.add_node(<Add as ConstructOperator>::construct_operator());

    quickly_wire(
        &mut graph,
        node_id1,
        [
            (
                0,
                ErasedSlot::Connection(NodeConnection {
                    node_id: node_id2,
                    output_index: 0,
                }),
            ),
            (
                0,
                ErasedSlot::Connection(NodeConnection {
                    node_id: node_id3,
                    output_index: 0,
                }),
            ),
            (0, ErasedSlot::Constant(TaggedValue::Float(1.0))),
        ],
    );

    quickly_wire(
        &mut graph,
        node_id2,
        [
            (
                0,
                ErasedSlot::Connection(NodeConnection {
                    node_id: node_id3,
                    output_index: 0,
                }),
            ),
            (0, ErasedSlot::Constant(TaggedValue::Float(2.0))),
        ],
    );

    quickly_wire(
        &mut graph,
        node_id3,
        [
            (0, ErasedSlot::Constant(TaggedValue::Float(7.0))),
            (0, ErasedSlot::Constant(TaggedValue::Float(3.0))),
            (0, ErasedSlot::Constant(TaggedValue::Float(10.0))),
        ],
    );

    let evaluator = Evaluator::new(&mut graph, EvaluationSystems::new());
    dbg!(evaluator.get_node_output(node_id1, 0));
}

fn quickly_wire(
    graph: &mut Graph,
    node_id: NodeId,
    what: impl IntoIterator<Item = (usize, ErasedSlot)>,
) {
    let node = graph.get_node_mut(node_id).unwrap();

    for (i, slot) in what.into_iter() {
        node.operator
            .get_input_mut(i)
            .unwrap()
            .set_slot(Placement::After(i), slot)
            .unwrap();
    }
}
