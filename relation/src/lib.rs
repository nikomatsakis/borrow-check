#![feature(catch_expr)]
#![feature(const_fn)]
#![feature(crate_in_paths)]
#![feature(crate_visibility_modifier)]
#![feature(proc_macro)]
#![feature(extern_prelude)]
#![feature(extern_absolute_paths)]
#![feature(in_band_lifetimes)]
#![feature(termination_trait_test)]
#![feature(macro_vis_matcher)]
#![feature(nonzero)]
// #![feature(infer_outlives_requirements)]

pub mod indices;
mod test;
pub mod vec_family;

use crate::indices::Indices;
use crate::vec_family::{IndexVec, VecFamily};

/// A graph data struture that preserve transitive reachability relationships.
///
/// For example, if we have a graph:
///
/// ```notrust
/// A --> B
/// B --> C
/// ```
///
/// Upon removing `B`, we preserve that there exists a path from `A` to `C`, and so
/// our graph becomes:
/// ```notrust
/// A --> C
/// ```
#[derive(Debug)]
pub struct Relation<F: VecFamily> {
    nodes: F::NodeVec,
    edges: F::EdgeVec,
    edge_free_list: Option<F::Edge>,
}

impl<F: VecFamily> Clone for Relation<F> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            edge_free_list: self.edge_free_list.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeData<F: VecFamily> {
    first_edges: Indices<Option<F::Edge>>,
}

