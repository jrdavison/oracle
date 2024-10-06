#ifndef BOARD_H_
#define BOARD_H_

#include <algorithm>
#include <iostream>
#include <string>

#include <SFML/Graphics.hpp>

#include "info_panel.h"
#include "position.h"
#include "utils.h"

namespace Oracle {

namespace GUI {

constexpr int ATLAS_GRID_W_PX = 170;

const sf::Color LIGHT_SQ = sf::Color(240, 217, 181);    // off white
const sf::Color DARK_SQ  = sf::Color(181, 136, 99);     // tan
const sf::Color VALID_SQ = sf::Color(35, 64, 153, 90);  // transparent blue
const sf::Color CHECK_SQ = sf::Color(252, 3, 3, 90);    // transparent red

class Piece {
   public:
    Piece(sf::Texture& pa, Utils::Piece p, Utils::Square sq);
    ~Piece() = default;

    void          draw(sf::RenderWindow& window) { window.draw(m_sprite); };
    void          drag(int x, int y) { m_sprite.setPosition(x, y); };
    Utils::Square square() { return m_square; };

    void move(Utils::Square sq);

   private:
    Utils::Square m_square;
    sf::Sprite    m_sprite;
};

class Board {
   public:
    Board();
    ~Board() { clear_board(); };

    void draw(sf::RenderWindow& window);
    void mouse_handler(sf::RenderWindow& window);
    void move(sf::RenderWindow& window);

   private:
    Piece*   m_board[Utils::SQUARE_NB] = {nullptr};
    Piece*   m_dragged_piece           = nullptr;
    Position m_position;

    InfoPanel m_info_panel;

    sf::Texture m_board_texture;
    sf::Texture m_piece_atlas;

    void init_board();
    void clear_board();
    void draw_board(sf::RenderWindow& window);
    void draw_pieces(sf::RenderWindow& window);
};

inline Utils::File file_from_x(int x) { return Utils::File(x / Utils::BOARD_SQ_PX); };
inline Utils::Rank rank_from_y(int y) { return Utils::Rank((Utils::BOARD_SQ_ROW_NB - 1) - (y / Utils::BOARD_SQ_PX)); };
sf::RectangleShape make_board_square(Utils::File file, Utils::Rank rank, sf::Color color);
sf::Texture        make_board_texture();
Utils::MouseCoords get_mouse_coords(sf::RenderWindow& window);

}  // namespace GUI

}  // namespace Oracle

#endif  // BOARD_H_
