use macroquad::prelude::*;

const TICKS_PER_SECOND: usize = 1000;

struct Player {
    pos: Vec2,
    vel: Vec2,
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut player = Player {
        pos: vec2(0.0, 0.0),
        vel: vec2(0.0, 0.0),
    };

    loop {
        let aspect = screen_width() / screen_height();

        let world_to_screen =
            Mat3::from_translation(vec2(screen_width() / 2.0, screen_height() / 2.0))
                * Mat3::from_scale(vec2(1.0 / screen_width(), 1.0 / screen_width()));

        clear_background(BLACK);

        let player_pos_screen = world_to_screen * player.pos.extend(1.0);
        draw_circle(player_pos_screen.x, player_pos_screen.y, 15.0, YELLOW);

        next_frame().await
    }
}
