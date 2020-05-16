use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::graphics::{window, Color, Rect};
use ggez::{Context, ContextBuilder, GameResult};
use num_derive::FromPrimitive;
use num_traits::cast::FromPrimitive;
use rand::prelude::{SliceRandom, ThreadRng};
use rand::Rng;
use std::mem::swap;
use std::time::{Duration, Instant};
use filled::FILLED;

mod filled;

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

impl Piece {
    fn filled(&self, rotation: u8) -> &[(u8, u8)] {
        match self {
            Self::O => &FILLED[rotation as usize][0..4],
            Self::T => &FILLED[rotation as usize][4..8],
            Self::L => &FILLED[rotation as usize][8..12],
            Self::J => &FILLED[rotation as usize][12..16],
            Self::S => &FILLED[rotation as usize][16..20],
            Self::Z => &FILLED[rotation as usize][20..24],
            Self::I => &FILLED[rotation as usize][24..28],
        }
    }
    fn height(&self, rotation: u8) -> usize {
        match rotation {
            0 => match self {
                    Self::I => 1,
                    _ => 2,
                },
            1 => match self {
                Self::O => 2,
                Self::I => 4,
                _ => 3,
            },
            _ => unimplemented!(),
        }
    }
    fn width(&self, rotation: u8) -> usize {
        match rotation {
            0 => match self {
                    Self::O => 2,
                    Self::I => 4,
                    _ => 3,
                },
            1 => match self {
                Self::I => 1,
                _ => 2,
            }
            _ => unimplemented!(),
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
    fn hard_drop(&mut self, piece: Piece, column: u8, rotation: u8) {
        if column >= 10 {
            return;
        }
        let mut target_row = 0;
        for row in (0..20).rev() {
            if {
                let mut collides = false;
                for (x, y) in piece.filled(rotation) {
                    collides |= match self.board.get(row + *x as usize).map(|x|x[(column + *y) as usize]
                        .filled
                        .is_some()) {
                        Some(val) => val,
                        None => continue,
                    }
                }
                collides
            } {
                if row < 20 - piece.height(rotation) {
                    target_row = row + 1;
                    break;
                } else {
                    *self = Board::default();
                    return;
                }
            }
        }
        for (x, y) in piece.filled(rotation) {
            self.board[target_row + *x as usize][(column + *y) as usize].filled = Some(piece)
        }
    }
}

fn generate_batch(rng: &mut ThreadRng, batch: &mut Vec<Piece>) {
    *batch = (0..7).map(|x| Piece::from_i8(x).unwrap()).collect();
    batch.shuffle(rng);
}

#[derive(Default, Debug)]
struct Tetris {
    next_tick: Option<Instant>,
    tick_speed: Duration,
    board: Board,
    rng: ThreadRng,
    current_batch: Vec<Piece>,
    next_batch: Vec<Piece>,
}

impl EventHandler for Tetris {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if self.current_batch.is_empty() {
            swap(&mut self.current_batch, &mut self.next_batch);
            generate_batch(&mut self.rng, &mut self.next_batch);
            if self.current_batch.is_empty() {
                generate_batch(&mut self.rng, &mut self.current_batch)
            }
        }
        if let Some(time) = &mut self.next_tick {
            while Instant::now() > *time {
                let piece = self.current_batch.pop().unwrap();
                let rotation = self.rng.gen_range(0, 2);
                let column = self.rng.gen_range(0, 11 - piece.width(rotation) as u8);
                self.board.hard_drop(piece, column, rotation);
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
            (63, 191, 191).into(),
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
            (0, 0, 0).into(),
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
    test.tick_speed = Duration::from_millis(500);
    test.next_tick = Some(Instant::now() + Duration::from_millis(500));
    match event::run(&mut ctx, &mut event_loop, &mut test) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}
