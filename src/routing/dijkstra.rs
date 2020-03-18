use super::paths::Path;
use crate::{
    configs::routing::Config,
    defaults::capacity::DimVec,
    helpers,
    network::{EdgeIdx, Graph, Node, NodeIdx},
};
use smallvec::smallvec;
use std::{cmp::Reverse, collections::BinaryHeap};

/// A bidirectional implementation of Dijkstra's algorithm.
pub struct Dijkstra {
    queue: BinaryHeap<Reverse<CostNode>>,
    // fwd
    fwd_costs: Vec<f64>,
    predecessors: Vec<Option<EdgeIdx>>,
    is_visited_by_src: Vec<bool>,
    // bwd
    bwd_costs: Vec<f64>,
    successors: Vec<Option<EdgeIdx>>,
    is_visited_by_dst: Vec<bool>,
}

impl Dijkstra {
    pub fn new() -> Dijkstra {
        Dijkstra {
            queue: BinaryHeap::new(),
            // fwd
            fwd_costs: Vec::new(),
            predecessors: Vec::new(),
            is_visited_by_src: Vec::new(),
            // bwd
            bwd_costs: Vec::new(),
            successors: Vec::new(),
            is_visited_by_dst: Vec::new(),
        }
    }

    /// Resizes existing datastructures storing routing-data like costs saving re-allocations.
    fn init_query(&mut self, new_len: usize) {
        // fwd
        self.fwd_costs.resize(new_len, std::f64::INFINITY);
        self.fwd_costs
            .iter_mut()
            .for_each(|c| *c = std::f64::INFINITY);

        self.predecessors.resize(new_len, None);
        self.predecessors.iter_mut().for_each(|p| *p = None);

        self.is_visited_by_src.resize(new_len, false);
        self.is_visited_by_src.iter_mut().for_each(|v| *v = false);

        // bwd
        self.bwd_costs.resize(new_len, std::f64::INFINITY);
        self.bwd_costs
            .iter_mut()
            .for_each(|c| *c = std::f64::INFINITY);

        self.successors.resize(new_len, None);
        self.successors.iter_mut().for_each(|s| *s = None);

        self.is_visited_by_dst.resize(new_len, false);
        self.is_visited_by_dst.iter_mut().for_each(|v| *v = false);

        self.queue.clear();
    }

    /// The given costnode is a meeting-costnode, if it is visited by both, the search starting in src and the search starting in dst.
    /// Further, to be correct for contraction hierarchies, it has to be checked whether the current total-cost are less or equal to the current node-cost.
    /// This is needed because the bidirectional Dijkstra processes sub-graphs, which are not equal.
    /// This leads to the possibility, that shortest-paths of a sub-graph could be non-optimal for the total graph, even if both sub-queries (forward and backward) have already found a common meeting-node.
    /// Paths in sub-graphs have only one direction wrt node-level, namely up for fwd-graph and down for bwd-graph.
    /// TODO
    fn is_meeting_costnode(&self, costnode: &CostNode) -> bool {
        self.is_visited_by_src[*costnode.idx] && self.is_visited_by_dst[*costnode.idx]
    }

    fn visit(&mut self, costnode: &CostNode) {
        match costnode.direction {
            Direction::FWD => self.is_visited_by_src[*costnode.idx] = true,
            Direction::BWD => self.is_visited_by_dst[*costnode.idx] = true,
        }
    }

    fn total_cost(&self, costnode: &CostNode) -> f64 {
        self.fwd_costs[*costnode.idx] + self.bwd_costs[*costnode.idx]
    }

