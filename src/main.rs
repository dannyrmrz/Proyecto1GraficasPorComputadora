// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]

mod line;
mod framebuffer;
mod maze;
mod caster;
mod player;
mod textures;
mod audio;

use line::line;
use maze::{Maze,load_maze};
use caster::{cast_ray, Intersect};
use framebuffer::Framebuffer;
use player::{Player, process_events};
use textures::TextureManager;
use audio::AudioManager;

use raylib::prelude::*;
use std::thread;
use std::time::Duration;
use std::f32::consts::PI;
use std::fs;

fn cell_to_color(cell: char) -> Color {
  match cell {
    '+' => {
      return Color::BLUEVIOLET;
    },
    '-' => {
      return Color::VIOLET;
    },
    '|' => {
      return Color::VIOLET;
    },
    'g' => {
      return Color::GREEN;
    },
    _ => {
      return Color::WHITE;
    },
  }
}

fn draw_cell(
  framebuffer: &mut Framebuffer,
  xo: usize,
  yo: usize,
  block_size: usize,
  cell: char,
) {
  if cell == ' ' {
    return;
  }
  let color = cell_to_color(cell);
  framebuffer.set_current_color(color);

  for x in xo..xo + block_size {
    for y in yo..yo + block_size {
      framebuffer.set_pixel(x as u32, y as u32);
    }
  }
}

pub fn render_maze(
  framebuffer: &mut Framebuffer,
  maze: &Maze,
  block_size: usize,
  player: &Player,
) {
  for (row_index, row) in maze.iter().enumerate() {
    for (col_index, &cell) in row.iter().enumerate() {
      let xo = col_index * block_size;
      let yo = row_index * block_size;
      draw_cell(framebuffer, xo, yo, block_size, cell);
    }
  }

  framebuffer.set_current_color(Color::WHITESMOKE);

  // draw what the player sees
  let num_rays = 5;
  for i in 0..num_rays {
    let current_ray = i as f32 / num_rays as f32; // current ray divided by total rays
    let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
    cast_ray(framebuffer, &maze, &player, a, block_size, true);
  }
}

