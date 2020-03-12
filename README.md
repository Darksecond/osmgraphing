# osmgraphing

[![Build Status nightly][github/self/actions/badge]][github/self/actions]

[![Tag][github/self/tags/badge]][github/self/tags]
[![Crates.io][crates.io/self/badge]][crates.io/self]
[![Docs][docs.rs/self/badge]][docs.rs/self]

[![Changelog][github/self/blob/changelog/badge]][github/self/blob/changelog]
[![Last commit][github/self/last-commit/badge]][github/self/last-commit]

[![License][github/self/license/badge]][github/self/license]

Welcome to the `osmgraphing`-repo! `:)`
Goal of this repo is parsing [openstreetmap][osm]-data to calculate traffic-routes and different related use-cases on it.
This repo deals with analyzing selfish routing and learning metrics for balancing load in street-networks.
All calculations should be done effectively on a single desktop instead of an expensive cluster.


## Setup and usage

`cargo` is the build-tool of Rust and can be used to run everything except scripts in `scripts/`.
`cargo run` will give you help, e.g. it tells you to use `cargo run --example`.
Running this command will print names of runnable examples.
Further, refer to the [examples][github/self/tree/examples] for more details, or to [cargo-docs][docs.rs/self] to get details about the repo's setup and implementation.

Downloaded osm-data is provided in xml (`osm`) or binary (`pbf`), where nodes are related to location in latitude and longitude.
Problems will be the size-limit when downloading from [openstreetmap][osm], but there are other osm data providers like [geofabrik][geofabrik] for instance.

For testing, some simple text-based format `fmi` is used.
Since they are created manually for certain tasks, parsing them - generally speaking - is unstable.
However, this repository has a generator, which can create such `fmi`-files from `pbf`- or other `fmi`-files (for different metric-order).
The binary `mapgenerator` (binaries are in `target/release` after release-building) helps with generating proper config-files, but have a look at `resources/configs/blueprint` to get further explanations.
A tool for creating `fmi`-map-files, which contain graphs contracted via contraction-hierarchies, is [multi-ch-constructor][github/lesstat/multi-ch-constructor].


## Requirements for large maps

In general, the requirements depend on the size of the parsed map and your machine.
Following numbers base on an __8-core-CPU__ and the `pbf`-map `Germany` running on `archlinux`.
Further, they base on the assumption, that you don't use more than 4 metrics (besides ignore and ids), because up to 4 metrics are inlined with `SmallVec`.
You should change the number of inlined metrics according to your needs in the module `defaults`, because, during build-phase, the memory is allocated anyways.

- Parsing `Germany` (~50 million nodes, ~103 million edges, pbf-file) needs around __10 GB of RAM__.
  (Using only one metric, inlined, the memory-peak is slightly over __8 GB__.)
  After parsing, the memory-needs are lower due to the optimized graph-structure.
- Preprocessing `Germany` (including parsing) needs around __4 minutes__.
  This highly depends on the number of cores.
- A __routing query__ on `Germany` of length `620 km` takes around __16 seconds__ with `bidirectional Dijkstra`.
  This could be improved by removing intermediate nodes (like `b` in `a->b->c`), but they are kept for now.
  An `Astar` is not used anymore, because its only purpose is reducing the search-space, which can be reduced much more using `Contraction Hierarchies`.
  Further, `Astar` has issues when it comes to multiple or custom metrics, because of the metrics' heuristics.

Small maps like `Isle of Man` run on every machine and are parsed in less than a second.


## Contraction-Hierarchies

For speedup, this repository supports graphs contracted via contraction-hierarchies.
The repository [`lesstat/multi-ch-constructor`][github/lesstat/multi-ch-constructor] generates contracted graphs from `fmi`-files of a certain format.
This repository, `osmgraphing`, uses the `lesstat/multi-ch-constructor/master`-branch (commit `bec548c1a1ebeae7ac19d3250d5473199336d6fe`) for its ch-graphs.
For reproducability, the used steps are listed below.

First of all, the tool `multi-ch` needs a `fmi`-map-file as input.
To generate such a `fmi`-map-file in the right format, the `mapgenerator` of `osmgraphing` can be used with the `generator-config` shown below, following the [defined requirements][github/lesstat/cyclops/blob/README].

