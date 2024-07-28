import time
import struct

from typing import Iterator, List, Dict


HORIZONTAL_MASK = 0xFF
VERTICAL_MASK = 0x0101010101010101
DIAGONAL_MASK = 0x8040201008040201


def get_square(r: int, f: int) -> int:
    return r * 8 + f


def get_rank(sq: int) -> int:
    return sq // 8


def get_file(sq: int) -> int:
    return sq % 8


def mask_rook_attacks(sq: int):
    h_mask = HORIZONTAL_MASK << get_rank(sq) * 8
    v_mask = VERTICAL_MASK << get_file(sq)

    # clear square that the rook is on
    return (h_mask | v_mask) & ~(1 << sq)


def mask_bishop_attacks(sq: int):
    rank = get_rank(sq)
    file = get_file(sq)
    diag1 = DIAGONAL_MASK << (rank - min(rank, file)) * 8 + (file - min(rank, file))
    diag2 = DIAGONAL_MASK << (rank - min(rank, 7 - file)) * 8 + (
        file + min(rank, 7 - file)
    )

    # clear square that the bishop is on
    return (diag1 | diag2) & ~(1 << sq)


def rook_attacks(sq: int, blockers: int):
    attacks = 0
    rank = sq // 8
    file = sq % 8

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


# def generate_bishop_move_database() -> List[Dict[int, int]]:
#     bishop_moves = [{} for _ in range(64)]
#     for sq in range(64):
#         start_time = time.perf_counter()
#         mask = mask_bishop_attacks(sq)
#         for blockers in generate_relevant_blockers(mask):
#             attacks = bishop_attacks(sq, blockers)
#             bishop_moves[sq][blockers] = attacks
#         print(
#             f"Computed {len(bishop_moves[sq])} moves for square {sq}. Done in {time.perf_counter() - start_time:.2f} seconds."
#         )
#     return bishop_moves


def save_move_database(filename, rook_move_database):
    with open(filename, "wb") as f:
        for square in range(64):
            num_entries = len(rook_move_database[square])
            f.write(
                struct.pack("I", num_entries)
            )  # Write number of entries for this square
            for blockers, attacks in rook_move_database[square].items():
                f.write(struct.pack("Q", blockers))  # Write the blocker bitboard
                f.write(struct.pack("Q", attacks))  # Write the attack bitboard


def print_bitboard(bb: int) -> None:
    for rank in range(7, -1, -1):
        for file in range(8):
            if bb & (1 << (rank * 8 + file)):
                print("1", end=" ")
            else:
                print("0", end=" ")
        print()
    print()


print_bitboard(mask_bishop_attacks(1))
print_bitboard(mask_bishop_attacks(8))

# rook_move_database = generate_rook_move_database()
# save_move_database("../resources/rook_moves.bin", rook_move_database)
