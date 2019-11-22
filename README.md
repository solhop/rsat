# rsat

SolHop SAT Solver.

[![Crates.io](https://img.shields.io/crates/v/rsat.svg?style=for-the-badge)](https://crates.io/crates/rsat)
[![Crates.io](https://img.shields.io/crates/d/rsat.svg?style=for-the-badge)](https://crates.io/crates/rsat)
![Crates.io](https://img.shields.io/crates/l/rsat?style=for-the-badge)
[![Docs](https://img.shields.io/badge/api-docs-blue?style=for-the-badge)](https://docs.rs/rsat)
<!-- [![Build Status](https://dev.azure.com/solhop/rsat/_apis/build/status/solhop.rsat?branchName=master)](https://dev.azure.com/solhop/rsat/_build/latest?definitionId=1&branchName=master) -->
<!-- [![Coverage Status](https://coveralls.io/repos/github/solhop/rsat/badge.svg?branch=master)](https://coveralls.io/github/solhop/rsat?branch=master) -->

Currently, a stochastic local search based on probSAT and a CDCL solver based on MiniSAT has been implemented.
More algorithms will be available soon.

This projetct is still in development.
The APIs can change a lot before the first stable release v1.0.0.

## Install and Run

### Install

```sh
cargo install rsat
```

### Help

```sh
$ rsat --help
rsat 0.1.6
SolHOP SAT Solver

USAGE:
    rsat [FLAGS] [OPTIONS] <file>

FLAGS:
    -h, --help        Prints help information
    -p, --parallel    Enables data parallelism (currently only for sls solver)
    -V, --version     Prints version information

OPTIONS:
    -a, --alg <alg>                Algorithm to use (1 -> CDCL, 2 -> SLS) [default: 1]
        --max-flips <max-flips>    Maxinum number of flips in each try of SLS [default: 1000]
        --max-tries <max-tries>    Maximum number of tries for SLS [default: 100]

ARGS:
    <file>    Input file in DIMACS format
```

### Usage

```sh
rsat input.cnf -a 1
```

where `input.cnf` is the input SAT instance in DIMACS format.
Use `-a 2` to invoke the SLS solver.
Also see [help](#Help) for some options.

Below are some examples:

#### Example 1

Input

```txt
c comment
p cnf 3 4
1 0
-1 -2 0
2 -3 0
-3 0
```

Output

```txt
SAT
1 -2 -3 0
```

#### Example 2

Input

```txt
c comment
p cnf 3 4
1 0
-1 -2 0
2 -3 0
3 0
```

Output

```txt
UNSAT
```

Note: The SLS solver will never be available to prove UNSAT.
It will give the best model that has been found so far.

```txt
UNKNOWN
-1 2 3 0
```

## License

[MIT](LICENSE)