```yaml
generator:
  map-file: 'resources/maps/isle-of-man_2019-09-05.ch.fmi'
  nodes:
  - category: 'NodeIdx'
    id: 'internal id'
  - category: 'NodeId'
    id: 'osmid'
  - category: 'Latitude'
  - category: 'Longitude'
  - category: 'Ignore' # height
  - category: 'Level' # for contraction-hierarchies
  edges:
  - id: 'SrcId'
  - id: 'DstId'
  - id: 'Meters'
  - id: 'KilometersPerHour'
  - id: 'Seconds'
```

The `multi-ch`-tool needs 3 counts at the file-beginning: metric-count, node-count, edge-count.

Before the `multi-ch`-tool can be used, it has to be built.
For the sake of optimization, you have to set the metric-count as dimension in [multi-ch-constructor/src/multi_lib/graph.hpp, line 49][github/lesstat/multi-ch-constructor/change-dim].

```zsh
git clone --recursive https://github.com/lesstat/multi-ch-constructor
cd multi-ch-constructor

cmake -Bbuild
cmake --build build

./build/multi-ch --text path/to/fmi/graph --percent 99.85 --stats --write path/to/new/fmi/graph
```


## Credits

The project started in the mid of 2019 as a student project.
This page honors the workers and helpers of this project, sorted by their last names.

__[Florian Barth][github/lesstat]__  
is the supervisor of the project since beginning and is always helping immediately with his experience and advice.

__[Dominic Parga Cacheiro][github/dominicparga]__  
has been part of the project's first weeks when project-planning and learning Rust was on the scope.
He continues the work and is writing and improving the simulation.

__[Jena Satkunarajan][github/jenasat]__  
has been part of the project's first weeks when project-planning and learning Rust was on the scope.
He has implemented the first (and running) approach of the `A*`-algorithm.


[crates.io/self]: https://crates.io/crates/osmgraphing
[crates.io/self/badge]: https://img.shields.io/crates/v/osmgraphing?style=for-the-badge
[docs.rs/self]: https://docs.rs/osmgraphing/0/
[docs.rs/self/badge]: https://img.shields.io/crates/v/osmgraphing?color=informational&label=docs&style=for-the-badge
[geofabrik]: https://geofabrik.de
[github/dominicparga]: https://github.com/dominicparga
[github/jenasat]: https://github.com/JenaSat
[github/lesstat]: https://github.com/lesstat
[github/lesstat/cyclops/blob/README]: https://github.com/Lesstat/cyclops/blob/master/README.md#graph-data
[github/lesstat/multi-ch-constructor]: https://github.com/Lesstat/multi-ch-constructor
[github/lesstat/multi-ch-constructor/change-dim]: https://github.com/Lesstat/multi-ch-constructor/blob/bec548c1a1ebeae7ac19d3250d5473199336d6fe/src/multi_lib/graph.hpp#L49
[github/self/actions]: https://github.com/dominicparga/osmgraphing/actions
[github/self/actions/badge]: https://img.shields.io/github/workflow/status/dominicparga/osmgraphing/Rust?label=nightly-build&style=for-the-badge
[github/self/blob/changelog]: https://github.com/dominicparga/osmgraphing/blob/nightly/CHANGELOG.md
[github/self/blob/changelog/badge]: https://img.shields.io/badge/CHANGELOG-nightly-blueviolet?style=for-the-badge
[github/self/last-commit]: https://github.com/dominicparga/osmgraphing/commits
[github/self/last-commit/badge]: https://img.shields.io/github/last-commit/dominicparga/osmgraphing?style=for-the-badge
[github/self/license]: https://github.com/dominicparga/osmgraphing/blob/nightly/LICENSE
[github/self/license/badge]: https://img.shields.io/github/license/dominicparga/osmgraphing?style=for-the-badge
[github/self/tags]: https://github.com/dominicparga/osmgraphing/tags
[github/self/tags/badge]: https://img.shields.io/github/v/tag/dominicparga/osmgraphing?sort=semver&style=for-the-badge
[github/self/tree/examples]: https://github.com/dominicparga/osmgraphing/tree/nightly/examples
[github/self/wiki/usage]: https://github.com/dominicparga/osmgraphing/wiki/Usage
[osm]: https://openstreetmap.org
