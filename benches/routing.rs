use criterion::{black_box, criterion_group, criterion_main, Criterion};
use log::error;
use osmgraphing::{
    configs::{
        graph,
        graph::{edges, vehicles},
        Config, MetricType, VehicleType,
    },
    network::{Graph, NodeIdx},
    routing, Parser,
};
use std::path::PathBuf;

fn init_logging(quietly: bool) {
    let mut builder = env_logger::Builder::new();
    // minimum filter-level: `warn`
    builder.filter(None, log::LevelFilter::Warn);
    // if quiet logging: doesn't log `info` for the server and this repo
    if !quietly {
        builder.filter(Some(env!("CARGO_PKG_NAME")), log::LevelFilter::Info);
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

fn criterion_benchmark(c: &mut Criterion) {
    init_logging(true);

    // parsing
    let cfg = Config::new(graph::Config {
        map_file: PathBuf::from("resources/maps/isle-of-man_2019-09-05.osm.pbf"),
        vehicles: vehicles::Config {
            is_driver_picky: false,
            vehicle_type: VehicleType::Car,
        },
        edges: edges::Config {
            metric_types: vec![
                MetricType::Id {
                    id: "src-id".to_owned(),
                },
                MetricType::Id {
                    id: "dst-id".to_owned(),
                },
                MetricType::Length { provided: false },
                MetricType::Maxspeed { provided: true },
                MetricType::Duration { provided: false },
            ],
        },
    });
    let graph = match Parser::parse_and_finalize(cfg.graph) {
        Ok(graph) => graph,
        Err(msg) => {
            error!("{}", msg);
            return;
        }
    };
    let nodes = graph.nodes();

    // routing
    let labelled_routes = vec![
        // short route (~3 km)
        (
            "",
            " with short routes (~3 km)",
            vec![(
                nodes.idx_from(283500532).expect("A"),
                nodes.idx_from(283501263).expect("B"),
            )],
        ),
        // medium route (~30 km)
        (
            "",
            " with medium routes (~30 km)",
            vec![(
                nodes.idx_from(283483998).expect("C"),
                nodes.idx_from(1746745421).expect("D"),
            )],
        ),
        // long route (~56 km)
        (
            "",
            " with long routes (~56 km)",
            vec![(
                nodes.idx_from(1151603193).expect("E"),
                nodes.idx_from(456478793).expect("F"),
            )],
        ),
    ];

    // benchmarking shortest routing
    for (prefix, suffix, routes) in labelled_routes.iter() {
        c.bench_function(
            &format!("{}Shortest Dijkstra (unidir){}", prefix, suffix),
            |b| b.iter(|| unidir_shortest_dijkstra(black_box(&graph), black_box(&routes))),
        );
        c.bench_function(
            &format!("{}Shortest Dijkstra (bidir){}", prefix, suffix),
            |b| b.iter(|| bidir_shortest_dijkstra(black_box(&graph), black_box(&routes))),
        );
        c.bench_function(
            &format!("{}Shortest Astar (unidir){}", prefix, suffix),
            |b| b.iter(|| unidir_shortest_astar(black_box(&graph), black_box(&routes))),
        );
        c.bench_function(
            &format!("{}Shortest Astar (bidir){}", prefix, suffix),
            |b| b.iter(|| bidir_shortest_astar(black_box(&graph), black_box(&routes))),
        );
    }

    // benchmarking fastest routing
    for (prefix, suffix, routes) in labelled_routes.iter() {
        c.bench_function(
            &format!("{}Fastest Dijkstra (unidir){}", prefix, suffix),
            |b| b.iter(|| unidir_fastest_dijkstra(black_box(&graph), black_box(&routes))),
        );
        c.bench_function(
            &format!("{}Fastest Dijkstra (bidir){}", prefix, suffix),
            |b| b.iter(|| bidir_fastest_dijkstra(black_box(&graph), black_box(&routes))),
        );
        c.bench_function(
            &format!("{}Fastest Astar (unidir){}", prefix, suffix),
            |b| b.iter(|| unidir_fastest_astar(black_box(&graph), black_box(&routes))),
        );
        c.bench_function(&format!("{}Fastest Astar (bidir){}", prefix, suffix), |b| {
            b.iter(|| bidir_fastest_astar(black_box(&graph), black_box(&routes)))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

//------------------------------------------------------------------------------------------------//

fn unidir_shortest_dijkstra(graph: &Graph, routes: &Vec<(NodeIdx, NodeIdx)>) {
    let mut dijkstra = routing::factory::dijkstra::unidirectional::shortest();

    let nodes = graph.nodes();
    for &(src_idx, dst_idx) in routes.iter() {
        let src = nodes.create(src_idx);
        let dst = nodes.create(dst_idx);
        let _option_path = dijkstra.compute_best_path(&src, &dst, graph);
    }
}

fn bidir_shortest_dijkstra(graph: &Graph, routes: &Vec<(NodeIdx, NodeIdx)>) {
    let mut dijkstra = routing::factory::dijkstra::bidirectional::shortest();

    let nodes = graph.nodes();
    for &(src_idx, dst_idx) in routes.iter() {
        let src = nodes.create(src_idx);
        let dst = nodes.create(dst_idx);
        let _option_path = dijkstra.compute_best_path(&src, &dst, graph);
    }
}

fn unidir_shortest_astar(graph: &Graph, routes: &Vec<(NodeIdx, NodeIdx)>) {
    let mut astar = routing::factory::astar::unidirectional::shortest();

    let nodes = graph.nodes();
    for &(src_idx, dst_idx) in routes.iter() {
        let src = nodes.create(src_idx);
        let dst = nodes.create(dst_idx);
        let _option_path = astar.compute_best_path(&src, &dst, graph);
    }
}

fn bidir_shortest_astar(graph: &Graph, routes: &Vec<(NodeIdx, NodeIdx)>) {
    let mut astar = routing::factory::astar::bidirectional::shortest();

    let nodes = graph.nodes();
    for &(src_idx, dst_idx) in routes.iter() {
        let src = nodes.create(src_idx);
        let dst = nodes.create(dst_idx);
        let _option_path = astar.compute_best_path(&src, &dst, graph);
    }
}

fn unidir_fastest_dijkstra(graph: &Graph, routes: &Vec<(NodeIdx, NodeIdx)>) {
    let mut dijkstra = routing::factory::dijkstra::unidirectional::fastest();

    let nodes = graph.nodes();
    for &(src_idx, dst_idx) in routes.iter() {
        let src = nodes.create(src_idx);
        let dst = nodes.create(dst_idx);
        let _option_path = dijkstra.compute_best_path(&src, &dst, graph);
    }
}

fn bidir_fastest_dijkstra(graph: &Graph, routes: &Vec<(NodeIdx, NodeIdx)>) {
    let mut dijkstra = routing::factory::dijkstra::bidirectional::fastest();

    let nodes = graph.nodes();
    for &(src_idx, dst_idx) in routes.iter() {
        let src = nodes.create(src_idx);
        let dst = nodes.create(dst_idx);
        let _option_path = dijkstra.compute_best_path(&src, &dst, graph);
    }
}

fn unidir_fastest_astar(graph: &Graph, routes: &Vec<(NodeIdx, NodeIdx)>) {
    let mut astar = routing::factory::astar::unidirectional::fastest();

    let nodes = graph.nodes();
    for &(src_idx, dst_idx) in routes.iter() {
        let src = nodes.create(src_idx);
        let dst = nodes.create(dst_idx);
        let _option_path = astar.compute_best_path(&src, &dst, graph);
    }
}

fn bidir_fastest_astar(graph: &Graph, routes: &Vec<(NodeIdx, NodeIdx)>) {
    let mut astar = routing::factory::astar::bidirectional::fastest();

    let nodes = graph.nodes();
    for &(src_idx, dst_idx) in routes.iter() {
        let src = nodes.create(src_idx);
        let dst = nodes.create(dst_idx);
        let _option_path = astar.compute_best_path(&src, &dst, graph);
    }
}
