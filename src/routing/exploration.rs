// LU-decomposition because of
// https://math.stackexchange.com/questions/1720806/lu-decomposition-vs-qr-decomposition-for-similar-problems
//
// https://crates.io/crates/nalgebra

use crate::{
    approximating::Approx,
    configs,
    defaults::{self, capacity::DimVec},
    helpers::{self, algebra},
    network::{Graph, NodeIdx},
    routing::{
        dijkstra::{self, Dijkstra},
        paths::Path,
    },
};
use log::{trace, warn};
use nd_triangulation::Triangulation;
use smallvec::smallvec;
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

// needed because convex-hull has dim+1 points per cell
type CHDimVec<T> = smallvec::SmallVec<[T; defaults::capacity::SMALL_VEC_INLINE_SIZE + 1]>;

struct Query<'a> {
    src_idx: NodeIdx,
    dst_idx: NodeIdx,
    graph: &'a Graph,
    routing_cfg: configs::routing::Config,
    graph_dim: usize,
    triangulation_dim: usize,
    tolerances: DimVec<f64>,
    is_metric_considered: DimVec<bool>,
}

impl<'a> Query<'a> {
    fn with(query: dijkstra::Query<'a>) -> Query<'a> {
        // init query

        let src_idx = query.src_idx;
        let dst_idx = query.dst_idx;
        let graph = query.graph;
        let routing_cfg = query.routing_cfg.clone();

        // config and stuff
        let graph_dim = graph.metrics().dim();
        // Every cost-value has to be below this value.
        let tolerances: DimVec<_> = smallvec![defaults::routing::TOLERATED_SCALE_INF; graph_dim];
        // don't consider ignored metrics
        let is_metric_considered: DimVec<_> = routing_cfg
            .alphas
            .iter()
            .map(|alpha| alpha > &0.0)
            .collect();
        trace!("is_metric_considered: {:?}", is_metric_considered);

        Query {
            src_idx,
            dst_idx,
            graph,
            routing_cfg,
            graph_dim,
            triangulation_dim: is_metric_considered
                .iter()
                .filter(|&&is_considered| is_considered)
                .count(),
            tolerances,
            is_metric_considered,
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct VertexId(usize);

impl Deref for VertexId {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

#[derive(Clone)]
struct Vertex<'a> {
    pub id: VertexId,
    pub path: &'a Path,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct CellId(pub usize);

impl Deref for CellId {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

#[derive(Clone)]
struct Cell<'a> {
    id: CellId,
    vertices: CHDimVec<Vertex<'a>>,
}

impl<'a> Cell<'a> {
    pub fn id(&self) -> &CellId {
        &self.id
    }

    pub fn vertices(&self) -> &CHDimVec<Vertex<'a>> {
        &self.vertices
    }
}

pub struct ConvexHullExplorator {
    found_paths: HashMap<VertexId, Path>,
    tolerated_found_paths: Vec<VertexId>,
    visited_cells: HashSet<CellId>,
}

impl ConvexHullExplorator {
    pub fn new() -> ConvexHullExplorator {
        ConvexHullExplorator {
            found_paths: HashMap::new(),
            tolerated_found_paths: Vec::new(),
            visited_cells: HashSet::new(),
        }
    }

    // TODO cap exploration with epsilon for routing-costs (1 + eps) * costs[i]
    //
    // New paths of a facet are linear-combinations of its defining paths
    // -> could not be better than the best of already defined paths

    pub fn fully_explorate(
        &mut self,
        query: dijkstra::Query,
        dijkstra: &mut Dijkstra,
    ) -> Vec<Path> {
        // init query

        let mut query = Query::with(query);

        let mut triangulation = Triangulation::new(query.triangulation_dim);
        let mut is_triangulation_dirty = false;

        self.found_paths.clear();
        self.tolerated_found_paths.clear();
        self.visited_cells.clear();
        let mut new_found_paths = Vec::new();
        ConvexHullExplorator::explore_initial_paths(&mut new_found_paths, &mut query, dijkstra);
        self.update(
            &query,
            &mut is_triangulation_dirty,
            &mut new_found_paths,
            &mut triangulation,
        );

        // explore

        // +1 because a convex-hull (volume) needs dim+1 points
        // For imagination:
        // - line vs triangle in 2D
        // - triangle vs tetrahedron in 3D
        if query.triangulation_dim > 1
            && self.found_paths.len() + new_found_paths.len() > query.triangulation_dim
        {
            // find new routes

            trace!(
                "Start exploring new alternative routes, because triangulation of dim {} is ready.",
                query.triangulation_dim
            );
            trace!("Use tolerances {:?}", query.tolerances);
            while is_triangulation_dirty {
                trace!("Found {} paths yet.", self.found_paths.len());
                for raw_cell in triangulation.convex_hull_cells() {
                    // don't look at cells twice
                    if self.visited_cells.contains(&CellId(raw_cell.id())) {
                        trace!(
                            "Jump over already explored cell of cell-id {}",
                            raw_cell.id()
                        );
                        continue;
                    }

                    let cell = ConvexHullExplorator::cell_from(raw_cell, &self.found_paths);
                    self.visited_cells.insert(*cell.id());

                    // A correct convex-hull implies following statements.
                    // (1) For every dimension, new paths can't be better
                    //     than any cell's path with best cost in this dimension.
                    // (2) On the other hand, no path can be best in all dimension
                    //     (due to the convex-hull's pareto-front).
                    // -> If any dimension has only costs worse than the dimension's tolerance,
                    //    there is no way to get a better path from the exploration.
                    // -> Don't look deeper in this cell.
                    //
                    if query
                        // For every tolerance...
                        .tolerances
                        .iter()
                        .enumerate()
                        // ...check if it is considered at all,
                        // which is unnecessary here, because unconsidered metrics
                        // have a tolerance of infinity.
                        // .filter(|(dim_i, _tolerance)| query.is_metric_considered[*dim_i])
                        // If tolerance is considered, check if any tolerance can't be undercut
                        // by any path's cost.
                        .any(|(dim_i, tolerance)| {
                            // So test here, if the given tolerance can be undercut by any cost.
                            // If not, this cell should not be considered.
                            !cell
                                .vertices()
                                .iter()
                                .map(|vertex| vertex.path.costs()[dim_i])
                                .any(|dim_cost| &dim_cost <= tolerance)
                        })
                    {
                        trace!(
                            "{}{}{}",
                            "Jump over cell (id: ",
                            **cell.id(),
                            "), that can't undercut at least one tolerance."
                        );
                        continue;
                    }
                    trace!("Explore cell of cell-id {}", **cell.id());

                    // Check candidate, whether it's shape is already sharp enough.
                    // This is done by computing the normal-vector for facets of the convex hull,
                    // which is the alpha-vector resulting from the linear system below.
                    // If Dijkstra finds a better path for this alpha-vector,
                    // the path's cost is part of the convex-hull.

                    let (rows, b) = if let Some((rows, b)) =
                        ConvexHullExplorator::create_linear_system(&cell, &query)
                    {
                        (rows, b)
                    } else {
                        warn!("graph-dim:  {}", query.graph_dim);
                        warn!(
                            "considered: {}",
                            query
                                .is_metric_considered
                                .iter()
                                .filter(|&ism| *ism)
                                .count()
                        );
                        warn!("cell-verts: {}", cell.vertices().len());
                        warn!(
                            "{}{}",
                            "The linear system has less rows than the convex-hull has dimensions.",
                            "This doesn't lead to a unique solution.",
                        );
                        continue;
                    };

                    // calculate alphas
                    query.routing_cfg.alphas =
                        if let Some(x) = algebra::Matrix::from_rows(rows).lu().solve(&b) {
                            x
                        } else {
                            continue;
                        };
                    trace!("alphas = {:?}", query.routing_cfg.alphas);
                    for (i, vertex) in cell.vertices().iter().enumerate() {
                        // for i in 0..candidate.len() {
                        trace!(
                            "alphas * path_{}.costs() = {:?}",
                            i,
                            helpers::dot_product(&query.routing_cfg.alphas, vertex.path.costs(),)
                        );
                    }

                    // find new path with new alpha

                    if let Some(mut best_path) = dijkstra.compute_best_path(dijkstra::Query {
                        src_idx: query.src_idx,
                        dst_idx: query.dst_idx,
                        graph: query.graph,
                        routing_cfg: &query.routing_cfg,
                    }) {
                        best_path.calc_costs(query.graph);
                        let new_path = best_path;

                        let new_alpha_cost =
                            helpers::dot_product(&query.routing_cfg.alphas, new_path.costs());
                        trace!("alphas * new_path.costs() = {:?}", new_alpha_cost);
                        // take any vertex, since alpha is chosen s.t. all dot-products are equal
                        let any_alpha_cost = helpers::dot_product(
                            &query.routing_cfg.alphas,
                            cell.vertices()[0].path.costs(),
                        );

                        // Add new path if it's cost-vector's projection onto the alpha-vector
                        // is smaller.

                        let is_path_new = Approx(new_alpha_cost) < Approx(any_alpha_cost)
                            && !new_found_paths.contains(&new_path);
                        if is_path_new {
                            trace!("Push {}", new_path);
                            new_found_paths.push(new_path);
                        } else {
                            trace!("Already found path {}", new_path);
                        }
                    } else {
                        trace!("No path found");
                    }
                }

                self.update(
                    &query,
                    &mut is_triangulation_dirty,
                    &mut new_found_paths,
                    &mut triangulation,
                );
            }
        }

        // if paths were found but no one is tolerated
        if self.found_paths.len() > 0 && self.tolerated_found_paths.len() == 0 {
            warn!(
                "{}{}{}{}{}",
                "Exploration found paths from src-id ",
                query.graph.nodes().id(query.src_idx),
                " to dst-id ",
                query.graph.nodes().id(query.dst_idx),
                ", but should not tolerate any path. Maybe your tolerances are too tight?"
            );
        }

        let mut result = Vec::with_capacity(self.tolerated_found_paths.len());
        for vertex_id in &self.tolerated_found_paths {
            result.push(
                self.found_paths
                    .remove(vertex_id)
                    .expect("A tolerated found path should have been found."),
            )
        }
        result

        // self.found_paths
        //     .drain()
        //     .map(|(_vertex_id, path)| path)
        //     .filter_map(|path| {
        //         if Approx(path.costs()) <= Approx(&query.tolerances) {
        //             Some(path)
        //         } else {
        //             None
        //         }
        //     })
        //     .collect()
    }

    fn explore_initial_paths(
        new_found_paths: &mut Vec<Path>,
        query: &mut Query,
        dijkstra: &mut Dijkstra,
    ) {
        // find initial convex-hull
        // -> go through all combinations, where at least one alpha-entry is > 0.0
        // -> at least d+1 points for dimension d
        // -> at least all points from lower (e.g. "previous") dimensions

        let mut init_alphas: CHDimVec<_> = CHDimVec::new();

        // create imc-mask from is_metric_considered
        // rev() is important, because vectors grow from left and integers from right
        let imc_mask = query
            .is_metric_considered
            .iter()
            .rev()
            .fold(0, |acc, &digit| 2 * acc + if digit { 1 } else { 0 });

        // if mask is a power of 2 (e.g. 2==0x10, e.g. not 6==0x110)
        // -> metric-idx should be set
        let is_pow_of_2 = |mask: u32| mask & (mask - 1) == 0;
        let mut metric_idx = 0;

        // this whole loop is checked with a rust-playground-example:
        // https://gist.github.com/dominicparga/069c014eb3a0c2cf655d4d89ae4e7391
        for mask in 1..2u32.pow(query.graph_dim as u32) {
            // this if-clause causes to discard masks, that have a 1 where imc_mask is 0
            if ((imc_mask | mask) ^ imc_mask) == 0 {
                // parse mask into vector of 0.0 and 1.0
                let alphas = (0..query.graph_dim)
                    .map(|idx| ((mask >> idx) & 1) as f64)
                    .collect();

                if is_pow_of_2(mask) {
                    init_alphas.push((Some(metric_idx), alphas));
                    metric_idx += 1;
                } else {
                    init_alphas.push((None, alphas));
                }
            } else if is_pow_of_2(mask) {
                metric_idx += 1;
            }
        }

        // add all init-alphas' paths

        let mut found_paths = CHDimVec::new();
        for (metric_idx, alphas) in init_alphas {
            trace!("Trying init-alpha {:?}", alphas);

            query.routing_cfg.alphas = alphas;
            if let Some(mut best_path) = dijkstra.compute_best_path(dijkstra::Query {
                src_idx: query.src_idx,
                dst_idx: query.dst_idx,
                graph: query.graph,
                routing_cfg: &query.routing_cfg,
            }) {
                best_path.calc_costs(query.graph);

                // Remember tolerated costs for filtering in the end.
                // The costs have to be checked in the end, since this iterative algorithm could
                // find a tolerated path by using an unacceptable path.

                if let Some(metric_idx) = metric_idx {
                    if query.routing_cfg.tolerated_scales[metric_idx] == std::f64::INFINITY {
                        query.tolerances[metric_idx] = std::f64::INFINITY;
                    } else {
                        // NaN when 0.0 * inf
                        query.tolerances[metric_idx] = best_path.costs()[metric_idx]
                            * query.routing_cfg.tolerated_scales[metric_idx];
                    }
                }

                if !found_paths
                    .iter()
                    .map(|path: &Path| path.costs())
                    .any(|costs| Approx(costs) == Approx(best_path.costs()))
                {
                    trace!("Found and pushing init-path {}", best_path);
                    found_paths.push(best_path);
                }
            }
        }

        for path in found_paths {
            new_found_paths.push(path);
        }
    }

    fn cell_from<'a>(
        cell: nd_triangulation::Cell,
        found_paths: &'a HashMap<VertexId, Path>,
    ) -> Cell<'a> {
        Cell {
            id: CellId(cell.id()),
            vertices: cell
                .vertices()
                .into_iter()
                .map(|vertex| VertexId(vertex.id()))
                .map(|vertex_id| Vertex {
                    id: vertex_id,
                    path: found_paths.get(&vertex_id).expect(
                        "For every vertex in the triangulation, a path should be registered.",
                    ),
                })
                .collect(),
        }
    }

    fn create_linear_system(
        cell: &Cell,
        query: &Query,
    ) -> Option<(DimVec<DimVec<f64>>, DimVec<f64>)> {
        trace!("Create linear system with paths:");
        for vertex in cell.vertices() {
            trace!("  {}", vertex.path);
        }

        // Solve LGS to get alpha, where all cell-vertex-costs (personalized with alpha)
        // are equal.
        // -> Determine rows of matrix

        let mut rows = DimVec::new();
        let mut b = DimVec::new();

        // all lines describe the equality of each dot-product between cost-vector and alpha
        let vertex_0 = &cell.vertices()[0];
        for vertex_i in &cell.vertices()[1..] {
            rows.push(helpers::sub(vertex_0.path.costs(), vertex_i.path.costs()));
            b.push(0.0);
        }

        // but ignored metrics should lead to zero alpha
        for (i, _) in query
            .is_metric_considered
            .iter()
            .enumerate()
            .filter(|&(_, imc)| !imc)
        {
            // set [0, ..., 0, 1, 0, ..., 0] to 0.0
            let mut row = smallvec![0.0; query.graph_dim];
            row[i] = 1.0;
            rows.push(row);
            b.push(0.0);
        }

        // if one condition is missing (depending on convex-hull-implementation),
        match query.graph_dim - rows.len() {
            0 => (),
            1 => {
                // you could normalize alpha
                // -> one row in matrix is 1.0

                rows.push(smallvec![1.0; query.graph_dim]);
                b.push(1.0);
            }
            _ => return None,
        }

        trace!("rows = {:?}", rows);
        trace!("b = {:?}", b);
        Some((rows, b))
    }

    fn update(
        &mut self,
        query: &Query,
        is_triangulation_dirty: &mut bool,
        new_found_paths: &mut Vec<Path>,
        triangulation: &mut Triangulation,
    ) {
        trace!(
            "Updating triangulation with {} new found paths.",
            new_found_paths.len()
        );
        *is_triangulation_dirty = new_found_paths.len() > 0;

        // add new paths to triangulation
        // but only with considered metrics

        for path in new_found_paths.drain(..) {
            let new_raw_id = triangulation
                .add_vertex(
                    &path
                        .costs()
                        .iter()
                        .enumerate()
                        .filter_map(|(i, c)| {
                            if query.is_metric_considered[i] {
                                Some(*c)
                            } else {
                                None
                            }
                        })
                        .collect::<DimVec<_>>(),
                )
                .expect("Path's cost should have right dimension.");
            let new_id = VertexId(new_raw_id);

            // Remember path if it can be returned in the end.
            if Approx(path.costs()) <= Approx(&query.tolerances) {
                self.tolerated_found_paths.push(new_id);
            }

            self.found_paths.insert(new_id, path);
        }
        debug_assert!(
            new_found_paths.is_empty(),
            "All new found paths should be added by now."
        );
        trace!(
            "Triangulation is {}dirty.",
            if *is_triangulation_dirty { "" } else { "not " }
        );
    }
}
