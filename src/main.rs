mod orbit;

use std::ops::Range;

use glam::{dvec2, DMat3, DVec2};
use macroquad::{
    color::colors,
    prelude::{
        clear_background, draw_circle, get_time, next_frame, screen_height, screen_width, vec2,
        Color, WHITE, YELLOW,
    },
    ui::root_ui,
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

    let vs_max: f32 = 2.0f32.sqrt() - 0.000001;
    let vs_min: f32 = 0.5;
    let mut initial_radius: f32 = 1.0 / 6.0;

    loop {
        let time = get_time();

        let screen_min_dim = screen_width().min(screen_height()) as f64;

        root_ui().window(
            12313,
            vec2(0.0, 0.0),
            vec2(screen_min_dim as f32 / 4.0, screen_min_dim as f32 / 15.0),
            |ui| {
                ui.drag(232342, "initial_r", Some((0.0, 1.0)), &mut initial_radius);
            },
        );

        let v_scale = (time / 32.0)
            .sin()
            .remap(-1.0..1.0, vs_min as f64..vs_max as f64) as f32;

        let pos = dvec2(initial_radius as f64, 0.0) * WORLD_RADIUS_METERS;
        let v_mag = (PULL / pos.length()).sqrt(); // velocity for a circular orbit
        let player = Player {
            sat: Satellite {
                pos,
                // bug?: any larger than 1.414213562 (sqrt(2)), and the orbit doesn't display
                // dvec2(0.0, v_mag * 1.4142135623)
                vel: dvec2(0.0, v_scale as f64 * v_mag),
                when: 0.0,
            },
        };

        let world_to_screen = DMat3::from_translation(dvec2(
            screen_width() as f64 / 2.0,
            screen_height() as f64 / 2.0,
        )) * DMat3::from_scale(dvec2(
            screen_min_dim / 2.0 / WORLD_RADIUS_METERS,
            screen_min_dim / 2.0 / WORLD_RADIUS_METERS,
        ));

        clear_background(colors::BLACK);

        draw_circle(
            screen_width() / 2.0,
            screen_height() / 2.0,
            20.0,
            colors::VIOLET,
        );

        for (point, color) in warp_points {
            let pos_screen = world_to_screen * point.extend(1.0);
            draw_circle(pos_screen.x as f32, pos_screen.y as f32, 15.0, color);
        }

        let points = 1000;
        let period = player.sat.period();
        let dot_dur = period; //.min(60.0);
        for i in 0..points {
            let t = (i as f64).remap(0.0..((points - 1) as f64), 0.0..dot_dur);
            let projected_pos_screen = world_to_screen * player.sat.pos_at(t).extend(1.0);
            draw_circle(
                projected_pos_screen.x as f32,
                projected_pos_screen.y as f32,
                2.0,
                WHITE,
            );
        }

        let player_pos_screen = world_to_screen * player.sat.pos_at(time).extend(1.0);
        draw_circle(
            player_pos_screen.x as f32,
            player_pos_screen.y as f32,
            15.0,
            YELLOW,
        );

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