impl<F: VecFamily> Default for NodeData<F> {
    fn default() -> Self {
        NodeData {
            first_edges: Indices::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EdgeData<F: VecFamily> {
    nodes: Indices<F::Node>,
    next_edges: Indices<Option<F::Edge>>,
}

/// Represents a direction of an edge
#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Incoming,
    Outgoing,
}

impl Direction {
    pub fn invert(self) -> Direction {
        match self {
            Direction::Incoming => Direction::Outgoing,
            Direction::Outgoing => Direction::Incoming,
        }
    }
}

impl<F: VecFamily> Relation<F> {
    /// Creates a new `Relation` with `num_nodes` elements.
    ///
    /// There are no methods for adding nodes to a `Relation`, they are all
    /// allocated and populated here.
    pub fn new(num_nodes: usize) -> Self {
        Self {
            nodes: F::NodeVec::with_default_elements(num_nodes),
            edges: F::EdgeVec::empty(),
            edge_free_list: None,
        }
    }

    fn alloc_edge(&mut self, edge_data: EdgeData<F>) -> F::Edge {
        if let Some(free_edge) = self.edge_free_list {
            let next_free_edge;
            {
                let free_edge_data = self.edge_mut(free_edge);
                next_free_edge = free_edge_data.next_edges.outgoing();
                *free_edge_data = edge_data;
            }
            self.edge_free_list = next_free_edge;
            free_edge
        } else {
            self.edges.push(edge_data)
        }
    }

    /// Adds an edge from `predecessor` to `successor`
    ///
    /// Returns true if the edge was added, and false if it already exists
    pub fn add_edge(&mut self, predecessor: F::UserNode, successor: F::UserNode) -> bool {
        let predecessor = F::into_node(predecessor);
        let successor = F::into_node(successor);
        self.add_edge_internal(predecessor, successor)
    }

    fn add_edge_internal(&mut self, predecessor: F::Node, successor: F::Node) -> bool {
        // Check that edge does not already exist.
        if self.successors_internal(predecessor)
            .any(|s| s == predecessor)
        {
            false
        } else {
            let next_incoming = self.node(successor).first_edges.incoming();
            let next_outgoing = self.node(predecessor).first_edges.outgoing();
            let edge_index = self.alloc_edge(EdgeData {
                nodes: Indices::new(predecessor, successor),
                next_edges: Indices::new(next_incoming, next_outgoing),
            });
            self.node_mut(successor)
                .first_edges
                .set_incoming(Some(edge_index));
            self.node_mut(predecessor)
                .first_edges
                .set_outgoing(Some(edge_index));
            true
        }
    }

    fn count_edges_saturating(&mut self, node: F::Node, direction: Direction) -> usize {
        let mut edges = self.edges(node, direction);
        if let Some(_) = edges.next() {
            if let Some(_) = edges.next() {
                2
            } else {
                1
            }
        } else {
            0
        }
    }

    /// Take all the edges incident to `node` in the given direction
    /// and move them over to the free list. When done, `node` will have
    /// no edges left from that given direction.
    ///
    /// Example if invoked with `A` and `Outgoing`, then the graph:
    ///
    /// ```notrust
    /// A -E0-> B
    /// A -E1-> C
    /// B -E2-> C
    /// ```
    ///
    /// becomes:
    ///
    /// ```notrust
    /// B -E2-> C
    /// free list: E0, E1
    /// ```
    fn move_edges_to_free_list(&mut self, node: F::Node, direction: Direction) {
        println!(
            "move_edges_to_free_list(node={:?}, direction={:?})",
            node, direction
        );
        let mut next_edge_to_remove = self.node(node).first_edges[direction];
        let inv_direction = direction.invert();

        // The new head of the free list.
        let mut next_free_list_edge = self.edge_free_list;
        while let Some(edge_to_remove) = next_edge_to_remove {
            let other_node;
            let other_next_edge;

            {
                let edge_data = self.edge_mut(edge_to_remove);
                debug_assert_eq!(edge_data.nodes[inv_direction], node);
                next_edge_to_remove = edge_data.next_edges[direction];
                other_node = edge_data.nodes[direction];
                other_next_edge = edge_data.next_edges[inv_direction];
                edge_data.next_edges.set_outgoing(next_free_list_edge);
            }

            self.unlink_edge(other_node, inv_direction, edge_to_remove, other_next_edge);
            next_free_list_edge = Some(edge_to_remove);
        }
        self.edge_free_list = next_free_list_edge;
        self.node_mut(node).first_edges[direction] = None;
    }

    /// Go through the list of edges for `node` (in the given
    /// direction) until you find `edge`; remove it from the
    /// list. Does not affect (or even read) the edge data for `edge`
    /// in any way.
    fn unlink_edge(
        &mut self,
        node: F::Node,
        direction: Direction,
        edge: F::Edge,
        next_edge: Option<F::Edge>,
    ) {
        println!(
            "unlink_edge(node={:?}, direction={:?}, edge={:?}, next_edge={:?})",
            node, direction, edge, next_edge
        );

        let mut cur_edge;

        {
            let node_data = self.node_mut(node);
            cur_edge = node_data.first_edges[direction].unwrap();
            if cur_edge == edge {
                node_data.first_edges[direction] = next_edge;
                return;
            }
        }

        loop {
            let edge_data = self.edge_mut(cur_edge);
            cur_edge = edge_data.next_edges[direction].unwrap();
            if cur_edge == edge {
                edge_data.next_edges[direction] = next_edge;
                return;
            }
        }
    }

    /// Remove all edges from `node`, preserving transitive
    /// relationships between other nodes.
    pub fn remove_edges(&mut self, node: F::UserNode) {
        let node = F::into_node(node);
        let incoming_count = self.count_edges_saturating(node, Direction::Incoming);
        if incoming_count == 0 {
            // Easy case: node with only outgoing edges (or no edges
            // at all). Just kill all the edges, as there can be no
            // transitive relationships.
            return self.move_edges_to_free_list(node, Direction::Outgoing);
        }

        let outgoing_count = self.count_edges_saturating(node, Direction::Outgoing);
        if outgoing_count == 0 {
            // Easy case: node with only incoming edges. Just kill all
            // the edges, as above.
            return self.move_edges_to_free_list(node, Direction::Incoming);
        }

        if outgoing_count == 1 {
            // Before                  After
            //
            // A --+
            //     |
            //     v
            //     C --> D         A --> D <-- B
            //     ^
            //     |
            // B --+
            let successor = self.move_only_outgoing_edge_to_free_list(node);
            return self.redirect_incoming_edges(node, successor);
        }

        if incoming_count == 1 {
            // Before                    After
            //
            //
            //           +--> D            +--> D
            //           |                 |
            // A --> B --+             A --+
            //           |                 |
            //           +--> C            +--> C
            //
            //
            let predecessor = self.move_only_incoming_edge_to_free_list(node);
            return self.redirect_outgoing_edges(node, predecessor);
        }

        // final case: multiple in, multiple out. This case allocates,
        // which is why it is after all of the others.
        self.redirect_all_edges(node);
    }

    /// Given a node that is known to have exactly one successor, move
    /// the outgoing edge to the free list, and return the node that
    /// was its target.
    fn move_only_outgoing_edge_to_free_list(&mut self, node: F::Node) -> F::Node {
        let edge_to_remove = self.node_mut(node).first_edges.take_outgoing().unwrap();
        let successor_node;
        let successor_next;
        {
            let edge_free_list = self.edge_free_list;
            let edge_data = self.edge_mut(edge_to_remove);
            debug_assert_eq!(edge_data.nodes.incoming(), node);
            successor_node = edge_data.nodes.outgoing();
            successor_next = edge_data.next_edges.outgoing();
            edge_data.next_edges.set_outgoing(edge_free_list);
        }
        self.unlink_edge(
            successor_node,
            Direction::Incoming,
            edge_to_remove,
            successor_next,
        );
        self.edge_free_list = Some(edge_to_remove);
        successor_node
    }

    /// Given a node that is known to have exactly one predecessor, move
    /// the incoming edge to the free list, and return the node that
    /// was its origin.
    fn move_only_incoming_edge_to_free_list(&mut self, node: F::Node) -> F::Node {
        let edge_to_remove = self.node_mut(node).first_edges.take_incoming().unwrap();
        let predecessor_node;
        let predecessor_next;
        {
            let edge_free_list = self.edge_free_list;
            let edge_data = self.edge_mut(edge_to_remove);
            debug_assert_eq!(edge_data.nodes.outgoing(), node);
            predecessor_node = edge_data.nodes.incoming();
            predecessor_next = edge_data.next_edges.outgoing();
            edge_data.next_edges.set_outgoing(edge_free_list);
        }
        self.unlink_edge(
            predecessor_node,
            Direction::Outgoing,
            edge_to_remove,
            predecessor_next,
        );
        self.edge_free_list = Some(edge_to_remove);
        predecessor_node
    }

    /// We model edges as a linked list, backed by a vector. Here we "push" each
    /// incoming edge to `node` and push it to the top of the stack of edges in `successor`.
    /// this has the effect of reversing the order of the edge stack from `node` compared to
    /// successor. That is, if we had N(0), with incoming edges E(0), E(1), and E(2), and we
    /// want to connect them to N(1) which may or may not have any other incoming edges.
    ///
    /// N(0) Edge Data:
    /// E(0) -> E(1) -> E(2) -> None
    ///
    /// N(1) Edge Data After:
    /// E(2) -> E(1) -> E(0) -> [head of N(1) subs] -> ...
    ///
    //  We also take care to set `node` to `None` as otherwise, the traversal over all nodes
    //  fails and gives inconsistent results, causing tests to fail.
    fn redirect_incoming_edges(&mut self, node: F::Node, successor: F::Node) {
        let mut edge_to_redirect = self.node(node).first_edges.incoming();
        while let Some(redirected_edge_ind) = edge_to_redirect {
            let tmp;
            let first_incoming_edge_of_successor = self.node(successor).first_edges.incoming();
            {
                let edge_to_redirect_data = self.edge_mut(redirected_edge_ind);
                edge_to_redirect_data.nodes.set_outgoing(successor);
                tmp = edge_to_redirect_data.next_edges.incoming();
                edge_to_redirect_data
                    .next_edges
                    .set_incoming(first_incoming_edge_of_successor);
            }
            self.node_mut(successor)
                .first_edges
                .set_incoming(edge_to_redirect);
            edge_to_redirect = tmp;
        }
        self.node_mut(node).first_edges.set_incoming(None);
    }

    fn redirect_outgoing_edges(&mut self, node: F::Node, predecessor: F::Node) {
        let mut edge_to_redirect = self.node(node).first_edges.outgoing();
        while let Some(redirected_edge_ind) = edge_to_redirect {
            let tmp;
            let first_outgoing_edge_of_predecessor = self.node(predecessor).first_edges.outgoing();
            {
                let edge_to_redirect_data = self.edge_mut(redirected_edge_ind);
                edge_to_redirect_data.nodes.set_incoming(predecessor);
                tmp = edge_to_redirect_data.next_edges.outgoing();
                edge_to_redirect_data
                    .next_edges
                    .set_outgoing(first_outgoing_edge_of_predecessor);
            }
            self.node_mut(predecessor)
                .first_edges
                .set_outgoing(edge_to_redirect);
            edge_to_redirect = tmp;
        }
        self.node_mut(node).first_edges.set_outgoing(None);
    }

    /// Redirects all edges coming into or out of a node, works by removing all
    /// edges, then inserting the necessary ones. Allocates twice for the list
    /// of nodes
    fn redirect_all_edges(&mut self, node: F::Node) {
        let successors: Vec<_> = self.successors_internal(node).collect();
        let predecessors: Vec<_> = self.predecessors_internal(node).collect();

        self.move_edges_to_free_list(node, Direction::Outgoing);
        self.move_edges_to_free_list(node, Direction::Incoming);
        println!("{:#?}", self);
        for s in successors {
            for &p in predecessors.iter() {
                self.add_edge_internal(p, s);
            }
        }
    }

    /// Iterate over all the edge indices coming out of a
    /// node. Careful, because edge indices get invalidated by removal
    /// operations.
    fn edges(&self, node: F::Node, direction: Direction) -> Edges<'_, F> {
        let edge_index = self.node(node).first_edges[direction];
        Edges {
            relation: self,
            edge_index,
            direction,
        }
    }

    pub fn successors(&self, node: F::UserNode) -> impl Iterator<Item = F::UserNode> + '_ {
        let node = F::into_node(node);
        self.successors_internal(node)
            .map(|n| F::from_node(n))
    }

