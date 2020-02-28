use crate::{
    configs::graph,
    network::{GraphBuilder, ProtoEdge},
};
use log::info;
use std::{fs::File, io::BufRead, ops::Range};

pub struct Parser {
    node_lines: Range<usize>,
    edge_lines: Range<usize>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            node_lines: 1..0,
            edge_lines: 1..0,
        }
    }

    fn is_line_functional(line: &String) -> bool {
        line != "" && line.chars().next() != Some('#')
    }
}

impl super::Parsing for Parser {
    /// Remembers range of edge-lines and node-lines
    fn preprocess(&mut self, file: File) -> Result<(), String> {
        info!("START Start preprocessing for fmi-parser.");
        // only functional-lines are counted
        let mut line_number = 0;
        let mut is_taking_counts = false;
        // counts are only metric-count, node-count, edge-count (in this order)
        let mut counts = vec![];
        for line in intern::Reader::new(file)
            .lines()
            .map(Result::unwrap)
            .filter(Self::is_line_functional)
        {
            // If there is a count, remember it.
            // The first occuring count let `is_taking_counts` getting true.
            // If all counts have been processed, `is_taking_counts` would change to false,
            // where the loop should stop and remember the line-number.
            let params: Vec<&str> = line.split_whitespace().collect();
            if params.len() == 1 {
                is_taking_counts = true;

                if let Ok(count) = params[0].parse::<usize>() {
                    counts.push(count);
                }
            } else if is_taking_counts {
                break;
            }

            line_number += 1;
        }

        // add counts
        if counts.len() < 2 {
            return Err(format!(
                "The provided fmi-map-file doesn't have enough (edge-, node-) counts."
            ));
        }

        // Current state: Last line-number is first node-line.
        // Further, the last two counts are the node- and edge-counts.
        let edge_count = counts.pop().unwrap();
        let node_count = counts.pop().unwrap();

        // nodes
        let start = line_number;
        let end = start + node_count;
        self.node_lines = start..end;

        // edges
        let start = end;
        let end = start + edge_count;
        self.edge_lines = start..end;

        info!("FINISHED");

        Ok(())
    }

    fn parse_ways(
        &self,
        file: File,
        graph_builder: &mut GraphBuilder,
        cfg: &graph::Config,
    ) -> Result<(), String> {
        info!("START Create edges from input-file.");
        let mut line_number = 0;
        for line in intern::Reader::new(file)
            .lines()
            .map(Result::unwrap)
            .filter(Self::is_line_functional)
        {
            // check if line contains edge
            if !self.edge_lines.contains(&line_number) {
                line_number += 1;
                continue;
            }
            line_number += 1;

            // create edge and add it
            let proto_edge = ProtoEdge::from_str(&line, &cfg.edges)?;
            graph_builder.push_edge(proto_edge);
        }
        info!("FINISHED");

        Ok(())
    }

    fn parse_nodes(
        &self,
        file: File,
        graph_builder: &mut GraphBuilder,
        _cfg: &graph::Config,
    ) -> Result<(), String> {
        info!("START Create nodes from input-file.");
        let mut line_number = 0;
        for line in intern::Reader::new(file)
            .lines()
            .map(Result::unwrap)
            .filter(Self::is_line_functional)
        {
            // check if line contains edge
            if !self.node_lines.contains(&line_number) {
                line_number += 1;
                continue;
            }
            line_number += 1;

            // create node and add it
            let proto_node = line.parse::<intern::ProtoNode>()?;
            graph_builder.push_node(proto_node.id, proto_node.coord);
        }
        info!("FINISHED");

        Ok(())
    }
}

mod intern {
    use crate::{
        configs::{graph::edges::Config, MetricCategory},
        network::{MetricIdx, ProtoEdge},
        units::geo,
    };
    pub use std::{io::BufReader as Reader, str};

    pub struct ProtoNode {
        pub id: i64,
        pub coord: geo::Coordinate,
    }

    impl str::FromStr for ProtoNode {
        type Err = String;

