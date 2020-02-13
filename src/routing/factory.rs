pub mod astar {
    pub mod unidirectional {
        use crate::{
            network,
            network::{HalfEdge, Node},
            routing::astar::{unidirectional::GenericAstar, Astar},
            units::{geo, length::Meters, speed::KilometersPerHour, time::Milliseconds},
        };

        pub fn shortest() -> Box<dyn Astar<Meters>> {
            let cost_fn = |edge: &HalfEdge| edge.meters().unwrap();
            let estimate_fn =
                |from: &Node, to: &Node| geo::haversine_distance_m(&from.coord(), &to.coord());
            Box::new(GenericAstar::new(cost_fn, estimate_fn))
        }

        pub fn fastest() -> Box<dyn Astar<Milliseconds>> {
            let cost_fn = |edge: &HalfEdge| edge.milliseconds().unwrap();
            let estimate_fn = |from: &Node, to: &Node| {
                let meters = geo::haversine_distance_m(&from.coord(), &to.coord());
                let maxspeed: KilometersPerHour = (network::defaults::MAX_SPEED_KMH as u16).into();
                meters / maxspeed
            };
            Box::new(GenericAstar::new(cost_fn, estimate_fn))
        }
    }

    pub mod bidirectional {
        use crate::{
            network,
            network::{HalfEdge, Node},
            routing::astar::{bidirectional::GenericAstar, Astar},
            units::{geo, length::Meters, speed::KilometersPerHour, time::Milliseconds},
        };

        pub fn shortest() -> Box<dyn Astar<Meters>> {
            let cost_fn = |edge: &HalfEdge| edge.meters().unwrap();
            let estimate_fn =
                |from: &Node, to: &Node| geo::haversine_distance_m(&from.coord(), &to.coord());
            Box::new(GenericAstar::new(cost_fn, estimate_fn))
        }

        pub fn fastest() -> Box<dyn Astar<Milliseconds>> {
            let cost_fn = |edge: &HalfEdge| edge.milliseconds().unwrap();
            let estimate_fn = |from: &Node, to: &Node| {
                let meters = geo::haversine_distance_m(&from.coord(), &to.coord());
                let maxspeed: KilometersPerHour = (network::defaults::MAX_SPEED_KMH as u16).into();
                meters / maxspeed
            };
            Box::new(GenericAstar::new(cost_fn, estimate_fn))
        }
    }
}

pub mod dijkstra {
    pub mod unidirectional {
        use crate::{
            network::HalfEdge,
            routing::dijkstra::{unidirectional::GenericAstar, Astar},
            units::{length::Meters, time::Milliseconds},
        };

        pub fn shortest() -> Box<dyn Astar<Meters>> {
            let cost_fn = |edge: &HalfEdge| edge.meters().unwrap();
            Box::new(GenericAstar::new(cost_fn))
        }

        pub fn fastest() -> Box<dyn Astar<Milliseconds>> {
            // length [m] / velocity [km/h]
            let cost_fn = |edge: &HalfEdge| edge.milliseconds().unwrap();
            Box::new(GenericAstar::new(cost_fn))
        }
    }

    pub mod bidirectional {
        use crate::{
            network::HalfEdge,
            routing::dijkstra::{bidirectional::GenericAstar, Astar},
            units::{length::Meters, time::Milliseconds},
        };

        pub fn shortest() -> Box<dyn Astar<Meters>> {
            let cost_fn = |edge: &HalfEdge| edge.meters().unwrap();
            Box::new(GenericAstar::new(cost_fn))
        }

        pub fn fastest() -> Box<dyn Astar<Milliseconds>> {
            // length [m] / velocity [km/h]
            let cost_fn = |edge: &HalfEdge| edge.milliseconds().unwrap();
            Box::new(GenericAstar::new(cost_fn))
        }
    }
}
