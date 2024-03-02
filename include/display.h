#ifndef DISPLAY_H_
#define DISPLAY_H_

#include <algorithm>
#include <iostream>
#include <string>

#include <SFML/Graphics.hpp>

#include "bitboards.h"
#include "utils.h"

namespace Oracle {

class PieceGUI {
   public:
    PieceGUI(sf::Texture& pa, Piece p, Square sq);
    ~PieceGUI() = default;

    void   draw(sf::RenderWindow& window) { window.draw(m_sprite); };
    void   drag(int x, int y) { m_sprite.setPosition(x, y); };
    Square square() { return m_square; };

    void move(Square sq);

   private:
    Square     m_square;
    sf::Sprite m_sprite;
};

class Board {
   public:
    Board();
    ~Board();


    void pause() { m_paused = true; };
    void resume() { m_paused = false; };
    bool is_paused() { return m_paused; };
    bool move_occurred() { return m_move_occurred; };


    void draw(sf::RenderWindow& window);
    void mouse_handler(sf::RenderWindow& window);
    void move(sf::RenderWindow& window);

   private:
    PieceGUI* m_board[SQUARE_NB] = {nullptr};
    PieceGUI* m_dragged_piece    = nullptr;

    Bitboards m_bitboards;

    sf::Font    m_font;
    sf::Texture m_board_texture;
    sf::Texture m_piece_atlas;

    bool m_paused        = false;
    bool m_move_occurred = false;

    void draw_board(sf::RenderWindow& window);
    void draw_pieces(sf::RenderWindow& window);
    void init_board();
};

inline File        file_from_x(int x) { return File(x / BOARD_SQ_PX); };
inline Rank        rank_from_y(int y) { return Rank((BOARD_SQ_ROW_NB - 1) - (y / BOARD_SQ_PX)); };
sf::RectangleShape make_board_square(File file, Rank rank, sf::Color color);
sf::Texture        make_board_texture();
MouseCoords        get_mouse_coords(sf::RenderWindow& window);

}  // namespace Oracle

#endif  // DISPLAY_H_
