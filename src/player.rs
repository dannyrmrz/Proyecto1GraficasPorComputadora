// player.rs

use raylib::prelude::*;
use std::f32::consts::PI;

use crate::maze::Maze;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32, // field of view
}

pub fn process_events(player: &mut Player, rl: &RaylibHandle, maze: &Maze, block_size: usize) {
    const MOVE_SPEED: f32 = 6.0;
    const ROTATION_SPEED: f32 = PI / 10.0;

    // Keyboard rotation
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a += ROTATION_SPEED;
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a -= ROTATION_SPEED;
    }

    // Mouse horizontal rotation
    let mouse_delta = rl.get_mouse_delta();
    player.a -= mouse_delta.x * 0.003;

    // Normalize angle to [0, 2PI)
    if player.a >= 2.0 * PI { player.a -= 2.0 * PI; }
    if player.a < 0.0 { player.a += 2.0 * PI; }

    // Attempt movement with collision detection
    let move_dir_forward = Vector2::new(player.a.cos(), player.a.sin());
    let move_dir_backward = Vector2::new(-player.a.cos(), -player.a.sin());

    let try_move = |pos: Vector2, dir: Vector2| -> Vector2 {
        let target = Vector2::new(pos.x + dir.x, pos.y + dir.y);
        let i = (target.x as usize) / block_size;
        let j = (target.y as usize) / block_size;
        if j < maze.len() && i < maze[0].len() && maze[j][i] == ' ' {
            target
        } else {
            pos
        }
    };

    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        // move backward with small steps to avoid tunneling
        for _ in 0..(MOVE_SPEED as i32) {
            player.pos = try_move(player.pos, Vector2::new(move_dir_backward.x, move_dir_backward.y));
        }
    }
    if rl.is_key_down(KeyboardKey::KEY_UP) {
        for _ in 0..(MOVE_SPEED as i32) {
            player.pos = try_move(player.pos, Vector2::new(move_dir_forward.x, move_dir_forward.y));
        }
    }
}
