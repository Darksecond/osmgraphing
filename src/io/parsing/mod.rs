pub mod fmi;
pub mod pbf;

use crate::{
    configs::parser::{self, EdgeCategory, NodeCategory},
    defaults::capacity,
    io::{MapFileExt, SupportingFileExts, SupportingMapFileExts},
    network::{EdgeBuilder, Graph, GraphBuilder, NodeBuilder},
};
use log::{info, warn};
use std::path::Path;

/// The parser parsing `*.osm.pbf`- and `*.fmi`-files into a graphbuilder or a graph.
///
///
/// ## The filter-pipeline
///
/// 1. Download raw osm-data (see [README](https://github.com/dominicparga/osmgraphing/blob/nightly/README.md))
/// 1. Read in this data
/// 1. Filter and process osm-components like nodes and edges, e.g. filtering via tags
/// 1. Create a memory- and runtime-efficient routing-graph.
///
///
/// ### Nodes
///
/// - Coordinates:
///   Nodes have coordinates given in `(latitude, longitude)`.
/// - Height: Nodes have a height, which is ignored right now.
///
///
/// ### Edges
///
/// Every edge will have a street-type with respective default speed-limit.
/// These defaults depend on the street-network and can be found in the respective module `network`.
///
///
/// ## Additional information
///
/// This `pbf`-parser uses [osmpbfreader-rs](https://crates.io/crates/osmpbfreader).
/// An own implementation would need [the pbf-impl of rust](https://github.com/stepancheg/rust-protobuf), but the previously mentioned osmpbfreader works well.
/// `*.osm`-xml-files are not supported, but could be read with [quick-xml](https://github.com/tafia/quick-xml).
///
/// Other libraries processing openstreetmap-data can be found [in the osm-wiki](https://wiki.openstreetmap.org/wiki/Frameworks#Data_Processing_or_Parsing_Libraries).
pub struct Parser;

impl Parser {
    pub fn parse(cfg: parser::Config) -> Result<GraphBuilder, String> {
        match Parser::from_path(&cfg.map_file)? {
            MapFileExt::PBF => pbf::Parser::new().parse(cfg),
            MapFileExt::FMI => fmi::Parser::new().parse(cfg),
        }
    }

    pub fn parse_and_finalize(cfg: parser::Config) -> Result<Graph, String> {
        match Parser::from_path(&cfg.map_file)? {
            MapFileExt::PBF => pbf::Parser::new().parse_and_finalize(cfg),
            MapFileExt::FMI => fmi::Parser::new().parse_and_finalize(cfg),
        }
    }
}

impl SupportingMapFileExts for Parser {}
impl SupportingFileExts for Parser {
    fn supported_exts<'a>() -> &'a [&'a str] {
        &["pbf", "fmi"]
    }
}

trait Parsing {
    fn preprocess(&mut self, cfg: &parser::Config) -> Result<(), String> {
        check_parser_config(cfg)
    }

    fn parse(&mut self, cfg: parser::Config) -> Result<GraphBuilder, String> {
        let mut builder = GraphBuilder::new(cfg);

        info!("START Process given file");
        self.preprocess(builder.cfg())?;
        self.parse_ways(&mut builder)?;
        let mut builder = builder.next();
        self.parse_nodes(&mut builder)?;
        let builder = builder.next();
        info!("FINISHED");

        builder
    }

    fn parse_ways(&self, builder: &mut EdgeBuilder) -> Result<(), String>;

    fn parse_nodes(&self, builder: &mut NodeBuilder) -> Result<(), String>;

    fn parse_and_finalize(&mut self, cfg: parser::Config) -> Result<Graph, String> {
        let path = Path::new(&cfg.map_file);
        info!("START Parse from given path {}", path.display());

        // TODO parse "cycleway" and other tags
        // see https://wiki.openstreetmap.org/wiki/Key:highway

        let result = self.parse(cfg)?.finalize();
        info!("FINISHED");
        result
    }
}

fn check_parser_config(cfg: &parser::Config) -> Result<(), String> {
    // check if yaml-config is correct

    // check nodes' meta-info

    if !cfg.nodes.categories().contains(&NodeCategory::NodeId) {
        return Err(String::from(
            "The provided config-file doesn't contain a NodeId, but needs to.",
        ));
    }

    // check nodes' coordinates

    if !cfg.nodes.categories().contains(&NodeCategory::Latitude) {
        return Err(String::from(
            "The provided config-file doesn't contain a latitude, but needs to.",
        ));
    }

    if !cfg.nodes.categories().contains(&NodeCategory::Longitude) {
        return Err(String::from(
            "The provided config-file doesn't contain a longitude, but needs to.",
        ));
    }

    // check edges' metric-memory-capacity

    if cfg.edges.metrics.dim() > capacity::SMALL_VEC_INLINE_SIZE {
        return Err(format!(
            "The provided config-file has more metrics for the graph ({}) \
             than the parser has been compiled to ({}).",
            cfg.edges.metrics.dim(),
            capacity::SMALL_VEC_INLINE_SIZE
        ));
    } else if cfg.edges.metrics.dim() < capacity::SMALL_VEC_INLINE_SIZE {
        warn!(
            "The provided config-file has less metrics for the graph ({}) \
             than the parser has been compiled to ({}). \
             Compiling accordingly saves memory.",
            cfg.edges.metrics.dim(),
            capacity::SMALL_VEC_INLINE_SIZE
        );
    }

    // check count of shortcut-edge-indices

    let count = cfg
        .edges
        .categories
        .iter()
        .filter(|&category| category == &EdgeCategory::ShortcutEdgeIdx)
        .count();
    if count > 0 && count != 2 {
        return Err(format!(
            "The config-file has a different number than 0 or 2 of edge-category '{}'",
            EdgeCategory::ShortcutEdgeIdx
        ));
    }

    Ok(())
}
