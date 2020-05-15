use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::graphics::{Rect, window};

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
    (0, 0), //L1
    (0, 1),
    (0, 2),
    (1, 2),
    (0, 0), //J1
    (0, 1),
    (0, 2),
    (1, 0),
    (0, 0), //S1
    (0, 1),
    (1, 1),
    (1, 2),
    (0, 1), //Z1
    (0, 2),
    (1, 0),
    (1, 1),
    (0, 0), //I1
    (0, 1),
    (0, 2),
    (0, 3),
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

impl EventHandler for Tetris {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let (width, height): (f64,f64) = window(&ctx).get_inner_size().unwrap().into();

        for (ypos, row) in self.board.board.iter().enumerate() {
            for (xpos, cell) in row.iter().enumerate() {
                if cell.filled {
                    let rectangle = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        Rect::new(16.0 * xpos as f32,height as f32 - 16.0 * (ypos + 1) as f32,14.0,14.0),
                        [0.25, 0.75, 0.75, 1.0].into(),
                    )?;
                    graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
                }
            }
        }

        graphics::present(ctx)
    }
}

fn main() {
    let (mut ctx, mut event_loop) =
        ContextBuilder::new("Tetris", "ix")
            .build()
            .unwrap();
    let mut test = Tetris::default();


    println!("{:?}", test);
    test.board.hard_drop(Piece::O, 3, 3);
    println!("{:?}", test);
    test.board.hard_drop(Piece::T, 4, 2);
    println!("{:?}", test);

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut test) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}
