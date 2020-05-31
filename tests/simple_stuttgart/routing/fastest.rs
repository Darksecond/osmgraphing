use crate::helpers::{defaults, test_dijkstra, TestNode};
use defaults::paths::resources::simple_stuttgart as resources;
use kissunits::{
    geo::Coordinate,
    time::{Hours, Minutes},
};
use osmgraphing::{
    configs::{self, SimpleId},
    defaults::capacity::DimVec,
    network::{MetricIdx, NodeIdx},
};
use smallvec::smallvec;

const METRIC_ID: &str = defaults::DURATION_ID;
const IS_CH_DIJKSTRA: bool = true;

#[test]
fn chdijkstra_on_map() {
    test_dijkstra(
        resources::FMI_YAML,
        METRIC_ID,
        IS_CH_DIJKSTRA,
        Box::new(expected_paths),
    )
}

#[test]
fn dijkstra_on_map() {
    test_dijkstra(
        resources::FMI_YAML,
        METRIC_ID,
        !IS_CH_DIJKSTRA,
        Box::new(expected_paths),
    )
}

fn expected_paths(
    parsing_cfg: &configs::parsing::Config,
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
        ch_level: 0,
    })
    .collect();

    let expected_paths = vec![
        // opp
        (opp, opp, Some((0.0, vec![vec![]]))),
        (opp, bac, Some((9.6, vec![vec![opp, bac]]))),
        (opp, wai, Some((21.1, vec![vec![opp, bac, wai]]))),
        (opp, end, Some((26.1, vec![vec![opp, bac, end]]))),
        (opp, dea, Some((11.738, vec![vec![opp, bac, dea]]))),
        (opp, stu, Some((31.3, vec![vec![opp, bac, wai, stu]]))),
        // bac
        (bac, opp, Some((9.6, vec![vec![bac, opp]]))),
        (bac, bac, Some((0.0, vec![vec![]]))),
        (bac, wai, Some((11.5, vec![vec![bac, wai]]))),
        (bac, end, Some((16.5, vec![vec![bac, end]]))),
        (bac, dea, Some((2.138, vec![vec![bac, dea]]))),
        (bac, stu, Some((21.7, vec![vec![bac, wai, stu]]))),
        // wai
        (wai, opp, Some((21.1, vec![vec![wai, bac, opp]]))),
        (wai, bac, Some((11.5, vec![vec![wai, bac]]))),
        (wai, wai, Some((0.0, vec![vec![]]))),
        (wai, end, Some((9.6, vec![vec![wai, end]]))),
        (wai, dea, Some((13.638, vec![vec![wai, bac, dea]]))),
        (wai, stu, Some((10.2, vec![vec![wai, stu]]))),
        // end
        (end, opp, Some((26.1, vec![vec![end, bac, opp]]))),
        (end, bac, Some((16.5, vec![vec![end, bac]]))),
        (end, wai, Some((9.6, vec![vec![end, wai]]))),
        (end, end, Some((0.0, vec![vec![]]))),
        (end, dea, Some((18.638, vec![vec![end, bac, dea]]))),
        (end, stu, Some((15.75, vec![vec![end, stu]]))),
        // dea
        (dea, opp, None),
        (dea, bac, None),
        (dea, wai, None),
        (dea, end, None),
        (dea, dea, Some((0.0, vec![vec![]]))),
        (dea, stu, None),
        // stu
        (stu, opp, Some((31.3, vec![vec![stu, wai, bac, opp]]))),
        (stu, bac, Some((21.7, vec![vec![stu, wai, bac]]))),
        (stu, wai, Some((10.2, vec![vec![stu, wai]]))),
        (stu, end, Some((15.75, vec![vec![stu, end]]))),
        (stu, dea, Some((23.838, vec![vec![stu, wai, bac, dea]]))),
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
                    let cost = Hours::from(Minutes(cost));
                    Some((smallvec![*cost], paths))
                }
                None => None,
            };
            (
                src,
                dst,
                smallvec![MetricIdx(
                    parsing_cfg
                        .edges
                        .metrics
                        .ids
                        .iter()
                        .position(|id| id == &SimpleId::from(METRIC_ID))
                        .expect("Expect simple-stuttgart's duration-id to be correct.")
                )],
                path_info,
            )
        })
        .collect()
}
