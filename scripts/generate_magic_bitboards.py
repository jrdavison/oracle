from collections import namedtuple
import os
import struct
import time

from typing import Iterator, List, Dict, Tuple

SAVE_PATH = os.path.join(os.path.dirname(__file__), "..", "data")

BishopDbs = namedtuple("BishopDbs", ["mask", "blockers"])

HORIZONTAL_MASK = 0xFF
VERTICAL_MASK = 0x0101010101010101

KNIGHT_DIRECTIONS = [
    (2, 1),  # NNW
    (2, -1),  # NNE
    (-2, 1),  # SSW
    (-2, -1),  # SSE
    (1, 2),  # NEW
    (-1, 2),  # SEW
    (1, -2),  # NWW
    (-1, -2),  # SWW
]

KING_DIRECTIONS = [
    (1, 0),  # N
    (1, 1),  # NE
    (0, 1),  # E
    (-1, 1),  # SE
    (-1, 0),  # S
    (-1, -1),  # SW
    (0, -1),  # W
    (1, -1),  # NW
]


def get_square(r: int, f: int) -> int:
    return r * 8 + f


def get_rank(sq: int) -> int:
    return sq // 8


def get_file(sq: int) -> int:
    return sq % 8


def mask_rook_attacks(sq: int) -> int:
    h_mask = HORIZONTAL_MASK << get_rank(sq) * 8
    v_mask = VERTICAL_MASK << get_file(sq)

    # clear square that the rook is on
    return (h_mask | v_mask) & ~(1 << sq)


def bishop_attacks(sq: int, blockers: int) -> int:
    attacks = 0
    rank = get_rank(sq)
    file = get_file(sq)

    # NE
    for r, f in zip(range(rank + 1, 8), range(file + 1, 8)):
        attacks |= 1 << get_square(r, f)
        if blockers & (1 << get_square(r, f)):
            break
    # SE
    for r, f in zip(range(rank - 1, -1, -1), range(file + 1, 8)):
        attacks |= 1 << get_square(r, f)
        if blockers & (1 << get_square(r, f)):
            break
    # SW
    for r, f in zip(range(rank - 1, -1, -1), range(file - 1, -1, -1)):
        attacks |= 1 << get_square(r, f)
        if blockers & (1 << get_square(r, f)):
            break
    # NW
    for r, f in zip(range(rank + 1, 8), range(file - 1, -1, -1)):
        attacks |= 1 << get_square(r, f)
        if blockers & (1 << get_square(r, f)):
            break

    return attacks


def rook_attacks(sq: int, blockers: int) -> int:
    attacks = 0
    rank = get_rank(sq)
    file = get_file(sq)

    # E
    for f in range(file + 1, 8):
        attacks |= 1 << get_square(rank, f)
        if blockers & (1 << get_square(rank, f)):
            break
    # W
    for f in range(file - 1, -1, -1):
        attacks |= 1 << get_square(rank, f)
        if blockers & (1 << get_square(rank, f)):
            break

    # N
    for r in range(rank + 1, 8):
        attacks |= 1 << get_square(r, file)
        if blockers & (1 << get_square(r, file)):
            break
    # S
    for r in range(rank - 1, -1, -1):
        attacks |= 1 << get_square(r, file)
        if blockers & (1 << get_square(r, file)):
            break

    return attacks


def jumping_attacks(sq: int, directions: List[Tuple[int, int]]) -> int:
    attacks = 0
    rank = get_rank(sq)
    file = get_file(sq)

    for direction in directions:
        r, f = rank + direction[0], file + direction[1]
        if 0 <= r < 8 and 0 <= f < 8:
            attacks |= 1 << get_square(r, f)
    return attacks


def generate_relevant_blockers(mask: int) -> Iterator[int]:
    relevant_bits = [i for i in range(64) if mask & (1 << i)]
    num_relevant_bits = len(relevant_bits)
    for index in range(1 << num_relevant_bits):
        blockers = 0
        for i in range(num_relevant_bits):
            if index & (1 << i):
                blockers |= 1 << relevant_bits[i]
        yield blockers


def generate_rook_move_database() -> List[Dict[int, int]]:
    print("Generating rook move database...")
    rook_moves = [{} for _ in range(64)]
    for sq in range(64):
        start_time = time.perf_counter()
        mask = mask_rook_attacks(sq)
        for blockers in generate_relevant_blockers(mask):
            attacks = rook_attacks(sq, blockers)
            rook_moves[sq][blockers] = attacks
        print(
            f"Computed {len(rook_moves[sq])} moves for square {sq}. Done in {time.perf_counter() - start_time:.2f} seconds."
        )
    return rook_moves


def generate_bishop_move_databases() -> BishopDbs:
    print("Generating bishop move databases...")
    bishop_moves = [{} for _ in range(64)]
    diagonal_masks = [0 for _ in range(64)]
    for sq in range(64):
        start_time = time.perf_counter()
        mask = bishop_attacks(sq, 0)
        diagonal_masks[sq] = mask
        print(f"Diagonal mask for square {sq}:")
        print_bitboard(mask)
        for blockers in generate_relevant_blockers(mask):
            attacks = bishop_attacks(sq, blockers)
            bishop_moves[sq][blockers] = attacks
        print(
            f"Computed {len(bishop_moves)} moves for square {sq}. Done in {time.perf_counter() - start_time:.2f} seconds."
        )
    return BishopDbs(mask=diagonal_masks, blockers=bishop_moves)


def generate_knight_move_database() -> List[int]:
    knight_moves = [0 for _ in range(64)]
    for sq in range(64):
        knight_moves[sq] = jumping_attacks(sq, KNIGHT_DIRECTIONS)
    print("Computed knight moves.")
    return knight_moves


def generate_king_move_database() -> List[int]:
    king_moves = [0 for _ in range(64)]
    for sq in range(64):
        king_moves[sq] = jumping_attacks(sq, KING_DIRECTIONS)
    print("Computed king moves.")
    return king_moves


def save_blockers_move_db(filename: str, move_db: List[Dict[int, int]]) -> None:
    full_path = os.path.join(SAVE_PATH, filename)
    with open(full_path, "wb") as f:
        for square in range(64):
            num_entries = len(move_db[square])
            f.write(
                struct.pack("I", num_entries)
            )  # Write number of entries for this square
            for blockers, attacks in move_db[square].items():
                f.write(struct.pack("Q", blockers))  # Write the blocker bitboard
                f.write(struct.pack("Q", attacks))  # Write the attack bitboard


def save_simple_move_db(filename: str, move_db: List[int]) -> None:
    full_path = os.path.join(SAVE_PATH, filename)
    with open(full_path, "wb") as f:
        for move in move_db:
            f.write(struct.pack("Q", move))


def print_bitboard(bb: int) -> None:
    for rank in range(7, -1, -1):
        for file in range(8):
            if bb & (1 << (rank * 8 + file)):
                print("1", end=" ")
            else:
                print("0", end=" ")
        print()
    print()


rook_move_database = generate_rook_move_database()
save_blockers_move_db("rook_moves.bin", rook_move_database)

knight_move_database = generate_knight_move_database()
save_simple_move_db("knight_moves.bin", knight_move_database)

king_move_database = generate_king_move_database()
save_simple_move_db("king_moves.bin", king_move_database)

bishop_move_databases = generate_bishop_move_databases()
save_blockers_move_db("bishop_moves.bin", bishop_move_databases.blockers)
save_simple_move_db("diagonal_masks.bin", bishop_move_databases.mask)
