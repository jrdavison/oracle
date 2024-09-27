#ifndef POSITION_H_INCLUDED
#define POSITION_H_INCLUDED

#include <algorithm>
#include <chrono>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <string>

#include "utils.h"

namespace Oracle {

class Position {
   public:
    Position(const std::string& fen);
    Position()  = default;
    ~Position() = default;

    Piece  piece_at(Square sq) const { return m_board[sq]; }
    bool   is_valid_move(Square from, Square to);
    double get_last_move_gen_speed() const { return last_move_gen_speed; }
    Color  turn_color() const { return m_turn_color; }

    void compute_valid_moves();
    void make_move(Square from, Square to);

    Bitboard get_all_checkers_bb() const { return m_checkers_bb[WHITE] | m_checkers_bb[BLACK]; };

   private:
    BoardArray m_board                  = {NO_PIECE};
    Bitboard   m_valid_moves[SQUARE_NB] = {0};
    Bitboard   m_checkers_bb[COLOR_NB]  = {0};

    Color m_turn_color = COLOR_NB;

    RookMoveDatabase   m_rook_moves;
    KnightMoveDatabase m_knight_moves = {0};

    double last_move_gen_speed = 0.0;

    Bitboard compute_pawn_moves(Piece p, Square sq);
    Bitboard compute_knight_moves(Piece p, Square sq);
    Bitboard compute_rook_moves(Piece p, Square sq);
    Bitboard compute_bishop_moves(Piece p, Square sq);

    void load_rook_move_db(const std::string& filename);
    void load_knight_move_db(const std::string& filename);
};

PieceType from_char(char c);
void      print_bitboard(Bitboard bb, const std::string& label);

}

#endif  // POSITION_H_INCLUDED
