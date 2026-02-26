#include "pieces.hpp"


Position::Position(int x,int y){
    this->x = static_cast<uint8_t>(x);
    this->y = static_cast<uint8_t>(y);
}

bool operator==(const Position& a,const Position& b){
    return ((a.x==b.x) && (a.y == b.y));
}

Piece::Piece(PieceType id,Position pos, PieceColor clr):ID(id),pos(pos),clr(clr){}

const Position Piece::getPos() const{
    return pos;
}
const PieceColor Piece::getColor() const{
    return clr;
}
const PieceType Piece::getType() const{
    return ID;
}
void Piece::goTo(Position new_pos){
    has_moved = true;
    pos = new_pos;

}

void Piece::get_captured(){
    captured = true;
}
