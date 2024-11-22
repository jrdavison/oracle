# Oracle Chess Agent

## About

A chess agent built using Rust.

## Usage

1. Run binary for development:

```bash
cargo run
```

2. Build optimized release binary:

```bash
cargo build --release
```

## Move Generation

We precompute attacks for knights, kings, bishops, and rooks. The attacks are loaded into look up table in rust and used to get valid attack squares in one-shot

### Precompute Magicbitboards

There are precomputed lookup tables in the `data/` folder. If you wish to recompute these do one of the following:

1. Run after build using `cargo`

```bash
cargo run -- --gen-magics
```

1. Using the executable

```bash
./oracle --gen-magics
```

### Knights

Knight attacks are stored in a `SimpleAttackDatabase` type. This is just a basic `[Bitboard; Square::Count as usize]` array where each index represents the valid attacks for that square.

Valid attacks for `KNIGHT_ATTACKS_DB[50]` (square: C7):

```text
1 0 0 0 1 0 0 0
0 0 0 0 0 0 0 0
1 0 0 0 1 0 0 0
0 1 0 1 0 0 0 0
0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0
```

### Kings

Just like knights, king attacks are also stored in `SimpleAttackDatabase` type.

Valid attacks for `KING_ATTACKS_DB[33]` (square: B5):

```text
0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0
1 1 1 0 0 0 0 0
1 0 1 0 0 0 0 0
1 1 1 0 0 0 0 0
0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0
```

### Diagonal Masks

Valid diagonal masks are stored in a `SimpleMoveDatabase`. This allows us to easily get the valid diagonals for a given square (useful for computing bishop moves)

Valid diagonal mask for `DIAGONAL_MASKS_DB[22]` (square: G3):

```text
0 1 0 0 0 0 0 0
0 0 1 0 0 0 0 0
0 0 0 1 0 0 0 0
0 0 0 0 1 0 0 0
0 0 0 0 0 1 0 1
0 0 0 0 0 0 0 0
0 0 0 0 0 1 0 1
0 0 0 0 1 0 0 0
```

### Rooks

Rook attacks are stored as a `BlockersAttackDatabase`. This is an array of 64 hashmaps. the hash key is the bitboard of all blocking pieces either horizontally or vertically aligned with the rook.

Computing moves for square B3, with a blockers key like this (some bitmasking is done to isolate only the file/rank the rook is on. represented by the `x`'s below):

```text
    x
  0 1 0 0 0 0 0 0
  0 0 0 0 0 0 0 0
  0 1 0 0 0 0 0 0
  0 0 0 0 0 0 0 0
  0 0 0 0 0 0 0 0
x 0 0 0 0 0 0 1 1
  0 1 0 0 0 0 0 0
  0 0 0 0 0 0 0 0
```

would result in valid attacks like this:

```text
    x
  0 0 0 0 0 0 0 0
  0 0 0 0 0 0 0 0
  0 1 0 0 0 0 0 0
  0 1 0 0 0 0 0 0
  0 1 0 0 0 0 0 0
x 0 0 1 1 1 1 1 0 x
  0 1 0 0 0 0 0 0
  0 0 0 0 0 0 0 0
    x
```

### Bishops

Just like rooks, bishops attacks are also stored as a `BlockersAttackDatabase`.

First we need to get the diagonal mask for the current square. This is used as the mask to get the blockers key for the diagonals the bishop is on.

Computing moves for square D4, with a blockers key like this (some bitmasking is done to isolate only the diagonals the bishop is on. represented by the `x`'s below):

```text
                  x
x 0 0 0 0 0 0 0 1
  1 0 0 0 0 0 0 0
  0 1 0 0 0 0 0 0
  0 0 0 0 1 0 0 0
  0 0 0 0 0 0 0 0
  0 0 0 0 0 0 0 0
  0 0 0 0 0 1 0 0
  0 0 0 0 0 0 0 0
x               x
```

would result in valid attacks like this:

```text
                  x
x 0 0 0 0 0 0 0 1
  0 0 0 0 0 0 0 0
  0 1 0 0 0 0 0 0
  0 0 1 0 1 0 0 0
  0 0 0 x 0 0 0 0
  0 0 1 0 1 0 0 0
  0 1 0 0 0 1 0 0
  1 0 0 0 0 0 0 0
x               x
```
