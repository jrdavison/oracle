#ifndef POSITION_H_INCLUDED
#define POSITION_H_INCLUDED

#include <algorithm>
#include <chrono>
#include <cstdint>
#include <iostream>
#include <string>

#include "utils.h"

namespace Oracle {

class Position {
   public:
    Position(const std::string& fen);
    Position()  = default;
    ~Position() = default;

    Piece piece_at(Square sq) const { return m_board[sq]; }
    bool  is_valid_move(Square from, Square to) { return m_valid_moves[from] & (1ULL << to); }

    void compute_valid_moves();
    void make_move(Square from, Square to);

    Bitboard get_all_checkers_bb() const { return m_checkers_bb[WHITE] | m_checkers_bb[BLACK]; };

   private:
    BoardArray m_board                  = {NO_PIECE};
    Bitboard   m_valid_moves[SQUARE_NB] = {0};
    Bitboard   m_checkers_bb[COLOR_NB]  = {0};

    Color m_side_to_move = COLOR_NB;

    Bitboard compute_pawn_moves(Piece p, Square sq);
    Bitboard compute_knight_moves(Piece p, Square sq);
};

PieceType from_char(char c);
void      print_bitboard(Bitboard bb, const std::string& label);
}

#endif  // POSITION_H_INCLUDED
