#ifndef INFO_PANEL_H_
#define INFO_PANEL_H_

#include <sstream>

#include <SFML/Graphics.hpp>

#include "position.h"
#include "utils.h"

namespace Oracle {

namespace GUI {

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
