extern crate render_engine;
extern crate rand;

use render_engine as re;

use re::exposed_tools::*;
use re::app::App;

const GRID_SIZE: u32 = 40;
// seconds in between movements of the snake
const MOVE_TIME: f32 = 0.1;

fn main() {
    let mut app = App::new();

    let mut snake = Snake::new();
    let mut apple = Apple::new();

    let mut direction = Direction::Right;
    let mut time_of_last_move = std::time::Instant::now();

    let mut score = 0;

    loop {
        app.clear_vertex_buffers();
        let snake_mesh = snake.create_vertices(&app);
        app.new_vbuf_from_verts(&snake_mesh);
        let apple_mesh = apple.create_vertices(&app);
        app.new_vbuf_from_verts(&apple_mesh);

        app.unprocessed_events
            .iter()
            .for_each(|&keycode| match keycode {
                VirtualKeyCode::W => direction = Direction::Up,
                VirtualKeyCode::A => direction = Direction::Left,
                VirtualKeyCode::S => direction = Direction::Down,
                VirtualKeyCode::D => direction = Direction::Right,
                _ => {}
            });

        app.draw_frame();

        if get_elapsed(time_of_last_move) > MOVE_TIME {
            snake.move_direction(&direction);
            time_of_last_move = std::time::Instant::now();

            if snake.ate_apple(&apple) {
                apple.randomize_position();
                snake.must_grow = true;
                score += 1;
                println!("New score: {}", score);
            }

            if snake.ran_into_self() {
                println!("You died -.-");
                break;
            }
        }

        if app.done {
            break;
        }
    }

    app.print_fps();
}

struct Apple {
    position: GridCoord,
}

impl Apple {
    fn new() -> Self {
        Apple {
            position: Self::random_grid_coord(),
        }
    }

    fn create_vertices(&self, app: &App) -> Vec<Vertex> {
        let square = Square {
            corner: self.position.to_pixel_coord(),
            size: GRID_SIZE,
            color: [1.0, 0.0, 0.0, 1.0],
        };

        square.create_vertices(app)
    }

    fn randomize_position(&mut self) {
        self.position = Self::random_grid_coord();
    }

    fn random_grid_coord() -> GridCoord {
        GridCoord {
            x: rand::random::<u32>() % 40 + 2,
            y: rand::random::<u32>() % 20 + 2,
        }
    }
}

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

struct Snake {
    pieces: Vec<GridCoord>,
    pub must_grow: bool,
}

impl Snake {
    fn new() -> Self {
        Snake {
            pieces: vec![
                GridCoord { x: 5, y: 5 },
                GridCoord { x: 5, y: 6 },
                GridCoord { x: 5, y: 7 },
                GridCoord { x: 5, y: 8 },
            ],
            must_grow: false,
        }
    }

    fn ran_into_self(&self) -> bool {
        self.pieces.iter().enumerate().any(|(idx1, coord1)| {
            self.pieces.iter().enumerate().any(|(idx2, coord2)| {
                idx1 != idx2 && coord1 == coord2
            })
        })
    }

    fn ate_apple(&self, apple: &Apple) -> bool {
        self.pieces.iter().any(|gc: &GridCoord| gc == &apple.position)
    }

    fn move_direction(&mut self, direction: &Direction) {
        match direction {
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::Up => self.move_up(),
            Direction::Down => self.move_down(),
        }
    }

    fn move_left(&mut self) {
        self.shift_all_except_head();
        self.pieces[0].x -= 1;
    }

    fn move_right(&mut self) {
        self.shift_all_except_head();
        self.pieces[0].x += 1;
    }

    fn move_up(&mut self) {
        self.shift_all_except_head();
        self.pieces[0].y -= 1;
    }

    fn move_down(&mut self) {
        self.shift_all_except_head();
        self.pieces[0].y += 1;
    }

    fn shift_all_except_head(&mut self) {
        let grown_piece_coord = if self.must_grow {
            self.must_grow = false;
            Some(self.pieces[self.pieces.len() - 1])
        } else {
            None
        };

        for idx in (1..self.pieces.len()).rev() {
            self.pieces[idx] = self.pieces[idx - 1];
        }

        if let Some(coord) = grown_piece_coord {
            self.pieces.push(coord);
        }
    }

    fn create_vertices(&self, app: &App) -> Vec<Vertex> {
        self.pieces
            .iter()
            .flat_map(|grid_coord| {
                let corner = grid_coord.to_pixel_coord();
                let square = Square {
                    corner,
                    size: GRID_SIZE,
                    color: [1.0, 1.0, 1.0, 1.0],
                };
                square.create_vertices(app)
            })
            .collect()
    }
}

#[derive(Clone, Copy, PartialEq)]
struct GridCoord {
    x: u32,
    y: u32,
}

impl GridCoord {
    fn to_pixel_coord(&self) -> PixelCoord {
        PixelCoord {
            x: self.x * GRID_SIZE,
            y: self.y * GRID_SIZE,
        }
    }
}
