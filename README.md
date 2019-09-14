# rsat
Local Search SAT Solver in Rust based on [probSAT](https://github.com/adrianopolus/probSAT).

## Install and Run

### Install

```sh
$ cargo install rsat
```

### Run

```sh
$ slp input.cnf --max-tries=100 --max-flips=1000
```

where `input.cnf` contains the input SAT instance to be solved in DIMACS format.

## License

[MIT](LICENSE)
