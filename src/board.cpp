#include "board.hpp"

std::vector<Position> Board::get_possible_positions(const Piece& a){

    std::vector<Position> moves;

    auto inside = [](int x, int y){
        return x >= 0 && x < 8 && y >= 0 && y < 8;
    };

    int x = a.getPos().x;
    int y = a.getPos().y;

    switch(a.getType()){

    case PieceType::Pawn:{
        int dir = (a.getColor() == PieceColor::White) ? 1 : -1;

        // Forward
        if(inside(x, y + dir))
            moves.emplace_back(x, y + dir);

        // Double move (we check validity later)
        if(!a.has_moved && inside(x, y + 2*dir))
            moves.emplace_back(x, y + 2*dir);

        // Diagonal captures (just possible pattern)
        if(inside(x + 1, y + dir))
            moves.emplace_back(x + 1, y + dir);

        if(inside(x - 1, y + dir))
            moves.emplace_back(x - 1, y + dir);

        break;
    }


    case PieceType::Rook:{
        for(int i = 0; i < 8; i++){
            if(i != x) moves.emplace_back(i, y);
            if(i != y) moves.emplace_back(x, i);
        }
        break;
    }


    case PieceType::Knight:{
        int dx[] = {2,2,-2,-2,1,1,-1,-1};
        int dy[] = {1,-1,1,-1,2,-2,2,-2};

        for(int i = 0; i < 8; i++){
            int nx = x + dx[i];
            int ny = y + dy[i];
            if(inside(nx, ny))
                moves.emplace_back(nx, ny);
        }
        break;
    }


    case PieceType::Bishop:{
        for(int i = 1; i < 8; i++){
            if(inside(x+i, y+i)) moves.emplace_back(x+i, y+i);
            if(inside(x-i, y+i)) moves.emplace_back(x-i, y+i);
            if(inside(x+i, y-i)) moves.emplace_back(x+i, y-i);
            if(inside(x-i, y-i)) moves.emplace_back(x-i, y-i);
        }
        break;
    }


    case PieceType::Queen:{
        // Rook part
        for(int i = 0; i < 8; i++){
            if(i != x) moves.emplace_back(i, y);
            if(i != y) moves.emplace_back(x, i);
        }

        // Bishop part
        for(int i = 1; i < 8; i++){
            if(inside(x+i, y+i)) moves.emplace_back(x+i, y+i);
            if(inside(x-i, y+i)) moves.emplace_back(x-i, y+i);
            if(inside(x+i, y-i)) moves.emplace_back(x+i, y-i);
            if(inside(x-i, y-i)) moves.emplace_back(x-i, y-i);
        }
        break;
    }


    case PieceType::King:{
        for(int dx = -1; dx <= 1; dx++){
            for(int dy = -1; dy <= 1; dy++){
                if(dx == 0 && dy == 0) continue;

                int nx = x + dx;
                int ny = y + dy;

                if(inside(nx, ny))
                    moves.emplace_back(nx, ny);
            }
        }
        break;
    }

    }

    return moves;
}


Board::Board(std::vector<Piece>& white, std::vector<Piece>& black){
    for(int i=0;i<white.size();i++){
        Position pos = white[i].getPos();
        grid[pos.x][pos.y] = &white[i];

        pos = black[i].getPos();
        grid[pos.x][pos.y] = &black[i];
    }
}