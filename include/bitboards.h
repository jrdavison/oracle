#ifndef BITBOARDS_H_
#define BITBOARDS_H_

#include <cstdint>
#include <string>

#include "utils.h"

namespace Oracle {

class Bitboards {
   public:
    Bitboards()  = default;
    ~Bitboards() = default;

    Piece get_piece_at_sq(Square square) { return m_board[square]; }

    void set_position(const std::string& fen);

   private:
    BoardArray m_board = {NO_PIECE};
};

PieceType from_char(char c);

}  // namespace Oracle

#endif  // BITBOARDS_H_
