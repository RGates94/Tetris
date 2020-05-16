use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::graphics::{window, Color, Rect};
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
    fn width(&self, _rotation: u8) -> usize {
        match self {
            Self::O => 2,
            Self::I => 4,
            _ => 3,
        }
    }
    fn color(&self) -> Color {
        match self {
            Self::O => (255, 255, 0),
            Self::T => (255, 0, 255),
            Self::L => (255, 127, 0),
            Self::J => (63, 63, 255),
            Self::S => (63, 255, 63),
            Self::Z => (255, 0, 0),
            Self::I => (0, 255, 255),
        }
        .into()
    }
}

#[derive(Default, Debug)]
struct Cell {
    filled: Option<Piece>,
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
                    collides |= self.board[row + *x as usize][(column + *y) as usize]
                        .filled
                        .is_some()
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
            self.board[target_row + *x as usize][(column + *y) as usize].filled = Some(piece)
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
        let column = self.rng.gen_range(0, 11 - piece.width(0) as u8);
        if let Some(time) = &mut self.next_tick {
            while Instant::now() > *time {
                self.board.hard_drop(piece, column, 2);
                *time += self.tick_speed;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let (width, height): (f64, f64) = window(&ctx).get_inner_size().unwrap().into();

        let outer = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(
                width as f32 / 2.0 - 89.0,
                height as f32 - 329.0,
                178.0,
                329.0,
            ),
            (63,191,191).into(),
        )?;

        let inner = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(
                width as f32 / 2.0 - 81.0,
                height as f32 - 321.0,
                162.0,
                321.0,
            ),
            (0,0,0).into(),
        )?;

        graphics::draw(ctx, &outer, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        graphics::draw(ctx, &inner, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;

        for (ypos, row) in self.board.board.iter().enumerate() {
            for (xpos, cell) in row.iter().enumerate() {
                if cell.filled.is_some() {
                    let rectangle = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        Rect::new(
                            width as f32 / 2.0 - 79.0 + 16.0 * xpos as f32,
                            height as f32 + 1.0 - 16.0 * (ypos + 1) as f32,
                            14.0,
                            14.0,
                        ),
                        cell.filled.unwrap().color(),
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
    test.rng = rng;
    test.tick_speed = Duration::from_millis(100);
    test.next_tick = Some(Instant::now() + Duration::from_millis(500));
    match event::run(&mut ctx, &mut event_loop, &mut test) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}
