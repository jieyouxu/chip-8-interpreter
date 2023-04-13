use raylib::prelude::*;

const RECT_LEN: usize = 20;
const GRID_WIDTH: usize = 64;
const GRID_HEIGHT: usize = 32;

pub fn main() {
    let (mut rl, thread) = raylib::init()
        .size((GRID_WIDTH * 10) as i32, (GRID_HEIGHT * 10) as i32)
        .title("CHIP-8 Interpreter")
        .build();

    let mut grid = [false; GRID_WIDTH * GRID_HEIGHT];

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        draw_grid(&grid, &mut d);
    }
}

fn draw_grid(grid: &[bool; GRID_WIDTH * GRID_HEIGHT], d: &mut RaylibDrawHandle) {
    let mut gx = 0;
    let mut gy = 0;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            match grid[(y * GRID_WIDTH + x) as usize] {
                true => d.draw_rectangle(
                    (gx * RECT_LEN) as i32,
                    (gy * RECT_LEN) as i32,
                    RECT_LEN as i32,
                    RECT_LEN as i32,
                    Color::WHITE,
                ),
                false => d.draw_rectangle(
                    (gx * RECT_LEN) as i32,
                    (gy * RECT_LEN) as i32,
                    RECT_LEN as i32,
                    RECT_LEN as i32,
                    Color::BLACK,
                ),
            }
            gx += 1;
        }
        gx = 0;
        gy += 1;
    }
}
