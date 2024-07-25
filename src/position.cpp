#include "position.h"

namespace Oracle {

// clang-format off
const Direction KNIGHT_MOVES[8] = {
    static_cast<Direction>(NORTH + NORTH_EAST), // NNE
    static_cast<Direction>(NORTH + NORTH_WEST), // NNW
    static_cast<Direction>(SOUTH + SOUTH_EAST), // SSE
    static_cast<Direction>(SOUTH + SOUTH_WEST), // SSW
    static_cast<Direction>(NORTH_EAST + EAST), // NEE
    static_cast<Direction>(SOUTH_EAST + EAST), // SEE
    static_cast<Direction>(NORTH_WEST + WEST), // NWW
    static_cast<Direction>(SOUTH_WEST + WEST) // SWW
};
// clang-format on

Position::Position(const std::string& fen) {
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

                set_bit(m_checkers_bb[color], sq);
                ++file;
            }
            else if (blank_space_nb == 1)
            {
                m_side_to_move = (c == 'w') ? WHITE : BLACK;
            }
        }
    }

    compute_valid_moves();
}

void Position::make_move(Square from, Square to) {
    Color color = color_of(m_board[from]);

    m_board[to]   = m_board[from];
    m_board[from] = NO_PIECE;

    clear_bit(m_checkers_bb[color], from);
    set_bit(m_checkers_bb[color], to);

    // TODO: save what was captured so we can undo the move
    m_side_to_move = ~color;
}

void Position::compute_valid_moves() {
    auto startTime = std::chrono::high_resolution_clock::now();

    for (Square sq = SQ_A1; sq < SQUARE_NB; ++sq)
    {
        Piece     p          = piece_at(sq);
        PieceType piece_type = type_of(p);
        Color     color      = color_of(p);

        switch (piece_type)
        {
        case PAWN :
            m_valid_moves[sq] = compute_pawn_moves(p, sq);
            break;
        case KNIGHT :
            m_valid_moves[sq] = compute_knight_moves(p, sq);
            break;
        }
    }

    auto endTime  = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(endTime - startTime);
    std::cout << "Move generation took " << duration.count() << " ms\n";
}

Bitboard Position::compute_pawn_moves(Piece p, Square sq) {
    Bitboard valid_moves = 0;

    Color     color       = color_of(p);
    Direction forward_dir = forward_direction(color);

    Square target_square = sq;
    for (int i = 0; i < 2; ++i)
    {
        // only allow double move if pawn is on starting rank
        if (i == 1 && relative_rank(color, rank_of(sq)) != RANK_2)
            break;

        target_square += forward_dir;
        if ((target_square < SQUARE_NB) && !is_bit_set(get_all_checkers_bb(), target_square))
            set_bit(valid_moves, target_square);
        else
            break;
    }

    // Capture moves
    Square target_square_east = sq + forward_dir + EAST;
    if (target_square_east < SQUARE_NB && is_bit_set(m_checkers_bb[~color], target_square_east))
        set_bit(valid_moves, target_square_east);

    Square target_square_west = sq;
    target_square_west += forward_dir + WEST;
    if (target_square_west < SQUARE_NB && is_bit_set(m_checkers_bb[~color], target_square_west))
        set_bit(valid_moves, target_square_west);

    return valid_moves;
}

Bitboard Position::compute_knight_moves(Piece p, Square sq) {
    Bitboard valid_moves = 0;
    Color    color       = color_of(p);

    for (Direction dir : KNIGHT_MOVES)
    {
        Square target_square = sq + dir;
        if (SQ_A1 <= target_square < SQUARE_NB)
            set_bit(valid_moves, target_square);
    }

    // can't capture own pieces
    valid_moves &= ~m_checkers_bb[color];

    return valid_moves;
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

void print_bitboard(Bitboard bb, const std::string& label) {
    std::string result = label + " bitboard:\n";

    for (int rank = (RANK_NB - 1); rank >= 0; --rank)
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
