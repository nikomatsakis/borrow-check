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

use crate::indices::{EdgeIndex, Indices, NodeIndex};
use crate::vec_family::{IndexVec, VecFamily};
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Relation<F: VecFamily> {
    nodes: F::NodeVec,
    edges: F::EdgeVec,
    edge_free_list: Option<EdgeIndex>,
}

#[derive(Default, Debug)]
pub struct NodeData {
    first_edges: Indices<Option<EdgeIndex>>,
}

#[derive(Debug)]
pub struct EdgeData {
    nodes: Indices<NodeIndex>,
    next_edges: Indices<Option<EdgeIndex>>,
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

    fn alloc_edge(&mut self, edge_data: EdgeData) -> EdgeIndex {
        if let Some(free_edge) = self.edge_free_list {
            let next_free_edge;
            {
                let free_edge_data = &mut self[free_edge];
                next_free_edge = free_edge_data.next_edges.outgoing();
                *free_edge_data = edge_data;
            }
            self.edge_free_list = next_free_edge;
            free_edge
        } else {
            self.edges.push(edge_data)
        }
    }

    pub fn add_edge(&mut self, predecessor: NodeIndex, successor: NodeIndex) {
        // Check that edge does not already exist.
        if self.successors(predecessor).any(|s| s == predecessor) {
            return;
        }

        let next_incoming = self[successor].first_edges.incoming();
        let next_outgoing = self[predecessor].first_edges.outgoing();
        let edge_index = self.alloc_edge(EdgeData {
            nodes: Indices::new(predecessor, successor),
            next_edges: Indices::new(next_incoming, next_outgoing),
        });
        self[successor].first_edges.set_incoming(Some(edge_index));
        self[predecessor].first_edges.set_outgoing(Some(edge_index));
    }

