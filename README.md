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

We precompute attacks for knights, kings, pawns, bishops, and rooks. The attacks are loaded into look up table in rust and used to get valid attack squares in one-shot

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

### Types

```rust
type BlockersTable = [HashMap<Bitboard, Bitboard>; Square::Count as usize];
type AttackMaskTable = [Bitboard; Square::Count as usize];
```

- `AttackMaskTable`:
  - **key** - Square on a chessboard
  - **value** - Valid moves for that square
- `BlockersTable`:
  - **key** - square on a chessboard
  - **value** - Hashmap with the following
    - **key** - all relevant blockers on the board for the piece
    - **value** - valid moves based on the blocking pieces

### Jumping Pieces

#### Knights

**Type:** `AttackMaskTable`

Valid attacks for  knight `n` on square C7 (`KNIGHT_MASKS[50]`):

```text
8: 1 0 0 0 1 0 0 0
7: 0 0 n 0 0 0 0 0
6: 1 0 0 0 1 0 0 0
5: 0 1 0 1 0 0 0 0
4: 0 0 0 0 0 0 0 0
3: 0 0 0 0 0 0 0 0
2: 0 0 0 0 0 0 0 0
1: 0 0 0 0 0 0 0 0
   A B C D E F G H
```

#### Kings

**Type:** `AttackMaskTable`

Just like knights, king attacks are also stored in `AttackMaskTable` type.

Valid attacks for king `k` on square B5 (`KING_MASKS[33]`):

```text
8: 0 0 0 0 0 0 0 0
7: 0 0 0 0 0 0 0 0
6: 1 1 1 0 0 0 0 0
5: 1 k 1 0 0 0 0 0
4: 1 1 1 0 0 0 0 0
3: 0 0 0 0 0 0 0 0
2: 0 0 0 0 0 0 0 0
1: 0 0 0 0 0 0 0 0
   A B C D E F G H
```

#### Pawns

**Type:** `AttackMaskTable`

We store valid pawn attacks for each color for simplicity.

Valid attacks for white pawn `p` on D4 (`PAWN_ATTACK_MASKS[Color::White as usize][27]`):

```text
8: 0 0 0 0 0 0 0 0
7: 0 0 0 0 0 0 0 0
6: 0 0 0 0 0 0 0 0
5: 0 0 1 0 1 0 0 0
4: 0 0 0 p 0 0 0 0
3: 0 0 0 0 0 0 0 0
2: 0 0 0 0 0 0 0 0
1: 0 0 0 0 0 0 0 0
   A B C D E F G H
```

Valid attacks for black pawn `p` on D4 (`PAWN_ATTACK_MASKS[Color::Black as usize][27]`):

```text
8: 0 0 0 0 0 0 0 0
7: 0 0 0 0 0 0 0 0
6: 0 0 0 0 0 0 0 0
5: 0 0 0 0 0 0 0 0
4: 0 0 0 p 0 0 0 0
3: 0 0 1 0 1 0 0 0
2: 0 0 0 0 0 0 0 0
1: 0 0 0 0 0 0 0 0
   A B C D E F G H
```

### Sliding Pieces

We precompute diagonal and orthogonal masks ahead of time so we don't have to have a loop to iterate in all possible directions. This allows us to get the sliding masks for a piece in 1 shot. We can also ignore the edges of the board in most cases when generating these masks. Doing so will greatly decrease the number of keys we need in each hashmap in the `BlockersTable`. Ignored edges will be represented by `-`s in the following examples.

#### Diagonal Masks

**Type:** `AttackMaskTable`

Diagonal mask for piece `x` on square D1 (`DIAGONAL_MASKS[4]`):

```text
8: - - - - - - - -
7: - 0 0 0 0 0 0 -
6: - 0 0 0 0 0 0 -
5: - 0 0 0 0 0 0 -
4: - 0 0 0 0 0 1 -
3: - 1 0 0 0 1 0 -
2: - 0 1 0 1 0 0 -
1: - 0 0 x 0 0 0 -
   A B C D E F G H
```

#### Orthogonal Masks

**Type:** `AttackMaskTable`

Orthogonal mask for piece `x` on square E6 (`ORTHOGONAL_MASKS[44]`):

```text
8: - - - - - - - -
7: - 0 0 0 1 0 0 -
6: - 1 1 1 x 1 1 -
5: - 0 0 0 1 0 0 -
4: - 0 0 0 1 0 0 -
3: - 0 0 0 1 0 0 -
2: - 0 0 0 1 0 0 -
1: - - - - - - - -
   A B C D E F G H
```

#### Rooks

**Type:** `BlockersTable`

Valid attacks for rook on square D6 with the following checkers bitboard representing all pieces on the board:

```text
8: 1 1 0 0 0 1 1 1
7: 1 0 1 0 1 0 0 0
6: 1 0 0 1 0 1 1 0
5: 0 1 0 0 1 0 0 1
4: 0 0 1 0 0 0 1 1
3: 0 0 1 1 0 0 0 0
2: 0 0 1 0 0 0 1 1
1: 0 1 1 0 1 0 0 1
   A B C D E F G H
```

First we need to get the orthogonal mask for the square the rook is on.

Orthogonal mask for rook `r` on square D6 (`ORTHOGONAL_MASKS[43]`):

```text
8: - - - - - - - -
7: - 0 0 1 0 0 0 -
6: - 1 1 r 1 1 1 -
5: - 0 0 1 0 0 0 -
4: - 0 0 1 0 0 0 -
3: - 0 0 1 0 0 0 -
2: - 0 0 1 0 0 0 -
1: - - - - - - - -
   A B C D E F G H
```

