use std::fmt;
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Class {
    Pawn    = 0,
    Knight  = 1,
    Bishop  = 2,
    Rook    = 3,
    Queen   = 4,
    King    = 5,
}

impl Class {
    pub const ALL: [Class; 6] = [
        Class::Pawn,
        Class::Knight,
        Class::Bishop,
        Class::Rook,
        Class::Queen,
        Class::King,
    ];

    #[inline(always)]
    pub fn iter() -> std::slice::Iter<'static, Class> {
        Self::ALL.iter()
    }
}
impl TryFrom<char> for Class {
    type Error = String;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'P' | 'p' => Ok(Class::Pawn),
            'N' | 'n' => Ok(Class::Knight),
            'B' | 'b' => Ok(Class::Bishop),
            'R' | 'r' => Ok(Class::Rook),
            'Q' | 'q' => Ok(Class::Queen),
            'K' | 'k' => Ok(Class::King),
            _ => Err(format!("'{}' is not a valid chess piece character", c)),
        }
    }
}









#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline(always)]
    pub fn opposite(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}







#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Piece {
    WhitePawn   = 0,
    WhiteKnight = 1,
    WhiteBishop = 2,
    WhiteRook   = 3,
    WhiteQueen  = 4,
    WhiteKing   = 5, 
    BlackPawn   = 6,
    BlackKnight = 7,
    BlackBishop = 8,
    BlackRook   = 9,
    BlackQueen  = 10,
    BlackKing   = 11,
    None        = 12,
}

impl Piece {
    #[inline(always)]
    pub fn new(class: Class, color: Color) -> Self {
        let val = (color as u8) * 6 + (class as u8);
        unsafe { std::mem::transmute::<u8, Piece>(val) }
    }

    #[inline(always)]
    pub fn to_char(&self) -> char {
        match self {
            Piece::WhitePawn   => 'P',
            Piece::WhiteKnight => 'N',
            Piece::WhiteBishop => 'B',
            Piece::WhiteRook   => 'R',
            Piece::WhiteQueen  => 'Q',
            Piece::WhiteKing   => 'K',
            Piece::BlackPawn   => 'p',
            Piece::BlackKnight => 'n',
            Piece::BlackBishop => 'b',
            Piece::BlackRook   => 'r',
            Piece::BlackQueen  => 'q',
            Piece::BlackKing   => 'k',
            Piece::None        => ' ', 
        }
    }
    #[inline(always)]
    pub fn class(&self) -> Class {
        let v = *self as u8;
        unsafe { std::mem::transmute::<u8, Class>(v % 6) }
    }

    #[inline(always)]
    pub fn color(&self) -> Color {
        let v = *self as u8;
        unsafe { std::mem::transmute::<u8, Color>(v / 6) }
    }

    #[inline(always)]
    pub fn is_none(&self) -> bool {
        matches!(self, Piece::None)
    }

    pub const ALL: [Piece; 12] = [
        Piece::WhitePawn, Piece::WhiteKnight, Piece::WhiteBishop, Piece::WhiteRook, Piece::WhiteQueen, Piece::WhiteKing,
        Piece::BlackPawn, Piece::BlackKnight, Piece::BlackBishop, Piece::BlackRook, Piece::BlackQueen, Piece::BlackKing,
    ];

    #[inline(always)]
    pub fn iter() -> std::slice::Iter<'static, Piece> {
        Self::ALL.iter()
    }
}

impl TryFrom<char> for Piece {
    type Error = String;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'P' => Ok(Piece::WhitePawn),
            'N' => Ok(Piece::WhiteKnight),
            'B' => Ok(Piece::WhiteBishop),
            'R' => Ok(Piece::WhiteRook),
            'Q' => Ok(Piece::WhiteQueen),
            'K' => Ok(Piece::WhiteKing),
            'p' => Ok(Piece::BlackPawn),
            'n' => Ok(Piece::BlackKnight),
            'b' => Ok(Piece::BlackBishop),
            'r' => Ok(Piece::BlackRook),
            'q' => Ok(Piece::BlackQueen),
            'k' => Ok(Piece::BlackKing),
            ' ' => Ok(Piece::None),
            _ => Err(format!("'{}' is not a valid chess piece character", c)),
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}