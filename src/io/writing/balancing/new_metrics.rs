use crate::{configs, defaults, helpers::err, network::Graph};
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
};

pub struct Writer {}

impl Writer {
    pub fn new() -> Writer {
        Writer {}
    }

    pub fn write(
        &mut self,
        iter: usize,
        graph: &Graph,
        balancing_cfg: &configs::balancing::Config,
    ) -> err::Feedback {
        // prepare

        let fwd_edges = graph.fwd_edges();

        // get writers

        let mut writer = {
            let path = balancing_cfg
                .results_dir
                .join(format!("{}", iter))
                .join(defaults::balancing::stats::DIR)
                .join(defaults::balancing::stats::files::NEW_METRICS);
            let output_file = match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(f) => f,
                Err(e) => {
                    return Err(err::Msg::from(format!(
                        "Couldn't open {} due to error: {}",
                        path.display(),
                        e
                    )))
                }
            };
            BufWriter::new(output_file)
        };

        // write header

        writeln!(writer, "edge-id new_metric")?;

        // write data

        let workload_idx = graph
            .cfg()
            .edges
            .metrics
            .try_idx_of(&balancing_cfg.monitoring.workload_id)?;
        let metrics = graph.metrics();

        for edge_idx in fwd_edges
            .iter()
            .filter(|&edge_idx| !fwd_edges.is_shortcut(edge_idx))
        {
            let edge_id = fwd_edges.id(edge_idx);

            let mut new_metric = metrics[edge_idx][*workload_idx];
            if graph.cfg().edges.metrics.are_normalized {
                if let Some(mean) = metrics.mean(workload_idx) {
                    new_metric *= mean;
                }
            }
            writeln!(writer, "{} {}", edge_id, new_metric)?;
        }

        Ok(())
    }
}