Then we bitwise `&` the checkers bitboard and orthogonal mask together to get a bitboard representing all blocking pieces

```text
Checkers Bitboard:         Orthogonal Mask:           Relevant blockers
8: 1 1 0 0 0 1 1 1         8: 0 0 0 0 0 0 0 0         8: 0 0 0 0 0 0 0 0
7: 1 0 1 0 1 0 0 0         7: 0 0 0 1 0 0 0 0         7: 0 0 0 0 0 0 0 0
6: 1 0 0 1 0 1 1 0         6: 0 1 1 0 1 1 1 0         6: 0 1 0 0 0 1 1 0
5: 0 1 0 0 1 0 0 1         5: 0 0 0 1 0 0 0 0         5: 0 0 0 0 0 0 0 0
4: 0 0 1 0 0 0 1 1    &    4: 0 0 0 1 0 0 0 0    =    4: 0 0 0 0 0 0 0 0
3: 0 0 1 1 0 0 0 0         3: 0 0 0 1 0 0 0 0         3: 0 0 0 1 0 0 0 0
2: 0 0 1 0 0 0 1 1         2: 0 0 0 1 0 0 0 0         2: 0 0 0 0 0 0 0 0
1: 0 1 1 1 1 0 0 1         1: 0 0 0 0 0 0 0 0         1: 0 0 0 0 0 0 0 0
   A B C D E F G H            A B C D E F G H            A B C D E F G H
```

Then we use the relevant blockers bitboard as the key to the hashmap at the D6 square to get the valid moves (`ROOK_BLOCKERS_LOOKUP[43][blockers]`):

```text
8: 0 0 0 1 0 0 0 0
7: 0 0 0 1 0 0 0 0
6: 1 1 1 r 1 1 0 0
5: 0 0 0 1 0 0 0 0
4: 0 0 0 1 0 0 0 0
3: 0 0 0 1 0 0 0 0
2: 0 0 0 0 0 0 0 0
1: 0 0 0 0 0 0 0 0
   A B C D E F G H
```

#### Bishops

**Type:** `BlockersTable`

Valid attacks for rook on square E8 with the following checkers bitboard representing all pieces on the board:

```text
8: 1 1 0 0 1 1 1 1
7: 1 0 1 0 1 0 0 0
6: 1 0 0 1 0 1 1 0
5: 0 1 0 0 1 0 0 1
4: 0 0 1 0 0 0 1 1
3: 0 0 1 1 0 0 0 0
2: 0 0 1 0 0 0 1 1
1: 0 1 1 0 1 0 0 1
   A B C D E F G H
```

First we need to get the orthogonal mask for the square the rook is on.

Orthogonal mask for bishop `b` on square D6 (`ORTHOGONAL_MASKS[60]`):

```text
8: - 0 0 0 b 0 0 -
7: - 0 0 1 0 1 0 -
6: - 0 1 0 0 0 1 -
5: - 1 0 0 0 0 0 -
4: - 0 0 0 0 0 0 -
3: - 0 0 0 0 0 0 -
2: - 0 0 0 0 0 0 -
1: - - - - - - - -
   A B C D E F G H
```

Then we bitwise `&` the checkers bitboard and orthogonal mask together to get a bitboard representing all blocking pieces

```text
Checkers Bitboard:         Orthogonal Mask:           Relevant blockers
8: 1 1 0 0 1 1 1 1         8: 0 0 0 0 0 0 0 0         8: 0 0 0 0 0 0 0 0
7: 1 0 1 0 1 0 0 0         7: 0 0 0 1 0 1 0 0         7: 0 0 0 0 0 0 0 0
6: 1 0 0 1 0 1 1 0         6: 0 0 1 0 0 0 1 0         6: 0 0 0 0 0 0 1 0
5: 0 1 0 0 1 0 0 1         5: 0 1 0 0 0 0 0 0         5: 0 1 0 0 0 0 0 0
4: 0 0 1 0 0 0 1 1    &    4: 0 0 0 0 0 0 0 0    =    4: 0 0 0 0 0 0 0 0
3: 0 0 1 1 0 0 0 0         3: 0 0 0 0 0 0 0 0         3: 0 0 0 0 0 0 0 0
2: 0 0 1 0 0 0 1 1         2: 0 0 0 0 0 0 0 0         2: 0 0 0 0 0 0 0 0
1: 0 1 1 1 1 0 0 1         1: 0 0 0 0 0 0 0 0         1: 0 0 0 0 0 0 0 0
   A B C D E F G H            A B C D E F G H            A B C D E F G H
```

Then we use the relevant blockers bitboard as the key to the hashmap at the D6 square to get the valid moves (`ROOK_BLOCKERS_LOOKUP[43][blockers]`):

```text
8: 0 0 0 0 0 0 0 0
7: 0 0 0 1 0 1 0 0
6: 0 0 1 0 0 0 1 0
5: 0 1 0 0 0 0 0 0
4: 0 0 0 0 0 0 0 0
3: 0 0 0 0 0 0 0 0
2: 0 0 0 0 0 0 0 0
1: 0 0 0 0 0 0 0 0
   A B C D E F G H
```

#### Queens

The method for computing queen moves is to compute the [rook moves](#rooks) and [bishop moves](#bishops) and then bitwise `OR` them together.
