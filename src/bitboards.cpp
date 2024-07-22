#include "bitboards.h"

namespace Oracle {

void Bitboards::init(const Position& pos) {
    for (Square sq = SQ_A1; sq < SQUARE_NB; ++sq)
    {
        Piece     p          = pos.piece_at(sq);
        PieceType piece_type = type_of(p);
        Color     color      = color_of(p);

        switch (piece_type)
        {
        case PAWN :
            m_valid_moves[sq] = compute_pawn_moves(p, sq);
            break;
        }
    }
}

Bitboard Bitboards::compute_pawn_moves(Piece p, Square sq) {
    Bitboard valid_moves = 0;

    Color     color       = color_of(p);
    Direction forward_dir = forward_direction(color);

    // Forward Move (single square)
    Square target_square = sq;
    target_square += forward_dir;
    if (target_square < SQUARE_NB)
        set_bit(valid_moves, target_square);

    return valid_moves;
}

// helpers
void print_bitboard(Bitboard bb, const std::string& label) {
    std::string result = label + " bitboard:\n";

    for (int rank = (RANK_NB - 1); rank > 0; --rank)
    {
        for (int file = FILE_A; file < FILE_NB; ++file)
        {
            Square sq = make_square(File(file), Rank(rank));
            result += is_bit_set(bb, sq) ? '1' : '0';
        }
        result += "\n";
    }

    std::cout << result;
}

}  // namespace Oracle
