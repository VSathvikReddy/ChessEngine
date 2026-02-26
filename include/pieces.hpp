#pragma once

#include <cstdint>
#include<vector>

class Board;

struct Position{
    uint8_t x;
    uint8_t y;

    Position(int x,int y);
}; 
bool operator==(const Position& a,const Position& b);

enum class PieceType{
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
    friend Board;

    const PieceType ID;
    const PieceColor clr;
    Position pos;

    bool has_moved=false;
    bool captured = false;
public:
    
    Piece(PieceType id,Position pos, PieceColor clr);

    const Position getPos() const;
    void goTo(Position new_pos);
    const PieceColor getColor() const;
    const PieceType getType() const;

    void get_captured();
};