    fn count_edges_saturating(&mut self, node: NodeIndex, direction: Direction) -> usize {
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
    fn move_edges_to_free_list(&mut self, node: NodeIndex, direction: Direction) {
        println!(
            "move_edges_to_free_list(node={:?}, direction={:?})",
            node, direction
        );
        let mut next_edge_to_remove = self[node].first_edges[direction];
        let inv_direction = direction.invert();

        // The new head of the free list.
        let mut next_free_list_edge = self.edge_free_list;
        while let Some(edge_to_remove) = next_edge_to_remove {
            let other_node;
            let other_next_edge;

            {
                let edge_data = &mut self[edge_to_remove];
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
        self[node].first_edges[direction] = None;
    }

    /// Go through the list of edges for `node` (in the given
    /// direction) until you find `edge`; remove it from the
    /// list. Does not affect (or even read) the edge data for `edge`
    /// in any way.
    fn unlink_edge(
        &mut self,
        node: NodeIndex,
        direction: Direction,
        edge: EdgeIndex,
        next_edge: Option<EdgeIndex>,
    ) {
        println!(
            "unlink_edge(node={:?}, direction={:?}, edge={:?}, next_edge={:?})",
            node, direction, edge, next_edge
        );

        let mut cur_edge;

        {
            let node_data = &mut self[node];
            cur_edge = node_data.first_edges[direction].unwrap();
            if cur_edge == edge {
                node_data.first_edges[direction] = next_edge;
                return;
            }
        }

        loop {
            let edge_data = &mut self[cur_edge];
            cur_edge = edge_data.next_edges[direction].unwrap();
            if cur_edge == edge {
                edge_data.next_edges[direction] = next_edge;
                return;
            }
        }
    }

    /// Remove all edges from `node`, preserving transitive
    /// relationships between other nodes.
    pub fn remove_edges(&mut self, node: NodeIndex) {
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

        if incoming_count == 1 && outgoing_count == 1 {
            // Another easy case. Only one predecessor and one
            // successor, like this (here, `P` and `S` represent the
            // edge indices):
            //
            // A -P-> B -S-> C
            //
            // In this case, to remove B, we can just redirect one of
            // those edges to go directly from A to C (we choose `P`),
            // and kill the other:
            //
            //     A -P-> C
            //     B
            //     Free list: S

            let successor = self.move_only_outgoing_edge_to_free_list(node);
            return self.redirect_only_incoming_edge(node, successor);
        }

        panic!("not yet implemented");
    }

    /// Given a node that is known to have exactly one successor, move
    /// the outgoing edge to the free list, and return the node that
    /// was its target.
    fn move_only_outgoing_edge_to_free_list(&mut self, node: NodeIndex) -> NodeIndex {
        let edge_to_remove = self[node].first_edges.take_outgoing().unwrap();
        let successor_node;
        let successor_next;
        {
            let edge_free_list = self.edge_free_list;
            let edge_data = &mut self[edge_to_remove];
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

    fn redirect_only_incoming_edge(&mut self, node: NodeIndex, successor: NodeIndex) {
        // unwrap will not panic as there must be an incoming edge for this function to be called
        let edge_to_redirect = self[node].first_edges.incoming().unwrap();
        let first_incoming_edge_of_successor = self[successor].first_edges.incoming();
        {
            let edge_to_redirect_data = &mut self[edge_to_redirect];
            edge_to_redirect_data.nodes.set_outgoing(successor);
            edge_to_redirect_data
                .next_edges
                .set_incoming(first_incoming_edge_of_successor);
        }
        self[successor]
            .first_edges
            .set_incoming(Some(edge_to_redirect));
    }

    /// Iterate over all the edge indices coming out of a
    /// node. Careful, because edge indices get invalidated by removal
    /// operations.
    fn edges(&self, node: NodeIndex, direction: Direction) -> Edges<'_, F> {
        let edge_index = self[node].first_edges[direction];
        Edges {
            relation: self,
            edge_index,
            direction,
        }
    }

    pub fn successors(&self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.edges(node, Direction::Outgoing)
            .map(move |edge| self[edge].nodes.outgoing())
    }

    pub fn nodes(&self) -> impl Iterator<Item = NodeIndex> {
        (0..self.nodes.len()).map(|i| NodeIndex::from(i))
    }

    #[cfg(test)]
    fn dump_and_assert(&self) -> Vec<String> {
        use std::collections::HashSet;

        let mut result = vec![];
        let mut edge_indices_observed = HashSet::new();

        for pred in self.nodes() {
            for edge in self.edges(pred, Direction::Outgoing) {
                let succ = self[edge].nodes.outgoing();
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
                let pred = self[edge].nodes.incoming();

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

            next_free_edge = self[free_edge].next_edges.outgoing();
        }

        result
    }
}

struct Edges<'r, F: VecFamily + 'r> {
    relation: &'r Relation<F>,
    edge_index: Option<EdgeIndex>,
    direction: Direction,
}

impl<F> Iterator for Edges<'r, F>
where
    F: VecFamily,
{
    type Item = EdgeIndex;

    fn next(&mut self) -> Option<EdgeIndex> {
        let current = self.edge_index;
        if let Some(edge) = current {
            self.edge_index = self.relation[edge].next_edges[self.direction];
        }
        current
    }
}

impl<F> Index<NodeIndex> for Relation<F>
where
    F: VecFamily,
{
    type Output = NodeData;

    fn index(&self, value: NodeIndex) -> &NodeData {
        self.nodes.get(value)
    }
}

impl<F> Index<EdgeIndex> for Relation<F>
where
    F: VecFamily,
{
    type Output = EdgeData;

    fn index(&self, value: EdgeIndex) -> &EdgeData {
        self.edges.get(value)
    }
}

impl<F> IndexMut<NodeIndex> for Relation<F>
where
    F: VecFamily,
{
    fn index_mut(&mut self, value: NodeIndex) -> &mut NodeData {
        self.nodes.get_mut(value)
    }
}

impl<F> IndexMut<EdgeIndex> for Relation<F>
where
    F: VecFamily,
{
    fn index_mut(&mut self, value: EdgeIndex) -> &mut EdgeData {
        self.edges.get_mut(value)
    }
}
