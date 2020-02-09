//------------------------------------------------------------------------------------------------//
// own modules

mod paths;

//------------------------------------------------------------------------------------------------//
// other modules

use crate::{
    network::{Graph, HalfEdge, Node, NodeIdx},
    units::Metric,
};
use std::{cmp::Ordering, collections::BinaryHeap, fmt, fmt::Display, ops::Add};

//------------------------------------------------------------------------------------------------//
// Path

/// A path from a src to a dst storing predecessors and successors.
///
/// The implementation bases either on vectors or on hashmaps.
/// Some words about it without doing a benchmark:
/// - Since the vector-approach stores two fully allocated vectors, it probably consumes more memory than the hashmap-approach.
/// - Just by looking at resulting times of long paths (~600 km) in Germany, the hashmap-approach seems to be slightly better in performance, but both approaches take around 7 seconds for it.
#[derive(Clone)]
pub struct Path<M>
where
    M: Metric,
{
    // core: paths::VecPath<M>,
    core: paths::HashPath<M>,
}

impl<M> Path<M>
where
    M: Metric,
{
    fn from(src_idx: NodeIdx, dst_idx: NodeIdx, _graph: &Graph) -> Self {
        // let core = paths::VecPath::with_capacity(src_idx, dst_idx, graph.nodes().count());
        let core = paths::HashPath::new(src_idx, dst_idx);
        Path { core }
    }

    //--------------------------------------------------------------------------------------------//

    pub fn src_idx(&self) -> NodeIdx {
        self.core.src_idx
    }

    pub fn dst_idx(&self) -> NodeIdx {
        self.core.dst_idx
    }

    pub fn cost(&self) -> M {
        self.core.cost
    }

    /// Return idx of predecessor-node
    pub fn pred_node_idx(&self, idx: NodeIdx) -> Option<NodeIdx> {
        self.core.pred_node_idx(idx)
    }

    /// Return idx of successor-node
    pub fn succ_node_idx(&self, idx: NodeIdx) -> Option<NodeIdx> {
        self.core.succ_node_idx(idx)
    }
}

//------------------------------------------------------------------------------------------------//
// storing costs and local information

#[derive(Copy, Clone, Debug)]
enum Direction {
    FWD,
    BWD,
}

impl Ord for Direction {
    fn cmp(&self, other: &Direction) -> Ordering {
        let self_value = match self {
            Direction::FWD => 1,
            Direction::BWD => -1,
        };
        let other_value = match other {
            Direction::FWD => 1,
            Direction::BWD => -1,
        };
        self_value.cmp(&other_value)
    }
}

impl PartialOrd for Direction {
    fn partial_cmp(&self, other: &Direction) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Direction {}

impl PartialEq for Direction {
    fn eq(&self, other: &Direction) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Direction::FWD => "forward",
                Direction::BWD => "backward",
            }
        )
    }
}

#[derive(Copy, Clone, Debug)]
struct CostNode<M>
where
    M: Metric,
{
    pub idx: NodeIdx,
    pub cost: M,
    pub estimation: M,
    pub pred_idx: Option<NodeIdx>,
    pub direction: Direction,
}

impl<M> Display for CostNode<M>
where
    M: Metric,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ idx: {}, cost: {}, esti: {}, pred-idx: {}, {} }}",
            self.idx,
            self.cost,
            self.estimation,
            match self.pred_idx {
                Some(idx) => format!("{}", idx),
                None => String::from("None"),
            },
            self.direction
        )
    }
}

impl<M> Ord for CostNode<M>
where
    M: Metric + Ord + Add<M, Output = M>,
{
    fn cmp(&self, other: &CostNode<M>) -> Ordering {
        // (1) cost in float, but cmp uses only m, which is ok
        // (2) inverse order since BinaryHeap is max-heap, but min-heap is needed
        (other.cost + other.estimation)
            .cmp(&(self.cost + self.estimation))
            .then_with(|| other.idx.cmp(&self.idx))
            .then_with(|| other.direction.cmp(&self.direction))
    }
}

