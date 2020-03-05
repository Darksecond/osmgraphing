use super::{assert_correct, create_config, TestNode, TestType};
use osmgraphing::network::NodeIdx;

const DURATION_ID: &str = "Seconds";

pub mod astar {
    pub mod unidirectional {
        use super::super::{assert_correct, create_config, expected_paths, TestType, DURATION_ID};
        use osmgraphing::routing;

        pub fn simple_stuttgart() {
            let cfg = create_config(TestType::SimpleStuttgart);

            let mut astar = routing::factory::astar::unidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::SimpleStuttgart);

            assert_correct(&mut astar, expected_paths, cfg.graph);
        }

        pub fn small() {
            let cfg = create_config(TestType::Small);

            let mut astar = routing::factory::astar::unidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::Small);

            assert_correct(&mut astar, expected_paths, cfg.graph);
        }

        pub fn bidirectional_bait() {
            let cfg = create_config(TestType::BidirectionalBait);

            let mut astar = routing::factory::astar::unidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::BidirectionalBait);

            assert_correct(&mut astar, expected_paths, cfg.graph);
        }

        pub fn isle_of_man() {
            let cfg = create_config(TestType::IsleOfMan);

            let mut astar = routing::factory::astar::unidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::BidirectionalBait);

            assert_correct(&mut astar, expected_paths, cfg.graph);
        }
    }

    pub mod bidirectional {
        use super::super::{assert_correct, create_config, expected_paths, TestType, DURATION_ID};
        use osmgraphing::routing;

        pub fn simple_stuttgart() {
            let cfg = create_config(TestType::SimpleStuttgart);

            let mut astar = routing::factory::astar::bidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::SimpleStuttgart);

            assert_correct(&mut astar, expected_paths, cfg.graph);
        }

        pub fn small() {
            let cfg = create_config(TestType::Small);

            let mut astar = routing::factory::astar::bidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::Small);

            assert_correct(&mut astar, expected_paths, cfg.graph);
        }

        pub fn bidirectional_bait() {
            let cfg = create_config(TestType::BidirectionalBait);

            let mut astar = routing::factory::astar::bidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::BidirectionalBait);

            assert_correct(&mut astar, expected_paths, cfg.graph);
        }

        pub fn isle_of_man() {
            let cfg = create_config(TestType::IsleOfMan);

            let mut astar = routing::factory::astar::bidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::BidirectionalBait);

            assert_correct(&mut astar, expected_paths, cfg.graph);
        }
    }
}

pub mod dijkstra {
    pub mod unidirectional {
        use super::super::{assert_correct, create_config, expected_paths, TestType, DURATION_ID};
        use osmgraphing::routing;

        pub fn simple_stuttgart() {
            let cfg = create_config(TestType::SimpleStuttgart);

            let mut dijkstra = routing::factory::dijkstra::unidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::SimpleStuttgart);

            assert_correct(&mut dijkstra, expected_paths, cfg.graph);
        }

        pub fn small() {
            let cfg = create_config(TestType::Small);

            let mut dijkstra = routing::factory::dijkstra::unidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::Small);

            assert_correct(&mut dijkstra, expected_paths, cfg.graph);
        }

        pub fn bidirectional_bait() {
            let cfg = create_config(TestType::BidirectionalBait);

            let mut dijkstra = routing::factory::dijkstra::unidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::BidirectionalBait);

            assert_correct(&mut dijkstra, expected_paths, cfg.graph);
        }

        pub fn isle_of_man() {
            let cfg = create_config(TestType::IsleOfMan);

            let mut dijkstra = routing::factory::dijkstra::unidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::BidirectionalBait);

            assert_correct(&mut dijkstra, expected_paths, cfg.graph);
        }
    }

    pub mod bidirectional {
        use super::super::{assert_correct, create_config, expected_paths, TestType, DURATION_ID};
        use osmgraphing::routing;

        pub fn simple_stuttgart() {
            let cfg = create_config(TestType::SimpleStuttgart);

            let mut dijkstra = routing::factory::dijkstra::bidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::SimpleStuttgart);

            assert_correct(&mut dijkstra, expected_paths, cfg.graph);
        }

        pub fn small() {
            let cfg = create_config(TestType::Small);

            let mut dijkstra = routing::factory::dijkstra::bidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::Small);

            assert_correct(&mut dijkstra, expected_paths, cfg.graph);
        }

        pub fn bidirectional_bait() {
            let cfg = create_config(TestType::BidirectionalBait);

            let mut dijkstra = routing::factory::dijkstra::bidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::BidirectionalBait);

            assert_correct(&mut dijkstra, expected_paths, cfg.graph);
        }

        pub fn isle_of_man() {
            let cfg = create_config(TestType::IsleOfMan);

            let mut dijkstra = routing::factory::dijkstra::bidirectional::fastest(
                cfg.graph.edges.metrics.idx(&DURATION_ID.into()),
            );
            let expected_paths = expected_paths(TestType::BidirectionalBait);

            assert_correct(&mut dijkstra, expected_paths, cfg.graph);
        }
    }
}

