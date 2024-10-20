#include <SFML/Graphics.hpp>

#include "game.h"
#include "utils.h"

int main() {
    // create the window
    sf::RenderWindow window(sf::VideoMode(Oracle::Utils::BOARD_W_PX * 2, Oracle::Utils::BOARD_W_PX),
                            "Oracle Chess Engine");

    Oracle::Game game;
    game.play(window);

    bool mouse_pressed = false;
    while (window.isOpen())
    {
        sf::Event event;
        while (window.pollEvent(event))
        {
            if (event.type == sf::Event::Closed)
                window.close();

            if (event.type == sf::Event::LostFocus)
                game.pause();

            if (event.type == sf::Event::GainedFocus)
                game.resume();

            if (event.type == sf::Event::MouseButtonPressed)
                mouse_pressed = true;

            // pressing escape will close the window
            if ((event.type == sf::Event::KeyPressed) && (event.key.code == sf::Keyboard::Escape))
                window.close();

            if (event.type == sf::Event::MouseButtonReleased)
            {
                mouse_pressed = false;
                game.move(window);
            }
        }

        if (mouse_pressed)
            game.play(window);
        else
            sf::sleep(sf::milliseconds(10));
    }

    return 0;
}
