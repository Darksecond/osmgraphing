use crate::{configs::SimpleId, defaults, io::SupportingFileExts};
use std::{
    convert::TryFrom,
    fs::OpenOptions,
    path::{Path, PathBuf},
};
pub mod metrics;
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct Config {
    pub results_dir: PathBuf,
    pub multi_ch_constructor: MultiChConstructor,
    pub new_graph_dim: usize,
    pub num_iter: usize,
    pub iter_0_cfg: PathBuf,
    pub iter_i_cfg: PathBuf,
    pub workload_id: SimpleId,
    pub lane_count_id: SimpleId,
    pub distance_id: SimpleId,
    pub optimization: Optimization,
}

impl SupportingFileExts for Config {
    fn supported_exts<'a>() -> &'a [&'a str] {
        &["yaml"]
    }
}

impl Config {
    pub fn try_from_str(yaml_str: &str) -> Result<Config, String> {
        let proto_cfg = {
            match serde_yaml::from_str(yaml_str) {
                Ok(proto_cfg) => proto_cfg,
                Err(e) => return Err(format!("{}", e)),
            }
        };
        Config::try_from_proto(proto_cfg)
    }

    pub fn from_str(yaml_str: &str) -> Config {
        match Config::try_from_str(yaml_str) {
            Ok(cfg) => cfg,
            Err(msg) => panic!("{}", msg),
        }
    }

    fn try_from_proto(proto_cfg: ProtoConfig) -> Result<Config, String> {
        Ok(Config {
            results_dir: proto_cfg.results_dir,
            multi_ch_constructor: MultiChConstructor {
                dir: PathBuf::from(defaults::balancing::paths::multi_ch_constructor::DIR),
                contraction_ratio: String::from(defaults::balancing::CONTRACTION_RATIO),
            },
            new_graph_dim: proto_cfg.new_graph_dim,
            num_iter: proto_cfg.num_iter,
            iter_0_cfg: proto_cfg.iter_0_cfg,
            iter_i_cfg: proto_cfg.iter_i_cfg,
            workload_id: proto_cfg.workload_id,
            lane_count_id: proto_cfg.lane_count_id,
            distance_id: proto_cfg.distance_id,
            optimization: Optimization::from(proto_cfg.optimization),
        })
    }

    fn _from_proto(proto_cfg: ProtoConfig) -> Config {
        match Config::try_from_proto(proto_cfg) {
            Ok(cfg) => cfg,
            Err(msg) => panic!("{}", msg),
        }
    }

    pub fn try_from_yaml<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Config, String> {
        let path = path.as_ref();
        let file = {
            Config::find_supported_ext(path)?;
            OpenOptions::new()
                .read(true)
                .open(path)
                .expect(&format!("Couldn't open {}", path.display()))
        };

        let proto_cfg = match serde_yaml::from_reader(file) {
            Ok(proto_cfg) => proto_cfg,
            Err(e) => return Err(format!("{}", e)),
        };
        Config::try_from_proto(proto_cfg)
    }

    pub fn from_yaml<P: AsRef<Path> + ?Sized>(path: &P) -> Config {
        match Config::try_from_yaml(path) {
            Ok(cfg) => cfg,
            Err(msg) => panic!("{}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MultiChConstructor {
    pub dir: PathBuf,
    pub contraction_ratio: String,
}

#[derive(Debug, Clone)]
pub enum Optimization {
    ExplicitEuler { correction: f64 },
}

impl From<ProtoOptimization> for Optimization {
    fn from(proto_optimization: ProtoOptimization) -> Optimization {
        match proto_optimization {
            ProtoOptimization::ExplicitEuler { correction } => Optimization::ExplicitEuler {
                correction: correction,
            },
        }
    }
}

/// Don't deny unknown fields to allow multiple configs in one yaml-file.
#[derive(Debug, Deserialize)]
#[serde(try_from = "RawConfig")]
pub struct ProtoConfig {
    pub results_dir: PathBuf,
    pub new_graph_dim: usize,
    pub num_iter: usize,
    pub iter_0_cfg: PathBuf,
    pub iter_i_cfg: PathBuf,
    pub workload_id: SimpleId,
    pub lane_count_id: SimpleId,
    pub distance_id: SimpleId,
    pub optimization: ProtoOptimization,
}

impl TryFrom<RawConfig> for ProtoConfig {
    type Error = String;

    fn try_from(raw_cfg: RawConfig) -> Result<ProtoConfig, String> {
        Ok(ProtoConfig {
            results_dir: raw_cfg.balancing.results_dir,
            new_graph_dim: raw_cfg.balancing.new_graph_dim,
            num_iter: raw_cfg.balancing.number_of_iterations,
            iter_0_cfg: raw_cfg.balancing.iter_0_cfg,
            iter_i_cfg: raw_cfg.balancing.iter_i_cfg,
            workload_id: raw_cfg.balancing.metric_ids.workload,
            lane_count_id: raw_cfg.balancing.metric_ids.lane_count,
            distance_id: raw_cfg.balancing.metric_ids.distance,
            optimization: ProtoOptimization::from(raw_cfg.balancing.optimization),
        })
    }
}

#[derive(Debug)]
pub enum ProtoOptimization {
    ExplicitEuler { correction: f64 },
}

impl From<RawOptimization> for ProtoOptimization {
    fn from(raw_optimization: RawOptimization) -> ProtoOptimization {
        match raw_optimization {
            RawOptimization::ExplicitEuler { correction } => ProtoOptimization::ExplicitEuler {
                correction: correction,
            },
        }
    }
}

/// Don't deny unknown fields to allow multiple configs in one yaml-file.
#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub balancing: RawContent,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawContent {
    #[serde(rename = "results-dir")]
    pub results_dir: PathBuf,
    #[serde(rename = "iter-0-cfg")]
    pub iter_0_cfg: PathBuf,
    #[serde(rename = "iter-i-cfg")]
    pub iter_i_cfg: PathBuf,
    #[serde(rename = "new_graph-dim")]
    pub new_graph_dim: usize,
    pub number_of_iterations: usize,
    #[serde(rename = "metric-ids")]
    pub metric_ids: metrics::RawConfig,
    #[serde(flatten)]
    pub optimization: RawOptimization,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RawOptimization {
    #[serde(rename = "explicit_euler")]
    ExplicitEuler {
        #[serde(rename = "correction")]
        correction: f64,
    },
    // some kind of correction-function:
    // interpolating linear between point-pairs given in a file?
}
