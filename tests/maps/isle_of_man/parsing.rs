use crate::helpers::{defaults, parse};
use osmgraphing::configs::Config;

const PBF_CONFIG: &str = defaults::paths::resources::configs::ISLE_OF_MAN_PBF;
const CH_FMI_CONFIG: &str = defaults::paths::resources::configs::ISLE_OF_MAN_CH_FMI;
const FMI_CONFIG: &str = defaults::paths::resources::configs::ISLE_OF_MAN_FMI;

#[test]
fn pbf_yaml() {
    Config::from_yaml(PBF_CONFIG).unwrap();
}

#[test]
fn fmi_yaml() {
    Config::from_yaml(FMI_CONFIG).unwrap();
}

#[test]
fn ch_fmi_yaml() {
    Config::from_yaml(CH_FMI_CONFIG).unwrap();
}

#[test]
fn yaml_str() {
    Config::from_yaml(PBF_CONFIG).unwrap();
}

#[test]
fn pbf_graph() {
    let cfg = Config::from_yaml(PBF_CONFIG).unwrap();
    let graph = parse(cfg.parser);

    let nodes = graph.nodes();
    let expected = 52_803;
    assert_eq!(
        nodes.count(),
        expected,
        "Number of nodes in graph should be {} but is {}.",
        expected,
        nodes.count()
    );
    let fwd_edges = graph.fwd_edges();
    let expected = 107_031;
    assert_eq!(
        fwd_edges.count(),
        expected,
        "Number of fwd-edges in graph should be {} but is {}.",
        expected,
        fwd_edges.count()
    );
}

#[test]
fn fmi_graph() {
    let cfg = Config::from_yaml(defaults::paths::resources::configs::ISLE_OF_MAN_FMI).unwrap();
    let graph = parse(cfg.parser);

    let nodes = graph.nodes();
    let expected = 52_803;
    assert_eq!(
        nodes.count(),
        expected,
        "Number of nodes in graph should be {} but is {}.",
        expected,
        nodes.count()
    );
    let fwd_edges = graph.fwd_edges();
    let expected = 107_031;
    assert_eq!(
        fwd_edges.count(),
        expected,
        "Number of fwd-edges in graph should be {} but is {}.",
        expected,
        fwd_edges.count()
    );
}

#[test]
fn ch_fmi_graph() {
    let cfg = Config::from_yaml(defaults::paths::resources::configs::ISLE_OF_MAN_CH_FMI).unwrap();
    let graph = parse(cfg.parser);

    let nodes = graph.nodes();
    let expected = 52_803;
    assert_eq!(
        nodes.count(),
        expected,
        "Number of nodes in graph should be {} but is {}.",
        expected,
        nodes.count()
    );
    let fwd_edges = graph.fwd_edges();
    let expected = 183_366;
    assert_eq!(
        fwd_edges.count(),
        expected,
        "Number of fwd-edges in graph should be {} but is {}.",
        expected,
        fwd_edges.count()
    );
}
