#ifndef BITBOARDS_H_
#define BITBOARDS_H_

#include <cstdint>
#include <iostream>
#include <string>

#include "position.h"
#include "utils.h"

namespace Oracle {

class Bitboards {
   public:
    Bitboards()  = default;
    ~Bitboards() = default;

    void init(const Position& pos);

    Bitboard is_valid_move(Square from, Square to) { return m_valid_moves[from] & (1ULL << to); }

   private:
    Bitboard m_valid_moves[SQUARE_NB] = {0};

    Bitboard compute_pawn_moves(Piece p, Square sq);
};

void print_bitboard(Bitboard bb, const std::string& label);

inline void set_bit(Bitboard& bb, Square sq) { bb |= (1ULL << sq); };
inline bool is_bit_set(Bitboard bb, Square sq) { return bb & (1ULL << sq); };

}  // namespace Oracle

#endif  // BITBOARDS_H_
