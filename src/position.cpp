#include "position.h"

namespace Oracle {

void Position::set(const std::string& fen) {
    File file = FILE_A;
    Rank rank = RANK_8;

    int blank_space_nb = 0;
    for (char c : fen)
    {
        if (c == '/')
        {
            --rank;
            file = FILE_A;
        }
        else if (isdigit(c))
        {
            file += (c - '0');
        }
        else if (c == ' ')
        {
            blank_space_nb++;
        }
        else
        {
            if (blank_space_nb == 0)
            {
                Color     color      = isupper(c) ? WHITE : BLACK;
                Square    sq         = make_square(file, rank);
                PieceType piece_type = from_char(c);
                m_board[sq]          = make_piece(from_char(c), color);
                ++file;
            }
        }
    }
}

void Position::make_move(Square from, Square to) {
    m_board[to]   = m_board[from];
    m_board[from] = NO_PIECE;
}

// helpers
PieceType from_char(char c) {
    switch (tolower(c))
    {
    case 'p' :
        return PAWN;
    case 'n' :
        return KNIGHT;
    case 'b' :
        return BISHOP;
    case 'r' :
        return ROOK;
    case 'q' :
        return QUEEN;
    case 'k' :
        return KING;
    default :
        return NO_PIECE_TYPE;
    }
}
}  // namespace Oracle