fn render_world(
  framebuffer: &mut Framebuffer,
  maze: &Maze,
  block_size: usize,
  player: &Player,
  texture_cache: &TextureManager,
  crumbs: &[(usize, usize)],
) {
  let num_rays = framebuffer.width;

  let hh = framebuffer.height as f32 / 2.0;  // precalculated half height
  let distance_to_projection_plane = 70.0; // how far is the "player" from the "camera"

  framebuffer.set_current_color(Color::WHITESMOKE);

  // Store wall distances for depth testing
  let mut wall_distances: Vec<f32> = vec![f32::MAX; num_rays as usize];

  // Render walls with real textures from wall.png
  for i in 0..num_rays {
    let current_ray = i as f32 / num_rays as f32; // current ray divided by total rays
    let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
    let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

    // Calculate the height of the stake
    let distance_to_wall = intersect.distance;// how far is this wall from the player
    // Store wall distance for depth testing
    wall_distances[i as usize] = distance_to_wall;
    
    // this ratio doesn't really matter as long as it is a function of distance
    let stake_height = (hh / distance_to_wall) * distance_to_projection_plane;

    // Calculate the position to draw the stake
    let stake_top = (hh - (stake_height / 2.0)) as usize;
    let stake_bottom = (hh + (stake_height / 2.0)) as usize;

              // Calculate texture coordinates for wall using wall.png
     let hit_u = if intersect.vertical_side {
       intersect.hit_y / block_size as f32
     } else {
       intersect.hit_x / block_size as f32
     };
     
     // Get texture coordinates from wall.png (64x64)
     let wall_tex_x = (hit_u * 64.0) as u32;
     
     for y in stake_top..stake_bottom {
       if y >= framebuffer.height as usize { continue; }
       
       let v = (y as f32 - stake_top as f32) / (stake_bottom as f32 - stake_top as f32 + 0.0001);
       let wall_tex_y = (v * 64.0) as u32;
       
       // Get real pixel color from wall.png texture
       let wall_color = texture_cache.get_wall_pixel_color(wall_tex_x, wall_tex_y);
       
       framebuffer.set_current_color(wall_color);
       framebuffer.set_pixel(i, y as u32);
     }
  }

         // Render sprites (crumbs) as simple yellow dots
   for (ci, cj) in crumbs {
     let cx = (*ci as f32) * block_size as f32 + (block_size as f32 * 0.5);
     let cy = (*cj as f32) * block_size as f32 + (block_size as f32 * 0.5);
     
     // Calculate sprite position relative to player
     let dx = cx - player.pos.x;
     let dy = cy - player.pos.y;
     
     // Calculate distance and angle to sprite
     let sprite_distance = (dx * dx + dy * dy).sqrt();
     let sprite_angle = dy.atan2(dx) - player.a;
     
     // Skip if sprite is behind player
     if sprite_angle.abs() > PI / 2.0 {
       continue;
     }
     
     // Calculate sprite screen position
     let sprite_screen_x = (sprite_angle / (player.fov / 2.0)) * (framebuffer.width as f32 / 2.0) + (framebuffer.width as f32 / 2.0);
     
     // Calculate sprite size on screen
     let sprite_size = (block_size as f32 / sprite_distance) * distance_to_projection_plane;
     let sprite_height = sprite_size.max(1.0) as usize;
     let sprite_width = sprite_size.max(1.0) as usize;
     
     // Skip rendering if sprite is too small or too far
     if sprite_width < 1 || sprite_height < 1 || sprite_distance > 1000.0 {
       continue;
     }
     
     // Draw sprite as simple yellow dot
     let sprite_top = (hh - sprite_height as f32 / 2.0) as usize;
     let sprite_bottom = (hh + sprite_height as f32 / 2.0) as usize;
     
     // Check if sprite is behind walls using depth testing
     let sprite_start_x = (sprite_screen_x as usize).saturating_sub(sprite_width / 2);
     let sprite_end_x = (sprite_screen_x as usize + sprite_width / 2).min(framebuffer.width as usize);
     
     for y in sprite_top..sprite_bottom {
       if y >= framebuffer.height as usize { continue; }
       
       for x in sprite_start_x..sprite_end_x {
         if x >= framebuffer.width as usize { continue; }
         
         // Depth test: only draw if sprite is closer than wall
         if x < wall_distances.len() && sprite_distance < wall_distances[x] {
           framebuffer.set_current_color(Color::YELLOW);
           framebuffer.set_pixel(x as u32, y as u32);
         }
       }
     }
   }
}

fn draw_minimap(window: &mut RaylibHandle, rl: &RaylibThread, maze: &Maze, block_size: usize, player: &Player) {
  let scale: i32 = 4;
  let padding: i32 = 10;
  let width = (maze[0].len() as i32) * scale;
  let height = (maze.len() as i32) * scale;
  let x0 = window.get_screen_width() - width - padding;
  let y0 = padding;
  let mut d = window.begin_drawing(rl);
  d.draw_rectangle_lines(x0-1, y0-1, width+2, height+2, Color::WHITE);
  for j in 0..maze.len() {
    for i in 0..maze[0].len() {
      let c = maze[j][i];
      if c != ' ' {
        d.draw_rectangle(x0 + (i as i32)*scale, y0 + (j as i32)*scale, scale, scale, Color::DARKGRAY);
      }
    }
  }
  let px = (player.pos.x as i32) / block_size as i32;
  let py = (player.pos.y as i32) / block_size as i32;
  d.draw_rectangle(x0 + px*scale, y0 + py*scale, scale, scale, Color::YELLOW);
}

