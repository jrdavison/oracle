import time
import struct

from typing import Iterator, List, Dict


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


def rook_attacks(sq: int, blockers: int) -> int:
    attacks = 0
    rank = get_rank(sq)
    file = get_file(sq)

    # horizontal attack (east)
    for f in range(file + 1, 8):
        attacks |= 1 << get_square(rank, f)
        if blockers & (1 << get_square(rank, f)):
            break
    # horizontal attack (west)
    for f in range(file - 1, -1, -1):
        attacks |= 1 << get_square(rank, f)
        if blockers & (1 << get_square(rank, f)):
            break

    # vertical attack (north)
    for r in range(rank + 1, 8):
        attacks |= 1 << get_square(r, file)
        if blockers & (1 << get_square(r, file)):
            break
    # vertical attack (south)
    for r in range(rank - 1, -1, -1):
        attacks |= 1 << get_square(r, file)
        if blockers & (1 << get_square(r, file)):
            break

    return attacks


def knight_attacks(sq: int) -> int:
    attacks = 0
    rank = get_rank(sq)
    file = get_file(sq)

    for direction in KNIGHT_DIRECTIONS:
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


def generate_knight_move_database() -> List[int]:
    knight_moves = [0 for _ in range(64)]
    for sq in range(64):
        knight_moves[sq] = knight_attacks(sq)
    print("Computed knight moves.")
    return knight_moves


def save_rook_move_database(filename, rook_move_database) -> None:
    with open(filename, "wb") as f:
        for square in range(64):
            num_entries = len(rook_move_database[square])
            f.write(
                struct.pack("I", num_entries)
            )  # Write number of entries for this square
            for blockers, attacks in rook_move_database[square].items():
                f.write(struct.pack("Q", blockers))  # Write the blocker bitboard
                f.write(struct.pack("Q", attacks))  # Write the attack bitboard


def save_knight_move_database(filename, knight_move_database) -> None:
    with open(filename, "wb") as f:
        for move in knight_move_database:
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
save_rook_move_database("../data/rook_moves.bin", rook_move_database)

knight_move_database = generate_knight_move_database()
save_knight_move_database("../data/knight_moves.bin", knight_move_database)
