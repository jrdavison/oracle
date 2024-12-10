# Oracle Chess Agent

## About

A chess agent built using Rust.

## Usage

Run binary for development:

```bash
cargo run
````

Build optimized release binary:

```bash
cargo build --release
```

## Move Generation

Attacks for knights, kings, pawns, bishops, and rooks are precomputed. The attacks are loaded into look up table in rust and used to get valid attack squares in one-shot

### Precompute Magicbitboards

There are precomputed lookup tables in the `data/` folder. If you wish to recompute these do one of the following:

Run after build using `cargo`:

```bash
cargo run -- --gen-magics
```

Using the executable:

```bash
./oracle --gen-magics
```