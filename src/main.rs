#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    unused_assignments,
    unreachable_code
)]
use renderer::*;
use winit::event::VirtualKeyCode;

const HEIGHT_RESOLUTION_HALF: f32 = 64.0;
const MAX_RESOLUTION_HALF: f32 = 256.0;

struct ArrowKey {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}
impl ArrowKey {
    fn vec(&self, speed: f32) -> (f32, f32) {
        let mut x = (self.right as i32 as f32 - self.left as i32 as f32) * speed;
        let mut y = (self.up as i32 as f32 - self.down as i32 as f32) * speed;
        if x != 0f32 && y != 0f32 {
            x /= 2f32.sqrt();
            y /= 2f32.sqrt();
        }
        (x, y)
    }
    fn new_empty() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

struct PlayerHandle(usize);

struct AdditionalTimeInfo {
    last_utime: f32,
    delta_time: f32,
}

struct DragStorePos(f32, f32);

fn prep_func(table: &mut ecs::Table) {
    table.add_state(ArrowKey::new_empty()).unwrap();
    let player_index = table.insert_new(Sprite {
        pos_x: -16.0,
        pos_y: 16.0,
        width: 32.0,
        height: 32.0,

        tex_x: 0.0,
        tex_y: 0.0,
        tex_width: 32.0,
        tex_height: 32.0,

        depth: 0.5,
        origin: 24.0,

        duration: 1.0,
        frames: 2,
    });

    let size = 1024.0;
    table.insert_new(Sprite {
        pos_x: -size / 2.0,
        pos_y: size / 2.0,
        width: size,
        height: size,

        tex_x: 0.0,
        tex_y: 64.0,
        tex_width: 32.0,
        tex_height: 32.0,

        depth: 0.9,
        origin: 24.0,

        duration: 1.0,
        frames: 1,
    });
    table.add_state(PlayerHandle(player_index)).unwrap();
    table
        .add_state(AdditionalTimeInfo {
            delta_time: 0.0,
            last_utime: 0.0,
        })
        .unwrap();
    table.add_state(DragStorePos(0.0, 0.0)).unwrap();
}

fn entry_point(table: &mut ecs::Table) {
    // important states
    let running_state = table.read_state::<RunningState>().unwrap();
    let uniform_data = table.read_state::<Uniform>().unwrap();
    let time_info = table.read_state::<AdditionalTimeInfo>().unwrap();
    time_info.delta_time = uniform_data.utime - time_info.last_utime;
    time_info.last_utime = uniform_data.utime;
    let player_index = table.read_state::<PlayerHandle>().unwrap().0;
    let player_sprite = table.read_single::<Sprite>(player_index).unwrap();
    let arrow_key_state = table.read_state::<ArrowKey>().unwrap();
    let drag_store_pos = table.read_state::<DragStorePos>().unwrap();
    let mod_state = table.read_state::<winit::event::ModifiersState>().unwrap();
    let mouse_state = table.read_state::<MouseState>().unwrap();
    let key_state = table.read_state::<KeyState>().unwrap();

    let ratio = uniform_data.window_height / uniform_data.window_width;
    let centralized_mouse_pos_in_pixel_in_screen = (
        (mouse_state.x() / (uniform_data.window_width * 0.5) - 1.0)
            * uniform_data.height_resolution
            / ratio,
        (1.0 - mouse_state.y() / (uniform_data.window_height * 0.5))
            * uniform_data.height_resolution,
    );

    // quitting
    if key_state.just_clicked(VirtualKeyCode::Q) {
        *running_state = RunningState::Closed;
    }
    // show dtime
    if mouse_state.middle_button_pressed() {
        println!("{:?}", time_info.delta_time);
    }

    // show uniform data
    if key_state.just_clicked(VirtualKeyCode::Space) {
        // println!("{:?}", uniform_data);
        println!("{:?}", (mouse_state.x(), mouse_state.y()));
    }

    // spawn minions
    if key_state.just_clicked(VirtualKeyCode::C) {
        let mut clone = player_sprite.clone();
        clone.pos_y += 0.001 * (rand::random::<f32>() - 0.5);
        table.insert_new(clone);
    }

    // check sprite num
    if key_state.just_clicked(VirtualKeyCode::Home) {
        println!("{:?}", table.read_column::<Sprite>().unwrap().len());
    }

    // move cam with mouse
    if mouse_state.right_button_clicked() {
        drag_store_pos.0 =
            centralized_mouse_pos_in_pixel_in_screen.0 - uniform_data.global_offset_x;
        drag_store_pos.1 =
            centralized_mouse_pos_in_pixel_in_screen.1 - uniform_data.global_offset_y;
    }
    if mouse_state.right_button_released() {
        drag_store_pos.0 = 0.0;
        drag_store_pos.1 = 0.0;
    }
    if mouse_state.right_button_pressed() {
        uniform_data.global_offset_x =
            centralized_mouse_pos_in_pixel_in_screen.0 - drag_store_pos.0;
        uniform_data.global_offset_y =
            centralized_mouse_pos_in_pixel_in_screen.1 - drag_store_pos.1;
        // println!("{:?}", centralized_mouse_pos_in_pixel);
    }

    // get character position
    if key_state.just_clicked(VirtualKeyCode::End) {
        println!("{:?}", (player_sprite.pos_x, player_sprite.pos_y));
    }

    // zooming in and out
    if !mouse_state.right_button_pressed() {
        if mouse_state.wheel_delta() < 0.0 {
            uniform_data.height_resolution =
                (uniform_data.height_resolution * 1.10).min(MAX_RESOLUTION_HALF);
        } else if mouse_state.wheel_delta() > 0.0 {
            uniform_data.height_resolution =
                (uniform_data.height_resolution * 0.90).max(HEIGHT_RESOLUTION_HALF);
        } else {
        }
    }

    // updating sprite position
    if key_state.is_pressed(VirtualKeyCode::Up) {
        arrow_key_state.up = true;
    } else {
        arrow_key_state.up = false;
    }
    if key_state.is_pressed(VirtualKeyCode::Down) {
        arrow_key_state.down = true;
    } else {
        arrow_key_state.down = false;
    }
    if key_state.is_pressed(VirtualKeyCode::Left) {
        arrow_key_state.left = true;
    } else {
        arrow_key_state.left = false;
    }
    if key_state.is_pressed(VirtualKeyCode::Right) {
        arrow_key_state.right = true;
    } else {
        arrow_key_state.right = false;
    }
    let vec = arrow_key_state.vec(75.0 * time_info.delta_time);
    player_sprite.pos_x += vec.0;
    player_sprite.pos_y += vec.1;
}

fn main() {
    renderer::run(HEIGHT_RESOLUTION_HALF, 1024, entry_point, prep_func);
}
