#include "game.h"

namespace Oracle {

// Board
Game::Game() {
    std::string fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    m_position      = Position(fen);

    m_board.init_board(m_position);
    m_info_panel = GUI::InfoPanel();
}

void Game::play(sf::RenderWindow& window) {
    if (m_paused)
        return;

    window.clear(sf::Color::Black);

    m_board.mouse_handler(window);
    m_board.draw(window, m_position);
    m_info_panel.draw(window, m_position);

    window.display();
}

}  // namespace Oracle
