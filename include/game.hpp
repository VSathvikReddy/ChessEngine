#pragma once
#include <memory>

#include "pieces.hpp"
#include "board.hpp"

class ChessEngine{
private:
    Board board;
    std::vector<std::unique_ptr<Piece>> Black_Pieces;
    std::vector<std::unique_ptr<Piece>> White_Pieces;
public:
    ChessEngine();
};