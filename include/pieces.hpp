#include <cstdint>
#include<vector>

struct Position{
    uint8_t pos;
}; 

std::vector<std::pair<uint8_t,uint8_t>> convert 

class Piece{
protected:
    Position pos;

    bool has_moved=false;
    bool captured = false;
public:
    const Position getPos() const;
    bool goTo(Position new_pos);
    virtual const std::vector<Position> getPossibleMoves() const;
};


class Pawn:public Piece{

};

class Rook:public Piece{

};

class Knight:public Piece{

};

class Bishop:public Piece{

};

class Queen:public Piece{

};

class King:public Piece{

};

