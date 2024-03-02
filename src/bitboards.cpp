#include "bitboards.h"

namespace Oracle {

void Bitboards::set_position(const std::string& fen) {
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
                Square    square     = make_square(file, rank);
                PieceType piece_type = from_char(c);
                Piece     piece      = make_piece(piece_type, color);
                m_board[square]      = piece;
                ++file;
            }
        }
    }
    return;
}

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
