use super::{EdgeIdx, Graph, NodeIdx};
use crate::{
    configs::graph,
    units::{geo, geo::Coordinate, MetricU32},
};
use log::info;
use progressing;
use progressing::Bar;
use std::collections::BTreeMap;

//------------------------------------------------------------------------------------------------//

#[derive(Debug)]
pub struct ProtoNode {
    id: i64,
    coord: Option<Coordinate>,
    edge_count: u16,
}

impl ProtoNode {
    fn is_in_edge(&self) -> bool {
        self.edge_count > 0
    }
}

//------------------------------------------------------------------------------------------------//

#[derive(Debug)]
pub struct ProtoEdge {
    src_id: i64,
    dst_id: i64,
    metrics: BTreeMap<String, MetricU32>,
}

impl ProtoEdge {
    pub fn new(src_id: i64, dst_id: i64) -> ProtoEdge {
        ProtoEdge {
            src_id,
            dst_id,
            metrics: BTreeMap::new(),
        }
    }

    pub fn src_id(&self) -> &i64 {
        &self.src_id
    }

    pub fn dst_id(&self) -> &i64 {
        &self.dst_id
    }

    pub fn add_metric(&mut self, metric_id: &str, value: MetricU32) -> Option<MetricU32> {
        self.metrics.insert(metric_id.to_owned(), value)
    }

    pub fn metric(&self, metric_idx: &str) -> Option<&MetricU32> {
        self.metrics.get(metric_idx)
    }
}

/// handy for remembering indices after sorting backwards
#[derive(Debug)]
struct MiniProtoEdge {
    src_id: i64,
    dst_id: i64,
    idx: usize,
}

//------------------------------------------------------------------------------------------------//
// graphbuilding

