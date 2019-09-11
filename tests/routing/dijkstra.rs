use osmgraphing::network::Graph;
use osmgraphing::routing::dijkstra;

//--------------------------------------------------------------------------------------------//
// helpers

fn idx(node: (usize, i64)) -> usize {
    node.0
}

fn id(node: (usize, i64)) -> i64 {
    node.1
}

//--------------------------------------------------------------------------------------------//
// test-path

struct TestPath {
    src: (usize, i64),
    dst: (usize, i64),
    cost: u32,
    alternative_nodes: Vec<Vec<(usize, i64)>>,
}
impl TestPath {
    fn from(src: (usize, i64), dst: (usize, i64), cost: u32, nodes: Vec<(usize, i64)>) -> TestPath {
        TestPath {
            src,
            dst,
            cost,
            alternative_nodes: vec![nodes],
        }
    }

    fn from_alternatives(
        src: (usize, i64),
        dst: (usize, i64),
        cost: u32,
        alternative_nodes: Vec<Vec<(usize, i64)>>,
    ) -> TestPath {
        TestPath {
            src,
            dst,
            cost,
            alternative_nodes,
        }
    }

    fn assert(&self, path: &dijkstra::Path, graph: &Graph) {
        let node = |idx: usize| -> (usize, i64) { (idx, graph.node(idx).id()) };

        //----------------------------------------------------------------------------------------//
        // check meta-info

        let path_src = node(path.src_idx());
        assert_eq!(
            path_src, self.src,
            "Path has wrong src (idx, id)={:?} (should be {:?})",
            path_src, self.src,
        );
        let path_dst = node(path.dst_idx());
        assert_eq!(
            path_dst, self.dst,
            "Path has wrong dst (idx, id)={:?} (should be {:?})",
            path_dst, self.dst,
        );
        assert_eq!(
            path.cost(),
            self.cost,
            "Path from src-id {} to dst-id {} should have cost={}",
            id(self.src),
            id(self.dst),
            self.cost,
        );

        //----------------------------------------------------------------------------------------//
        // check predecessors/successors

        // src has no predecessor
        assert_eq!(
            path.predecessor(idx(self.src)),
            None,
            "Predecessor of src-idx {} should be None",
            idx(self.src)
        );
        // dst has no successor
        assert_eq!(
            path.successor(idx(self.dst)),
            None,
            "Predecessor of dst-idx {} should be None",
            idx(self.dst)
        );

        let mut is_pred_eq = false;
        let mut is_succ_eq = false;
        for nodes in &self.alternative_nodes {
            if nodes.len() > 0 {
                // predecessor-path
                let mut current = path_dst;
                let mut pred_path = vec![current];
                while let Some(pred) = path.predecessor(idx(current)) {
                    let pred = node(pred);
                    pred_path.push(pred);
                    current = pred;
                }
                pred_path.reverse();
                is_pred_eq |= &pred_path == nodes;

                // successor-path
                let mut current = path_src;
                let mut succ_path = vec![current];
                while let Some(succ) = path.successor(idx(current)) {
                    let succ = node(succ);
                    succ_path.push(succ);
                    current = succ;
                }
                is_succ_eq |= &succ_path == nodes;
            } else {
                is_pred_eq = true;
                is_succ_eq = true;
            }
        }
        assert!(
            is_pred_eq,
            "Predecessor-path from src={:?} to dst={:?} is wrong.",
            self.src, self.dst
        );
        assert!(
            is_succ_eq,
            "Successor-path from src={:?} to dst={:?} is wrong.",
            self.src, self.dst
        );
    }
}

//--------------------------------------------------------------------------------------------//
// tests

