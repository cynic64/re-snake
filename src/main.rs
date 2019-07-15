extern crate render_engine;
extern crate rand;

use render_engine as re;
use re::*;

const GRID_SIZE: u32 = 40;
// seconds in between movements of the snake
const MOVE_TIME: f32 = 0.1;

fn main() {
    let mut app = App::new();
    app.camera = Box::new(OrthoCamera { });

    let mut snake = Snake::new();
    let mut apple = Apple::new();

    let mut direction = Direction::Right;
    let mut time_of_last_move = std::time::Instant::now();

    let mut score = 0;

    loop {
        app.clear_vertex_buffers();
        let snake_mesh = snake.create_vertices(&app.dimensions);
        app.new_vbuf_from_verts(&snake_mesh);
        let apple_mesh = apple.create_vertices(&app.dimensions);
        app.new_vbuf_from_verts(&apple_mesh);

        app.unprocessed_keydown_events
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

    fn create_vertices(&self, dimensions: &[u32; 2]) -> Vec<Vertex> {
        // convert the top-left corner of the grid coordinate into pixels
        let (tl_px, tl_py) = (self.position.x * GRID_SIZE, self.position.y * GRID_SIZE);
        // convert the top-left corner into vulkan coordinates (-1..1) by:
        // dividing by the dimensions (0..1), multiplying by 2 (0..2) and subtracting 1 (-1..1)
        let (tl_vx, tl_vy) = ((tl_px as f32) / (dimensions[0] as f32) * 2.0 - 1.0, (tl_py as f32) / (dimensions[1] as f32) * 2.0 - 1.0);
        // convert the grid size into a vertical distance in vulkano thingies and a horizontal one
        // now that was a terrible explanation
        let (gs_vx, gs_vy) = ((GRID_SIZE as f32) / (dimensions[0] as f32) * 2.0, (GRID_SIZE as f32) / (dimensions[1] as f32) * 2.0);

        vec![
            Vertex {
                position: [tl_vx, tl_vy, 0.0],
                color: [1.0, 0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [tl_vx + gs_vx, tl_vy, 0.0],
                color: [1.0, 0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [tl_vx + gs_vx, tl_vy + gs_vy, 0.0],
                color: [1.0, 0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [tl_vx, tl_vy, 0.0],
                color: [1.0, 0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [tl_vx, tl_vy + gs_vy, 0.0],
                color: [1.0, 0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [tl_vx + gs_vx, tl_vy + gs_vy, 0.0],
                color: [1.0, 0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            },
        ]
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

    fn create_vertices(&self, dimensions: &[u32; 2]) -> Vec<Vertex> {
        self.pieces
            .iter()
            .flat_map(|grid_coord| {
                // convert the top-left corner of the grid coordinate into pixels
                let (tl_px, tl_py) = (grid_coord.x * GRID_SIZE, grid_coord.y * GRID_SIZE);
                // convert the top-left corner into vulkan coordinates (-1..1) by:
                // dividing by the dimensions (0..1), multiplying by 2 (0..2) and subtracting 1 (-1..1)
                let (tl_vx, tl_vy) = ((tl_px as f32) / (dimensions[0] as f32) * 2.0 - 1.0, (tl_py as f32) / (dimensions[1] as f32) * 2.0 - 1.0);
                // convert the grid size into a vertical distance in vulkano thingies and a horizontal one
                // now that was a terrible explanation
                let (gs_vx, gs_vy) = ((GRID_SIZE as f32) / (dimensions[0] as f32) * 2.0, (GRID_SIZE as f32) / (dimensions[1] as f32) * 2.0);

                vec![
                    Vertex {
                        position: [tl_vx, tl_vy, 0.0],
                        color: [1.0, 1.0, 1.0],
                        normal: [1.0, 0.0, 0.0],
                    },
                    Vertex {
                        position: [tl_vx + gs_vx, tl_vy, 0.0],
                        color: [1.0, 1.0, 1.0],
                        normal: [1.0, 0.0, 0.0],
                    },
                    Vertex {
                        position: [tl_vx + gs_vx, tl_vy + gs_vy, 0.0],
                        color: [1.0, 1.0, 1.0],
                        normal: [1.0, 0.0, 0.0],
                    },
                    Vertex {
                        position: [tl_vx, tl_vy, 0.0],
                        color: [1.0, 1.0, 1.0],
                        normal: [1.0, 0.0, 0.0],
                    },
                    Vertex {
                        position: [tl_vx, tl_vy + gs_vy, 0.0],
                        color: [1.0, 1.0, 1.0],
                        normal: [1.0, 0.0, 0.0],
                    },
                    Vertex {
                        position: [tl_vx + gs_vx, tl_vy + gs_vy, 0.0],
                        color: [1.0, 1.0, 1.0],
                        normal: [1.0, 0.0, 0.0],
                    },
                ]
            })
            .collect()
    }
}

#[derive(Clone, Copy, PartialEq)]
struct GridCoord {
    x: u32,
    y: u32,
}