pub struct GraphBuilder {
    proto_nodes: BTreeMap<i64, ProtoNode>,
    proto_edges: Vec<ProtoEdge>,
}
impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            proto_nodes: BTreeMap::new(),
            proto_edges: Vec::new(),
        }
    }

    pub fn push_node(&mut self, id: i64, coord: geo::Coordinate) -> &mut Self {
        // if already added -> update coord
        // if not -> add new node
        if let Some(proto_node) = self.proto_nodes.get_mut(&id) {
            proto_node.coord = Some(coord);
        } else {
            self.proto_nodes.insert(
                id,
                ProtoNode {
                    id,
                    coord: Some(coord),
                    edge_count: 0,
                },
            );
        }
        self
    }

    pub fn is_node_in_edge(&self, id: i64) -> bool {
        if let Some(proto_node) = self.proto_nodes.get(&id) {
            proto_node.is_in_edge()
        } else {
            false
        }
    }

    /// Duration will be calculated from length and maxspeed if not provided.
    pub fn push_edge(&mut self, proto_edge: ProtoEdge) -> &mut Self {
        // add or update src-node
        if let Some(proto_node) = self.proto_nodes.get_mut(&proto_edge.src_id) {
            proto_node.edge_count += 1;
        } else {
            self.proto_nodes.insert(
                proto_edge.src_id,
                ProtoNode {
                    id: proto_edge.src_id,
                    coord: None,
                    edge_count: 1,
                },
            );
        }

        // add or update dst-node
        if let Some(proto_node) = self.proto_nodes.get_mut(&proto_edge.dst_id) {
            proto_node.edge_count += 1;
        } else {
            self.proto_nodes.insert(
                proto_edge.dst_id,
                ProtoNode {
                    id: proto_edge.dst_id,
                    coord: None,
                    edge_count: 1,
                },
            );
        }

        // add edge
        self.proto_edges.push(proto_edge);

        self
    }

    pub fn finalize(mut self, cfg: &graph::Config) -> Result<Graph, String> {
        //----------------------------------------------------------------------------------------//
        // init graph

        info!(
            "START Finalize graph with {} proto-nodes and {} proto-edges.",
            self.proto_nodes.len(),
            self.proto_edges.len()
        );
        let mut graph = Graph::new();

        //----------------------------------------------------------------------------------------//
        // add nodes to graph which belong to edges (sorted by asc id)

        // logging
        info!("START Add nodes (sorted) which belongs to an edge.");
        let mut progress_bar = progressing::BernoulliBar::from_goal(self.proto_nodes.len() as u32);
        info!("{}", progress_bar);
        // start looping
        let mut node_idx = 0;
        // BTreeMap's iter returns sorted by key (asc)
        for (_id, proto_node) in self.proto_nodes.into_iter() {
            // add nodes only if they belong to an edge
            if !proto_node.is_in_edge() {
                progress_bar.add((0, 1));
                continue;
            }

            // add new node
            if let Some(coord) = proto_node.coord {
                graph.node_ids.push(proto_node.id);
                graph.node_coords.push(coord);
                node_idx += 1;
                progress_bar.add((1, 1));
            } else {
                // should not happen if file is okay
                return Err(format!(
                    "Proto-node (id: {}) has no coordinates, but belongs to an edge.",
                    proto_node.id
                ));
            }

            // print progress
            if progress_bar.progress().successes % (1 + (progress_bar.end() / 10)) == 0 {
                info!("{}", progress_bar);
            }
        }
        info!("{}", progress_bar);
        // reduce and optimize memory-usage
        // already dropped via iterator: drop(self.proto_nodes);
        graph.shrink_to_fit();
        assert_eq!(
            graph.node_ids.len() == graph.node_coords.len(),
            node_idx == graph.node_ids.len(),
            "The (maximum index - 1) should not be more than the number of nodes in the graph."
        );
        info!("FINISHED");

        //----------------------------------------------------------------------------------------//
        // sort forward-edges by ascending src-id, then by ascending dst-id -> offset-array

        info!("START Sort proto-forward-edges by their src/dst-IDs.");
        self.proto_edges.sort_by(|e0, e1| {
            e0.src_id
                .cmp(&e1.src_id)
                .then_with(|| e0.dst_id.cmp(&e1.dst_id))
        });
        info!("FINISHED");

        //----------------------------------------------------------------------------------------//
        // build metrics
        // If metrics are built before indices and offsets are built, the need of memory while
        // building is reduced.

        info!("START Create/store/filter metrics.");
        let mut progress_bar = progressing::MappingBar::new(0..=self.proto_edges.len());
        info!("{}", progress_bar);
        // init metric-collections in graph
        graph.init_metrics(cfg, self.proto_edges.len());
        // start looping
        let mut edge_idx = 0usize;
        for proto_edge in self.proto_edges.iter_mut() {
            graph.add_metrics(proto_edge, cfg)?;

            // print progress
            progress_bar.set(edge_idx);
            if progress_bar.progress() % (1 + (progress_bar.end() / 10)) == 0 {
                info!("{}", progress_bar);
            }

            // update edge-idx
            edge_idx += 1;
        }
        // last node needs an upper bound as well for `leaving_edges(...)`
        info!("{}", progress_bar.set(edge_idx));
        // reduce and optimize memory-usage
        graph.shrink_to_fit();
        // Drop edges
        let mut new_proto_edges: Vec<MiniProtoEdge> = self
            .proto_edges
            .into_iter()
            .map(|e| MiniProtoEdge {
                src_id: e.src_id,
                dst_id: e.dst_id,
                idx: 0,
            })
            .collect();
        info!("FINISHED");

        //----------------------------------------------------------------------------------------//
        // build forward-offset-array and edges

        // logging
        info!("START Create the forward-offset-array and the forward-mapping.");
        let mut progress_bar = progressing::MappingBar::new(0..=new_proto_edges.len());
        info!("{}", progress_bar);
        // start looping
        let mut src_idx = NodeIdx::zero();
        let mut offset = 0;
        graph.fwd_offsets.push(offset);
        // high-level-idea
        // count offset for each proto_edge (sorted) and apply offset as far as src doesn't change
        let mut edge_idx = 0;
        for proto_edge in new_proto_edges.iter_mut() {
            // find edge-data to compare it with expected data later (when setting offset)
            let src_id = proto_edge.src_id;
            let dst_id = proto_edge.dst_id;

            // Add edge-idx here to remember it for indirect mapping bwd->fwd.
            // Update it at the end of the loop.
            proto_edge.idx = edge_idx;

            // do not swap src and dst since this is a forward-edge
            let edge_src_idx = match graph.nodes().idx_from(src_id) {
                Ok(idx) => idx,
                Err(_) => {
                    return Err(format!(
                        "The given src-id `{:?}` doesn't exist as node",
                        proto_edge.src_id
                    ))
                }
            };
            let edge_dst_idx = match graph.nodes().idx_from(dst_id) {
                Ok(idx) => idx,
                Err(_) => {
                    return Err(format!(
                        "The given dst-id `{:?}` doesn't exist as node",
                        proto_edge.dst_id
                    ))
                }
            };

            // If coming edges have new src, then update offset of new src.
            // Loop because of nodes with no leaving edges.
            // Nodes of id y with no leaving edge must have the same offset as the node of id (y+1)
            // to remember it.
            while src_idx != edge_src_idx.into() {
                src_idx += 1;
                graph.fwd_offsets.push(offset);
            }
            offset += 1;
            graph.bwd_dsts.push(edge_src_idx);
            graph.fwd_dsts.push(edge_dst_idx);
            // mapping fwd to fwd is just the identity
            graph.fwd_to_fwd_map.push(EdgeIdx::new(edge_idx));

            // print progress
            progress_bar.set(edge_idx);
            if progress_bar.progress() % (1 + (progress_bar.end() / 10)) == 0 {
                info!("{}", progress_bar);
            }

            // update edge-idx
            edge_idx += 1;
        }
        // last node needs an upper bound as well for `leaving_edges(...)`
        graph.fwd_offsets.push(offset);
        info!("{}", progress_bar.set(offset));
        // reduce and optimize memory-usage
        // already dropped via iterator: drop(self.proto_edges);
        graph.shrink_to_fit();
        info!("FINISHED");

        //----------------------------------------------------------------------------------------//
        // sort backward-edges by ascending dst-id, then by ascending src-id -> offset-array

        info!("START Sort proto-backward-edges by their dst/src-IDs.");
        new_proto_edges.sort_by(|e0, e1| {
            e0.dst_id
                .cmp(&e1.dst_id)
                .then_with(|| e0.src_id.cmp(&e1.src_id))
        });
        info!("FINISHED");

        //----------------------------------------------------------------------------------------//
        // build backward-offset-array

        // logging
        info!("START Create the backward-offset-array.");
        let mut progress_bar = progressing::MappingBar::new(0..=new_proto_edges.len());
        info!("{}", progress_bar);
        // start looping
        let mut src_idx = NodeIdx::zero();
        let mut offset = 0;
        graph.bwd_offsets.push(offset);
        // high-level-idea
        // count offset for each proto_edge (sorted) and apply offset as far as src doesn't change
        for edge_idx in 0..new_proto_edges.len() {
            let proto_edge = &mut new_proto_edges[edge_idx];

            // find edge-data to compare it with expected data later (when setting offset)
            let dst_id = proto_edge.dst_id;
            // swap src and dst since this is the backward-edge
            let edge_src_idx = match graph.nodes().idx_from(dst_id) {
                Ok(idx) => idx,
                Err(_) => {
                    return Err(format!(
                        "The given dst-id `{:?}` doesn't exist as node",
                        proto_edge.dst_id
                    ));
                }
            };

            // If coming edges have new src, then update offset of new src.
            // Loop because of nodes with no leaving edges.
            // Nodes of id y with no leaving edge must have the same offset as the node of id (y+1)
            // to remember it.
            while src_idx != edge_src_idx {
                src_idx += 1;
                graph.bwd_offsets.push(offset);
            }
            offset += 1;
            // For the backward-mapping, bwd-indices have been remembered above,
            // but applied to forward-sorted-edges.
            // Now, that's used to generate the mapping from backward to forward,
            // which is needed for the offset-arrays.
            graph.bwd_to_fwd_map.push(EdgeIdx::new(proto_edge.idx));

            // print progress
            progress_bar.set(edge_idx);
            if progress_bar.progress() % (1 + (progress_bar.end() / 10)) == 0 {
                info!("{}", progress_bar);
            }
        }
        // last node needs an upper bound as well for `leaving_edges(...)`
        debug_assert_eq!(
            offset,
            new_proto_edges.len(),
            "Last offset-value should be as big as the number of proto-edges."
        );
        graph.bwd_offsets.push(offset);
        info!("{}", progress_bar.set(edge_idx));
        // reduce and optimize memory-usage
        graph.shrink_to_fit();
        info!("FINISHED");

        info!("FINISHED");

        Ok(graph)
    }
}
