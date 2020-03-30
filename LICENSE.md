# License and Copyright

As long as you are not building with or depending on the build-feature `gpl-3.0`, the `Apache-2.0`-license holds.

## Apache-2.0

This repository `osmgraphing` parses maps of own format or maps from
openstreetmap. To do this, a binary `osmgraphing` is built and an own-
defined config-file can be provided to specify parser- and routing-
settings. Besides that, a binary `mapgenerator` is built and can be used
to generate map-files.

The description above gives a (maybe uncomplete) overview about the part of this repository (and resulting binaries), which is licensed under the `Apache License, Version 2.0`.
You may not use content of this repository or its files, which are directly or indirectly related to above mentioned parts, except in compliance with the `Apache License, Version 2.0`.
You may obtain a copy of the License at

`https://www.apache.org/licenses/LICENSE-2.0`

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and limitations under the License.


## GPL-3.0

Besides that, this repository also explorates graphs and optimizes resulting routes with the module `exploration` and the binary `balancer`.
Since these parts of this repository use code, which is licensed under the `GPL-3.0`, every other module and binary, using it as described in the `GPL-3.0`, has to be licensed respectively under the `GPL-3.0`.
You may obtain a copy of the License at

`https://www.gnu.org/licenses/`


## Mentioning this Copyright in own projects

Please include this Copyright- and License-notice in your project.
As suggested in the `Apache License, Version 2.0`, you may choose a separate file like "NOTICE" for this.
Instead of copying the whole notice, you may copy only the following short version.

```text
osmgraphing, mapgenerator
https://github.com/dominicparga/osmgraphing
Copyright 2019 Dominic Parga Cacheiro
License Apache-2.0

explorator, balancer
https://github.com/dominicparga/osmgraphing
Copyright (C) 2020 Dominic Parga Cacheiro
License GPL-3.0-only
```


## Dependencies

```text
clap
https://github.com/clap-rs/clap
Copyright (c) 2015-2016 Kevin B. Knapp
License MIT AND Apache-2.0
```

```text
env_logger
https://github.com/sebasmagri/env_logger
Copyright (c) 2014 The Rust Project Developers
License MIT/Apache-2.0
```

```text
kissunits
https://github.com/dominicparga/kissunits
Copyright 2020 Dominic Parga Cacheiro
License Apache-2.0
```

```text
log
https://github.com/rust-lang/log
Copyright (c) 2014 The Rust Project Developers
License MIT OR Apache-2.0
```

```text
nd-triangulation
https://github.com/lesstat/nd-triangulation
License GPL-3.0-only
```

```text
osmpbfreader
https://github.com/TeXitoi/osmpbfreader-rs
License WTFPL WITH LGPLv3
```

```text
progressing
https://github.com/dominicparga/progressing
Copyright 2020 Dominic Parga Cacheiro
License Apache-2.0
```

```text
rand
rand_pcg
https://github.com/rust-random/rand
Copyright 2018 Developers of the Rand project
Copyright (c) 2014 The Rust Project Developers
License MIT OR Apache-2.0
```

```text
serde
https://github.com/serde-rs/serde
License MIT OR Apache-2.0
```

```text
serde_yaml
https://github.com/dtolnay/serde-yaml
License MIT OR Apache-2.0
```

```text
smallvec
https://github.com/servo/rust-smallvec
Copyright (c) 2018 The Servo Project Developers
License MIT/Apache-2.0
```
