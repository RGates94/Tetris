use filled::FILLED;
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

mod filled;

#[derive(Debug, Clone, Copy)]
struct Piece {
    kind: Tetromino,
    column: u8,
    row: u8,
    rotation: u8,
}

#[derive(Debug, FromPrimitive, Clone, Copy)]
enum Tetromino {
    O,
    T,
    L,
    J,
    S,
    Z,
    I,
}

impl Tetromino {
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
        if rotation % 2 == 0 {
            match self {
                Self::I => 1,
                _ => 2,
            }
        } else {
            match self {
                Self::O => 2,
                Self::I => 4,
                _ => 3,
            }
        }
    }
    fn width(&self, rotation: u8) -> usize {
        if rotation % 2 == 0 {
            match self {
                Self::O => 2,
                Self::I => 4,
                _ => 3,
            }
        } else {
            match self {
                Self::I => 1,
                _ => 2,
            }
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
    filled: Option<Tetromino>,
}

#[derive(Default, Debug)]
struct Board {
    board: [[Cell; 10]; 20],
}

impl Board {
    fn check_collision(&self, piece: Piece) -> bool {
        let mut collides = false;
        for (x, y) in piece.kind.filled(piece.rotation) {
            collides |= match self
                .board
                .get((piece.row + *x) as usize)
                .map(|x| x[(piece.column + *y) as usize].filled.is_some())
            {
                Some(val) => val,
                None => continue,
            }
        }
        collides
    }
    fn hard_drop(&mut self, mut piece: Piece) {
        if piece.column >= 10 {
            return;
        }
        while piece.row > 0 {
            piece.row -= 1;
            if { self.check_collision(piece) } {
                if piece.row < 20 - piece.kind.height(piece.rotation) as u8 {
                    piece.row += 1;
                    break;
                } else {
                    *self = Board::default();
                    return;
                }
            }
        }
        self.place_unchecked(piece);
    }
    fn _place_checked(&mut self, piece: Piece) -> bool {
        if self.check_collision(piece) {
            self.place_unchecked(piece);
            true
        } else {
            false
        }
    }
    fn place_unchecked(&mut self, piece: Piece) {
        for (x, y) in piece.kind.filled(piece.rotation) {
            self.board[(piece.row + *x) as usize][(piece.column + *y) as usize].filled = Some(piece.kind)
        }
    }
    fn draw_board_ggez(&self, ctx: &mut Context, x: f32, y: f32) -> GameResult {
        for (ypos, row) in self.board.iter().enumerate() {
            for (xpos, cell) in row.iter().enumerate() {
                if cell.filled.is_some() {
                    let rectangle = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        Rect::new(
                            x + 1.0 + 16.0 * xpos as f32,
                            y + 1.0 - 16.0 * (ypos + 1) as f32,
                            14.0,
                            14.0,
                        ),
                        cell.filled.unwrap().color(),
                    )?;
                    graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
                }
            }
        }
        Ok(())
    }
}

fn generate_batch(rng: &mut ThreadRng, batch: &mut Vec<Tetromino>) {
    *batch = (0..7).map(|x| Tetromino::from_i8(x).unwrap()).collect();
    batch.shuffle(rng);
}

#[derive(Default, Debug)]
struct Tetris {
    next_tick: Option<Instant>,
    tick_speed: Duration,
    board: Board,
    rng: ThreadRng,
    current_batch: Vec<Tetromino>,
    next_batch: Vec<Tetromino>,
    next_piece: Option<Piece>,
}

impl Tetris {
    fn next_piece(&mut self) -> Tetromino {
        if self.current_batch.is_empty() {
            swap(&mut self.current_batch, &mut self.next_batch);
            generate_batch(&mut self.rng, &mut self.next_batch);
            if self.current_batch.is_empty() {
                generate_batch(&mut self.rng, &mut self.current_batch)
            }
        }
        self.current_batch.pop().unwrap()
    }
    fn place_random(&mut self) {
        let kind = self.next_piece();
        let rotation = self.rng.gen_range(0, 4);
        let column = self.rng.gen_range(0, 11 - kind.width(rotation) as u8);
        let piece = Piece {
            kind,
            column,
            row: 20,
            rotation,
        };
        self.board.hard_drop(piece);
    }
}

impl EventHandler for Tetris {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if let Some(mut time) = self.next_tick {
            while Instant::now() > time {
                self.place_random();
                time += self.tick_speed;
            }
            self.next_tick = Some(time);
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

        self.board
            .draw_board_ggez(ctx, width as f32 / 2.0 - 80.0, height as f32)?;

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
