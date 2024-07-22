#ifndef POSITION_H_INCLUDED
#define POSITION_H_INCLUDED

#include <algorithm>
#include <iostream>
#include <string>

#include "utils.h"

namespace Oracle {

class Position {
   public:
    Position()  = default;
    ~Position() = default;

    Piece piece_at(Square sq) const { return m_board[sq]; }

    void set(const std::string& fen);
    void make_move(Square from, Square to);

   private:
    BoardArray m_board = {NO_PIECE};
};

PieceType from_char(char c);
}

#endif  // POSITION_H_INCLUDED
