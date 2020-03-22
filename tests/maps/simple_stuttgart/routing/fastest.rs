use crate::helpers::{defaults, test_dijkstra_on_map, TestNode};
use osmgraphing::{
    configs::{self, SimpleId},
    defaults::capacity::DimVec,
    network::{MetricIdx, NodeIdx},
    units::geo::Coordinate,
};
use smallvec::smallvec;

const METRIC_ID: &str = defaults::DURATION_ID;
const CONFIG: &str = defaults::paths::resources::configs::SIMPLE_STUTTGART_FMI;
const IS_CH_DIJKSTRA: bool = true;

#[test]
fn chdijkstra_on_map() {
    test_dijkstra_on_map(CONFIG, METRIC_ID, IS_CH_DIJKSTRA, Box::new(expected_paths))
}

#[test]
fn dijkstra_on_map() {
    test_dijkstra_on_map(CONFIG, METRIC_ID, !IS_CH_DIJKSTRA, Box::new(expected_paths))
}

fn expected_paths(
    cfg_parser: &configs::parser::Config,
) -> Vec<(
    TestNode,
    TestNode,
    DimVec<MetricIdx>,
    Option<(DimVec<f64>, Vec<Vec<TestNode>>)>,
)> {
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
        level: 0,
    })
    .collect();

    let expected_paths = vec![
        // opp
        (opp, opp, Some((0.0, vec![vec![]]))),
        (opp, bac, Some((576.0, vec![vec![opp, bac]]))),
        (opp, wai, Some((1_266.0, vec![vec![opp, bac, wai]]))),
        (opp, end, Some((1_566.0, vec![vec![opp, bac, end]]))),
        (opp, dea, Some((704.28, vec![vec![opp, bac, dea]]))),
        (opp, stu, Some((1_878.0, vec![vec![opp, bac, wai, stu]]))),
        // bac
        (bac, opp, Some((576.0, vec![vec![bac, opp]]))),
        (bac, bac, Some((0.0, vec![vec![]]))),
        (bac, wai, Some((690.0, vec![vec![bac, wai]]))),
        (bac, end, Some((990.0, vec![vec![bac, end]]))),
        (bac, dea, Some((128.28, vec![vec![bac, dea]]))),
        (bac, stu, Some((1_302.0, vec![vec![bac, wai, stu]]))),
        // wai
        (wai, opp, Some((1_266.0, vec![vec![wai, bac, opp]]))),
        (wai, bac, Some((690.0, vec![vec![wai, bac]]))),
        (wai, wai, Some((0.0, vec![vec![]]))),
        (wai, end, Some((576.0, vec![vec![wai, end]]))),
        (wai, dea, Some((818.28, vec![vec![wai, bac, dea]]))),
        (wai, stu, Some((612.0, vec![vec![wai, stu]]))),
        // end
        (end, opp, Some((1_566.0, vec![vec![end, bac, opp]]))),
        (end, bac, Some((990.0, vec![vec![end, bac]]))),
        (end, wai, Some((576.0, vec![vec![end, wai]]))),
        (end, end, Some((0.0, vec![vec![]]))),
        (end, dea, Some((1_118.28, vec![vec![end, bac, dea]]))),
        (end, stu, Some((945.0, vec![vec![end, stu]]))),
        // dea
        (dea, opp, None),
        (dea, bac, None),
        (dea, wai, None),
        (dea, end, None),
        (dea, dea, Some((0.0, vec![vec![]]))),
        (dea, stu, None),
        // stu
        (stu, opp, Some((1_878.0, vec![vec![stu, wai, bac, opp]]))),
        (stu, bac, Some((1_302.0, vec![vec![stu, wai, bac]]))),
        (stu, wai, Some((612.0, vec![vec![stu, wai]]))),
        (stu, end, Some((945.0, vec![vec![stu, end]]))),
        (stu, dea, Some((1_430.28, vec![vec![stu, wai, bac, dea]]))),
        (stu, stu, Some((0.0, vec![vec![]]))),
    ];

    // map indices to nodes
    expected_paths
        .into_iter()
        .map(|(src_idx, dst_idx, path_info)| {
            let src = nodes[src_idx].clone();
            let dst = nodes[dst_idx].clone();
            let path_info: Option<(DimVec<f64>, Vec<Vec<TestNode>>)> = match path_info {
                Some((cost, paths)) => {
                    let paths = paths
                        .into_iter()
                        .map(|path| {
                            path.into_iter()
                                .map(|node_idx| nodes[node_idx].clone())
                                .collect()
                        })
                        .collect();
                    Some((smallvec![cost], paths))
                }
                None => None,
            };
            (
                src,
                dst,
                smallvec![cfg_parser.edges.metric_idx(&SimpleId::from(METRIC_ID))],
                path_info,
            )
        })
        .collect()
}
