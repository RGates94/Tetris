use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::graphics::{window, Rect};
use ggez::{Context, ContextBuilder, GameResult};
use num_derive::FromPrimitive;
use num_traits::cast::FromPrimitive;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::time::{Duration, Instant};

#[derive(Debug, FromPrimitive, Clone, Copy)]
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
    fn height(&self, _rotation: u8) -> usize {
        match self {
            Self::I => 1,
            _ => 2,
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
    fn hard_drop(&mut self, piece: Piece, column: u8, _rotation: u8) {
        if column >= 10 {
            return;
        }
        let mut target_row = 0;
        for row in (0..19).rev() {
            if {
                let mut collides = false;
                for (x, y) in piece.filled() {
                    collides |= self.board[row + *x as usize][(column + *y) as usize].filled
                }
                collides
            } {
                if row < 20 - piece.height(0) {
                    target_row = row + 1;
                    break;
                } else {
                    *self = Board::default();
                    return;
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
    next_tick: Option<Instant>,
    tick_speed: Duration,
    board: Board,
    rng: ThreadRng,
}

impl EventHandler for Tetris {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let piece = Piece::from_isize(self.rng.gen_range::<isize, _, _>(0, 7)).unwrap();
        if let Some(time) = &mut self.next_tick {
            while Instant::now() > *time {
                self.board.hard_drop(piece, 6, 2);
                *time += self.tick_speed;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let (width, height): (f64, f64) = window(&ctx).get_inner_size().unwrap().into();

        for (ypos, row) in self.board.board.iter().enumerate() {
            for (xpos, cell) in row.iter().enumerate() {
                if cell.filled {
                    let rectangle = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        Rect::new(
                            width as f32 / 2.0 - 80.0 + 16.0 * xpos as f32,
                            height as f32 - 16.0 * (ypos + 1) as f32,
                            14.0,
                            14.0,
                        ),
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
    let (mut ctx, mut event_loop) = ContextBuilder::new("Tetris", "ix").build().unwrap();
    let mut test = Tetris::default();
    let rng = rand::thread_rng();

    println!("{:?}", test);
    test.board.hard_drop(Piece::O, 3, 3);
    println!("{:?}", test);
    test.board.hard_drop(Piece::T, 4, 2);
    println!("{:?}", test);
    test.rng = rng;
    test.tick_speed = Duration::from_millis(500);
    test.next_tick = Some(Instant::now() + Duration::from_millis(2000));

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut test) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}