impl<M> PartialOrd for CostNode<M>
where
    M: Metric + Ord + Add<M, Output = M>,
{
    fn partial_cmp(&self, other: &CostNode<M>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<M> Eq for CostNode<M> where M: Metric + Ord + Add<M, Output = M> {}

impl<M> PartialEq for CostNode<M>
where
    M: Metric + Ord + Add<M, Output = M>,
{
    fn eq(&self, other: &CostNode<M>) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

//------------------------------------------------------------------------------------------------//
// Astar

pub trait Astar<M>
where
    M: Metric,
{
    fn compute_best_path(&mut self, src: &Node, dst: &Node, graph: &Graph) -> Option<Path<M>>
    where
        M: Metric;
}

//------------------------------------------------------------------------------------------------//
// GenericAstar

/// Cost-function, Estimation-function and Metric
pub struct GenericAstar<C, E, M>
where
    C: Fn(&HalfEdge) -> M,
    E: Fn(&Node, &Node) -> M,
    M: Metric,
{
    cost_fn: C,
    estimate_fn: E,
    queue: BinaryHeap<CostNode<M>>, // max-heap, but CostNode's natural order is reversed
    // fwd
    fwd_costs: Vec<M>,
    predecessors: Vec<Option<NodeIdx>>,
    is_visited_by_src: Vec<bool>,
    // bwd
    bwd_costs: Vec<M>,
    successors: Vec<Option<NodeIdx>>,
    is_visited_by_dst: Vec<bool>,
}

impl<C, E, M> GenericAstar<C, E, M>
where
    C: Fn(&HalfEdge) -> M,
    E: Fn(&Node, &Node) -> M,
    M: Metric + Ord + Add<M, Output = M>,
{
    pub fn from(cost_fn: C, estimate_fn: E) -> GenericAstar<C, E, M> {
        GenericAstar {
            cost_fn,
            estimate_fn,
            queue: BinaryHeap::new(),
            // fwd
            fwd_costs: vec![M::inf(); 0],
            predecessors: vec![None; 0],
            is_visited_by_src: vec![false; 0],
            // bwd
            bwd_costs: vec![M::inf(); 0],
            successors: vec![None; 0],
            is_visited_by_dst: vec![false; 0],
        }
    }

    /// Resizes existing datastructures storing routing-data like costs saving re-allocations.
    fn resize(&mut self, new_len: usize) {
        // fwd
        self.fwd_costs.splice(.., vec![M::inf(); new_len]);
        self.predecessors.splice(.., vec![None; new_len]);
        self.is_visited_by_src.splice(.., vec![false; new_len]);
        // bwd
        self.bwd_costs.splice(.., vec![M::inf(); new_len]);
        self.successors.splice(.., vec![None; new_len]);
        self.is_visited_by_dst.splice(.., vec![false; new_len]);

        self.queue.clear();
    }

    /// The given costnode is a meeting-costnode, if it is visited by both, the search starting in src and the search starting in dst.
    fn is_meeting_costnode(&self, costnode: &CostNode<M>) -> bool {
        self.is_visited_by_src[costnode.idx.to_usize()]
            && self.is_visited_by_dst[costnode.idx.to_usize()]
    }

    fn visit(&mut self, costnode: &CostNode<M>) {
        match costnode.direction {
            Direction::FWD => self.is_visited_by_src[costnode.idx.to_usize()] = true,
            Direction::BWD => self.is_visited_by_dst[costnode.idx.to_usize()] = true,
        }
    }

    fn total_cost(&self, costnode: &CostNode<M>) -> M {
        self.fwd_costs[costnode.idx.to_usize()] + self.bwd_costs[costnode.idx.to_usize()]
    }
}

impl<C, E, M> Astar<M> for GenericAstar<C, E, M>
where
    C: Fn(&HalfEdge) -> M,
    E: Fn(&Node, &Node) -> M,
    M: Metric + Ord + Add<M, Output = M>,
{
    fn compute_best_path(&mut self, src: &Node, dst: &Node, graph: &Graph) -> Option<Path<M>> {
        //----------------------------------------------------------------------------------------//
        // initialization-stuff

        let nodes = graph.nodes();
        let fwd_edges = graph.fwd_edges();
        let bwd_edges = graph.bwd_edges();
        self.resize(nodes.count());
        let mut best_meeting: Option<(CostNode<M>, M)> = None;

        //----------------------------------------------------------------------------------------//
        // prepare first iteration(s)

        // push src-node
        self.queue.push(CostNode {
            idx: src.idx(),
            cost: M::zero(),
            estimation: M::zero(),
            pred_idx: None,
            direction: Direction::FWD,
        });
        // push dst-node
        self.queue.push(CostNode {
            idx: dst.idx(),
            cost: M::zero(),
            estimation: M::zero(),
            pred_idx: None,
            direction: Direction::BWD,
        });
        // update fwd-stats
        self.fwd_costs[src.idx().to_usize()] = M::zero();
        // update bwd-stats
        self.bwd_costs[dst.idx().to_usize()] = M::zero();

        //----------------------------------------------------------------------------------------//
        // search for shortest path

        while let Some(current) = self.queue.pop() {
            // if path is found
            // -> remember best meeting-node
            self.visit(&current);
            if self.is_meeting_costnode(&current) {
                if let Some((_meeting_node, total_cost)) = best_meeting {
                    // if meeting-node is already found
                    // check if new meeting-node is better
                    let new_total_cost = self.total_cost(&current);
                    if new_total_cost < total_cost {
                        best_meeting = Some((current, new_total_cost));
                    }
                } else {
                    best_meeting = Some((current, self.total_cost(&current)));
                }
            }

            // distinguish between fwd and bwd
            let (xwd_costs, xwd_edges, xwd_predecessors, xwd_dst) = match current.direction {
                Direction::FWD => (
                    &mut self.fwd_costs,
                    &fwd_edges,
                    &mut self.predecessors,
                    &dst,
                ),
                Direction::BWD => (&mut self.bwd_costs, &bwd_edges, &mut self.successors, &src),
            };

            // first occurrence has lowest cost
            // -> check if current has already been expanded
            if current.cost > xwd_costs[current.idx.to_usize()] {
                continue;
            }

            // update costs and add predecessors
            // of nodes, which are dst of current's leaving edges
            let leaving_edges = match xwd_edges.starting_from(current.idx) {
                Some(e) => e,
                None => continue,
            };
            for leaving_edge in leaving_edges {
                let new_cost = current.cost + (self.cost_fn)(&leaving_edge);
                if new_cost < xwd_costs[leaving_edge.dst_idx().to_usize()] {
                    xwd_predecessors[leaving_edge.dst_idx().to_usize()] = Some(current.idx);
                    xwd_costs[leaving_edge.dst_idx().to_usize()] = new_cost;

                    // if path is found
                    // -> Run until queue is empty
                    //    since the shortest path could have longer hop-distance
                    //    with shorter weight-distance than currently found node.
                    if best_meeting.is_none() {
                        let leaving_edge_dst = nodes.create(leaving_edge.dst_idx());
                        let estimation = (self.estimate_fn)(&leaving_edge_dst, xwd_dst);
                        self.queue.push(CostNode {
                            idx: leaving_edge.dst_idx(),
                            cost: new_cost,
                            estimation: estimation,
                            pred_idx: Some(current.idx),
                            direction: current.direction,
                        });
                    }
                }
            }
        }

        // create path if found
        if let Some((meeting_node, total_cost)) = best_meeting {
            let mut path = Path::from(src.idx(), dst.idx(), &graph);
            path.core.cost = total_cost;

            // iterate backwards over fwd-path
            let mut cur_idx = meeting_node.idx;
            while let Some(pred_idx) = self.predecessors[cur_idx.to_usize()] {
                path.core.add_pred_succ(pred_idx, cur_idx);
                cur_idx = pred_idx;
            }

            // iterate backwards over bwd-path
            let mut cur_idx = meeting_node.idx;
            while let Some(succ_idx) = self.successors[cur_idx.to_usize()] {
                path.core.add_pred_succ(cur_idx, succ_idx);
                cur_idx = succ_idx;
            }

            // predecessor of src is not set
            // successor of dst is not set
            Some(path)
        } else {
            None
        }
    }
}
