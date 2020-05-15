#[derive(Debug)]
enum Piece {
    O,
    T,
    L,
    J,
    S,
    Z,
    I,
}

static FILLED: [(u8, u8); 28] = [
    (0, 0), //O1
    (0, 1),
    (1, 0),
    (1, 1),
    (0, 0), //T1
    (0, 1),
    (0, 2),
    (1, 1),
    (0, 0),
    (0, 1),
    (1, 0),
    (1, 1),
    (0, 0),
    (0, 1),
    (1, 0),
    (1, 1),
    (0, 0),
    (0, 1),
    (1, 0),
    (1, 1),
    (0, 0),
    (0, 1),
    (1, 0),
    (1, 1),
    (0, 0),
    (0, 1),
    (1, 0),
    (1, 1),
];

impl Piece {
    fn filled(&self) -> &[(u8, u8)] {
        match self {
            Self::O => &FILLED[0..4],
            Self::T => &FILLED[4..8],
            Self::L => &FILLED[8..12],
            Self::J => &FILLED[12..16],
            Self::S => &FILLED[16..20],
            Self::Z => &FILLED[20..24],
            Self::I => &FILLED[24..28],
        }
    }
}

#[derive(Default, Debug)]
struct Cell {
    filled: bool,
}

#[derive(Default, Debug)]
struct Board {
    board: [[Cell; 10]; 20],
}

impl Board {
    fn hard_drop(&mut self, piece: Piece, column: u8, rotation: u8) {
        if column >= 10 {
            return;
        }
        let mut target_row = 0;
        for (index, row) in self.board.iter().enumerate().rev() {
            if row[column as usize].filled {
                if index < 19 {
                    target_row = index + 1;
                    break;
                }
            }
        }
        for (x, y) in piece.filled() {
            self.board[target_row + *x as usize][(column + *y) as usize].filled = true
        }
    }
}

#[derive(Default, Debug)]
struct Tetris {
    board: Board,
}

fn main() {
    let mut test = Tetris::default();
    println!("{:?}", test);
    test.board.hard_drop(Piece::O, 3, 3);
    println!("{:?}", test);
    test.board.hard_drop(Piece::T, 4, 2);
    println!("{:?}", test);
}
