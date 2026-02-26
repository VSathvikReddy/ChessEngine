#include <cstdint>
#include<vector>

struct Position{
    uint8_t pos;
}; 


class Piece{
private:
    Position pos;
public:
    const Position getPos() const;
    virtual const std::vector<Position> getPossibleMoves() const;
};