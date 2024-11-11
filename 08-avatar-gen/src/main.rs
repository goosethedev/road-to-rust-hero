use svg::node::element::path::{Command, Data, Position};
use svg::node::element::{Path, Rectangle};
use svg::Document;

use crate::Operation::*;
use crate::Orientation::*;

const WIDTH: isize = 400;
const HEIGHT: isize = 400;
const STEP_SIZE: isize = HEIGHT / 10;
const STROKE_WIDTH: usize = 5;
const HOME_X: isize = 200;
const HOME_Y: isize = 200;

#[derive(Debug)]
enum Operation {
    Forward(isize),
    TurnLeft,
    TurnRight,
    Home,
    Noop(u8),
}

#[derive(Debug)]
enum Orientation {
    North,
    South,
    East,
    West,
}

/// Coordinates tracker to draw.
/// isize since at moments, coordinates can be negative.
#[derive(Debug)]
struct Artist {
    x: isize,
    y: isize,
    heading: Orientation,
}

impl Artist {
    fn new() -> Self {
        Artist {
            x: HOME_X,
            y: HOME_Y,
            heading: North,
        }
    }

    fn forward(&mut self, distance: isize) {
        match self.heading {
            North => self.y += distance,
            South => self.y -= distance,
            West => self.x += distance,
            East => self.x -= distance,
        };
    }

    fn go_home(&mut self) {
        (self.x, self.y) = (HOME_X, HOME_Y);
    }

    fn turn_left(&mut self) {
        self.heading = match self.heading {
            North => West,
            West => South,
            South => East,
            East => North,
        };
    }

    fn turn_right(&mut self) {
        self.heading = match self.heading {
            North => East,
            East => South,
            South => West,
            West => North,
        };
    }

    /// When X or Y go out of range, go to axis center.
    fn wrap(&mut self) {
        if self.x < 0 {
            self.x = HOME_X;
            self.heading = West;
        } else if self.x > WIDTH {
            self.x = HOME_X;
            self.heading = East;
        }

        if self.y < 0 {
            self.y = HOME_Y;
            self.heading = North;
        } else if self.y > HEIGHT {
            self.y = HOME_Y;
            self.heading = South;
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let hash = args.get(1).expect("Input hex code not provided");
    let filename = format!("{}.svg", args.get(2).unwrap_or(hash));

    let operations = parse_input(&hash.to_lowercase());
    let commands = parse_operations(&operations);
    let svg_doc = generate_svg(commands);

    svg::save(filename, &svg_doc).expect("Error saving the SVG file");
}

/// Parse input hash string into operations, one per character.
fn parse_input(input: &str) -> Vec<Operation> {
    input
        .bytes()
        .map(|byte| match byte {
            b'0' => Home,
            b'1'..=b'9' => {
                // ASCII numbers start at 48 i.e. 0x30
                let steps = (byte - 0x30) as isize;
                Forward(steps)
            }
            b'a' | b'b' | b'c' => TurnLeft,
            b'd' | b'e' | b'f' => TurnRight,
            _ => Noop(byte),
        })
        .collect()
}

/// Parse operations into SVG data commands.
fn parse_operations(ops: &Vec<Operation>) -> Vec<Command> {
    let mut turtle = Artist::new();

    let mut commands: Vec<Command> = Vec::with_capacity(1 + ops.len());
    commands.push(Command::Move(Position::Absolute, (HOME_X, HOME_Y).into()));

    for op in ops {
        match *op {
            Forward(steps) => turtle.forward(steps * STEP_SIZE),
            TurnLeft => turtle.turn_left(),
            TurnRight => turtle.turn_right(),
            Home => turtle.go_home(),
            Noop(byte) => eprintln!("warning: illegal byte encountered: {:?}", byte),
        };
        commands.push(Command::Line(
            Position::Absolute,
            (turtle.x, turtle.y).into(),
        ));
        turtle.wrap();
    }
    commands
}

/// Generate an SVG document object from a list of data commands.
fn generate_svg(path_data: Vec<Command>) -> Document {
    let background = Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("width", WIDTH)
        .set("height", HEIGHT)
        .set("fill", "#ffffff");

    let border = background
        .clone()
        .set("fill-opacity", 0)
        .set("stroke", "#aaaaaa")
        .set("stroke-width", STROKE_WIDTH * 3);

    let sketch = Path::new()
        .set("fill", "none")
        .set("stroke", "#2f2f2f")
        .set("stroke-width", STROKE_WIDTH)
        .set("stroke-opacity", "0.9")
        .set("d", Data::from(path_data));

    Document::new()
        .set("viewBox", (0, 0, HEIGHT, WIDTH))
        .set("height", HEIGHT)
        .set("width", WIDTH)
        .set("style", "style=\"outline: 5px solid #800000;\"")
        .add(background)
        .add(sketch)
        .add(border)
}
