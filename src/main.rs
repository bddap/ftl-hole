mod orbit;

use std::ops::Range;

use glam::{dvec2, DMat3, DVec2};
use macroquad::{
    color::colors,
    prelude::{
        clear_background, draw_circle, get_time, next_frame, screen_height, screen_width, BLACK,
        WHITE, YELLOW,
    },
};

const WORLD_RADIUS_METERS: f64 = 1024.0;
const BLACK_HOLE_MASS: f64 = 5.97219_e17;
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
        let csv = orbit::Csv::new(self.pos.extend(0.0), self.vel.extend(0.0));
        let koe = orbit::Koe::from_csv(csv, PULL);
        let koe = koe.tick(t - self.when, PULL);
        let csv = orbit::Csv::from_koe(koe, PULL);
        dvec2(csv.pos.x, csv.pos.y)
    }

    fn period(&self) -> f64 {
        let csv = orbit::Csv::new(self.pos.extend(0.0), self.vel.extend(0.0));
        let koe = orbit::Koe::from_csv(csv, PULL);
        koe.period(PULL)
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let warp_points = [
        (dvec2(1.0, -1.0), colors::BEIGE),
        (dvec2(1.0, 1.0), colors::DARKBROWN),
        (dvec2(-1.0, 1.0), colors::DARKBROWN),
        (dvec2(-1.0, -1.0), colors::DARKBROWN),
    ]
    .map(|(p, c)| (p * WORLD_RADIUS_METERS / 2.0, c));

    let pos = dvec2(1.0, 0.0) * WORLD_RADIUS_METERS / 3.0;
    let v_mag = (PULL / pos.length()).sqrt(); // velocity for a circular orbit
    let player = Player {
        sat: Satellite {
            pos,
            // bug?: any larger than 1.414213562 (sqrt(2)), and the orbit doesn't display
            // dvec2(0.0, v_mag * 1.4142135623)
            vel: dvec2(0.0, v_mag * 1.2),
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

        draw_circle(
            screen_width() / 2.0,
            screen_height() / 2.0,
            20.0,
            colors::VIOLET,
        );

        let player_pos_screen = world_to_screen * player.sat.pos_at(time).extend(1.0);
        draw_circle(
            player_pos_screen.x as f32,
            player_pos_screen.y as f32,
            15.0,
            YELLOW,
        );

        let points = 100;
        let period = player.sat.period();
        for i in 0..points {
            let t = (i as f64).remap(0.0..((points - 1) as f64), 0.0..period);
            let projected_pos_screen = world_to_screen * player.sat.pos_at(t).extend(1.0);
            draw_circle(
                projected_pos_screen.x as f32,
                projected_pos_screen.y as f32,
                2.0,
                WHITE,
            );
        }

        for (point, color) in warp_points {
            let pos_screen = world_to_screen * point.extend(1.0);
            draw_circle(pos_screen.x as f32, pos_screen.y as f32, 15.0, color);
        }

        next_frame().await
    }
}

trait Remap: Sized {
    fn remap(self, current: Range<Self>, target: Range<Self>) -> Self;
}

impl Remap for f64 {
    fn remap(self, current: Range<Self>, target: Range<Self>) -> Self {
        (self - current.start) / (current.end - current.start) * (target.end - target.start)
            + target.start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remap() {
        assert_eq!(0.0.remap(0.0..1.0, 0.0..1.0), 0.0);
        assert_eq!(1.0.remap(0.0..1.0, 0.0..1.0), 1.0);
        assert_eq!(0.5.remap(0.0..1.0, 0.0..2.0), 1.0);
        assert_eq!((-0.5).remap(0.0..-1.0, 0.0..2.0), 1.0);
    }
}
