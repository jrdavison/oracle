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

    Utils::Bitboard get_all_checkers_bb() const { return m_checkers_bb[Utils::WHITE] | m_checkers_bb[Utils::BLACK]; };
    Utils::Color    turn_color() const { return m_turn_color; }
    Utils::Piece    piece_at(Utils::Square sq) const { return m_board[sq]; }
    double          last_move_gen_speed() const { return m_last_move_gen_speed; }
    int             move_count() const { return m_move_count; }

    bool is_valid_move(Utils::Square from, Utils::Square to);
    void compute_valid_moves(Utils::Color color);
    void make_move(Utils::Square from, Utils::Square to);


   private:
    Utils::Bitboard   m_checkers_bb[Utils::COLOR_NB]  = {0};
    Utils::Bitboard   m_valid_moves[Utils::SQUARE_NB] = {0};
    Utils::BoardArray m_board                         = {Utils::NO_PIECE};

    Utils::Color m_turn_color          = Utils::COLOR_NB;
    double       m_last_move_gen_speed = 0.0;
    int          m_move_count          = 0;

    Utils::RookMoveDatabase   m_rook_moves;
    Utils::KnightMoveDatabase m_knight_moves = {0};

    Utils::Bitboard compute_bishop_moves(Utils::Piece p, Utils::Square sq);
    Utils::Bitboard compute_knight_moves(Utils::Piece p, Utils::Square sq);
    Utils::Bitboard compute_pawn_moves(Utils::Piece p, Utils::Square sq);
    Utils::Bitboard compute_rook_moves(Utils::Piece p, Utils::Square sq);
    Utils::Bitboard compute_king_moves(Utils::Piece p, Utils::Square sq);

    void load_knight_move_db(const std::string& filename);
    void load_rook_move_db(const std::string& filename);
};

Utils::PieceType from_char(char c);
void             print_bitboard(Utils::Bitboard bb, const std::string& label);

}

#endif  // POSITION_H_INCLUDED
