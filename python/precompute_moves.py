import os
import struct
import time
from collections import namedtuple

import perfection

from typing import Iterator, List, Dict, Tuple

SAVE_PATH = os.path.join(os.path.dirname(__file__), "..", "data")

MaskBlockerDbs = namedtuple("MaskBlockerDbs", ["masks", "blockers"])

HORIZONTAL_MASK = 0xFF
VERTICAL_MASK = 0x0101010101010101
FRIST_RANK_MASK = 0xFF
LAST_RANK_MASK = 0xFF00000000000000
FIRST_FILE_MASK = 0x0101010101010101
LAST_FILE_MASK = 0x8080808080808080

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
    h_mask = HORIZONTAL_MASK << (get_rank(sq) * 8)
    v_mask = VERTICAL_MASK << get_file(sq)
    attack_mask = h_mask | v_mask

    if get_rank(sq) != 0:
        attack_mask &= ~FRIST_RANK_MASK
    if get_rank(sq) != 7:
        attack_mask &= ~LAST_RANK_MASK
    if get_file(sq) != 0:
        attack_mask &= ~FIRST_FILE_MASK
    if get_file(sq) != 7:
        attack_mask &= ~LAST_FILE_MASK

    # Combine horizontal and vertical, and clear the rook's square
    return attack_mask & ~(1 << sq)


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


def generate_rook_attack_db() -> MaskBlockerDbs:
    print("Generating rook move database...")
    rook_moves = [{} for _ in range(64)]
    horizontal_masks = [0 for _ in range(64)]
    for sq in range(64):
        start_time = time.perf_counter()
        mask = mask_rook_attacks(sq)
        horizontal_masks[sq] = mask
        for blockers in generate_relevant_blockers(mask):
            attacks = rook_attacks(sq, blockers)
            rook_moves[sq][blockers] = attacks
        print(
            f"Computed {len(rook_moves[sq])} moves for square {sq}. Done in {time.perf_counter() - start_time:.2f} seconds."
        )

    return MaskBlockerDbs(masks=horizontal_masks, blockers=rook_moves)


def generate_bishop_attack_dbs() -> MaskBlockerDbs:
    print("Generating bishop move databases...")
    bishop_moves = [{} for _ in range(64)]
    diagonal_masks = [0 for _ in range(64)]
    for sq in range(64):
        start_time = time.perf_counter()
        mask = bishop_attacks(sq, 0)
        diagonal_masks[sq] = mask
        for blockers in generate_relevant_blockers(mask):
            attacks = bishop_attacks(sq, blockers)
            bishop_moves[sq][blockers] = attacks
        print(
            f"Computed {len(bishop_moves)} moves for square {sq}. Done in {time.perf_counter() - start_time:.2f} seconds."
        )
    return MaskBlockerDbs(masks=diagonal_masks, blockers=bishop_moves)


def generate_knight_attack_db() -> List[int]:
    knight_moves = [0 for _ in range(64)]
    for sq in range(64):
        knight_moves[sq] = jumping_attacks(sq, KNIGHT_DIRECTIONS)
    print("Computed knight moves.")
    return knight_moves


def generate_king_attack_db() -> List[int]:
    king_moves = [0 for _ in range(64)]
    for sq in range(64):
        king_moves[sq] = jumping_attacks(sq, KING_DIRECTIONS)
    print("Computed king moves.")
    return king_moves


def save_blockers_db(filename: str, move_db: List[Dict[int, int]]) -> None:
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


def save_attack_db(filename: str, move_db: List[int]) -> None:
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


rook_move_dbs = generate_rook_attack_db()
save_blockers_db("rook_moves.bin", rook_move_dbs.blockers)
save_attack_db("horizontal_vertical_masks.bin", rook_move_dbs.masks)

knight_move_db = generate_knight_attack_db()
save_attack_db("knight_moves.bin", knight_move_db)

king_move_db = generate_king_attack_db()
save_attack_db("king_moves.bin", king_move_db)

bishop_move_dbs = generate_bishop_attack_dbs()
save_blockers_db("bishop_moves.bin", bishop_move_dbs.blockers)
save_attack_db("diagonal_masks.bin", bishop_move_dbs.masks)
