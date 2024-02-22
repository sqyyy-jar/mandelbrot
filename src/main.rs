use std::{
    io::Cursor,
    ops::{Add, Div, Mul, Sub},
};

use image::{ImageFormat, ImageOutputFormat, Rgb, RgbImage};
use macroquad::{
    color::WHITE,
    input::{is_mouse_button_released, mouse_position_local, MouseButton},
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
    window::{next_frame, Conf},
};

pub type Complex = Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn len(self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn hadamard(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.x * other.x - self.y * other.y,
            self.x * other.y + self.y * other.x,
        )
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f64> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

/// [n]: amount of iterations
/// [c]: position
pub fn mandelbrot(n: u32, c: Vec2) -> f64 {
    let mut z = c;
    for _ in 0..n {
        z = z.cross(z) + c;
    }
    z.len()
}

/// Iterations per depth
const ACCUARACY: u32 = 30;
/// Bias per depth
const BIAS: f64 = 500.0;

pub fn render_mandelbrot(pos: Vec2, size: Vec2, width: u32, depth: u32) -> RgbImage {
    let height = (width as f64 * size.y / size.x) as u32;
    let x_step = size.x / width as f64;
    let y_step = size.y / height as f64;
    let mut image = RgbImage::new(width, height);
    for ix in 0..width {
        for iy in 0..height {
            let mx = pos.x + ix as f64 * x_step;
            let my = pos.y + iy as f64 * y_step;
            let scale = BIAS * depth as f64;
            let z = mandelbrot(depth * ACCUARACY, Vec2::new(mx, my)).min(scale);
            let color = (z / scale * 255.0) as u8;
            image.put_pixel(ix, iy, Rgb([0, color, 0]));
        }
    }
    image
}

pub fn render_to_texture(pos: Vec2, size: Vec2, depth: u32) -> Texture2D {
    let image = render_mandelbrot(pos, size, 1200, depth);
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageOutputFormat::Png)
        .unwrap();
    Texture2D::from_file_with_format(&bytes, Some(ImageFormat::Png))
}

fn conf() -> Conf {
    Conf {
        window_title: "Mandelbrot".into(),
        window_width: 1200,
        window_height: 900,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut stack = Vec::new();
    let mut texture;
    {
        // The default boundaries are: (-2.5, -1.2) to (0.7, 1.2)
        let pos = Vec2::new(-2.5, -1.2);
        let size = Vec2::new(3.2, 2.4);
        texture = render_to_texture(pos, size, 1);
        stack.push((pos, size));
    }
    loop {
        if is_mouse_button_released(MouseButton::Left) {
            let &(pos, size) = stack.last().unwrap();
            let mouse_pos = mouse_position_local();
            let new_size = size / 4.0;
            let half_new_size = new_size / 2.0;
            let clicked_pos = Vec2::new(
                pos.x + (mouse_pos.x as f64 + 1.0) / 2.0 * size.x,
                pos.y + (mouse_pos.y as f64 + 1.0) / 2.0 * size.y,
            );
            let new_pos = clicked_pos - half_new_size;
            texture = render_to_texture(new_pos, new_size, stack.len() as u32);
            stack.push((new_pos, new_size));
        } else if is_mouse_button_released(MouseButton::Right) && stack.len() > 1 {
            stack.pop().unwrap();
            let &(pos, size) = stack.last().unwrap();
            texture = render_to_texture(pos, size, stack.len() as u32);
        }
        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(macroquad::math::Vec2::new(1200.0, 900.0)),
                ..Default::default()
            },
        );
        next_frame().await
    }
}