        fn from_str(line: &str) -> Result<Self, Self::Err> {
            let params: Vec<&str> = line.split_whitespace().collect();

            let n = 4;
            if params.len() < n {
                return Err(format!(
                    "Not enough params for a node ({}, but should be {}).",
                    params.len(),
                    n
                ));
            }

            let id = match params[0].parse::<i64>() {
                Ok(id) => id,
                Err(_) => {
                    return Err(format!(
                        "Parsing id '{:?}' from fmi-file, which is not i64.",
                        params[0]
                    ))
                }
            };
            let lat = match params[2].parse::<f64>() {
                Ok(lat) => lat,
                Err(_) => {
                    return Err(format!(
                        "Parsing lat '{:?}' from fmi-file, which is not f64.",
                        params[2]
                    ))
                }
            };
            let lon = match params[3].parse::<f64>() {
                Ok(lon) => lon,
                Err(_) => {
                    return Err(format!(
                        "Parsing lon '{:?}' from fmi-file, which is not f64.",
                        params[3]
                    ))
                }
            };

            Ok(ProtoNode {
                id,
                coord: geo::Coordinate::from((lat, lon)),
            })
        }
    }

    //--------------------------------------------------------------------------------------------//

    impl ProtoEdge {
        /// Parse a line of metrics into an edge.
        ///
        /// - When NodeIds are parsed, the first one is interpreted as src-id and the second one as dst-id.
        /// - If a metric is given as float, it tries to parse it into `u32` of accuracy in millis.
        ///   Example: given metric of `3.1415 km` becomes `3141 m`.
        ///   The maximum possible value would be around `10^7 m`
        ///   (`2^32 km` ~ `10^10 km`, which is divided by `1_000` for accuracy in `m`).
        pub fn from_str(line: &str, cfg: &Config) -> Result<ProtoEdge, String> {
            let mut metric_values = Vec::with_capacity(cfg.metrics.count());
            let mut src_id = None;
            let mut dst_id = None;

            // Loop over metric-types and parse params accordingly.
            let params: Vec<&str> = line.split_whitespace().collect();

            // metric-idx has to be counted separatedly.
            for param_idx in 0..cfg.metrics.all_categories().len() {
                let metric_type = cfg.metrics.all_categories()[param_idx];

                let param = *params.get(param_idx).ok_or(
                    "The fmi-map-file is expected to have more edge-params \
                     than actually has.",
                )?;

                match metric_type {
                    MetricCategory::Id => {
                        // get src-id and dst-id to create unfinished-edge afterwards
                        if src_id.is_none() {
                            src_id = Some(param.parse::<i64>().ok().ok_or(format!(
                                "Parsing {} (for edge-src) '{:?}' from fmi-file, which is not i64.",
                                metric_type, param
                            ))?);
                        } else if dst_id.is_none() {
                            dst_id = Some(param.parse::<i64>().ok().ok_or(format!(
                                "Parsing {} (for edge-dst) '{:?}' from fmi-file, which is not i64.",
                                metric_type, param
                            ))?);
                        } else {
                            return Err(format!(
                                "Both src-id and dst-id are already set, \
                                 but another {} should be parsed.",
                                metric_type
                            ));
                        }
                    }
                    MetricCategory::Length
                    | MetricCategory::Maxspeed
                    | MetricCategory::Duration
                    | MetricCategory::LaneCount
                    | MetricCategory::Custom => {
                        let metric_idx = MetricIdx(metric_values.len());
                        let is_provided = cfg.metrics.is_provided(metric_idx);

                        if is_provided {
                            // Try parsing as u32.
                            // If value is float, parsing keeps accuracy of millis.
                            // Example: given as 3.1415 km becomes 3141 m
                            // Maximum possible value would be around 10^7 m
                            // (2^32 km ~ 10^10 km, which is divided by 1_000 for accuracy in m)
                            let value = {
                                if let Ok(value_u32) = param.parse::<u32>() {
                                    value_u32
                                } else if let Ok(value_f64) = param.parse::<f64>() {
                                    (value_f64 * 1e3) as u32
                                } else {
                                    return Err(format!(
                                        "Parsing {} '{}' of edge-param #{} didn't work.",
                                        metric_type, param, param_idx
                                    ));
                                }
                            };
                            metric_values.push(Some(value.into()));
                        } else {
                            metric_values.push(None);
                        }
                    }
                    MetricCategory::Ignore => (),
                }
            }

            debug_assert_eq!(
                cfg.metrics.count(),
                metric_values.len(),
                "Metric-vec of proto-edge has {} elements, but should have {}.",
                metric_values.len(),
                cfg.metrics.count()
            );
            Ok(ProtoEdge {
                src_id: src_id.ok_or("Proto-edge should have a src-id, but doesn't.".to_owned())?,
                dst_id: dst_id.ok_or("Proto-edge should have a dst-id, but doesn't.".to_owned())?,
                metrics: metric_values,
            })
        }
    }
}
