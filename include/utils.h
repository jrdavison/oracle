#ifndef UTILS_H_
#define UTILS_H_

#include <cstdint>
#include <unordered_map>

#include <SFML/Graphics.hpp>

namespace Oracle {

constexpr int BOARD_W_PX      = 640;
constexpr int BOARD_SQ_PX     = 80;
constexpr int BOARD_SQ_ROW_NB = 8;

constexpr int ATLAS_GRID_W_PX = 170;

const sf::Color LIGHT_SQ = sf::Color(240, 217, 181);    // off white
const sf::Color DARK_SQ  = sf::Color(181, 136, 99);     // tan
const sf::Color VALID_SQ = sf::Color(35, 64, 153, 90);  // transparent blue
const sf::Color CHECK_SQ = sf::Color(252, 3, 3, 90);    // transparent red
const sf::Color INFO_BG  = sf::Color(60, 60, 60);       // gray

constexpr uint64_t HORIZONTAL_MASK = 0x00000000000000FF;
constexpr uint64_t VERTICAL_MASK   = 0x0101010101010101;

struct MouseCoords {
    int x;
    int y;
};

enum Color {
    WHITE,
    BLACK,
    COLOR_NB = 2
};

enum CastlingRights {
    NO_CASTLING,
    WHITE_OO,
    WHITE_OOO = WHITE_OO << 1,
    BLACK_OO  = WHITE_OO << 2,
    BLACK_OOO = WHITE_OO << 3,

    KING_SIDE      = WHITE_OO | BLACK_OO,
    QUEEN_SIDE     = WHITE_OOO | BLACK_OOO,
    WHITE_CASTLING = WHITE_OO | WHITE_OOO,
    BLACK_CASTLING = BLACK_OO | BLACK_OOO,
    ANY_CASTLING   = WHITE_CASTLING | BLACK_CASTLING,

    CASTLING_RIGHT_NB = 16
};

// clang-format off
enum PieceType {
    NO_PIECE_TYPE, KING, QUEEN, BISHOP, KNIGHT, ROOK, PAWN,
    ALL_PIECES = 0,
    PIECE_TYPE_NB = 8
};

enum Piece {
    NO_PIECE,
    W_KING = KING, W_QUEEN, W_BISHOP, W_KNIGHT, W_ROOK, W_PAWN,
    B_KING = KING + 8, B_QUEEN, B_BISHOP, B_KNIGHT, B_ROOK, B_PAWN,
    PIECE_NB = 16
};

enum Square : int {
    SQ_A1, SQ_B1, SQ_C1, SQ_D1, SQ_E1, SQ_F1, SQ_G1, SQ_H1,
    SQ_A2, SQ_B2, SQ_C2, SQ_D2, SQ_E2, SQ_F2, SQ_G2, SQ_H2,
    SQ_A3, SQ_B3, SQ_C3, SQ_D3, SQ_E3, SQ_F3, SQ_G3, SQ_H3,
    SQ_A4, SQ_B4, SQ_C4, SQ_D4, SQ_E4, SQ_F4, SQ_G4, SQ_H4,
    SQ_A5, SQ_B5, SQ_C5, SQ_D5, SQ_E5, SQ_F5, SQ_G5, SQ_H5,
    SQ_A6, SQ_B6, SQ_C6, SQ_D6, SQ_E6, SQ_F6, SQ_G6, SQ_H6,
    SQ_A7, SQ_B7, SQ_C7, SQ_D7, SQ_E7, SQ_F7, SQ_G7, SQ_H7,
    SQ_A8, SQ_B8, SQ_C8, SQ_D8, SQ_E8, SQ_F8, SQ_G8, SQ_H8,

    SQUARE_NB   = 64
};
// clang-format on

typedef Piece BoardArray[SQUARE_NB];

typedef uint64_t                               Bitboard;
typedef Bitboard                               KnightMoveDatabase[SQUARE_NB];
typedef std::unordered_map<Bitboard, Bitboard> RookMoveDatabase[SQUARE_NB];

enum Direction : int {
    NORTH = 8,
    EAST  = 1,
    SOUTH = -NORTH,
    WEST  = -EAST,

    NORTH_EAST = NORTH + EAST,
    SOUTH_EAST = SOUTH + EAST,
    SOUTH_WEST = SOUTH + WEST,
    NORTH_WEST = NORTH + WEST,
};

enum File : int {
    FILE_A,
    FILE_B,
    FILE_C,
    FILE_D,
    FILE_E,
    FILE_F,
    FILE_G,
    FILE_H,
    FILE_NB
};

enum Rank : int {
    RANK_1,
    RANK_2,
    RANK_3,
    RANK_4,
    RANK_5,
    RANK_6,
    RANK_7,
    RANK_8,
    RANK_NB
};

// Swap color of piece B_KNIGHT <-> W_KNIGHT
constexpr Piece operator~(Piece p) { return Piece(p ^ 8); }
// Swap color
constexpr Color operator~(Color c) { return Color(c ^ 1); }

constexpr PieceType type_of(Piece p) { return PieceType(p & 7); };
constexpr Color     color_of(Piece p) { return Color(p >> 3); };
constexpr File      file_of(Square s) { return File(s & 7); };
constexpr Rank      rank_of(Square s) { return Rank(s >> 3); };
constexpr Rank      relative_rank(Color c, Rank r) { return c == WHITE ? r : Rank(RANK_8 - r); };
constexpr Square    make_square(File f, Rank r) { return Square((r << 3) + f); };
constexpr Piece     make_piece(PieceType pt, Color c) { return Piece(pt + (c << 3)); };
constexpr Direction forward_direction(Color c) { return c == WHITE ? NORTH : SOUTH; };
constexpr bool      valid_square(int square) { return SQ_A1 <= square && square < SQUARE_NB; }

// Bitboard operations
inline void    set_bit(Bitboard& bb, Square sq) { bb |= (1ULL << sq); };
inline void    clear_bit(Bitboard& bb, Square sq) { bb &= ~(1ULL << sq); };
constexpr bool is_bit_set(Bitboard bb, Square sq) { return bb & (1ULL << sq); };

// Allow directions to increment/decrement squares (defaulting to SQUARE_NB if out of bounds)
constexpr Square operator+(Square sq, Direction dir) {
    int new_sq = static_cast<int>(sq) + static_cast<int>(dir);

    // Check if new square is out of bounds
    if (!valid_square(new_sq))
        return SQUARE_NB;

    // Prevent wrapping for basic east-west movement
    File file = file_of(sq);
    if ((file == FILE_A && (dir == WEST)) || (file == FILE_H && (dir == EAST)))
        return SQUARE_NB;

    // Break down compound directions into basic directions
    switch (dir)
    {
    case NORTH_EAST :
    case NORTH_WEST :
        return sq + NORTH + (dir == NORTH_EAST ? EAST : WEST);
    case SOUTH_EAST :
    case SOUTH_WEST :
        return sq + SOUTH + (dir == SOUTH_EAST ? EAST : WEST);
    default :
        return static_cast<Square>(new_sq);
    }
}

// Allow increment/decrement of enum types
template<typename T>
inline T& operator++(T& d) {
    return d = T(int(d) + 1);
};
template<typename T>
inline T& operator--(T& d) {
    return d = T(int(d) - 1);
};

// Allow add/subtract of enum types
template<typename T>
inline T& operator+=(T& d, int i) {
    return d = T(int(d) + i);
};
template<typename T>
inline T& operator-=(T& d, int i) {
    return d = T(int(d) - i);
};

}  // namespace Oracle

#endif  // UTILS_H_
