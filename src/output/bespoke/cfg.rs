use crate::facts::{AllFacts, Point};
use crate::intern::InternerTables;
use petgraph::{self, Direction};

type InternalGraph = petgraph::graph::Graph<(), ()>;
type InternalNode = petgraph::graph::NodeIndex;

crate struct ControlFlowGraph {
    graph: InternalGraph,
}

impl ControlFlowGraph {
    crate fn new(tables: &InternerTables, all_facts: &AllFacts) -> Self {
        let mut graph =
            InternalGraph::with_capacity(tables.len::<Point>(), all_facts.cfg_edge.len());

        for _ in tables.each::<Point>() {
            graph.add_node(());
        }

        for (p, q) in &all_facts.cfg_edge {
            graph.add_edge(
                InternalNode::new(p.index()),
                InternalNode::new(q.index()),
                (),
            );
        }

        ControlFlowGraph { graph }
    }

    crate fn successors(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        self.graph
            .neighbors_directed(InternalNode::new(point.index()), Direction::Outgoing)
            .map(|node| Point::from(node.index()))
    }

    crate fn predecessors(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        self.graph
            .neighbors_directed(InternalNode::new(point.index()), Direction::Incoming)
            .map(|node| Point::from(node.index()))
    }

    crate fn has_predecessors(&self, point: Point) -> bool {
        self.predecessors(point).next().is_some()
    }
}
