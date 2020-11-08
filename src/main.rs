extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{ButtonArgs, ButtonEvent, Button, MouseButton, ButtonState, MouseCursorEvent};
use piston::event_loop::{Events, EventSettings};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use graphics::rectangle::Border;
use piston::window::WindowSettings;
use table::*;
use color_rgba::ColorsRGBA::*;

mod table;
mod color_rgba;

const WIDTH: f64 = 1200.0;
const HEIGHT: f64 = 600.0;
const CELL_SIZE: f64 = 30.0;

pub struct App {
    table: Table,
    time: f64,
    view_grid: bool,
    step_time: f64,
}

impl App {

    fn new(table: Table) -> App {
        App {
            table, 
            time: 0.0,
            view_grid: true,
            step_time: 0.5
        }
    }

    fn get_table(&self) -> &Table {
        &self.table
    }

    fn render(&mut self, args: &RenderArgs, glx: &mut GlGraphics) {
        use graphics::*;

        let table = self.get_table();
        let (rows, columns) = self.table.get_size();

        use Cell::*;
        glx.draw(args.viewport(), |c, gl| {
            for row in 0..rows {
                for column in 0..columns {
                    let color = match table.get((row, column)) {
                        Signal => BLUE.get(),
                        TailSignal => RED.get(),
                        Wire => YELLOW.get(),
                        _ => WHITE.get()
                    };
                    let square_x = column as f64 * CELL_SIZE; 
                    let square_y = row as f64 * CELL_SIZE; 
                    let mut delta = 0.0;
                    if self.view_grid {
                        delta = 0.5;
                    }
                    Rectangle::new(color)
                        .border(Border {color: BLACK.get(), radius: 0.1} )
                        .draw([square_x + delta, square_y + delta, CELL_SIZE - delta, CELL_SIZE - delta], &c.draw_state, c.transform, gl);
                }
            }
        });
    }

    fn update(&mut self, dt: f64) {
        self.time += dt;
        if self.time < self.step_time {
            return;
        }
        self.time -= self.step_time;
        let (rows, columns) = self.table.get_size();
        let mut updated_table = Table::new(rows, columns);
        for row in 0..rows {
            for column in 0..columns {
                use Cell::*;
                match self.table.get((row, column)) {
                    Wire => {
                        let count = self.table.around_count((row, column), Some(Signal));
                        if count == 1 || count == 2 {
                            updated_table.set((row, column), Signal);
                        } else {
                            updated_table.set((row, column), Wire);
                        }
                    },
                    Signal => updated_table.set((row, column), TailSignal),
                    TailSignal => updated_table.set((row, column), Wire),
                    _ => {}
                }
            }
        }
        self.table = updated_table
    }

    fn process_mouse(&mut self, button: &ButtonArgs, coord: (f64, f64)) {
        let table: &mut Table = &mut self.table;
        let (rows, columns) = table.get_size();
        let mut delta = 0.0;
        if self.view_grid {
            delta = 0.5;
        }
        let pos = (
            (coord.0 / (CELL_SIZE + delta)) as usize,
            (coord.1 / (CELL_SIZE + delta)) as usize
        );
        if pos.0 >= rows || pos.1 >= columns {
            return
        }
        if button.state == ButtonState::Press {
            if let Button::Mouse(mouse_button) = button.button {
                use Cell::*;
                if mouse_button == MouseButton::Left {
                    table.set(pos, Wire);
                } else if mouse_button == MouseButton::Middle {
                    table.set(pos, Void);
                } else if mouse_button == MouseButton::Right && *table.get(pos) == Wire {
                    table.set(pos, Signal);
                }
            }
        }
    }

}

#[warn(unused_imports)]
fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("Wireworld", [WIDTH, HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();


    let rows = (HEIGHT / CELL_SIZE) as usize;
    let columns = (WIDTH / CELL_SIZE) as usize;
    let table = Table::new(rows, columns);

    let mut gl = GlGraphics::new(opengl);

    let mut app = App::new(table);

    let mut events = Events::new(EventSettings::new());
    let mut cursor: Option<[f64; 2]> = None;
    let mut last_button_args: Option<ButtonArgs> = None;
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args, &mut gl);
        }

        let mut mouse_moved = true;
        e.mouse_cursor(|pos| {
            if cursor != Some(pos) {
                mouse_moved = true;
                cursor = Some(pos);
            } else {
                mouse_moved = false;
            }
        });
        if let Some(button) = e.button_args() {
            if last_button_args != Some(button) {
                last_button_args = Some(button);
            }
        }
        if mouse_moved  {
            if let Some(button) = last_button_args {
                if let Some(pos) = cursor {
                    app.process_mouse(&button, (
                            pos[1],
                            pos[0]
                        ),
                    );
                }
                if button.state == ButtonState::Release {
                    last_button_args = None;
                }
            }
        }
        if let Some(update_args) = e.update_args() {
            app.update(update_args.dt);
        }
    }
}