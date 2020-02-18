use log::{error, info};
use osmgraphing::{
    configs::{edges, graph, paths, MetricType},
    network::NodeIdx,
    Parser,
};
use std::{path::PathBuf, time::Instant};

//------------------------------------------------------------------------------------------------//

fn init_logging(quietly: bool) {
    let mut builder = env_logger::Builder::new();
    // minimum filter-level: `warn`
    builder.filter(None, log::LevelFilter::Warn);
    // if quiet logging: doesn't log `info` for the server and this repo
    if !quietly {
        builder.filter(Some(env!("CARGO_PKG_NAME")), log::LevelFilter::Info);
        builder.filter(Some("parser"), log::LevelFilter::Info);
    }
    // overwrite default with environment-variables
    if let Ok(filters) = std::env::var("RUST_LOG") {
        builder.parse_filters(&filters);
    }
    if let Ok(write_style) = std::env::var("RUST_LOG_STYLE") {
        builder.parse_write_style(&write_style);
    }
    // init
    builder.init();
}

fn main() {
    init_logging(false);
    info!("Executing example: parser");

    let cfg = graph::Config {
        is_graph_suitable: false,
        paths: paths::Config {
            map_file: match std::env::args_os().nth(1) {
                Some(path) => PathBuf::from(path),
                None => PathBuf::from("resources/maps/isle-of-man_2019-09-05.osm.pbf"),
            },
        },
        edges: edges::Config {
            metric_ids: vec![
                String::from("src-id"),
                String::from("dst-id"),
                String::from("length"),
                String::from("maxspeed"),
            ],
            metric_types: vec![
                MetricType::Id,
                MetricType::Id,
                MetricType::Length { provided: false },
                MetricType::Maxspeed { provided: true },
            ],
        },
        ..Default::default()
    };

    let now = Instant::now();
    let graph = match Parser::parse_and_finalize(&cfg) {
        Ok(graph) => graph,
        Err(msg) => {
            error!("{}", msg);
            return;
        }
    };
    info!(
        "Finished parsing in {} seconds ({} µs).",
        now.elapsed().as_secs(),
        now.elapsed().as_micros(),
    );
    info!("");
    info!("{}", graph);
}
