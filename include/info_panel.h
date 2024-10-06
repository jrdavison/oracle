#ifndef INFO_PANEL_H_
#define INFO_PANEL_H_

#include <sstream>

#include <SFML/Graphics.hpp>

#include "position.h"
#include "utils.h"

namespace Oracle {

namespace GUI {

constexpr int   FONT_SIZE_LG      = 32;
constexpr int   FONT_SIZE_SM      = 16;
constexpr int   PADDING           = 10;
constexpr int   DECIMAL_PRECISION = 4;
const sf::Color INFO_BG           = sf::Color(60, 60, 60);  // gray

class InfoPanel {
   public:
    InfoPanel();

    void draw(sf::RenderWindow& window, Position& position);

   private:
    sf::Font m_font;
};

}  // namespace GUI

}  // namespace Oracle

#endif  // INFO_PANEL_H_
