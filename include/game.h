#ifndef GAME_H_
#define GAME_H_

#include <algorithm>
#include <iostream>
#include <string>

#include <SFML/Graphics.hpp>

#include "board.h"
#include "info_panel.h"
#include "position.h"
#include "utils.h"

namespace Oracle {

class Game {
   public:
    Game()  = default;
    ~Game() = default;

    void pause() { m_paused = true; };
    void resume() { m_paused = false; };
    bool is_paused() { return m_paused; };

    void play(sf::RenderWindow& window);
    void move(sf::RenderWindow& window);

   private:
    GUI::Board m_board;
    bool       m_paused = false;
};

}  // namespace Oracle

#endif  // GAME_H_
