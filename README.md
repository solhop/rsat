# rsat
Local Search SAT Solver in Rust based on [probSAT](https://github.com/adrianopolus/probSAT).

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

## License

[MIT](LICENSE)
