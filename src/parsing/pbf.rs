mod pbf {
    pub use osmpbfreader::{reader::OsmPbfReader as Reader, OsmObj};
}

use crate::{
    configs::{graph, MetricCategory},
    helpers,
    network::{GraphBuilder, MetricIdx, ProtoEdge, StreetType},
    units::geo::Coordinate,
};
use log::info;

pub struct Parser;

impl Parser {
    pub fn new() -> Parser {
        Parser {}
    }
}

impl super::Parsing for Parser {
    fn parse_ways(
        &self,
        cfg: &graph::Config,
        graph_builder: &mut GraphBuilder,
    ) -> Result<(), String> {
        info!("START Create edges from input-file.");
        let file = helpers::open_file(cfg.map_file())?;
        for mut way in pbf::Reader::new(file)
            .iter()
            .filter_map(Result::ok)
            .filter_map(|obj| match obj {
                pbf::OsmObj::Way(way) => Some(way),
                _ => None,
            })
        {
            if way.nodes.len() < 2 {
                continue;
            }

            // collect relevant data from file, if way-type is as expected by user
            let highway_tag = match StreetType::from(&way) {
                Some(highway_tag) => highway_tag,
                None => continue,
            };
            if !highway_tag.is_for(&cfg.vehicles.category, cfg.vehicles.are_drivers_picky) {
                continue;
            }

            // get nodes of way to create proto-edges later
            let (is_oneway, is_reverse) = highway_tag.parse_oneway(&way);
            if is_reverse {
                way.nodes.reverse();
            }
            let iter_range = if is_oneway {
                0..0
            } else {
                // if not oneway
                // -> add node-IDs reversed to generate edges forwards and backwards
                0..way.nodes.len() - 1
            };
            let nodes: Vec<i64> = way
                .nodes
                .iter()
                .chain(way.nodes[iter_range].iter().rev())
                .map(|id| id.0)
                .collect();

            // Collect metrics as expected by user-config
            // ATTENTION: A way contains multiple edges, thus be careful when adding new metrics.
            let cfg = &cfg.edges.metrics;
            let mut metrics = vec![None; cfg.count()];
            for metric_idx in (0..cfg.count()).map(MetricIdx) {
                let metric_type = cfg.category(metric_idx);
                let is_provided = cfg.is_provided(metric_idx);

                match metric_type {
                    MetricCategory::Length | MetricCategory::Duration | MetricCategory::Custom => {
                        if is_provided {
                            return Err(format!(
                                "The {} of an edge in a pbf-file has to be calculated, \
                                 but is expected to be provided.",
                                metric_type
                            ));
                        }
                    }
                    MetricCategory::Maxspeed => {
                        if is_provided {
                            let maxspeed = highway_tag.parse_maxspeed(&way);
                            metrics[*metric_idx] = Some(maxspeed as u32);
                        } else {
                            return Err(format!(
                                "The {} of an edge in a pbf-file has to be provided, \
                                 but is expected to be calculated.",
                                metric_type
                            ));
                        }
                    }
                    MetricCategory::LaneCount => {
                        if is_provided {
                            let lane_count = highway_tag.parse_lane_count(&way);
                            metrics[*metric_idx] = Some(lane_count as u32);
                        } else {
                            return Err(format!(
                                "The {} of an edge in a pbf-file has to be provided, \
                                 but is expected to be calculated.",
                                metric_type
                            ));
                        }
                    }
                    MetricCategory::Id | MetricCategory::Ignore => (),
                }
            }

            // for n nodes in a way, you can create (n-1) edges
            for (node_idx, values) in vec![metrics; nodes.len() - 1].into_iter().enumerate() {
                // add proto-edge to graph
                graph_builder.push_edge(ProtoEdge {
                    src_id: nodes[node_idx],
                    dst_id: nodes[node_idx + 1],
                    metrics: values,
                });
            }
        }
        info!("FINISHED");
        Ok(())
    }

    fn parse_nodes(
        &self,
        cfg: &graph::Config,
        graph_builder: &mut GraphBuilder,
    ) -> Result<(), String> {
        info!("START Create nodes from input-file.");
        let file = helpers::open_file(cfg.map_file())?;
        for node in pbf::Reader::new(file)
            .iter()
            .filter_map(Result::ok)
            .filter_map(|obj| match obj {
                pbf::OsmObj::Node(node) => Some(node),
                _ => None,
            })
        {
            // add node to graph if it's part of an edge
            if graph_builder.is_node_in_edge(node.id.0) {
                graph_builder.push_node(
                    node.id.0,
                    Coordinate::from((node.decimicro_lat, node.decimicro_lon)),
                );
            }
        }
        info!("FINISHED");
        Ok(())
    }
}