#[test]
fn small() {
    // (idx, id)
    let a = (0, 0);
    let b = (1, 1);
    let c = (2, 2);
    let d = (3, 3);
    let e = (4, 4);
    let f = (5, 5);
    let g = (6, 6);
    let h = (7, 7);

    let test_paths = vec![
        // a
        (a, a, Some(TestPath::from(a, a, 0, vec![]))),
        (a, b, None),
        (a, c, None),
        (a, d, None),
        (a, e, None),
        (a, f, None),
        (a, g, None),
        (a, h, None),
        // b
        (b, a, Some(TestPath::from(b, a, 1, vec![b, a]))),
        (b, b, Some(TestPath::from(b, b, 0, vec![]))),
        (b, c, Some(TestPath::from(b, c, 1, vec![b, c]))),
        (b, d, None),
        (b, e, None),
        (b, f, None),
        (b, g, None),
        (b, h, None),
        // c
        (c, a, Some(TestPath::from(c, a, 1, vec![c, a]))),
        (c, b, Some(TestPath::from(c, b, 1, vec![c, b]))),
        (c, c, Some(TestPath::from(c, c, 0, vec![]))),
        (c, d, None),
        (c, e, None),
        (c, f, None),
        (c, g, None),
        (c, h, None),
        // d
        (d, a, Some(TestPath::from(d, a, 2, vec![d, b, a]))),
        (d, b, Some(TestPath::from(d, b, 1, vec![d, b]))),
        (d, c, Some(TestPath::from(d, c, 2, vec![d, b, c]))),
        (d, d, Some(TestPath::from(d, d, 0, vec![]))),
        (d, e, Some(TestPath::from(d, e, 2, vec![d, e]))),
        (d, f, Some(TestPath::from(d, f, 2, vec![d, h, f]))),
        (d, g, None),
        (d, h, Some(TestPath::from(d, h, 1, vec![d, h]))),
        // e
        (e, a, Some(TestPath::from(e, a, 4, vec![e, d, b, a]))),
        (e, b, Some(TestPath::from(e, b, 3, vec![e, d, b]))),
        (e, c, Some(TestPath::from(e, c, 4, vec![e, d, b, c]))),
        (e, d, Some(TestPath::from(e, d, 2, vec![e, d]))),
        (e, e, Some(TestPath::from(e, e, 0, vec![]))),
        (e, f, Some(TestPath::from(e, f, 1, vec![e, f]))),
        (e, g, None),
        (e, h, Some(TestPath::from(e, h, 2, vec![e, f, h]))),
        // f
        (f, a, Some(TestPath::from(f, a, 4, vec![f, h, d, b, a]))),
        (f, b, Some(TestPath::from(f, b, 3, vec![f, h, d, b]))),
        (f, c, Some(TestPath::from(f, c, 4, vec![f, h, d, b, c]))),
        (f, d, Some(TestPath::from(f, d, 2, vec![f, h, d]))),
        (f, e, Some(TestPath::from(f, e, 1, vec![f, e]))),
        (f, f, Some(TestPath::from(f, f, 0, vec![]))),
        (f, g, None),
        (f, h, Some(TestPath::from(f, h, 1, vec![f, h]))),
        // g
        (g, a, Some(TestPath::from(g, a, 5, vec![g, e, d, b, a]))),
        (g, b, Some(TestPath::from(g, b, 4, vec![g, e, d, b]))),
        (g, c, Some(TestPath::from(g, c, 5, vec![g, e, d, b, c]))),
        (
            g,
            d,
            Some(TestPath::from_alternatives(
                g,
                d,
                3,
                vec![vec![g, e, d], vec![g, f, d]],
            )),
        ),
        (g, e, Some(TestPath::from(g, e, 1, vec![g, e]))),
        (g, f, Some(TestPath::from(g, f, 1, vec![g, f]))),
        (g, g, Some(TestPath::from(g, g, 0, vec![]))),
        (g, h, Some(TestPath::from(g, h, 2, vec![g, f, h]))),
        // h
        (h, a, Some(TestPath::from(h, a, 3, vec![h, d, b, a]))),
        (h, b, Some(TestPath::from(h, b, 2, vec![h, d, b]))),
        (h, c, Some(TestPath::from(h, c, 3, vec![h, d, b, c]))),
        (h, d, Some(TestPath::from(h, d, 1, vec![h, d]))),
        (h, e, Some(TestPath::from(h, e, 2, vec![h, f, e]))),
        (h, f, Some(TestPath::from(h, f, 1, vec![h, f]))),
        (h, g, None),
        (h, h, Some(TestPath::from(h, h, 0, vec![]))),
    ];

    let graph = super::parse("resources/maps/small.fmi");

    for (src, dst, option_test_path) in test_paths {
        let option_path = dijkstra::compute_shortest_path(id(src), id(dst), &graph);
        assert_eq!(
            option_path.is_some(),
            option_test_path.is_some(),
            "Path from (idx,id)={:?} to (idx,id)={:?} should be {}",
            src,
            dst,
            if option_test_path.is_some() {
                "Some"
            } else {
                "None"
            }
        );

        if let (Some(test_path), Some(path)) = (option_test_path, option_path) {
            test_path.assert(&path, &graph);
        }
    }
}