fn expected_paths(
    test_type: TestType,
) -> Vec<(TestNode, TestNode, Option<(f32, Vec<Vec<TestNode>>)>)> {
    match test_type {
        TestType::BidirectionalBait => expected_paths_bait(),
        TestType::IsleOfMan => expected_paths_isle_of_man(),
        TestType::SimpleStuttgart => expected_paths_simple_stuttgart(),
        TestType::Small => expected_paths_small(),
    }
}

fn expected_paths_simple_stuttgart() -> Vec<(TestNode, TestNode, Option<(f32, Vec<Vec<TestNode>>)>)>
{
    // (idx, id)
    let opp = TestNode {
        idx: NodeIdx(0),
        id: 26_033_921,
    };
    let bac = TestNode {
        idx: NodeIdx(1),
        id: 26_160_028,
    };
    let wai = TestNode {
        idx: NodeIdx(2),
        id: 252_787_940,
    };
    let end = TestNode {
        idx: NodeIdx(3),
        id: 298_249_467,
    };
    let dea = TestNode {
        idx: NodeIdx(4),
        id: 1_621_605_361,
    };
    let stu = TestNode {
        idx: NodeIdx(5),
        id: 2_933_335_353,
    };

    vec![
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
    ]
}

fn expected_paths_small() -> Vec<(TestNode, TestNode, Option<(f32, Vec<Vec<TestNode>>)>)> {
    // (idx, id)
    let a = TestNode {
        idx: NodeIdx(0),
        id: 0,
    };
    let b = TestNode {
        idx: NodeIdx(1),
        id: 1,
    };
    let c = TestNode {
        idx: NodeIdx(2),
        id: 2,
    };
    let d = TestNode {
        idx: NodeIdx(3),
        id: 3,
    };
    let e = TestNode {
        idx: NodeIdx(4),
        id: 4,
    };
    let f = TestNode {
        idx: NodeIdx(5),
        id: 5,
    };
    let g = TestNode {
        idx: NodeIdx(6),
        id: 6,
    };
    let h = TestNode {
        idx: NodeIdx(7),
        id: 7,
    };

    vec![
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
        (b, a, Some((0.12, vec![vec![b, a]]))),
        (b, b, Some((0.0, vec![vec![]]))),
        (b, c, Some((0.12, vec![vec![b, c]]))),
        (b, d, None),
        (b, e, None),
        (b, f, None),
        (b, g, None),
        (b, h, None),
        // c
        (c, a, Some((0.12, vec![vec![c, a]]))),
        (c, b, Some((0.12, vec![vec![c, b]]))),
        (c, c, Some((0.0, vec![vec![]]))),
        (c, d, None),
        (c, e, None),
        (c, f, None),
        (c, g, None),
        (c, h, None),
        // d
        (d, a, Some((0.24, vec![vec![d, b, a]]))),
        (d, b, Some((0.12, vec![vec![d, b]]))),
        (d, c, Some((0.24, vec![vec![d, b, c]]))),
        (d, d, Some((0.0, vec![vec![]]))),
        (d, e, Some((0.24, vec![vec![d, e]]))),
        (d, f, Some((0.24, vec![vec![d, h, f]]))),
        (d, g, None),
        (d, h, Some((0.12, vec![vec![d, h]]))),
        // e
        (e, a, Some((0.48, vec![vec![e, d, b, a]]))),
        (e, b, Some((0.36, vec![vec![e, d, b]]))),
        (e, c, Some((0.48, vec![vec![e, d, b, c]]))),
        (e, d, Some((0.24, vec![vec![e, d]]))),
        (e, e, Some((0.0, vec![vec![]]))),
        (e, f, Some((0.12, vec![vec![e, f]]))),
        (e, g, None),
        (e, h, Some((0.24, vec![vec![e, f, h]]))),
        // f
        (f, a, Some((0.48, vec![vec![f, h, d, b, a]]))),
        (f, b, Some((0.36, vec![vec![f, h, d, b]]))),
        (f, c, Some((0.48, vec![vec![f, h, d, b, c]]))),
        (f, d, Some((0.24, vec![vec![f, h, d]]))),
        (f, e, Some((0.12, vec![vec![f, e]]))),
        (f, f, Some((0.0, vec![vec![]]))),
        (f, g, None),
        (f, h, Some((0.12, vec![vec![f, h]]))),
        // g
        (
            g,
            a,
            Some((0.6, vec![vec![g, f, h, d, b, a], vec![g, e, d, b, a]])),
        ),
        (
            g,
            b,
            Some((0.48, vec![vec![g, e, d, b], vec![g, f, h, d, b]])),
        ),
        (
            g,
            c,
            Some((0.6, vec![vec![g, e, d, b, c], vec![g, f, h, d, b, c]])),
        ),
        (g, d, Some((0.36, vec![vec![g, e, d], vec![g, f, h, d]]))),
        (g, e, Some((0.12, vec![vec![g, e]]))),
        (g, f, Some((0.12, vec![vec![g, f]]))),
        (g, g, Some((0.0, vec![vec![]]))),
        (g, h, Some((0.24, vec![vec![g, f, h]]))),
        // h
        (h, a, Some((0.36, vec![vec![h, d, b, a]]))),
        (h, b, Some((0.24, vec![vec![h, d, b]]))),
        (h, c, Some((0.36, vec![vec![h, d, b, c]]))),
        (h, d, Some((0.12, vec![vec![h, d]]))),
        (h, e, Some((0.24, vec![vec![h, f, e]]))),
        (h, f, Some((0.12, vec![vec![h, f]]))),
        (h, g, None),
        (h, h, Some((0.0, vec![vec![]]))),
    ]
}

