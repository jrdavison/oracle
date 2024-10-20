#include "game.h"

namespace Oracle {

// Board

void Game::play(sf::RenderWindow& window) {
    if (m_paused)
        return;

    m_board.draw(window);
}

void Game::move(sf::RenderWindow& window) {
    if (m_paused)
        return;

    m_board.move(window);
}
}  // namespace Oracle
