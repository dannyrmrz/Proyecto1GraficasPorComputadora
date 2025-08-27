// caster.rs

use raylib::color::Color;

use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
  pub distance: f32,
  pub impact: char,
  pub hit_x: f32,
  pub hit_y: f32,
  pub vertical_side: bool,
}

pub fn cast_ray(
  framebuffer: &mut Framebuffer,
  maze: &Maze,
  player: &Player,
  a: f32,
  block_size: usize,
  draw_line: bool,
) -> Intersect {
  let mut d = 0.0;
  let mut prev_i: Option<usize> = None;
  let mut prev_j: Option<usize> = None;

  framebuffer.set_current_color(Color::WHITESMOKE);

  loop {
    let cos = d * a.cos();
    let sin = d * a.sin();
    let x = (player.pos.x + cos) as isize;
    let y = (player.pos.y + sin) as isize;

    if x < 0 || y < 0 { return Intersect{ distance: f32::MAX, impact: ' ', hit_x: player.pos.x, hit_y: player.pos.y, vertical_side: false }; }
    let i = (x as usize) / block_size;
    let j = (y as usize) / block_size;

    if j >= maze.len() || i >= maze[0].len() {
      return Intersect{ distance: f32::MAX, impact: ' ', hit_x: player.pos.x, hit_y: player.pos.y, vertical_side: false };
    }

    if maze[j][i] != ' ' && maze[j][i] != 'g' {
      let hit_x = player.pos.x + (d * a.cos());
      let hit_y = player.pos.y + (d * a.sin());
      let vertical_side = if let (Some(pi), Some(_pj)) = (prev_i, prev_j) {
        i != pi
      } else { false };
      return Intersect{ distance: d, impact: maze[j][i], hit_x, hit_y, vertical_side };
    }

    if draw_line {
      framebuffer.set_pixel(x as u32, y as u32);
    }

    prev_i = Some(i);
    prev_j = Some(j);
    d += 10.0;
  }
}



