mod graph;
pub use graph::{
    building::{GraphBuilder, ProtoEdge, ProtoNode},
    EdgeContainer, EdgeIdx, Graph, HalfEdge, MetricContainer, MetricIdx, Node, NodeContainer,
    NodeIdx,
};
use serde::Deserialize;

//------------------------------------------------------------------------------------------------//

/// TODO
///
/// ## Street-types
///
/// Every edge will have a street-type with respective default speed-limit.
/// See [osm-wiki Key:highway](https://wiki.openstreetmap.org/wiki/Key:highway) for details and descriptions.
///
/// Accepted tags are listed below (sorted by descending priority) with respective default values.
/// International equivalents are depicted [here](https://wiki.openstreetmap.org/wiki/Highway:International_equivalence) and shortened/extended below.
///
/// `(*)` means the street type is allowed or possible but uncomfortable or unusual.
///
/// | street-type | Respective rural roads in Germany | Respective urban roads in Germany | default speed limit in km/h | for vehicles | for bicycles | for pedestrians |
/// |-|-|-|-:|:-:|:-:|:-:|
/// | Motorway | "Autobahn" | "Autobahn" | 130 | yes | no | no |
/// | MotorwayLink | | | 50 | yes | no | no |
/// | Trunk | "Schnellstraße" | "Schnellstraße" | 100 | yes | no | no |
/// | TrunkLink | | | 50 | yes | no | no |
/// | Primary | "Bundesstraße" (national roads) | Highest-level streets | 100 | yes | yes`(*)` | no |
/// | PrimaryLink | | | 30 | yes | yes`(*)` | no |
/// | Secondary | "Landesstraße" (regional roads) | Major streets | 70 | yes | yes`(*)` | no |
/// | SecondaryLink | | | 30 | yes | yes`(*)` | no |
/// | Tertiary | "Kreisstraße" (local roads) | Streets providing access to suburbs with priority | 70 | yes | yes | no |
/// | TertiaryLink | | | 30 | yes | yes | no  |
/// | Unclassified | Streets connecting towns | Industrial areas and providing access to neighborhoods without priority | 50 | yes | yes | no  |
/// | Residential | | Roads to access houses | 50 | yes | yes | yes |
/// | LivingStreet | | Pedestrians have right over cars | 15 | yes | yes | yes |
/// | Service | | Roads to something (e.g. a park) | 20 | yes`(*)` | yes | yes |
/// | Track | Roads mostly used for agricultural- or forestry-uses | Roads mostly used for agricultural- or forestry-uses | 30 | yes`(*)` | yes`(*)` | yes |
/// | Road | Undefined roads | Undefined roads | 50 | yes`(*)` | yes`(*)` | yes`(*)` |
/// | Cycleway | For cycles | For cycles | 25 | no | yes | no |
/// | Pedestrian | Mainly for pedestrians | Mainly for pedestrians | 5 | no | yes`(*)` | yes |
/// | Path | Non-specific path, e.g. for walkers | Non-specific path, e.g. for walkers | 15 | no | yes`(*)` | yes |
///
/// The mapping of given `key:value`-pairs to above street-types is too verbose to maintain it here in addition to the code.
/// Unknown snippets are printed with a warning and their respective id.
///
///
/// ## Speed-limit
///
/// The speed-limit is used in `km/h`, which is the provided unit by osm.
/// > Default: See table above
///
/// ## Length
///
/// The length is used in `km`, which is the provided unit by osm.
/// > Default: Calculated by coordinates of involved nodes.
///
///
/// ## Tag `oneway`
///
/// This tag seems to be very creative.
///
/// |              tag-value             | tag-value for default behavior |  way-ID  |
/// |:-----------------------------------|--------------------------------|---------:|
/// | `yes @ (2018 Aug 0 - 2018 Dec 21)` |              `yes`             | [24379239](https://www.openstreetmap.org/way/24379239) |
/// | `use_sidepath`                     |              `yes`             | [3701112](https://www.openstreetmap.org/way/3701112) |
/// | `alternating`                      |               `no`             | [5051072](https://www.openstreetmap.org/way/5051072) |
/// | `reversible`                       |               `no`             | [4005347](https://www.openstreetmap.org/way/4005347) |
/// | `bicycle`                          |               `no`             | [25596393](https://www.openstreetmap.org/way/25596393) |
/// | `recommended`                      |              `yes`             | [38250792](https://www.openstreetmap.org/way/38250792) |
/// | `-1;no`                            |               `-1`             | [180680762](https://www.openstreetmap.org/way/180680762) |
/// | `fixme`                            |               `no`             | [199388177](https://www.openstreetmap.org/way/199388177) |
/// | `left;through`                     |              `yes`             | [679817792](https://www.openstreetmap.org/way/679817792) |
/// | `undefined`                        |               `no`             | [331847642](https://www.openstreetmap.org/way/331847642) |
/// | `unknown`                          |               `no`             | [380885551](https://www.openstreetmap.org/way/380885551) |
/// | `shelter`                          |              `yes`             | [680612616](https://www.openstreetmap.org/way/680612616) |
/// | `cycle_barrier`                    |               `no`             | [691452957](https://www.openstreetmap.org/way/691452957) |
pub enum StreetCategory {
    Motorway,
    MotorwayLink,
    Trunk,
    TrunkLink,
    Primary,
    PrimaryLink,
    Secondary,
    SecondaryLink,
    Tertiary,
    TertiaryLink,
    Unclassified,
    Residential,
    LivingStreet,
    Service,
    Track,
    Road,
    Cycleway,
    Pedestrian,
    Path,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum VehicleCategory {
    Car,
    Bicycle,
    Pedestrian,
}
