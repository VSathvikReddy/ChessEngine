#include <cstdint>
#include<vector>

struct Position{
    uint8_t x;
    uint8_t y;

    Position(int x,int y);
}; 
bool operator==(const Position& a,const Position& b);

enum class Type{
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King
};

enum class PieceColor{
    Black,
    White
};

class Piece{
private:
    const Type ID;
    Position pos;

    bool has_moved=false;
    bool captured = false;
public:
    
    Piece(Type id,Position pos);

    const Position getPos() const;
    bool goTo(Position new_pos,std::vector<Position> valid_pos);
    void get_captured();
};
