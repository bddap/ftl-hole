use std::ops::Range;

use glam::{dvec2, DMat3, DVec2, Vec3Swizzles};
use itertools::Itertools;
use macroquad::{
    color::colors::{self, BEIGE, DARKBLUE, DARKBROWN, MAROON},
    prelude::{
        clear_background, draw_circle, draw_line, get_time, is_mouse_button_pressed,
        mouse_position, next_frame, screen_height, screen_width, vec2, Color, MouseButton, YELLOW,
    },
    rand::gen_range,
    shapes::draw_rectangle,
};

const WORLD_RADIUS_METERS: f64 = 1024.0;
const BLACK_HOLE_MASS: f64 = 5.97219_e17;
const GRAVITATIONAL_CONSTANT: f64 = 6.67_e-11;
const PULL: f64 = BLACK_HOLE_MASS * GRAVITATIONAL_CONSTANT;

struct Player {
    sat: Sat,
}

#[derive(Clone, Copy, Debug)]
struct Sat {
    pos: DVec2,
    vel: DVec2,
    // when were we at this pos and vel
    when: f64,
}

impl Sat {
    fn tick_to(&mut self, when: f64) {
        let dt = 0.001;
        while self.when < when {
            let acc = self.acceleration();
            self.vel += acc * dt;
            self.pos += self.vel * dt;
            self.when += dt;
        }
    }

    fn acceleration(&self) -> DVec2 {
        let r = self.pos.length();
        let r3 = r * r * r;
        -self.pos * PULL / r3
    }
}

#[derive(Clone)]
struct WarpPoint {
    pos: DVec2,
    color: Color,
    win_destination: DVec2,
}

#[macroquad::main("ftl-hole")]
async fn main() {
    // create 4 warp points with random positions and destinations
    let mut warp_points = [DARKBROWN, MAROON, DARKBLUE, BEIGE].map(|color| WarpPoint {
        color,
        pos: dvec2(gen_range(-1.0, 1.0), gen_range(-1.0, 1.0)).normalize() * WORLD_RADIUS_METERS,
        win_destination: dvec2(gen_range(-1.0, 1.0), gen_range(-1.0, 1.0)).normalize()
            * WORLD_RADIUS_METERS,
    });

    let initial_radius: f32 = 1.0 / 6.0;
    let pos = dvec2(initial_radius as f64, 0.0) * WORLD_RADIUS_METERS;
    let v_scale = 0.8;
    let v_mag = (PULL / pos.length()).sqrt();
    let mut player = Player {
        sat: Sat {
            pos,
            vel: dvec2(0.0, v_scale * v_mag),
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
        let screen_to_world = world_to_screen.inverse();

        if is_mouse_button_pressed(MouseButton::Left) {
            // find nearest warp point to the cursor
            let mouse_pos = dvec2(mouse_position().0.into(), mouse_position().1.into());
            let mouse_pos = (screen_to_world * mouse_pos.extend(1.0)).xy();

            let warp_pos = warp_points
                .iter_mut()
                .min_by_key(|p| (p.pos - mouse_pos).length_squared() as i64)
                .unwrap();

            dbg!(player.sat);
            std::mem::swap(&mut player.sat.pos, &mut warp_pos.pos);
            dbg!(player.sat);
        }

        player.sat.tick_to(time);

        clear_background(colors::BLACK);

        draw_circle(
            screen_width() / 2.0,
            screen_height() / 2.0,
            20.0,
            colors::VIOLET,
        );

        for wp in &warp_points {
            let pos_screen = world_to_screen * wp.pos.extend(1.0);
            draw_circle(pos_screen.x as f32, pos_screen.y as f32, 15.0, wp.color);

            let dest_screen = world_to_screen * wp.win_destination.extend(1.0);
            draw_rectangle(
                dest_screen.x as f32 - 10.0,
                dest_screen.y as f32 - 10.0,
                20.0,
                20.0,
                wp.color,
            );

            // draw a thin line from the warp point to its destination
            draw_line(
                pos_screen.x as f32,
                pos_screen.y as f32,
                dest_screen.x as f32,
                dest_screen.y as f32,
                1.0,
                wp.color,
            );
        }

        let points = 32;
        let dot_dur = 1.0;
        let mut p = player.sat;
        p.vel = -p.vel;
        let point_poses = (0..points).map(|i| {
            let t = (i as f64).remap(0.0..((points - 1) as f64), 0.0..dot_dur);
            p.tick_to(time + t);
            let projected_pos_screen = world_to_screen * p.pos.extend(1.0);
            vec2(projected_pos_screen.x as f32, projected_pos_screen.y as f32)
        });
        for (a, b) in point_poses.tuple_windows() {
            draw_line(a.x, a.y, b.x, b.y, 2.0, YELLOW);
        }

        let player_pos_screen = world_to_screen * player.sat.pos.extend(1.0);
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