    fn successors_internal(&self, node: F::Node) -> impl Iterator<Item = F::Node> + '_ {
        self.edges(node, Direction::Outgoing)
            .map(move |edge| self.edge(edge).nodes.outgoing())
    }

    pub fn predecessors(&self, node: F::UserNode) -> impl Iterator<Item = F::Node> + '_ {
        let node = F::into_node(node);
        self.predecessors_internal(node)
    }

    fn predecessors_internal(&self, node: F::Node) -> impl Iterator<Item = F::Node> + '_ {
        self.edges(node, Direction::Incoming)
            .map(move |edge| self.edge(edge).nodes.incoming())
    }

    pub fn nodes(&self) -> impl Iterator<Item = F::Node> {
        (0..self.nodes.len()).map(|i| F::Node::from(i))
    }

    #[cfg(test)]
    fn dump_and_assert(&self) -> Vec<String> {
        use std::collections::HashSet;

        let mut result = vec![];
        let mut edge_indices_observed = HashSet::new();

        for pred in self.nodes() {
            for edge in self.edges(pred, Direction::Outgoing) {
                let succ = self.edge(edge).nodes.outgoing();
                result.push(format!("{:?} --{:?}--> {:?}", pred, edge, succ));

                if !edge_indices_observed.insert(edge) {
                    panic!(
                        "observed edge {:?} twice; graph so far:\n{:#?}",
                        edge, result
                    );
                }

                assert!(
                    self.edges(succ, Direction::Incoming).any(|e| e == edge),
                    "edge {:?} not found in incoming list of node {:?}, graph = {:#?}",
                    edge,
                    succ,
                    self
                );
            }
        }

        for succ in self.nodes() {
            for edge in self.edges(succ, Direction::Incoming) {
                let pred = self.edge(edge).nodes.incoming();

                if edge_indices_observed.insert(edge) {
                    panic!(
                        "edge {:?} found in pred list of {:?} but not in succ lists; graph:\n{:#?}",
                        edge, succ, self
                    );
                }

                assert!(
                    self.edges(pred, Direction::Outgoing).any(|e| e == edge),
                    "edge {:?} not found in incoming list of node {:?}, graph = {:#?}",
                    edge,
                    succ,
                    self
                );
            }
        }

        let mut next_free_edge = self.edge_free_list;
        while let Some(free_edge) = next_free_edge {
            result.push(format!("free edge {:?}", free_edge));

            if !edge_indices_observed.insert(free_edge) {
                panic!(
                    "observed edge {:?} twice; graph so far:\n{:#?}",
                    free_edge, result
                );
            }

            next_free_edge = self.edge(free_edge).next_edges.outgoing();
        }

        result
    }

    fn node(&self, node: F::Node) -> &NodeData<F> {
        self.nodes.get(node)
    }

    fn node_mut(&mut self, node: F::Node) -> &mut NodeData<F> {
        self.nodes.get_mut(node)
    }

    fn edge(&self, edge: F::Edge) -> &EdgeData<F> {
        self.edges.get(edge)
    }

    fn edge_mut(&mut self, edge: F::Edge) -> &mut EdgeData<F> {
        self.edges.get_mut(edge)
    }
}

struct Edges<'r, F: VecFamily + 'r> {
    relation: &'r Relation<F>,
    edge_index: Option<F::Edge>,
    direction: Direction,
}

impl<F> Iterator for Edges<'r, F>
where
    F: VecFamily,
{
    type Item = F::Edge;

    fn next(&mut self) -> Option<F::Edge> {
        let current = self.edge_index;
        if let Some(edge) = current {
            self.edge_index = self.relation.edge(edge).next_edges[self.direction];
        }
        current
    }
}
