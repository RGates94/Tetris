use filled::{FILLED, ROTATION_OFFSETS};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics;
use ggez::graphics::{window, Color, Rect};
use ggez::{Context, ContextBuilder, GameResult};
use num_derive::FromPrimitive;
use num_traits::cast::FromPrimitive;
use rand::prelude::{SliceRandom, ThreadRng};
use rand::Rng;
use std::mem::swap;
use std::time::{Duration, Instant};
use std::cmp::min;

mod filled;

#[derive(Debug, Clone, Copy)]
struct Piece {
    kind: Tetromino,
    column: u8,
    row: u8,
    rotation: u8,
}

#[derive(Debug, Clone, Copy)]
struct PieceBlockIter<'a> {
    block_kind: &'a [(u8, u8)],
    column: u8,
    row: u8,
}

impl<'a> Iterator for PieceBlockIter<'a> {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<(u8, u8)> {
        let ((x, y), remaining) = self.block_kind.split_first()?;
        self.block_kind = remaining;
        Some((*x + self.row, *y + self.column))
    }
}

impl Piece {
    fn filled(&self) -> PieceBlockIter {
        PieceBlockIter {
            block_kind: self.kind.filled(self.rotation),
            column: self.column,
            row: self.row,
        }
    }
    fn draw_ggez(self, ctx: &mut Context, x: f32, y: f32) -> GameResult {
        for (ypos, xpos) in self.filled() {
            let rectangle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(
                    x + 1.0 + 16.0 * xpos as f32,
                    y + 1.0 - 16.0 * (ypos + 1) as f32,
                    14.0,
                    14.0,
                ),
                self.kind.color(),
            )?;
            graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        }
        Ok(())
    }
    fn draw_ghost_ggez(self, ctx: &mut Context, x: f32, y: f32) -> GameResult {
        for (ypos, xpos) in self.filled() {
            let mut ghost_color = self.kind.color();
            ghost_color.a = 0.2;
            let rectangle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(
                    x + 1.0 + 16.0 * xpos as f32,
                    y + 1.0 - 16.0 * (ypos + 1) as f32,
                    14.0,
                    14.0,
                ),
                ghost_color,
            )?;
            graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        }
        Ok(())
    }
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
    fn height(self, rotation: u8) -> usize {
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
    fn width(self, rotation: u8) -> usize {
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
    fn color(self) -> Color {
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

#[derive(Default, Debug, Copy, Clone)]
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
        for (x, y) in piece.filled() {
            collides |= match self.board.get(x as usize).map(|x| {
                if let Some(z) = x.get(y as usize) {
                    z.filled.is_some()
                } else {
                    true
                }
            }) {
                Some(val) => val,
                None => continue,
            }
        }
        collides
    }
    fn move_piece_left(&self, piece: &mut Piece) -> bool {
        if piece.column == 0 {
            return false;
        }
        piece.column -= 1;
        if self.check_collision(*piece) {
            piece.column += 1;
            false
        } else {
            true
        }
    }
    fn move_piece_right(&self, piece: &mut Piece) -> bool {
        if piece.column >= 10 - piece.kind.width(piece.rotation as u8) as u8 {
            return false;
        }
        piece.column += 1;
        if self.check_collision(*piece) {
            piece.column -= 1;
            false
        } else {
            true
        }
    }
    fn das_left(&self, piece: &mut Piece) {
        while self.move_piece_left(piece) {}
    }
    fn das_right(&self, piece: &mut Piece) {
        while self.move_piece_right(piece) {}
    }
    fn move_piece_down(&mut self, piece: &mut Piece) -> bool {
        if piece.row == 0 {
            //soft drop when this is implemented
            self.hard_drop(*piece);
            true
        } else {
            piece.row -= 1;
            if self.check_collision(*piece) {
                piece.row += 1;
                self.hard_drop(*piece);
                return true;
            }
            false
        }
    }
    fn rotate_piece_clockwise(&self, piece: &mut Piece) {
        let (orginal_x, original_y) = ROTATION_OFFSETS[piece.kind as usize][piece.rotation as usize];
        piece.rotation = (piece.rotation + 1) % 4;
        let (new_x, new_y) = ROTATION_OFFSETS[piece.kind as usize][piece.rotation as usize];
        let (column, row) = (piece.column, piece.row);
        piece.row += new_y;
        piece.column += new_x;
        piece.row = min(piece.row.checked_sub(original_y).unwrap_or(0),20 - piece.kind.height(piece.rotation) as u8);
        piece.column = min(piece.column.checked_sub(orginal_x).unwrap_or(0),10 - piece.kind.width(piece.rotation) as u8);
        if self.check_collision(*piece) {
            piece.rotation = (piece.rotation + 3) % 4;
        }
    }
    fn rotate_piece_counterclockwise(&self, piece: &mut Piece) {
        let (orginal_x, original_y) = ROTATION_OFFSETS[piece.kind as usize][piece.rotation as usize];
        piece.rotation = (piece.rotation + 3) % 4;
        let (new_x, new_y) = ROTATION_OFFSETS[piece.kind as usize][piece.rotation as usize];
        let (column, row) = (piece.column, piece.row);
        piece.row += new_y;
        piece.column += new_x;
        piece.row = min(piece.row.checked_sub(original_y).unwrap_or(0),20 - piece.kind.height(piece.rotation) as u8);
        piece.column = min(piece.column.checked_sub(orginal_x).unwrap_or(0),10 - piece.kind.width(piece.rotation) as u8);
        if self.check_collision(*piece) {
            piece.rotation = (piece.rotation + 1) % 4;
        }
    }
    fn drop(&self, piece: &mut Piece) -> bool {
        if piece.column >= 10 {
            return false;
        }
        while piece.row > 0 {
            piece.row -= 1;
            if self.check_collision(*piece) {
                if piece.row < 20 - piece.kind.height(piece.rotation) as u8 {
                    piece.row += 1;
                    break;
                } else {
                    return false;
                }
            }
        }
        true
    }
    fn hard_drop(&mut self, mut piece: Piece) {
        if !self.drop(&mut piece) {
            *self = Board::default();
        }
        self.place_unchecked(piece);
        self.clear_lines();
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
        for (x, y) in piece.filled() {
            self.board[x as usize][y as usize].filled = Some(piece.kind)
        }
    }
    fn clear_lines(&mut self) {
        self.board = array_init::from_iter(
            self.board
                .iter()
                .filter(|row| row.iter().any(|cell| cell.filled.is_none()))
                .copied()
                .chain([[Cell::default(); 10]; 1].iter().cycle().copied()),
        )
        .unwrap();
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
    current_piece: Option<Piece>,
    das_time: Option<(Instant, bool)>,
    hold: (bool, Option<Tetromino>),
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
    fn _place_random(&mut self) {
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
    fn switch_hold(&mut self) {
        if !self.hold.0 {
            if let Some(mut current) = self.current_piece {
                if let (_, Some(kind)) = &mut self.hold {
                    swap(&mut current.kind, kind);
                    current.column = 3;
                    current.row = 18;
                } else {
                    self.hold.1 = Some(current.kind);
                    current = Piece {
                        kind: self.next_piece(),
                        column: 3,
                        row: 18,
                        rotation: 0,
                    };
                }
                self.hold.0 = true;
                self.current_piece = Some(current);
            }
        }
    }
}

impl EventHandler for Tetris {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if self.current_piece.is_none() {
            self.current_piece = Some(Piece {
                kind: self.next_piece(),
                column: 3,
                row: 18,
                rotation: 0,
            });
        }
        if let Some(mut time) = self.next_tick {
            if let Some(mut piece) = self.current_piece {
                if let Some((das_time, right)) = self.das_time {
                    if das_time < Instant::now() {
                        if right {
                            self.board.das_right(&mut piece)
                        } else {
                            self.board.das_left(&mut piece)
                        }
                        self.das_time = None;
                    }
                }
                self.current_piece = Some(piece);
                while Instant::now() > time {
                    if self.board.move_piece_down(&mut piece) {
                        self.current_piece = None;
                    } else {
                        self.current_piece = Some(piece);
                        self.tick_speed *= 499;
                        self.tick_speed /= 500;
                    }
                    time += self.tick_speed;
                }
                self.next_tick = Some(time);
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

        self.board
            .draw_board_ggez(ctx, width as f32 / 2.0 - 80.0, height as f32)?;
        if let Some(piece) = self.current_piece {
            piece.draw_ggez(ctx, width as f32 / 2.0 - 80.0, height as f32)?;
            let mut ghost = piece;
            self.board.drop(&mut ghost);
            ghost.draw_ghost_ggez(ctx, width as f32 / 2.0 - 80.0, height as f32)?;
        }

        if let (_, Some(kind)) = self.hold {
            Piece {
                kind,
                column: 0,
                row: 18,
                rotation: 0,
            }
            .draw_ggez(ctx, width as f32 / 2.0 - 160.0, height as f32)?;
        }

        self.current_batch
            .iter()
            .rev()
            .chain(self.next_batch.iter().rev())
            .take(5)
            .enumerate()
            .map(|(index, &kind)| Piece {
                kind,
                column: 12,
                row: 18 - 3 * index as u8,
                rotation: 0,
            })
            .map(|piece| piece.draw_ggez(ctx, width as f32 / 2.0 - 80.0, height as f32))
            .collect::<Result<Vec<()>, _>>()?;

        graphics::present(ctx)
    }
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        repeat: bool,
    ) {
        if repeat {
            return;
        }
        match keycode {
            KeyCode::Up => {
                if let Some(piece) = self.current_piece {
                    self.board.hard_drop(piece);
                    self.hold.0 = false;
                }
                self.current_piece = None;
            }
            KeyCode::Left => {
                if let Some(mut piece) = self.current_piece {
                    self.board.move_piece_left(&mut piece);
                    self.das_time = Some((Instant::now() + Duration::from_millis(60), false));
                    self.current_piece = Some(piece);
                }
            }
            KeyCode::Right => {
                if let Some(mut piece) = self.current_piece {
                    self.board.move_piece_right(&mut piece);
                    self.das_time = Some((Instant::now() + Duration::from_millis(60), true));
                    self.current_piece = Some(piece);
                }
            }
            KeyCode::C => {
                self.switch_hold();
            }
            KeyCode::X => {
                if let Some(mut piece) = self.current_piece {
                    self.board.rotate_piece_clockwise(&mut piece);
                    self.current_piece = Some(piece);
                }
            }
            KeyCode::Z => {
                if let Some(mut piece) = self.current_piece {
                    self.board.rotate_piece_counterclockwise(&mut piece);
                    self.current_piece = Some(piece);
                }
            }
            _ => {}
        };
    }
    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::Left => {
                if !if let Some((_, right)) = self.das_time {
                    right
                } else {
                    true
                } {
                    self.das_time = None;
                }
            }
            KeyCode::Right => {
                if if let Some((_, right)) = self.das_time {
                    right
                } else {
                    false
                } {
                    self.das_time = None;
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let (mut ctx, mut event_loop) = ContextBuilder::new("Tetris", "ix").build().unwrap();
    let mut test = Tetris::default();
    let rng = rand::thread_rng();
    test.rng = rng;
    test.tick_speed = Duration::from_millis(1000);
    test.next_tick = Some(Instant::now() + Duration::from_millis(1000));
    match event::run(&mut ctx, &mut event_loop, &mut test) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}
