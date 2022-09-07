use std::ops::{Div, Neg};

use macroquad::prelude::*;

const TICKS_PER_SECOND: usize = 1000;
const TICK_DURATION: f64 = 1.0 / TICKS_PER_SECOND as f64;
const WORLD_RADIUS_METERS: f64 = 1024.0;
const BLACK_HOLE_MASS: f64 = 5.97219_e14;
const GRAVITATIONAL_CONSTANT: f64 = 6.67_e-11;
const PULL: f64 = (BLACK_HOLE_MASS * GRAVITATIONAL_CONSTANT) as f64;

struct Player {
    sat: Satellite,
}

#[derive(Clone, Copy)]
struct Satellite {
    pos: DVec2,
    vel: DVec2,
}

impl Satellite {
    fn tick(&mut self) {
        let force_dir = self.pos.normalize().neg();
        let rsquared = self.pos.dot(self.pos);
        self.vel += force_dir * PULL * TICK_DURATION / rsquared;
        self.pos += self.vel * TICK_DURATION;
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let warp_points = [
        dvec2(1.0, -1.0),
        dvec2(1.0, 1.0),
        dvec2(-1.0, 1.0),
        dvec2(-1.0, -1.0),
    ]
    .map(|p| p * WORLD_RADIUS_METERS / 2.0);

    let pos = dvec2(1.0, 0.0) * WORLD_RADIUS_METERS / 1.3;
    let v_mag = (PULL / pos.length()).sqrt(); // velocity for a circular orbit
    let mut player = Player {
        sat: Satellite {
            pos,
            vel: dvec2(0.0, v_mag),
        },
    };

    let mut sim_time: f64 = 0.0;

    loop {
        let time = get_time();
        while sim_time <= time {
            sim_time += TICK_DURATION;
            player.sat.tick();
        }

        let screen_min_dim = screen_width().min(screen_height()) as f64;
        let world_to_screen = DMat3::from_translation(dvec2(
            screen_width() as f64 / 2.0,
            screen_height() as f64 / 2.0,
        )) * DMat3::from_scale(dvec2(
            screen_min_dim / 2.0 / WORLD_RADIUS_METERS,
            screen_min_dim / 2.0 / WORLD_RADIUS_METERS,
        ));

        clear_background(BLACK);

        let player_pos_screen = world_to_screen * player.sat.pos.extend(1.0);
        draw_circle(
            player_pos_screen.x as f32,
            player_pos_screen.y as f32,
            15.0,
            YELLOW,
        );

        // TODO: orbital_period is only correct for circular orbits
        let orbital_period = pos.length().powf(1.5) * core::f64::consts::PI;
        let dots = 128;
        let orbital_period_ticks = orbital_period as usize * TICKS_PER_SECOND / 100;
        let ticks_per_dot = orbital_period_ticks / dots;
        let mut projected = player.sat;
        for _ in 0..dots {
            for _ in 0..ticks_per_dot {
                projected.tick();
            }
            let projected_pos_screen = world_to_screen * projected.pos.extend(1.0);
            draw_circle(
                projected_pos_screen.x as f32,
                projected_pos_screen.y as f32,
                2.0,
                WHITE,
            );
        }

        for point in warp_points {
            let pos_screen = world_to_screen * point.extend(1.0);
            draw_circle(pos_screen.x as f32, pos_screen.y as f32, 15.0, DARKBROWN);
        }

        next_frame().await
    }
}
