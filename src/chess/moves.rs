use std::str::FromStr;
use std::fmt;

use crate::chess::piece::Class;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tile {
    idx: u8,
}

impl Tile {
    #[inline(always)]
    unsafe fn from_str_unchecked(s: &str) -> Self {
        let bytes = s.as_bytes();
        
        unsafe {
            let f = *bytes.get_unchecked(0) - b'a';
            let r = *bytes.get_unchecked(1) - b'1';
            Self { idx: r * 8 + f } 
        }
    }

    #[inline(always)]
    pub fn from_idx(idx: u8) -> Self {
        debug_assert!(idx < 64, "tile index out of range: {}", idx);
        Self { idx: idx }
    }


    #[inline(always)]
    pub fn check_valid_tile(s: &str) -> bool {
        let bytes = s.as_bytes();
        s.len() == 2 && (b'a'..=b'h').contains(&bytes[0]) && (b'1'..=b'8').contains(&bytes[1])
    }

    #[inline(always)]
    pub fn get_idx(&self) -> usize {
        self.idx as usize
    }
    #[inline(always)]
    pub fn get_file_rank(&self) -> (u8,u8){
        let file = self.idx % 8;
        let rank = self.idx / 8;
        (file,rank)
    }




    pub fn iter() -> impl Iterator<Item = Tile> {
        (0u8..64).map(|idx| Tile { idx })
    }
}


impl FromStr for Tile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if Self::check_valid_tile(s) {
            Ok(unsafe { Self::from_str_unchecked(s) })
        } else {
            Err(format!("Invalid tile string: {}", s))
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = (self.idx % 8) + b'a';
        let rank = (self.idx / 8) + b'1';
        write!(f, "{}{}", file as char, rank as char)
    }
}









pub struct MoveInstruction{
    pub from: Tile,
    pub to: Tile,
    pub promotion: Option<Class>,
}


impl FromStr for MoveInstruction{
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.len() != 4 && s.len() != 5 {
            return Err(format!("Expected a move like \"e2e4\" or \"e7e8q\", got \"{}\"", s));
        }

        let from: Tile = s[0..2].parse()?;
        let to: Tile = s[2..4].parse()?;

        let promotion = if s.len() == 5 {
            let class = Class::try_from(s.as_bytes()[4] as char)?;
            if class == Class::Pawn || class == Class::King {
                return Err(format!("Cannot promote to {:?}", class));
            }
            Some(class)
        } else {
            None
        };

        Ok(Self { from, to, promotion })
    }
}









#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum MoveFlag {
    Quiet = 0b0000,
    DoublePawnPush = 0b0001,
    KingCastle = 0b0010,
    QueenCastle = 0b0011,
    Capture = 0b0100,
    EpCapture = 0b0101,
    // Add 0b0110 and 0b0111 if you need intermediate states, but 16 states total must exist for safe transmuting
    _Unused1 = 0b0110, 
    _Unused2 = 0b0111,
    KnightPromo = 0b1000,
    BishopPromo = 0b1001,
    RookPromo = 0b1010,
    QueenPromo = 0b1011,
    KnightPromoCapture = 0b1100,
    BishopPromoCapture = 0b1101,
    RookPromoCapture = 0b1110,
    QueenPromoCapture = 0b1111,
}

impl MoveFlag {
    #[inline(always)]
    fn from_u16(val: u16) -> Self {
        unsafe { std::mem::transmute(val & 0b1111) }
    }
}







#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    signature: u16,
}

impl Move {
    #[inline(always)]
    pub(crate) fn new(from: Tile, to: Tile, flag: MoveFlag) -> Self {
        Move {
            signature: (to.idx as u16) | ((from.idx as u16) << 6) | ((flag as u16) << 12)
        }
    }

    #[inline(always)]
    pub fn to(&self) -> Tile {
        Tile { idx: (self.signature & 0b111111) as u8 }
    }

    #[inline(always)]
    pub fn from(&self) -> Tile {
        Tile { idx: ((self.signature >> 6) & 0b111111) as u8 }
    }

    #[inline(always)]
    pub fn flag(&self) -> MoveFlag {
        MoveFlag::from_u16(self.signature >> 12)
    }

}