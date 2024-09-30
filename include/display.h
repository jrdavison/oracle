#ifndef DISPLAY_H_
#define DISPLAY_H_

#include <algorithm>
#include <iostream>
#include <string>

#include <SFML/Graphics.hpp>

#include "info_panel.h"
#include "position.h"
#include "utils.h"

namespace Oracle {

class PieceGUI {
   public:
    PieceGUI(sf::Texture& pa, Utils::Piece p, Utils::Square sq);
    ~PieceGUI() = default;

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
    ~Board();

    void pause() { m_paused = true; };
    void resume() { m_paused = false; };
    bool is_paused() { return m_paused; };

    void draw(sf::RenderWindow& window);
    void mouse_handler(sf::RenderWindow& window);
    void move(sf::RenderWindow& window);

   private:
    PieceGUI* m_board[Utils::SQUARE_NB] = {nullptr};
    PieceGUI* m_dragged_piece           = nullptr;

    Position m_position;

    sf::Texture m_board_texture;
    sf::Texture m_piece_atlas;
    InfoPanel   m_info_panel;

    bool m_paused = false;

    void draw_board(sf::RenderWindow& window);
    void draw_pieces(sf::RenderWindow& window);
    void init_board();
    void clear_board();
};

inline Utils::File file_from_x(int x) { return Utils::File(x / Utils::BOARD_SQ_PX); };
inline Utils::Rank rank_from_y(int y) { return Utils::Rank((Utils::BOARD_SQ_ROW_NB - 1) - (y / Utils::BOARD_SQ_PX)); };
sf::RectangleShape make_board_square(Utils::File file, Utils::Rank rank, sf::Color color);
sf::Texture        make_board_texture();
Utils::MouseCoords get_mouse_coords(sf::RenderWindow& window);

}  // namespace Oracle

#endif  // DISPLAY_H_
