#pragma once
#include <memory>

#include <SFML/Window.hpp>

#include "pieces.hpp"
#include "board.hpp"

class ChessEngine{
private:
    Board board;
    std::vector<Piece> Black_Pieces;
    std::vector<Piece> White_Pieces;

    sf::Window window;


public:
    ChessEngine();
};