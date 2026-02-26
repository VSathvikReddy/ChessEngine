#include "game.hpp"

ChessEngine::ChessEngine(){
    //Init_board
    Black_Pieces = std::vector<Piece>(16);
    for(int i=0;i<8;i++){
        Black_Pieces[i] = Piece(PieceColor::Black,Position(i,1));
    }
}