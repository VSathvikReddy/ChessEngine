#pragma once

#include <vector>
#include "pieces.hpp"

class Board{
private:
    Piece* grid[8][8];
    bool hot[8][8];
public:
    Board() = default;
    Board(std::vector<Piece>& white, std::vector<Piece>& black);
    ~Board() = default;

    std::vector<Position> get_possible_positions(const Piece& a);
};
