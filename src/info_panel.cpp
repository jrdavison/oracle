#include "info_panel.h"

namespace Oracle {

InfoPanel::InfoPanel() {
    if (!m_font.loadFromFile("../../resources/font.ttf"))
        throw std::runtime_error("Font could not be loaded");
}

void InfoPanel::draw(sf::RenderWindow& window, Position& position) {
    const int font_size_lg             = 32;
    const int font_size_sm             = 16;
    const int padding                  = 10;
    const int move_gen_speed_precision = 4;

    // background
    sf::RectangleShape info_pane_bg(
      sf::Vector2f(Utils::BOARD_SQ_PX * Utils::BOARD_SQ_ROW_NB, Utils::BOARD_SQ_PX * Utils::BOARD_SQ_ROW_NB));
    info_pane_bg.setFillColor(Utils::INFO_BG);
    info_pane_bg.setPosition(Utils::BOARD_SQ_PX * Utils::BOARD_SQ_ROW_NB, 0);
    window.draw(info_pane_bg);

    // turn banner
    sf::Color turn_banner_color = (position.turn_color() == Utils::WHITE) ? sf::Color::White : sf::Color::Black;
    sf::RectangleShape turn_banner(
      sf::Vector2f((Utils::BOARD_SQ_PX * Utils::BOARD_SQ_ROW_NB) - padding, font_size_lg + padding));
    turn_banner.setFillColor(turn_banner_color);
    turn_banner.setPosition(Utils::BOARD_SQ_PX * Utils::BOARD_SQ_ROW_NB + 5, 5);
    window.draw(turn_banner);

    // turn text
    sf::Text  turn_text;
    sf::Color turn_text_color = (position.turn_color() == Utils::WHITE) ? sf::Color::Black : sf::Color::White;
    turn_text.setFont(m_font);
    if (position.turn_color() == Utils::WHITE)
        turn_text.setString("White's Move");
    else
        turn_text.setString("Black's Move");
    turn_text.setCharacterSize(font_size_lg);
    turn_text.setFillColor(turn_text_color);
    float centerX = turn_banner.getPosition().x + (turn_banner.getSize().x / 2) - (turn_text.getLocalBounds().width / 2)
                  + (padding / 2);
    float centerY = padding / 2;
    turn_text.setPosition(centerX, centerY);
    window.draw(turn_text);


    sf::Text           move_gen_speed_text;
    std::ostringstream move_gen_speed;
    move_gen_speed.precision(move_gen_speed_precision);
    move_gen_speed << std::fixed << position.last_move_gen_speed();
    move_gen_speed_text.setFont(m_font);
    move_gen_speed_text.setString("Last move gen speed: " + move_gen_speed.str() + " ms");
    move_gen_speed_text.setCharacterSize(font_size_sm);
    move_gen_speed_text.setFillColor(sf::Color::White);
    move_gen_speed_text.setPosition((Utils::BOARD_SQ_PX * Utils::BOARD_SQ_ROW_NB) + padding,
                                    (Utils::BOARD_SQ_PX * Utils::BOARD_SQ_ROW_NB)
                                      - (move_gen_speed_text.getLocalBounds().height + padding));
    window.draw(move_gen_speed_text);

    // draw move count above move speed
    sf::Text move_count_text;
    move_count_text.setFont(m_font);
    move_count_text.setString("Moves: " + std::to_string(position.move_count()));
    move_count_text.setCharacterSize(font_size_sm);
    move_count_text.setFillColor(sf::Color::White);
    move_count_text.setPosition((Utils::BOARD_SQ_PX * Utils::BOARD_SQ_ROW_NB) + padding,
                                (Utils::BOARD_SQ_PX * (Utils::BOARD_SQ_ROW_NB - 1))
                                  - (move_gen_speed_text.getLocalBounds().height + padding));
    window.draw(move_count_text);
}


}  // namespace Oracle
