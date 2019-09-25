# rsat
Local Search SAT and MaxSAT Solver in Rust based on [probSAT](https://github.com/adrianopolus/probSAT).
Partial MaxSAT is not supported yet.

[![Crates.io](https://img.shields.io/crates/v/rsat.svg)](https://crates.io/crates/rsat)
[![Crates.io](https://img.shields.io/crates/d/rsat.svg)](https://crates.io/crates/rsat)
![Crates.io](https://img.shields.io/crates/l/rsat)
[![Build Status](https://dev.azure.com/solhop/rsat/_apis/build/status/solhop.rsat?branchName=master)](https://dev.azure.com/solhop/rsat/_build/latest?definitionId=1&branchName=master)

## Install and Run

### Install

```sh
$ cargo install rsat
```

### Run

```sh
$ rsat input.cnf --max-tries=100 --max-flips=1000
```

where `input.cnf` contains the input SAT instance to be solved in DIMACS format.

### Example input

```
c comment
p cnf 3 4
1 0
-1 -2 0
2 -3 0
-3 0
```

### Example Output

```
SAT
1 -2 -3 0
```

## License

[MIT](LICENSE)
