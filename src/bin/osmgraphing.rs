use log::info;
use osmgraphing::{configs, helpers, io, routing};
use std::{path::PathBuf, time::Instant};

//------------------------------------------------------------------------------------------------//
// points in Germany

// somewhere in Stuttgart (Schwabstrasse)
// id 20_443_604 osm-id 2_933_335_353 lat 48.77017570000000291 lon 9.15657690000000102

// "near" Esslingen
// id:4_647 osm-id:163_354 lat:48.66743380000000485 lon:9.24459110000000095

// somewhere in Ulm
// id 9_058_109 osm-id 580_012_224 lat 48.39352330000000535 lon 9.9816315000000006

// near Aalen
// id 54_288 osm-id 2_237_652 lat 48.88542720000000230 lon 10.13642900000000147

// somewhere in Berlin
// id 296_679 osm-id 26_765_334 lat 52.50536590000000103 lon 13.38662390000000002

//------------------------------------------------------------------------------------------------//

fn main() -> Result<(), String> {
    // process user-input

    let matches = parse_cmdline();
    match helpers::init_logging(matches.value_of("log").unwrap(), vec![]) {
        Ok(_) => (),
        Err(msg) => return Err(format!("{}", msg)),
    };

    info!("EXECUTE {}", env!("CARGO_PKG_NAME"));

    // parse graph

    let graph = {
        // get config by provided user-input

        let parsing_cfg = {
            let raw_parsing_cfg = PathBuf::from(matches.value_of("config").unwrap());
            match configs::parsing::Config::try_from_yaml(&raw_parsing_cfg) {
                Ok(cfg) => cfg,
                Err(msg) => return Err(format!("{}", msg)),
            }
        };

        // parse and create graph

        // measure parsing-time
        let now = Instant::now();

        let graph = match io::network::Parser::parse_and_finalize(parsing_cfg) {
            Ok(graph) => graph,
            Err(msg) => return Err(format!("{}", msg)),
        };
        info!(
            "Finished parsing in {} seconds ({} µs).",
            now.elapsed().as_secs(),
            now.elapsed().as_micros(),
        );
        info!("");
        info!("{}", graph);
        info!("");

        graph
    };

    // writing built graph

    if matches.is_present("is-writing-graph") {
        // get config by provided user-input

        let writing_cfg = {
            // take parsing-cfg if no other config is given

            let raw_cfg = match matches.value_of("writing-graph-cfg") {
                Some(path) => PathBuf::from(&path),
                None => PathBuf::from(&matches.value_of("config").unwrap()),
            };

            // parse config

            match configs::writing::network::Config::try_from_yaml(&raw_cfg) {
                Ok(cfg) => cfg,
                Err(msg) => return Err(format!("{}", msg)),
            }
        };

        // check if new file does already exist

        if writing_cfg.map_file.exists() {
            return Err(format!(
                "New map-file {} does already exist. Please remove it.",
                writing_cfg.map_file.display()
            ));
        }

        // writing to file

        // measure writing-time
        let now = Instant::now();

        match io::network::Writer::write(&graph, &writing_cfg) {
            Ok(()) => (),
            Err(msg) => return Err(format!("{}", msg)),
        };
        info!(
            "Finished writing in {} seconds ({} µs).",
            now.elapsed().as_secs(),
            now.elapsed().as_micros(),
        );
        info!("");
    }

    // writing routes to file

    if matches.is_present("is-writing-routes") {
        // get config by provided user-input

        let writing_cfg = {
            // take parsing-cfg if no other config is given

            let raw_cfg = match matches.value_of("writing-routes-cfg") {
                Some(path) => PathBuf::from(&path),
                None => PathBuf::from(&matches.value_of("config").unwrap()),
            };

            // parse config

            match configs::writing::routing::Config::try_from_yaml(&raw_cfg) {
                Ok(cfg) => cfg,
                Err(msg) => return Err(format!("{}", msg)),
            }
        };

        // check if new file does already exist

        if writing_cfg.file.exists() {
            return Err(format!(
                "New routes-file {} does already exist. Please remove it.",
                writing_cfg.file.display()
            ));
        }

        // writing to file

        // measure writing-time
        let now = Instant::now();

        match io::routing::Writer::write(&graph, &writing_cfg) {
            Ok(()) => (),
            Err(msg) => return Err(format!("{}", msg)),
        };
        info!(
            "Finished writing in {} seconds ({} µs).",
            now.elapsed().as_secs(),
            now.elapsed().as_micros(),
        );
        info!("");
    }

    // routing-example

    if matches.is_present("is-routing") {
        // get config by provided user-input

        let routing_cfg = {
            // take parsing-cfg if no other config is given

            let raw_cfg = match matches.value_of("routing-cfg") {
                Some(path) => PathBuf::from(&path),
                None => PathBuf::from(&matches.value_of("config").unwrap()),
            };

            // parse config

            match configs::routing::Config::try_from_yaml(&raw_cfg, graph.cfg()) {
                Ok(cfg) => cfg,
                Err(msg) => return Err(format!("{}", msg)),
            }
        };

        info!("EXECUTE Do routing with alphas: {:?}", routing_cfg.alphas);

        let nodes = graph.nodes();
        let mut dijkstra = routing::Dijkstra::new();

        // calculate best paths

        for (src, dst) in io::routing::Parser::parse_and_finalize(&routing_cfg, &graph)?
            .iter()
            .map(|&(src_idx, dst_idx, _)| (nodes.create(src_idx), nodes.create(dst_idx)))
        {
            info!("");

            let now = Instant::now();
            let best_path = dijkstra.compute_best_path(src.idx(), dst.idx(), &graph, &routing_cfg);
            info!(
                "Ran Dijkstra-query in {} ms",
                now.elapsed().as_micros() as f64 / 1_000.0,
            );
            if let Some(best_path) = best_path {
                let best_path = best_path.flatten(&graph);
                info!("Found path {}.", best_path);
            } else {
                info!("No path from ({}) to ({}).", src, dst);
            }
        }
    }
    Ok(())
}

