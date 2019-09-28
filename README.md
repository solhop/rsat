# rsat

SolHop SAT and MaxSAT Solver.

Currently, a stochastic local search based on probSAT and a CDCL solver based on MiniSAT
has been implemented. More algorithms will be available soon.

[![Crates.io](https://img.shields.io/crates/v/rsat.svg)](https://crates.io/crates/rsat)
[![Crates.io](https://img.shields.io/crates/d/rsat.svg)](https://crates.io/crates/rsat)
![Crates.io](https://img.shields.io/crates/l/rsat)
[![Build Status](https://dev.azure.com/solhop/rsat/_apis/build/status/solhop.rsat?branchName=master)](https://dev.azure.com/solhop/rsat/_build/latest?definitionId=1&branchName=master)

## Install and Run

### Install

```sh
$ cargo install rsat
```

### Help

```sh
$ rsat --help
```

### Usage

```sh
$ rsat input.cnf -a 1
```

where `input.cnf` is the input SAT instance in DIMACS format.
Use `-a 2` to invoke the SLS solver.
Also see [help](#Help) for some options.

Below are some examples:

#### Example 1

##### Input

```
c comment
p cnf 3 4
1 0
-1 -2 0
2 -3 0
-3 0
```

##### Output

```
SAT
1 -2 -3 0
```

#### Example 2

##### Input

```
c comment
p cnf 3 4
1 0
-1 -2 0
2 -3 0
3 0
```

##### Output

```
UNSAT
```

Note: The SLS solver will never be available to prove UNSAT.
It will give the best model that has been found so far.

```
UNKNOWN
-1 2 3 0
```

## License

[MIT](LICENSE)
