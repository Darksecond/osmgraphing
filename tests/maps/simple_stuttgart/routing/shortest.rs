use crate::helpers::{assert_path, create_config, defaults, TestNode, TestType};
use osmgraphing::{network::NodeIdx, routing, units::geo::Coordinate};

#[test]
fn bidirectional_dijkstra() {
    let cfg = create_config(
        TestType::SimpleStuttgart,
        Some(&format!("routing: [{{ id: '{}' }}]", defaults::LENGTH_ID)),
    );

    let mut dijkstra = routing::Dijkstra::new();
    let expected_paths = expected_paths();

    assert_path(&mut dijkstra, expected_paths, cfg);
}

fn expected_paths() -> Vec<(TestNode, TestNode, Option<(f32, Vec<Vec<TestNode>>)>)> {
    let opp: usize = 0;
    let bac: usize = 1;
    let wai: usize = 2;
    let end: usize = 3;
    let dea: usize = 4;
    let stu: usize = 5;

    let nodes: Vec<TestNode> = vec![
        ("opp", opp, 26_033_921, 48.9840100, 9.4589188),
        ("bac", bac, 26_160_028, 48.9416023, 9.4332023),
        ("wai", wai, 252_787_940, 48.8271096, 9.3098661),
        ("end", end, 298_249_467, 48.8108510, 9.3679493),
        ("dea", dea, 1_621_605_361, 48.9396327, 9.4188681),
        ("stu", stu, 2_933_335_353, 48.7701757, 9.1565768),
    ]
    .into_iter()
    .map(|(name, idx, id, lat, lon)| TestNode {
        name: String::from(name),
        idx: NodeIdx(idx),
        id,
        coord: Coordinate { lat, lon },
    })
    .collect();

    let expected_paths = vec![
        // opp
        (opp, opp, Some((0.0, vec![vec![]]))),
        (opp, bac, Some((8.0, vec![vec![opp, bac]]))),
        (opp, wai, Some((31.0, vec![vec![opp, bac, wai]]))),
        (opp, end, Some((30.0, vec![vec![opp, bac, end]]))),
        (opp, dea, Some((9.069, vec![vec![opp, bac, dea]]))),
        (opp, stu, Some((48.0, vec![vec![opp, bac, wai, stu]]))),
        // bac
        (bac, opp, Some((8.0, vec![vec![bac, opp]]))),
        (bac, bac, Some((0.0, vec![vec![]]))),
        (bac, wai, Some((23.0, vec![vec![bac, wai]]))),
        (bac, end, Some((22.0, vec![vec![bac, end]]))),
        (bac, dea, Some((1.069, vec![vec![bac, dea]]))),
        (bac, stu, Some((40.0, vec![vec![bac, wai, stu]]))),
        // wai
        (wai, opp, Some((31.0, vec![vec![wai, bac, opp]]))),
        (wai, bac, Some((23.0, vec![vec![wai, bac]]))),
        (wai, wai, Some((0.0, vec![vec![]]))),
        (wai, end, Some((8.0, vec![vec![wai, end]]))),
        (wai, dea, Some((24.069, vec![vec![wai, bac, dea]]))),
        (wai, stu, Some((17.0, vec![vec![wai, stu]]))),
        // end
        (end, opp, Some((30.0, vec![vec![end, bac, opp]]))),
        (end, bac, Some((22.0, vec![vec![end, bac]]))),
        (end, wai, Some((8.0, vec![vec![end, wai]]))),
        (end, end, Some((0.0, vec![vec![]]))),
        (end, dea, Some((23.069, vec![vec![end, bac, dea]]))),
        (end, stu, Some((21.0, vec![vec![end, stu]]))),
        // dea
        (dea, opp, None),
        (dea, bac, None),
        (dea, wai, None),
        (dea, end, None),
        (dea, dea, Some((0.0, vec![vec![]]))),
        (dea, stu, None),
        // stu
        (stu, opp, Some((48.0, vec![vec![stu, wai, bac, opp]]))),
        (stu, bac, Some((40.0, vec![vec![stu, wai, bac]]))),
        (stu, wai, Some((17.0, vec![vec![stu, wai]]))),
        (stu, end, Some((21.0, vec![vec![stu, end]]))),
        (stu, dea, Some((41.069, vec![vec![stu, wai, bac, dea]]))),
        (stu, stu, Some((0.0, vec![vec![]]))),
    ];

    // map indices to nodes
    expected_paths
        .into_iter()
        .map(|(src_idx, dst_idx, path_info)| {
            let src = nodes[src_idx].clone();
            let dst = nodes[dst_idx].clone();
            let path_info: Option<(f32, Vec<Vec<TestNode>>)> = match path_info {
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
