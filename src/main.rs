mod orbit;

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
    // when were we at this pos and vel
    when: f64,
}

impl Satellite {
    fn pos_at(&self, t: f64) -> DVec2 {
        let csv = orbit::Csv::new(
            self.pos.extend(0.0),
            self.vel.extend(0.0),
            orbit::CB::new(PULL).into(),
        );
        let koe = orbit::Koe::from_csv(csv);
        let koe = koe.tick(t - self.when);
        let csv = orbit::Csv::from_koe(koe);
        dvec2(csv.r.x, csv.r.y)
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
    let player = Player {
        sat: Satellite {
            pos,
            vel: dvec2(0.0, v_mag),
            when: 0.0,
        },
    };

    loop {
        let time = get_time();

        let screen_min_dim = screen_width().min(screen_height()) as f64;
        let world_to_screen = DMat3::from_translation(dvec2(
            screen_width() as f64 / 2.0,
            screen_height() as f64 / 2.0,
        )) * DMat3::from_scale(dvec2(
            screen_min_dim / 2.0 / WORLD_RADIUS_METERS,
            screen_min_dim / 2.0 / WORLD_RADIUS_METERS,
        ));

        clear_background(BLACK);

        let player_pos_screen = world_to_screen * player.sat.pos_at(time).extend(1.0);
        draw_circle(
            player_pos_screen.x as f32,
            player_pos_screen.y as f32,
            15.0,
            YELLOW,
        );

        for i in 0..100 {
            let projected_pos_screen =
                world_to_screen * player.sat.pos_at(time + i as f64).extend(1.0);
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
