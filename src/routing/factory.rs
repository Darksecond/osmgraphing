//------------------------------------------------------------------------------------------------//
// other modules

//------------------------------------------------------------------------------------------------//
// own modules

pub mod astar {
    use crate::network;
    use crate::network::{Edge, Node};
    use crate::units::geo;
    use crate::units::length::Meters;
    use crate::units::speed::KilometersPerHour;
    use crate::units::time::Milliseconds;

    use crate::routing::astar::GenericAstar;
    use crate::routing::Astar;

    pub fn shortest() -> Box<dyn Astar<Meters>> {
        let cost_fn = |edge: &Edge| edge.meters();
        let estimate_fn =
            |from: &Node, to: &Node| geo::haversine_distance_m(from.coord(), to.coord());
        Box::new(GenericAstar::from(cost_fn, estimate_fn))
    }

    pub fn fastest() -> Box<dyn Astar<Milliseconds>> {
        let cost_fn = |edge: &Edge| edge.milliseconds();
        let estimate_fn = |from: &Node, to: &Node| {
            let meters = geo::haversine_distance_m(from.coord(), to.coord());
            let maxspeed: KilometersPerHour = (network::defaults::MAX_SPEED_KMH as u16).into();
            meters / maxspeed
        };
        Box::new(GenericAstar::from(cost_fn, estimate_fn))
    }
}

pub mod dijkstra {
    use crate::network::{Edge, Node};
    use crate::units::length::Meters;
    use crate::units::time::Milliseconds;
    use crate::units::Metric;

    use crate::routing::astar::GenericAstar;
    use crate::routing::Astar;

    pub fn shortest() -> Box<dyn Astar<Meters>> {
        let cost_fn = |edge: &Edge| edge.meters();
        let estimate_fn = |_from: &Node, _to: &Node| Meters::zero();
        Box::new(GenericAstar::from(cost_fn, estimate_fn))
    }

    pub fn fastest() -> Box<dyn Astar<Milliseconds>> {
        // length [m] / velocity [km/h]
        let cost_fn = |edge: &Edge| edge.milliseconds();
        let estimate_fn = |_from: &Node, _to: &Node| Milliseconds::zero();
        Box::new(GenericAstar::from(cost_fn, estimate_fn))
    }
}
