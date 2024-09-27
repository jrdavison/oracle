#include "display.h"

namespace Oracle {

// Board
Board::Board() {
    if (!m_piece_atlas.loadFromFile("../../resources/piece-atlas.png"))
        throw std::runtime_error("Piece atlas could not be loaded");

    if (!m_font.loadFromFile("../../resources/font.ttf"))
        throw std::runtime_error("Font could not be loaded");

    m_piece_atlas.setSmooth(true);
    m_board_texture = make_board_texture();

    std::string fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    m_position      = Position(fen);

    init_board();
}

Board::~Board() {
    for (auto& piece : m_board)
    {
        if (piece != nullptr)
            delete piece;
    }
}

void Board::clear_board() {
    for (Square sq = SQ_A1; sq <= SQ_H8; ++sq)
    {
        delete m_board[sq];
        m_board[sq] = nullptr;
    }
}

void Board::init_board() {
    for (Square sq = SQ_A1; sq <= SQ_H8; ++sq)
    {
        Piece piece = m_position.piece_at(sq);
        if (piece != NO_PIECE)
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
    draw_info(window);

    window.display();
}

void Board::draw_board(sf::RenderWindow& window) {
    sf::Sprite board;
    board.setTexture(m_board_texture);
    window.draw(board);
}

void Board::draw_pieces(sf::RenderWindow& window) {
    for (Square sq = SQ_A1; sq <= SQ_H8; ++sq)
    {
        // to avoid having to loop again, draw any valid moves for a selected piece as we iterate over board squares
        if (m_dragged_piece != nullptr)
        {
            if (m_position.is_valid_move(m_dragged_piece->square(), sq))
            {
                sf::RectangleShape board_sq = make_board_square(file_of(sq), rank_of(sq), VALID_SQ);
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

void Board::draw_info(sf::RenderWindow& window) {
    const int font_size_lg             = 32;
    const int font_size_sm             = 16;
    const int padding                  = 10;
    const int move_gen_speed_precision = 4;

    Color turn_color = m_position.turn_color();  // TODO: get turn color from position

    // background
    sf::RectangleShape info_pane_bg(sf::Vector2f(BOARD_SQ_PX * BOARD_SQ_ROW_NB, BOARD_SQ_PX * BOARD_SQ_ROW_NB));
    info_pane_bg.setFillColor(INFO_BG);
    info_pane_bg.setPosition(BOARD_SQ_PX * BOARD_SQ_ROW_NB, 0);
    window.draw(info_pane_bg);

    // turn banner
    sf::Color          turn_banner_color = (turn_color == WHITE) ? sf::Color::White : sf::Color::Black;
    sf::RectangleShape turn_banner(sf::Vector2f((BOARD_SQ_PX * BOARD_SQ_ROW_NB) - padding, font_size_lg + padding));
    turn_banner.setFillColor(turn_banner_color);
    turn_banner.setPosition(BOARD_SQ_PX * BOARD_SQ_ROW_NB + 5, 5);
    window.draw(turn_banner);

    // turn text
    sf::Text  turn_text;
    sf::Color turn_text_color = (turn_color == WHITE) ? sf::Color::Black : sf::Color::White;
    turn_text.setFont(m_font);
    if (turn_color == WHITE)
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
    move_gen_speed << std::fixed << m_position.get_last_move_gen_speed();
    move_gen_speed_text.setFont(m_font);
    move_gen_speed_text.setString("Last move gen speed: " + move_gen_speed.str() + " ms");
    move_gen_speed_text.setCharacterSize(font_size_sm);
    move_gen_speed_text.setFillColor(sf::Color::White);
    move_gen_speed_text.setPosition((BOARD_SQ_PX * BOARD_SQ_ROW_NB) + padding,
                                    (BOARD_SQ_PX * BOARD_SQ_ROW_NB)
                                      - (move_gen_speed_text.getLocalBounds().height + padding));
    window.draw(move_gen_speed_text);
}

void Board::mouse_handler(sf::RenderWindow& window) {
    sf::Vector2i mouse_coords = sf::Mouse::getPosition(window);
    // contain mouse coords inside of the board
    int x = std::max(0, std::min(mouse_coords.x, BOARD_W_PX - 1));
    int y = std::max(0, std::min(mouse_coords.y, BOARD_W_PX - 1));

    if (sf::Mouse::isButtonPressed(sf::Mouse::Left))
    {
        if (m_dragged_piece == nullptr)
        {
            File   file     = file_from_x(x);
            Rank   rank     = rank_from_y(y);
            Square sq       = make_square(file, rank);
            m_dragged_piece = m_board[sq];
        }
        else
        {
            // TODO: only friendly pieces can be selected
            m_dragged_piece->drag(x, y);
        }
    }
}

void Board::move(sf::RenderWindow& window) {
    MouseCoords mouse_coords = get_mouse_coords(window);
    if (m_dragged_piece != nullptr)
    {
        Square src_sq  = m_dragged_piece->square();
        File   dest_f  = file_from_x(mouse_coords.x);
        Rank   dest_r  = rank_from_y(mouse_coords.y);
        Square dest_sq = make_square(dest_f, dest_r);
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
PieceGUI::PieceGUI(sf::Texture& pa, Piece p, Square sq) {
    int x_offset = (int(type_of(p)) - 1) * ATLAS_GRID_W_PX;
    int y_offset = (color_of(p) == WHITE) ? 0 : ATLAS_GRID_W_PX;

    m_sprite.setTexture(pa);
    m_sprite.setTextureRect(sf::IntRect(x_offset, y_offset, ATLAS_GRID_W_PX, ATLAS_GRID_W_PX));
    m_sprite.setOrigin(ATLAS_GRID_W_PX / 2, ATLAS_GRID_W_PX / 2);
    m_sprite.setScale((1.0f * BOARD_SQ_PX) / ATLAS_GRID_W_PX, (1.0f * BOARD_SQ_PX) / ATLAS_GRID_W_PX);

    move(sq);
};

void PieceGUI::move(Square sq) {
    m_square = sq;

    int x = (file_of(m_square) * BOARD_SQ_PX) + (BOARD_SQ_PX / 2);
    int y = (BOARD_SQ_PX * BOARD_SQ_ROW_NB) - (rank_of(m_square) * BOARD_SQ_PX) - BOARD_SQ_PX + (BOARD_SQ_PX / 2);
    m_sprite.setPosition(x, y);
};

// helpers
sf::Texture make_board_texture() {
    sf::Texture board_texture;
    board_texture.create(BOARD_W_PX, BOARD_W_PX);

    sf::RenderTexture render_texture;
    render_texture.create(BOARD_W_PX, BOARD_W_PX);
    render_texture.clear();
    for (Rank r = RANK_8; r >= RANK_1; --r)
    {
        for (File f = FILE_A; f <= FILE_H; ++f)
        {
            sf::Color          sq_color = (f + r) % 2 == 0 ? DARK_SQ : LIGHT_SQ;
            sf::RectangleShape board_sq = make_board_square(f, r, sq_color);
            render_texture.draw(board_sq);
        }
    }
    render_texture.display();

    board_texture.update(render_texture.getTexture());
    return board_texture;
}

sf::RectangleShape make_board_square(File file, Rank rank, sf::Color color) {
    sf::RectangleShape square(sf::Vector2f(BOARD_SQ_PX, BOARD_SQ_PX));
    square.setFillColor(color);

    int x_offset = file * BOARD_SQ_PX;
    int y_offset = (BOARD_W_PX - (rank * BOARD_SQ_PX) - BOARD_SQ_PX);
    square.setPosition(x_offset, y_offset);
    return square;
}

MouseCoords get_mouse_coords(sf::RenderWindow& window) {
    sf::Vector2i sf_mouse_coords = sf::Mouse::getPosition(window);

    // contain mouse coords inside of the board
    MouseCoords coords;
    coords.x = std::max(0, std::min(sf_mouse_coords.x, BOARD_W_PX - 1));
    coords.y = std::max(0, std::min(sf_mouse_coords.y, BOARD_W_PX - 1));
    return coords;
}

}  // namespace Oracle
