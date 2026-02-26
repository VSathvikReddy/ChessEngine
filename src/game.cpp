#include "game.hpp"

ChessEngine::ChessEngine(){
    window.create(sf::VideoMode(800, 600), "My window");

    Black_Pieces.clear();
    White_Pieces.clear();

    Black_Pieces.reserve(16);
    White_Pieces.reserve(16);

    // ===== Pawns =====
    for(int i = 0; i < 8; i++){
        White_Pieces.emplace_back(PieceType::Pawn, Position(i,1), PieceColor::White);
        Black_Pieces.emplace_back(PieceType::Pawn, Position(i,6), PieceColor::Black);
    }

    // ===== Rooks =====
    White_Pieces.emplace_back(PieceType::Rook, Position(0,0), PieceColor::White);
    White_Pieces.emplace_back(PieceType::Rook, Position(7,0), PieceColor::White);

    Black_Pieces.emplace_back(PieceType::Rook, Position(0,7), PieceColor::Black);
    Black_Pieces.emplace_back(PieceType::Rook, Position(7,7), PieceColor::Black);

    // ===== Knights =====
    White_Pieces.emplace_back(PieceType::Knight, Position(1,0), PieceColor::White);
    White_Pieces.emplace_back(PieceType::Knight, Position(6,0), PieceColor::White);

    Black_Pieces.emplace_back(PieceType::Knight, Position(1,7), PieceColor::Black);
    Black_Pieces.emplace_back(PieceType::Knight, Position(6,7), PieceColor::Black);

    // ===== Bishops =====
    White_Pieces.emplace_back(PieceType::Bishop, Position(2,0), PieceColor::White);
    White_Pieces.emplace_back(PieceType::Bishop, Position(5,0), PieceColor::White);

    Black_Pieces.emplace_back(PieceType::Bishop, Position(2,7), PieceColor::Black);
    Black_Pieces.emplace_back(PieceType::Bishop, Position(5,7), PieceColor::Black);

    // ===== Queens =====
    White_Pieces.emplace_back(PieceType::Queen, Position(3,0), PieceColor::White);
    Black_Pieces.emplace_back(PieceType::Queen, Position(3,7), PieceColor::Black);

    // ===== Kings =====
    White_Pieces.emplace_back(PieceType::King, Position(4,0), PieceColor::White);
    Black_Pieces.emplace_back(PieceType::King, Position(4,7), PieceColor::Black);

    board = Board(White_Pieces,Black_Pieces);
}