    pub fn compute_best_path(
        &mut self,
        src: &Node,
        dst: &Node,
        graph: &Graph,
        cfg: &Config,
    ) -> Option<Path<DimVec<f64>>> {
        if cfg.dim() <= 0 {
            panic!("Best path should be computed, but no metric is specified.");
        }

        //----------------------------------------------------------------------------------------//
        // initialization-stuff

        let nodes = graph.nodes();
        let fwd_edges = graph.fwd_edges();
        let bwd_edges = graph.bwd_edges();
        self.init_query(nodes.count());
        let mut best_meeting: Option<(NodeIdx, f64)> = None;

        //----------------------------------------------------------------------------------------//
        // prepare first iteration(s)

        // push src-node
        self.queue.push(Reverse(CostNode {
            idx: src.idx(),
            cost: 0.0,
            direction: Direction::FWD,
        }));
        // push dst-node
        self.queue.push(Reverse(CostNode {
            idx: dst.idx(),
            cost: 0.0,
            direction: Direction::BWD,
        }));
        // update fwd-stats
        self.fwd_costs[*src.idx()] = 0.0;
        // update bwd-stats
        self.bwd_costs[*dst.idx()] = 0.0;

        //------------------------------------------------------------------------------------//
        // search for shortest path

        while let Some(Reverse(current)) = self.queue.pop() {
            // if path is found
            // -> remember best meeting-node
            self.visit(&current);
            if self.is_meeting_costnode(&current) {
                if let Some((_meeting_node, total_cost)) = best_meeting {
                    // if meeting-node is already found
                    // check if new meeting-node is better
                    let new_total_cost = self.total_cost(&current);
                    if new_total_cost < total_cost {
                        best_meeting = Some((current.idx, new_total_cost));
                    }
                } else {
                    let total_cost = self.total_cost(&current);
                    best_meeting = Some((current.idx, total_cost));
                }
            }

            // distinguish between fwd and bwd
            // TODO performance: replace by 2d-array and Direction is mapped to idx
            let (xwd_costs, xwd_edges, xwd_predecessors) = match current.direction {
                Direction::FWD => (&mut self.fwd_costs, &fwd_edges, &mut self.predecessors),
                Direction::BWD => (&mut self.bwd_costs, &bwd_edges, &mut self.successors),
            };

            // first occurrence has lowest cost
            // -> check if current has already been expanded
            if current.cost > xwd_costs[*current.idx] {
                continue;
            }

            // update costs and add predecessors
            // of nodes, which are dst of current's leaving edges
            let leaving_edges = match xwd_edges.starting_from(current.idx) {
                Some(e) => e,
                None => continue,
            };
            for leaving_edge in leaving_edges {
                // ch-dijkstra
                if nodes.level(current.idx) > nodes.level(leaving_edge.dst_idx()) {
                    // TODO break with sorted leaving-edges
                    break;
                }
                // TODO
                let new_cost = current.cost
                    + helpers::dot_product(
                        &cfg.alphas(),
                        &leaving_edge.metrics(&cfg.metric_indices()),
                    );
                if new_cost < xwd_costs[*leaving_edge.dst_idx()] {
                    xwd_predecessors[*leaving_edge.dst_idx()] = Some(leaving_edge.idx());
                    xwd_costs[*leaving_edge.dst_idx()] = new_cost;

                    // if path is found
                    // -> Run until queue is empty
                    //    since the shortest path could have longer hop-distance
                    //    with shorter weight-distance than currently found node.
                    if best_meeting.is_none() {
                        self.queue.push(Reverse(CostNode {
                            idx: leaving_edge.dst_idx(),
                            cost: new_cost,
                            direction: current.direction,
                        }));
                    }
                }
            }
        }

        //------------------------------------------------------------------------------------//
        // create path if found

        if let Some((meeting_node_idx, _total_cost)) = best_meeting {
            let mut path = Path::with_capacity(
                src.idx(),
                dst.idx(),
                smallvec![0.0; cfg.dim()],
                nodes.count(),
            );

            // iterate backwards over fwd-path
            let mut cur_idx = meeting_node_idx;
            while let Some(incoming_idx) = self.predecessors[*cur_idx] {
                // get incoming edge, but reversed to get the forward's src-node
                let reverse_incoming_edge = bwd_edges.half_edge(incoming_idx);

                // update real path-costs
                helpers::add_to(
                    path.cost_mut(),
                    &reverse_incoming_edge.metrics(&cfg.metric_indices()),
                );

                // add predecessor/successor and prepare next loop-run
                let pred_idx = reverse_incoming_edge.dst_idx();
                path.add_pred_succ(pred_idx, cur_idx);
                cur_idx = pred_idx;
            }

            // iterate backwards over bwd-path
            let mut cur_idx = meeting_node_idx;
            while let Some(leaving_idx) = self.successors[*cur_idx] {
                // get leaving edge, but reversed to get the backward's src-node
                let reverse_leaving_edge = fwd_edges.half_edge(leaving_idx);

                // update real path-costs
                helpers::add_to(
                    path.cost_mut(),
                    &reverse_leaving_edge.metrics(&cfg.metric_indices()),
                );

                // add predecessor/successor and prepare next loop-run
                let succ_idx = reverse_leaving_edge.dst_idx();
                path.add_pred_succ(cur_idx, succ_idx);
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

#[derive(Copy, Clone, Debug)]
enum Direction {
    FWD,
    BWD,
}

#[derive(Clone)]
struct CostNode {
    idx: NodeIdx,
    cost: f64,
    direction: Direction,
}

mod costnode {
    use super::{CostNode, Direction};
    use crate::helpers::{ApproxCmp, ApproxEq};
    use std::{
        cmp::Ordering,
        fmt::{self, Display},
    };

    impl Display for CostNode {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{{ idx: {}, cost: {}, {} }}",
                self.idx, self.cost, self.direction
            )
        }
    }

    impl Ord for CostNode {
        fn cmp(&self, other: &CostNode) -> Ordering {
            self.cost
                .approx_cmp(&other.cost)
                .then_with(|| self.idx.cmp(&other.idx))
                .then_with(|| self.direction.cmp(&other.direction))
        }
    }

    impl PartialOrd for CostNode {
        fn partial_cmp(&self, other: &CostNode) -> Option<Ordering> {
            Some(
                self.cost
                    .approx_partial_cmp(&other.cost)?
                    .then_with(|| self.idx.cmp(&other.idx))
                    .then_with(|| self.direction.cmp(&other.direction)),
            )
        }
    }

    impl Eq for CostNode {}

    impl PartialEq for CostNode {
        fn eq(&self, other: &CostNode) -> bool {
            self.idx == other.idx
                && self.direction == other.direction
                && self.cost.approx_eq(&other.cost)
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
}