/// Consider a path from left to right.
/// It is important to have the smaller hop-distance at the bottom-path,
/// but the smaller weight-distance at the top-path.
fn expected_paths_bait() -> Vec<(TestNode, TestNode, Option<(f32, Vec<Vec<TestNode>>)>)> {
    // (idx, id)
    // ll left
    // bb bottom
    // rr right
    // tr top-right
    // tl top-left
    let ll = TestNode {
        idx: NodeIdx(0),
        id: 0,
    };
    let bb = TestNode {
        idx: NodeIdx(1),
        id: 1,
    };
    let rr = TestNode {
        idx: NodeIdx(2),
        id: 2,
    };
    let tr = TestNode {
        idx: NodeIdx(3),
        id: 3,
    };
    let tl = TestNode {
        idx: NodeIdx(4),
        id: 4,
    };

    vec![
        // ll
        (ll, ll, Some((0.0, vec![vec![]]))),
        (ll, bb, Some((0.6, vec![vec![ll, bb]]))),
        (ll, rr, Some((1.08, vec![vec![ll, tl, tr, rr]]))),
        (ll, tr, Some((0.72, vec![vec![ll, tl, tr]]))),
        (ll, tl, Some((0.36, vec![vec![ll, tl]]))),
        // bb
        (bb, ll, Some((0.6, vec![vec![bb, ll]]))),
        (bb, bb, Some((0.0, vec![vec![]]))),
        (bb, rr, Some((0.6, vec![vec![bb, rr]]))),
        (bb, tr, Some((0.96, vec![vec![bb, rr, tr]]))),
        (bb, tl, Some((0.96, vec![vec![bb, ll, tl]]))),
        // rr
        (rr, ll, Some((1.08, vec![vec![rr, tr, tl, ll]]))),
        (rr, bb, Some((0.6, vec![vec![rr, bb]]))),
        (rr, rr, Some((0.0, vec![vec![]]))),
        (rr, tr, Some((0.36, vec![vec![rr, tr]]))),
        (rr, tl, Some((0.72, vec![vec![rr, tr, tl]]))),
        // tr
        (tr, ll, Some((0.72, vec![vec![tr, tl, ll]]))),
        (tr, bb, Some((0.96, vec![vec![tr, rr, bb]]))),
        (tr, rr, Some((0.36, vec![vec![tr, rr]]))),
        (tr, tr, Some((0.0, vec![vec![]]))),
        (tr, tl, Some((0.36, vec![vec![tr, tl]]))),
        // tl
        (tl, ll, Some((0.36, vec![vec![tl, ll]]))),
        (tl, bb, Some((0.96, vec![vec![tl, ll, bb]]))),
        (tl, rr, Some((0.72, vec![vec![tl, tr, rr]]))),
        (tl, tr, Some((0.36, vec![vec![tl, tr]]))),
        (tl, tl, Some((0.0, vec![vec![]]))),
    ]
}

fn expected_paths_isle_of_man() -> Vec<(TestNode, TestNode, Option<(f32, Vec<Vec<TestNode>>)>)> {
    unimplemented!("Testing routing on isle-of-man is not supported yet.")
}
