#include "position.h"

namespace Oracle {

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

    load_rook_move_db("../../resources/precalculated_moves/rook_moves.bin");
    load_knight_move_db("../../resources/precalculated_moves/knight_moves.bin");

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
        case ROOK :
            m_valid_moves[sq] = compute_rook_moves(p, sq);
            break;
        case BISHOP :
            m_valid_moves[sq] = compute_bishop_moves(p, sq);
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
    Color color    = color_of(p);
    Rank  src_rank = rank_of(sq);
    File  src_file = file_of(sq);
    return m_knight_moves[sq] & ~m_checkers_bb[color];
}

Bitboard Position::compute_rook_moves(Piece p, Square sq) {
    Bitboard valid_moves = 0;
    Color    color       = color_of(p);

    // shift rook moves to relevant square (assuming no blockers)
    Bitboard h_mask    = HORIZONTAL_MASK << (rank_of(sq) * 8);
    Bitboard v_mask    = VERTICAL_MASK << file_of(sq);
    Bitboard move_mask = (h_mask | v_mask) & ~(1ULL << sq);

    Bitboard blockers_key = get_all_checkers_bb() & move_mask;
    if (m_rook_moves[sq].find(blockers_key) != m_rook_moves[sq].end())
        valid_moves = m_rook_moves[sq][blockers_key];
    else
        std::cerr << "Blockers key not found in lookup table." << std::endl;

    return valid_moves & ~m_checkers_bb[color];
}

Bitboard Position::compute_bishop_moves(Piece p, Square sq) {
    Bitboard valid_moves = 0;
    Color    color       = color_of(p);

    return valid_moves & ~m_checkers_bb[color];
}

void Position::load_rook_move_db(const std::string& filename) {
    std::ifstream file(filename, std::ios::binary);
    if (!file.is_open())
        throw std::runtime_error("Failed to open file: " + filename);

    for (Square sq = SQ_A1; sq < SQUARE_NB; ++sq)
    {
        std::unordered_map<Bitboard, Bitboard> moves;
        uint32_t                               num_entries;
        file.read(reinterpret_cast<char*>(&num_entries), sizeof(num_entries));

        for (uint32_t i = 0; i < num_entries; ++i)
        {
            uint64_t blockers;
            uint64_t attacks;
            file.read(reinterpret_cast<char*>(&blockers), sizeof(blockers));
            file.read(reinterpret_cast<char*>(&attacks), sizeof(attacks));
            moves[blockers] = attacks;
        }
        m_rook_moves[sq] = moves;
    }
}

void Position::load_knight_move_db(const std::string& filename) {
    std::ifstream file(filename, std::ios::binary);
    if (!file.is_open())
        throw std::runtime_error("Failed to open file: " + filename);

    Bitboard bb = 0;
    for (int sq = SQ_A1; sq < SQUARE_NB; ++sq)
    {
        file.read(reinterpret_cast<char*>(&bb), sizeof(bb));
        m_knight_moves[sq] = bb;
    }
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
