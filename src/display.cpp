#include "display.h"

namespace Oracle {

// Board
Board::Board() {
    if (!m_piece_atlas.loadFromFile("../../resources/piece-atlas.png"))
        throw std::runtime_error("Piece atlas could not be loaded");

    m_piece_atlas.setSmooth(true);
    m_board_texture = make_board_texture();

    std::string fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    m_position      = Position(fen);

    init_board();
    m_info_panel = InfoPanel();
}

Board::~Board() {
    for (auto& piece : m_board)
    {
        if (piece != nullptr)
            delete piece;
    }
}

void Board::clear_board() {
    for (Utils::Square sq = Utils::SQ_A1; sq <= Utils::SQ_H8; ++sq)
    {
        delete m_board[sq];
        m_board[sq] = nullptr;
    }
}

void Board::init_board() {
    for (Utils::Square sq = Utils::SQ_A1; sq <= Utils::SQ_H8; ++sq)
    {
        Utils::Piece piece = m_position.piece_at(sq);
        if (piece != Utils::NO_PIECE)
            m_board[sq] = new PieceGUI(m_piece_atlas, piece, sq);
    }
}

void Board::draw(sf::RenderWindow& window) {
    if (m_paused)
        return;

    window.clear(sf::Color::Black);

    mouse_handler(window);
    draw_board(window);
    draw_pieces(window);
    m_info_panel.draw(window, m_position);

    window.display();
}

void Board::draw_board(sf::RenderWindow& window) {
    sf::Sprite board;
    board.setTexture(m_board_texture);
    window.draw(board);
}

void Board::draw_pieces(sf::RenderWindow& window) {
    for (Utils::Square sq = Utils::SQ_A1; sq <= Utils::SQ_H8; ++sq)
    {
        // to avoid having to loop again, draw any valid moves for a selected piece as we iterate over board squares
        if (m_dragged_piece != nullptr)
        {
            if (m_position.is_valid_move(m_dragged_piece->square(), sq))
            {
                sf::RectangleShape board_sq = make_board_square(file_of(sq), rank_of(sq), Utils::VALID_SQ);
                window.draw(board_sq);
            }
        }

        PieceGUI* piece = m_board[sq];
        if ((piece != nullptr) && (piece != m_dragged_piece))
            m_board[sq]->draw(window);
    }

    if (m_dragged_piece != nullptr)
        m_dragged_piece->draw(window);
}

void Board::mouse_handler(sf::RenderWindow& window) {
    sf::Vector2i mouse_coords = sf::Mouse::getPosition(window);
    // contain mouse coords inside of the board
    int x = std::max(0, std::min(mouse_coords.x, Utils::BOARD_W_PX - 1));
    int y = std::max(0, std::min(mouse_coords.y, Utils::BOARD_W_PX - 1));

    if (sf::Mouse::isButtonPressed(sf::Mouse::Left))
    {
        if (m_dragged_piece == nullptr)
        {
            Utils::File   file = file_from_x(x);
            Utils::Rank   rank = rank_from_y(y);
            Utils::Square sq   = make_square(file, rank);
            m_dragged_piece    = m_board[sq];
        }
        else
            m_dragged_piece->drag(x, y);
    }
}

void Board::move(sf::RenderWindow& window) {
    Utils::MouseCoords mouse_coords = get_mouse_coords(window);
    if (m_dragged_piece != nullptr)
    {
        Utils::Square src_sq  = m_dragged_piece->square();
        Utils::File   dest_f  = file_from_x(mouse_coords.x);
        Utils::Rank   dest_r  = rank_from_y(mouse_coords.y);
        Utils::Square dest_sq = make_square(dest_f, dest_r);
        if (m_position.is_valid_move(src_sq, dest_sq))
        {
            m_position.make_move(src_sq, dest_sq);
            m_dragged_piece->move(dest_sq);

            clear_board();
            init_board();

            m_dragged_piece = nullptr;
            draw(window);
            m_position.compute_valid_moves();
            draw(window);  // draw again to update info pane
        }
        else
        {
            m_dragged_piece->move(src_sq);  // reset piece to original position
            m_dragged_piece = nullptr;
            draw(window);
        }
    }
}

// PieceGUI
PieceGUI::PieceGUI(sf::Texture& pa, Utils::Piece p, Utils::Square sq) {
    int x_offset = (int(type_of(p)) - 1) * Utils::ATLAS_GRID_W_PX;
    int y_offset = (color_of(p) == Utils::WHITE) ? 0 : Utils::ATLAS_GRID_W_PX;

    m_sprite.setTexture(pa);
    m_sprite.setTextureRect(sf::IntRect(x_offset, y_offset, Utils::ATLAS_GRID_W_PX, Utils::ATLAS_GRID_W_PX));
    m_sprite.setOrigin(Utils::ATLAS_GRID_W_PX / 2, Utils::ATLAS_GRID_W_PX / 2);
    m_sprite.setScale((1.0f * Utils::BOARD_SQ_PX) / Utils::ATLAS_GRID_W_PX,
                      (1.0f * Utils::BOARD_SQ_PX) / Utils::ATLAS_GRID_W_PX);

    move(sq);
};

void PieceGUI::move(Utils::Square sq) {
    m_square = sq;

    int x = (file_of(m_square) * Utils::BOARD_SQ_PX) + (Utils::BOARD_SQ_PX / 2);
    int y = (Utils::BOARD_SQ_PX * Utils::BOARD_SQ_ROW_NB) - (rank_of(m_square) * Utils::BOARD_SQ_PX)
          - Utils::BOARD_SQ_PX + (Utils::BOARD_SQ_PX / 2);
    m_sprite.setPosition(x, y);
};

// helpers
sf::Texture make_board_texture() {
    sf::Texture board_texture;
    board_texture.create(Utils::BOARD_W_PX, Utils::BOARD_W_PX);

    sf::RenderTexture render_texture;
    render_texture.create(Utils::BOARD_W_PX, Utils::BOARD_W_PX);
    render_texture.clear();
    for (Utils::Rank r = Utils::RANK_8; r >= Utils::RANK_1; --r)
    {
        for (Utils::File f = Utils::FILE_A; f <= Utils::FILE_H; ++f)
        {
            sf::Color          sq_color = (f + r) % 2 == 0 ? Utils::DARK_SQ : Utils::LIGHT_SQ;
            sf::RectangleShape board_sq = make_board_square(f, r, sq_color);
            render_texture.draw(board_sq);
        }
    }
    render_texture.display();

    board_texture.update(render_texture.getTexture());
    return board_texture;
}

sf::RectangleShape make_board_square(Utils::File file, Utils::Rank rank, sf::Color color) {
    sf::RectangleShape square(sf::Vector2f(Utils::BOARD_SQ_PX, Utils::BOARD_SQ_PX));
    square.setFillColor(color);

    int x_offset = file * Utils::BOARD_SQ_PX;
    int y_offset = (Utils::BOARD_W_PX - (rank * Utils::BOARD_SQ_PX) - Utils::BOARD_SQ_PX);
    square.setPosition(x_offset, y_offset);
    return square;
}

Utils::MouseCoords get_mouse_coords(sf::RenderWindow& window) {
    sf::Vector2i sf_mouse_coords = sf::Mouse::getPosition(window);

    // contain mouse coords inside of the board
    Utils::MouseCoords coords;
    coords.x = std::max(0, std::min(sf_mouse_coords.x, Utils::BOARD_W_PX - 1));
    coords.y = std::max(0, std::min(sf_mouse_coords.y, Utils::BOARD_W_PX - 1));
    return coords;
}

}  // namespace Oracle
