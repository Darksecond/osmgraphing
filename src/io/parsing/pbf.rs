use crate::{
    configs::{parser, EdgeCategory, NodeCategory},
    defaults::DimVec,
    helpers,
    network::{GraphBuilder, MetricIdx, ProtoEdge, ProtoNode, StreetCategory},
    units::geo::Coordinate,
};
use log::info;
use osmpbfreader::{reader::OsmPbfReader, OsmObj};
use smallvec::smallvec;

pub struct Parser;

impl Parser {
    pub fn new() -> Parser {
        Parser {}
    }
}

impl super::Parsing for Parser {
    fn preprocess(&mut self, cfg: &parser::Config) -> Result<(), String> {
        // check if yaml-config is correct
        if !cfg.nodes.categories().contains(&NodeCategory::NodeId) {
            Err(String::from(
                "The provided config-file doesn't contain a NodeId, but needs to.",
            ))
        } else if !cfg.nodes.categories().contains(&NodeCategory::Latitude) {
            Err(String::from(
                "The provided config-file doesn't contain a latitude, but needs to.",
            ))
        } else if !cfg.nodes.categories().contains(&NodeCategory::Longitude) {
            Err(String::from(
                "The provided config-file doesn't contain a longitude, but needs to.",
            ))
        } else {
            Ok(())
        }
    }

    fn parse_ways(
        &self,
        cfg: &parser::Config,
        graph_builder: &mut GraphBuilder,
    ) -> Result<(), String> {
        info!("START Create edges from input-file.");
        let file = helpers::open_file(&cfg.map_file)?;
        for mut way in OsmPbfReader::new(file)
            .par_iter()
            .filter_map(Result::ok)
            .filter_map(|obj| match obj {
                OsmObj::Way(way) => Some(way),
                _ => None,
            })
        {
            if way.nodes.len() < 2 {
                continue;
            }

            // collect relevant data from file, if way-type is as expected by user
            let highway_tag = match StreetCategory::from(&way) {
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
            let cfg = &cfg.edges;
            let mut metrics: DimVec<_> = smallvec![None; cfg.dim()];
            for metric_idx in (0..cfg.dim()).map(MetricIdx) {
                let category = cfg.metric_category(metric_idx);
                let is_provided = cfg.is_metric_provided(metric_idx);

                match category {
                    EdgeCategory::Meters | EdgeCategory::Seconds | EdgeCategory::Custom => {
                        if is_provided {
                            return Err(format!(
                                "The {} of an edge in a pbf-file has to be calculated, \
                                 but is expected to be provided.",
                                category
                            ));
                        }
                    }
                    EdgeCategory::KilometersPerHour => {
                        if is_provided {
                            let maxspeed = highway_tag.parse_maxspeed(&way);
                            metrics[*metric_idx] = Some(maxspeed as f32);
                        } else {
                            return Err(format!(
                                "The {} of an edge in a pbf-file has to be provided, \
                                 but is expected to be calculated.",
                                category
                            ));
                        }
                    }
                    EdgeCategory::LaneCount => {
                        if is_provided {
                            let lane_count = highway_tag.parse_lane_count(&way);
                            metrics[*metric_idx] = Some(lane_count as f32);
                        } else {
                            return Err(format!(
                                "The {} of an edge in a pbf-file has to be provided, \
                                 but is expected to be calculated.",
                                category
                            ));
                        }
                    }
                    EdgeCategory::SrcId | EdgeCategory::DstId | EdgeCategory::Ignore => (),
                }
            }

            // for n nodes in a way, you can create (n-1) edges
            for node_idx in 0..(nodes.len() - 1) {
                // add proto-edge to graph
                graph_builder.push_edge(ProtoEdge {
                    src_id: nodes[node_idx],
                    dst_id: nodes[node_idx + 1],
                    metrics: metrics.clone(),
                });
            }
        }
        info!("FINISHED");
        Ok(())
    }

    fn parse_nodes(
        &self,
        cfg: &parser::Config,
        graph_builder: &mut GraphBuilder,
    ) -> Result<(), String> {
        info!("START Create nodes from input-file.");
        let file = helpers::open_file(&cfg.map_file)?;
        for node in OsmPbfReader::new(file)
            .par_iter()
            .filter_map(Result::ok)
            .filter_map(|obj| match obj {
                OsmObj::Node(node) => Some(node),
                _ => None,
            })
        {
            // add node to graph if it's part of an edge
            if graph_builder.is_node_in_edge(node.id.0) {
                graph_builder.push_node(ProtoNode::new(
                    node.id.0,
                    Some(Coordinate::from_decimicro(
                        node.decimicro_lat,
                        node.decimicro_lon,
                    )),
                    None,
                ));
            }
        }
        info!("FINISHED");
        Ok(())
    }
}