fn main() {
  let window_width = 1300;
  let window_height = 900;
  let block_size = 100;

  let (mut window, raylib_thread) = raylib::init()
    .size(window_width, window_height)
    .title("Raycaster Example")
    .log_level(TraceLogLevel::LOG_WARNING)
    .build();

  let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
  framebuffer.set_background_color(Color::new(50, 50, 100, 255));

  let maze = load_maze("maze.txt");
  // Collectible crumbs from 'g' cells
  let mut crumbs: Vec<(usize, usize)> = Vec::new();
  for (j, row) in maze.iter().enumerate() {
    for (i, &c) in row.iter().enumerate() {
      if c == 'g' { crumbs.push((i, j)); }
    }
  }

     // Initialize texture manager
   let texture_cache = TextureManager::new(&mut window, &raylib_thread);
   
       // Initialize audio manager
    let mut audio_manager = AudioManager::new(&mut window, &raylib_thread);

  let mut player = Player {
    pos: Vector2::new(150.0, 150.0),
    a: PI / 3.0,
    fov: PI / 3.0,
  };

  // Game state
  #[derive(PartialEq, Eq, Clone, Copy)]
  enum GameState { Start, Playing, Success }
  let mut state = GameState::Start;

  

  window.set_target_fps(60);

  while !window.window_should_close() {
    // 1. clear framebuffer
    framebuffer.clear();

    // 2. move the player on user input (only in Playing)
    if state == GameState::Playing {
      process_events(&mut player, &window, &maze, block_size);
    }

    let mut mode = "3D";

    if window.is_key_down(KeyboardKey::KEY_M) {
      mode = if mode == "2D" { "3D" } else { "2D" };
    }

    // 3. draw stuff
    if state == GameState::Playing {
             if mode == "2D" {
         render_maze(&mut framebuffer, &maze, block_size, &player);
       } else {
         render_world(&mut framebuffer, &maze, block_size, &player, &texture_cache, &crumbs);
       }
    }

         // 4. swap buffers
     framebuffer.swap_buffers(&mut window, &raylib_thread);
     
     // Update background music
     if state == GameState::Playing {
       audio_manager.update_music(&mut window);
     }
    
    // HUD overlays and screens
    {
      let mut d = window.begin_drawing(&raylib_thread);
      match state {
        GameState::Start => {
          let sw = d.get_screen_width();
          let sh = d.get_screen_height();
          d.clear_background(Color::new(30, 30, 60, 255));
          d.draw_text("Presiona cualquier tecla para iniciar", sw/2 - 220, sh/2 - 10, 20, Color::WHITE);
          d.draw_text("Recoge las 3 migajas para ganar!", sw/2 - 200, sh/2 + 20, 18, Color::YELLOW);
        }
                 GameState::Playing => {
           d.draw_fps(10, 10);
           // Show crumb counter
           let remaining = crumbs.len();
           d.draw_text(&format!("Migajas restantes: {}/3", 3 - remaining), 10, 30, 20, Color::WHITE);
           
           // Show audio status
           let audio_status = if audio_manager.has_background_music() && audio_manager.has_pickup_sound() {
             "Audio: ON"
           } else if audio_manager.has_background_music() || audio_manager.has_pickup_sound() {
             "Audio: PARTIAL"
           } else {
             "Audio: OFF"
           };
           d.draw_text(audio_status, 10, 50, 16, Color::YELLOW);
         }
        GameState::Success => {
          let sw = d.get_screen_width();
          let sh = d.get_screen_height();
          d.clear_background(Color::new(20, 60, 20, 255));
          d.draw_text("¡Felicidades! Eres el mejor migajero", sw/2 - 260, sh/2 - 10, 20, Color::WHITE);
          d.draw_text("Presiona ESC para salir", sw/2 - 150, sh/2 + 20, 18, Color::YELLOW);
        }
      }
    }
    
         // Check for key press in Start state (outside of drawing context)
     if state == GameState::Start {
       if window.get_key_pressed().is_some() { 
         state = GameState::Playing;
         // Start background music when game begins
         audio_manager.play_background_music(&mut window);
       }
     }
    
    // Check for ESC in Success state
    if state == GameState::Success && window.is_key_down(KeyboardKey::KEY_ESCAPE) {
      break;
    }
    
    if state == GameState::Playing { draw_minimap(&mut window, &raylib_thread, &maze, block_size, &player); }

    // Collectibles check in Playing state
    if state == GameState::Playing {
      let mut collected = None;
      for (idx, (ci, cj)) in crumbs.iter().enumerate() {
        let cx = (*ci as f32) * block_size as f32 + (block_size as f32 * 0.5);
        let cy = (*cj as f32) * block_size as f32 + (block_size as f32 * 0.5);
        let dx = player.pos.x - cx;
        let dy = player.pos.y - cy;
        // Increased collection radius from 40 to 80 pixels for easier collection
        if (dx*dx + dy*dy).sqrt() < 80.0 {
          collected = Some(idx);
          break;
        }
      }
             if let Some(idx) = collected {
         crumbs.remove(idx);
         // Play pickup sound effect
         audio_manager.play_pickup_sound(&mut window);
         if crumbs.is_empty() {
           state = GameState::Success;
         }
       }
    }

    thread::sleep(Duration::from_millis(16));
  }
}



