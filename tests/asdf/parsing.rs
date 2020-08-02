use crate::helpers::defaults;
use osmgraphing::{configs, io::network::graph::Parser};
use std::path::PathBuf;

#[test]
fn wrong_extension() {
    let mut parsing_cfg =
        configs::parsing::Config::from_yaml(defaults::paths::resources::small::FMI_YAML);
    parsing_cfg.map_file = PathBuf::from("foo.asdf");
    assert!(
        Parser::parse(parsing_cfg).is_err(),
        "File-extension 'asdf' should not be supported."
    );
}

#[test]
fn routing_config_from_str() {
    let parsing_cfg =
        configs::parsing::Config::from_yaml(defaults::paths::resources::small::FMI_YAML);

    let yaml_str = &format!(
        "routing: {{ route-pairs-file: 'asdf', metrics: [{{ id: '{}' }}], algorithm: CHDijkstra }}",
        defaults::SPEED_ID
    );
    let routing_cfg = configs::routing::Config::from_str(yaml_str, &parsing_cfg);
    assert_eq!(
        routing_cfg.routing_algo,
        configs::routing::RoutingAlgo::CHDijkstra,
        "Routing-config should specify ch-dijkstra."
    );

    let yaml_str = &format!(
        "routing: {{ route-pairs-file: 'asdf', metrics: [{{ id: '{}' }}], algorithm: Dijkstra }}",
        defaults::SPEED_ID
    );
    let routing_cfg = configs::routing::Config::from_str(yaml_str, &parsing_cfg);
    assert_ne!(
        routing_cfg.routing_algo,
        configs::routing::RoutingAlgo::CHDijkstra,
        "Routing-config should specify normal dijkstra."
    );
}

#[test]
#[ignore]
fn config_from_str() {
    // TODO implement testing yaml-str
}
