#include "position.h"

namespace Oracle {

Position::Position(const std::string& fen) {
    Utils::File file = Utils::FILE_A;
    Utils::Rank rank = Utils::RANK_8;

    int blank_space_nb = 0;
    for (char c : fen)
    {
        if (c == '/')
        {
            --rank;
            file = Utils::FILE_A;
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
                Utils::Color     color      = isupper(c) ? Utils::WHITE : Utils::BLACK;
                Utils::Square    sq         = make_square(file, rank);
                Utils::PieceType piece_type = from_char(c);
                m_board[sq]                 = Utils::make_piece(from_char(c), color);

                Utils::set_bit(m_checkers_bb[color], sq);
                ++file;
            }
            else if (blank_space_nb == 1)
            {
                m_turn_color = (c == 'w') ? Utils::WHITE : Utils::BLACK;
            }
        }
    }

    load_rook_move_db("../../resources/precalculated_moves/rook_moves.bin");
    load_knight_move_db("../../resources/precalculated_moves/knight_moves.bin");

    compute_valid_moves();
}

bool Position::is_valid_move(Utils::Square from, Utils::Square to) {
    Utils::Piece p = m_board[from];
    if (color_of(p) != m_turn_color)
        return false;
    return m_valid_moves[from] & (1ULL << to);
}

void Position::make_move(Utils::Square from, Utils::Square to) {
    Utils::Color color = Utils::color_of(m_board[from]);

    m_board[to]   = m_board[from];
    m_board[from] = Utils::NO_PIECE;

    clear_bit(m_checkers_bb[color], from);
    set_bit(m_checkers_bb[color], to);

    // TODO: save what was captured so we can undo the move
    m_turn_color = ~color;
}

void Position::compute_valid_moves() {
    auto start = std::chrono::high_resolution_clock::now();

    for (Utils::Square sq = Utils::SQ_A1; sq < Utils::SQUARE_NB; ++sq)
    {
        Utils::Piece     p          = piece_at(sq);
        Utils::PieceType piece_type = Utils::type_of(p);
        Utils::Color     color      = Utils::color_of(p);

        switch (piece_type)
        {
        case Utils::PAWN :
            m_valid_moves[sq] = compute_pawn_moves(p, sq);
            break;
        case Utils::KNIGHT :
            m_valid_moves[sq] = compute_knight_moves(p, sq);
            break;
        case Utils::ROOK :
            m_valid_moves[sq] = compute_rook_moves(p, sq);
            break;
        case Utils::BISHOP :
            m_valid_moves[sq] = compute_bishop_moves(p, sq);
            break;
        }
    }

    auto duration =
      std::chrono::duration_cast<std::chrono::nanoseconds>(std::chrono::high_resolution_clock::now() - start);
    m_last_move_gen_speed = duration.count() / 1'000'000.0;
}

Utils::Bitboard Position::compute_pawn_moves(Utils::Piece p, Utils::Square sq) {
    Utils::Bitboard valid_moves = 0;

    Utils::Color     color       = Utils::color_of(p);
    Utils::Direction forward_dir = Utils::forward_direction(color);

    Utils::Square target_square = sq;
    for (int i = 0; i < 2; ++i)
    {
        // only allow double move if pawn is on starting rank
        if (i == 1 && Utils::relative_rank(color, Utils::rank_of(sq)) != Utils::RANK_2)
            break;

        target_square += forward_dir;
        if ((target_square < Utils::SQUARE_NB) && !is_bit_set(get_all_checkers_bb(), target_square))
            set_bit(valid_moves, target_square);
        else
            break;
    }

    // Capture moves
    Utils::Square target_square_east = sq + forward_dir + Utils::EAST;
    if (target_square_east < Utils::SQUARE_NB && is_bit_set(m_checkers_bb[~color], target_square_east))
        set_bit(valid_moves, target_square_east);

    Utils::Square target_square_west = sq;
    target_square_west += forward_dir + Utils::WEST;
    if (target_square_west < Utils::SQUARE_NB && is_bit_set(m_checkers_bb[~color], target_square_west))
        set_bit(valid_moves, target_square_west);

    return valid_moves;
}

Utils::Bitboard Position::compute_knight_moves(Utils::Piece p, Utils::Square sq) {
    Utils::Color color    = color_of(p);
    Utils::Rank  src_rank = rank_of(sq);
    Utils::File  src_file = file_of(sq);
    return m_knight_moves[sq] & ~m_checkers_bb[color];
}

Utils::Bitboard Position::compute_rook_moves(Utils::Piece p, Utils::Square sq) {
    Utils::Bitboard valid_moves = 0;
    Utils::Color    color       = color_of(p);

    // shift rook moves to relevant square (assuming no blockers)
    Utils::Bitboard h_mask    = Utils::HORIZONTAL_MASK << (rank_of(sq) * 8);
    Utils::Bitboard v_mask    = Utils::VERTICAL_MASK << file_of(sq);
    Utils::Bitboard move_mask = (h_mask | v_mask) & ~(1ULL << sq);

    Utils::Bitboard blockers_key = get_all_checkers_bb() & move_mask;
    if (m_rook_moves[sq].find(blockers_key) != m_rook_moves[sq].end())
        valid_moves = m_rook_moves[sq][blockers_key];
    else
        std::cerr << "Blockers key not found in lookup table." << std::endl;

    return valid_moves & ~m_checkers_bb[color];
}

Utils::Bitboard Position::compute_bishop_moves(Utils::Piece p, Utils::Square sq) {
    Utils::Bitboard valid_moves = 0;
    Utils::Color    color       = color_of(p);

    return valid_moves & ~m_checkers_bb[color];
}

void Position::load_rook_move_db(const std::string& filename) {
    std::ifstream file(filename, std::ios::binary);
    if (!file.is_open())
        throw std::runtime_error("Failed to open file: " + filename);

    for (Utils::Square sq = Utils::SQ_A1; sq < Utils::SQUARE_NB; ++sq)
    {
        std::unordered_map<Utils::Bitboard, Utils::Bitboard> moves;
        uint32_t                                             num_entries;
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

    Utils::Bitboard bb = 0;
    for (Utils::Square sq = Utils::SQ_A1; sq < Utils::SQUARE_NB; ++sq)
    {
        file.read(reinterpret_cast<char*>(&bb), sizeof(bb));
        m_knight_moves[sq] = bb;
    }
}

// helpers
Utils::PieceType from_char(char c) {
    switch (tolower(c))
    {
    case 'p' :
        return Utils::PAWN;
    case 'n' :
        return Utils::KNIGHT;
    case 'b' :
        return Utils::BISHOP;
    case 'r' :
        return Utils::ROOK;
    case 'q' :
        return Utils::QUEEN;
    case 'k' :
        return Utils::KING;
    default :
        return Utils::NO_PIECE_TYPE;
    }
}

void print_bitboard(Utils::Bitboard bb, const std::string& label) {
    std::string result = label + " bitboard:\n";

    for (Utils::Rank rank = Utils::RANK_8; rank >= Utils::RANK_LB; --rank)
    {
        for (Utils::File file = Utils::FILE_A; file < Utils::FILE_UB; ++file)
        {
            Utils::Square sq = Utils::make_square(file, rank);
            result += is_bit_set(bb, sq) ? '1' : '0';
        }
        result += "\n";
    }

    std::cout << result;
}


}  // namespace Oracle
