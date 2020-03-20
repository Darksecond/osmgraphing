use crate::helpers::{assert_path, defaults, TestNode};
use osmgraphing::{
    configs::{self, Config},
    network::NodeIdx,
    routing,
    units::geo::Coordinate,
};

#[test]
fn chdijkstra() {
    let mut cfg = Config::from_yaml(defaults::paths::resources::configs::SMALL_FMI).unwrap();
    cfg.routing = configs::routing::Config::from_str(
        &format!(
            "routing: {{ metrics: [{{ id: '{}' }}], is-ch-dijkstra: true }}",
            defaults::LENGTH_ID
        ),
        &cfg.parser,
    )
    .ok();

    let mut dijkstra = routing::Dijkstra::new();
    let expected_paths = expected_paths();

    assert_path(&mut dijkstra, expected_paths, cfg);
}

#[test]
fn dijkstra() {
    let mut cfg = Config::from_yaml(defaults::paths::resources::configs::SMALL_FMI).unwrap();
    cfg.routing = configs::routing::Config::from_str(
        &format!(
            "routing: {{ metrics: [{{ id: '{}' }}], is-ch-dijkstra: false }}",
            defaults::LENGTH_ID
        ),
        &cfg.parser,
    )
    .ok();

    let mut dijkstra = routing::Dijkstra::new();
    let expected_paths = expected_paths();

    assert_path(&mut dijkstra, expected_paths, cfg);
}

fn expected_paths() -> Vec<(TestNode, TestNode, Option<(f64, Vec<Vec<TestNode>>)>)> {
    let a: usize = 0;
    let b: usize = 1;
    let c: usize = 2;
    let d: usize = 3;
    let e: usize = 4;
    let f: usize = 5;
    let g: usize = 6;
    let h: usize = 7;

    let nodes: Vec<TestNode> = vec![
        ("a", a),
        ("b", b),
        ("c", c),
        ("d", d),
        ("e", e),
        ("f", f),
        ("g", g),
        ("h", h),
    ]
    .into_iter()
    .map(|(name, idx)| TestNode {
        name: String::from(name),
        idx: NodeIdx(idx),
        id: idx as i64,
        coord: Coordinate::zero(),
        level: 0,
    })
    .collect();

    let expected_paths = vec![
        // a
        (a, a, Some((0.0, vec![vec![]]))),
        (a, b, None),
        (a, c, None),
        (a, d, None),
        (a, e, None),
        (a, f, None),
        (a, g, None),
        (a, h, None),
        // b
        (b, a, Some((0.001, vec![vec![b, a]]))),
        (b, b, Some((0.0, vec![vec![]]))),
        (b, c, Some((0.001, vec![vec![b, c]]))),
        (b, d, None),
        (b, e, None),
        (b, f, None),
        (b, g, None),
        (b, h, None),
        // c
        (c, a, Some((0.001, vec![vec![c, a]]))),
        (c, b, Some((0.001, vec![vec![c, b]]))),
        (c, c, Some((0.0, vec![vec![]]))),
        (c, d, None),
        (c, e, None),
        (c, f, None),
        (c, g, None),
        (c, h, None),
        // d
        (d, a, Some((0.002, vec![vec![d, b, a]]))),
        (d, b, Some((0.001, vec![vec![d, b]]))),
        (d, c, Some((0.002, vec![vec![d, b, c]]))),
        (d, d, Some((0.0, vec![vec![]]))),
        (d, e, Some((0.002, vec![vec![d, e]]))),
        (d, f, Some((0.002, vec![vec![d, h, f]]))),
        (d, g, None),
        (d, h, Some((0.001, vec![vec![d, h]]))),
        // e
        (e, a, Some((0.004, vec![vec![e, d, b, a]]))),
        (e, b, Some((0.003, vec![vec![e, d, b]]))),
        (e, c, Some((0.004, vec![vec![e, d, b, c]]))),
        (e, d, Some((0.002, vec![vec![e, d]]))),
        (e, e, Some((0.0, vec![vec![]]))),
        (e, f, Some((0.001, vec![vec![e, f]]))),
        (e, g, None),
        (e, h, Some((0.002, vec![vec![e, f, h]]))),
        // f
        (f, a, Some((0.004, vec![vec![f, h, d, b, a]]))),
        (f, b, Some((0.003, vec![vec![f, h, d, b]]))),
        (f, c, Some((0.004, vec![vec![f, h, d, b, c]]))),
        (f, d, Some((0.002, vec![vec![f, h, d]]))),
        (f, e, Some((0.001, vec![vec![f, e]]))),
        (f, f, Some((0.0, vec![vec![]]))),
        (f, g, None),
        (f, h, Some((0.001, vec![vec![f, h]]))),
        // g
        (g, a, Some((0.005, vec![vec![g, e, d, b, a]]))),
        (
            g,
            b,
            Some((0.004, vec![vec![g, e, d, b], vec![g, f, h, d, b]])),
        ),
        (g, c, Some((0.005, vec![vec![g, e, d, b, c]]))),
        (g, d, Some((0.003, vec![vec![g, e, d], vec![g, f, d]]))),
        (g, e, Some((0.001, vec![vec![g, e]]))),
        (g, f, Some((0.001, vec![vec![g, f]]))),
        (g, g, Some((0.0, vec![vec![]]))),
        (g, h, Some((0.002, vec![vec![g, f, h]]))),
        // h
        (h, a, Some((0.003, vec![vec![h, d, b, a]]))),
        (h, b, Some((0.002, vec![vec![h, d, b]]))),
        (h, c, Some((0.003, vec![vec![h, d, b, c]]))),
        (h, d, Some((0.001, vec![vec![h, d]]))),
        (h, e, Some((0.002, vec![vec![h, f, e]]))),
        (h, f, Some((0.001, vec![vec![h, f]]))),
        (h, g, None),
        (h, h, Some((0.0, vec![vec![]]))),
    ];

    // map indices to nodes
    expected_paths
        .into_iter()
        .map(|(src_idx, dst_idx, path_info)| {
            let src = nodes[src_idx].clone();
            let dst = nodes[dst_idx].clone();
            let path_info: Option<(f64, Vec<Vec<TestNode>>)> = match path_info {
                Some((cost, paths)) => {
                    let paths = paths
                        .into_iter()
                        .map(|path| {
                            path.into_iter()
                                .map(|node_idx| nodes[node_idx].clone())
                                .collect()
                        })
                        .collect();
                    Some((cost, paths))
                }
                None => None,
            };
            (src, dst, path_info)
        })
        .collect()
}