fn parse_cmdline<'a>() -> clap::ArgMatches<'a> {
    let tmp = &[
        "Sets the logging-level by setting environment-variable 'RUST_LOG'.",
        "The env-variable 'RUST_LOG' has precedence.",
        "It takes values of modules, e.g.",
        "export RUST_LOG='warn,osmgraphing=info'",
        "for getting warn's by default, but 'info' about the others",
    ]
    .join("\n");
    let arg_log_level = clap::Arg::with_name("log")
        .long("log")
        .short("l")
        .value_name("FILTER-LEVEL")
        .help(tmp)
        .takes_value(true)
        .required(false)
        .default_value("INFO")
        .possible_values(&vec!["TRACE", "DEBUG", "INFO", "WARN", "ERROR"]);

    let arg_parser_cfg = clap::Arg::with_name("config")
        .long("config")
        .alias("parsing")
        .value_name("PATH")
        .help("Sets the parser and other configurations according to this config.")
        .takes_value(true);

    let arg_is_routing = clap::Arg::with_name("is-routing")
        .long("routing")
        .help("Does routing as specified in the provided config.")
        .takes_value(false)
        .requires("config");

    let arg_is_writing_graph = clap::Arg::with_name("is-writing-graph")
        .long("writing-graph")
        .help(
            "The generated graph will be exported \
               as described in the provided config.",
        )
        .takes_value(false)
        .requires("config");

    let arg_is_writing_routes = clap::Arg::with_name("is-writing-routes")
        .long("writing-routes")
        .help(
            "The generated graph will be used to \
               generate and export valid routes \
               as described in the provided config.",
        )
        .takes_value(false)
        .requires("config");

    clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .long_about(
            (&[
                "",
                "This tool takes a config-file, parses the chosen graph with specified",
                "settings, and can execute specified tasks.",
                "Such tasks may be exporting the graph as fmi-map-file or doing some ",
                "routing-queries (if provided in config-file).",
            ]
            .join("\n"))
                .as_ref(),
        )
        .arg(arg_log_level)
        .arg(arg_parser_cfg)
        .arg(arg_is_routing)
        .arg(arg_is_writing_graph)
        .arg(arg_is_writing_routes)
        .get_matches()
}
