//------------------------------------------------------------------------------------------------//
// other modules

use std::collections::HashMap;
use std::io::Write;
use std::time::SystemTime;
use std::{fs, io, path};

use log::{info, warn};
use osmgraphing::network::Graph;
use osmgraphing::{routing, Parser};
use serde::{Deserialize, Serialize};

//------------------------------------------------------------------------------------------------//
// own modules

pub mod routes;

//------------------------------------------------------------------------------------------------//
// config

pub struct Config<'a, P: AsRef<path::Path> + ?Sized> {
    pub map_file_path: &'a P,
    pub out_dir_path: &'a P,
}

//------------------------------------------------------------------------------------------------//
// simulation

#[derive(Serialize, Deserialize, Clone)]
struct EdgeInfo {
    src_id: i64,
    dst_id: i64,
    decimicro_lat: i32,
    decimicro_lon: i32,
    is_src: bool,
    is_dst: bool,
    lane_count: u8,
    length_m: u32,
    route_count: u16,
}

pub fn run<P: AsRef<path::Path> + ?Sized>(cfg: Config<P>) -> Result<(), String> {
    info!("Executing braess-optimization");

    //--------------------------------------------------------------------------------------------//
    // prepare simulation

    // check path of io-files before expensive simulation
    let out_dir_path = check_and_prepare_out_dir_path(cfg.out_dir_path)?;
    let out_file_path = out_dir_path.join("results.json");
    create_out_file(&out_file_path)?;
    let proto_routes = read_in_proto_routes();

    let graph = Parser::parse_and_finalize(&cfg.map_file_path)?;

    //--------------------------------------------------------------------------------------------//
    // prepare statistics

    let mut data: Vec<Option<EdgeInfo>> = vec![None; graph.edge_count()];

    //--------------------------------------------------------------------------------------------//
    // routing and statistics-update

    let mut astar = routing::factory::new_fastest_path_astar();

    // routes
    for (src_idx, dst_idx) in proto_routes {
        let src = graph.node(src_idx);
        let dst = graph.node(dst_idx);

        // compute best path
        let option_path = astar.compute_best_path(src, dst, &graph);
        if let Some(path) = option_path {
            info!("Duration {} ms from ({}) to ({}).", path.cost(), src, dst);

            update_edge_info(&mut data, &path, &graph);
        } else {
            warn!("No path from ({}) to ({}).", src, dst);
        }
    }

    //--------------------------------------------------------------------------------------------//
    // export statistics

    export_statistics(data, &out_file_path)?;

    Ok(())
}

//------------------------------------------------------------------------------------------------//
// helpers: io

/// Returns output-path, which is "{out_dir_path}/{%Y-%m-%d}/{%H:%M:%S}"
fn check_and_prepare_out_dir_path<P: AsRef<path::Path> + ?Sized>(
    out_dir_path: &P,
) -> Result<path::PathBuf, String> {
    // get and format current time
    let now = SystemTime::now();
    let now: chrono::DateTime<chrono::Utc> = now.into();
    let now_ymd = format!("{}", now.format("%Y-%m-%d"));
    let now_hms = format!("{}", now.format("%T")); // %T == %H:%M:%S
    drop(now);

    // check if necessary directories do already exist
    let out_dir_path = out_dir_path.as_ref();
    if !out_dir_path.exists() {
        return Err(format!("Path {} does not exist.", out_dir_path.display()));
    }
    let out_dir_path = out_dir_path.join(now_ymd).join(now_hms);
    match fs::create_dir_all(&out_dir_path) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!(
                "Problem with path {}: {}",
                out_dir_path.display(),
                e
            ))
        }
    };

    Ok(out_dir_path)
}

fn create_out_file<P: AsRef<path::Path> + ?Sized>(out_file_path: &P) -> Result<(), String> {
    let out_file_path = out_file_path.as_ref();
    if out_file_path.exists() {
        return Err(format!(
            "File {} does already exist. Please (re)move it.",
            out_file_path.display()
        ));
    } else {
        match fs::File::create(out_file_path) {
            Ok(file) => file,
            Err(_) => return Err(format!("Could not open file {}", out_file_path.display())),
        }
    };

    Ok(())
}

fn read_in_proto_routes() -> Vec<(usize, usize)> {
    // TODO
    vec![(0, 5), (0, 3), (2, 4)]
}

fn export_statistics<P: AsRef<path::Path> + ?Sized>(
    mut data: Vec<Option<EdgeInfo>>,
    out_file_path: &P,
) -> Result<(), String> {
    // file should have been created
    let out_file = {
        let out_file_path = out_file_path.as_ref();
        match fs::File::create(out_file_path) {
            Ok(file) => file,
            Err(_) => return Err(format!("Could not open file {}", out_file_path.display())),
        }
    };

    // remove None's from data
    data.retain(|ei| ei.is_some());

    // write data to json-file
    let mut writer = io::BufWriter::new(out_file);
    let mut json_data = HashMap::new();
    json_data.insert("edges", &data);
    match serde_json::to_string_pretty(&json_data) {
        Ok(json_data) => {
            match &mut writer.write(json_data.as_bytes()) {
                Ok(_) => (),
                Err(e) => return Err(format!("Could not write data to file Errmsg: {}", e)),
            };
        }
        Err(e) => return Err(format!("Could not convert data to json. Errmsg: {}", e)),
    }

    Ok(())
}

//------------------------------------------------------------------------------------------------//
// helpers: routing

fn update_edge_info(data: &mut Vec<Option<EdgeInfo>>, path: &routing::astar::Path, graph: &Graph) {
    // reconstruct path to update edge-statistics
    let mut current_idx = path.src_idx();
    while let Some(edge_dst_idx) = path.succ_node_idx(current_idx) {
        // get edge from its nodes
        let (edge, edge_idx) = graph
            .edge_from(current_idx, edge_dst_idx)
            .expect("Path should only use edges from the graph.");
        debug_assert_eq!(edge.src_idx(), current_idx, "edge.src_idx() != current_idx");
        debug_assert_eq!(
            edge.dst_idx(),
            edge_dst_idx,
            "edge.dst_idx() != edge_dst_idx"
        );

        // create EdgeInfo if not existing
        {
            let (edge_src, edge_dst) = (graph.node(edge.src_idx()), graph.node(edge.dst_idx()));

            if data[edge_idx].is_none() {
                data[edge_idx] = Some(EdgeInfo {
                    src_id: edge_src.id(),
                    dst_id: edge_dst.id(),
                    decimicro_lat: {
                        (edge_src.coord().decimicro_lat() + edge_dst.coord().decimicro_lat()) / 2
                    },
                    decimicro_lon: {
                        (edge_src.coord().decimicro_lon() + edge_dst.coord().decimicro_lon()) / 2
                    },
                    is_src: false,
                    is_dst: false,
                    lane_count: edge.lane_count(),
                    length_m: edge.meters(),
                    route_count: 0,
                });
            }
        }

        // update edge-info of path-edges
        {
            let edge_info = data[edge_idx]
                // Option<EdgeInfo> -> already set above
                .as_mut()
                .expect("EdgeInfo should have been set a few lines above.");
            // update path-edges' usages
            edge_info.route_count += 1;
            // update if edge is src/dst
            if edge.src_idx() == path.src_idx() {
                edge_info.is_src = true;
            }
            if edge.dst_idx() == path.dst_idx() {
                edge_info.is_dst = true;
            }
        }

        current_idx = edge_dst_idx;
    }
}
