#include "pieces.hpp"


Position::Position(int x,int y){
    this->x = static_cast<uint8_t>(x);
    this->y = static_cast<uint8_t>(y);
}

bool operator==(const Position& a,const Position& b){
    return ((a.x==b.x) && (a.y == b.y));
}

Piece::Piece(Type id, Position pos):ID(id),pos(pos){

}

const Position Piece::getPos() const{
    return pos;
}
bool Piece::goTo(Position new_pos, std::vector<Position> valid_pos){
    has_moved = true;
    for(auto x : valid_pos){
        if(x == new_pos){
            pos = new_pos;
            return true;
        }
    }
    return false;
}

void Piece::get_captured(){
    captured = true;
}
