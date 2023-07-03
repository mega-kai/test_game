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

struct GlobalMouseClickPos(f32, f32);

struct MinionsSpawn(u32);

fn prep_func(table: &mut ecs::Table) {
    table.add_state(ArrowKey::new_empty()).unwrap();
    let sprite_master = table.read_state::<SpriteMaster3000>().unwrap();

    let start_cam_offset = (0.0, -32.0);
    let (player_index, player_sprite) = sprite_master.add_sprite(table, "run_right").unwrap();
    player_sprite.pos_x = -16.0 + start_cam_offset.0;
    player_sprite.pos_y = 16.0 + start_cam_offset.1;
    player_sprite.depth = 0.5;
    player_sprite.duration = 1.0 / 24.0;

    // println!("{:?}", sprite_master.available_buffer_indices);
    // println!("{:?}", player_sprite);
    let size = 1024.0;
    let (_, bg_sprite) = sprite_master.add_sprite(table, "bg_grass").unwrap();
    bg_sprite.pos_x = -size / 2.0 + start_cam_offset.0;
    bg_sprite.pos_y = -16.0 + start_cam_offset.1;
    bg_sprite.depth = 0.9;
    bg_sprite.duration = 0.0;
    // for repeating patterns
    bg_sprite.width = size;
    bg_sprite.height = size / 2.0;

    table.add_state(PlayerHandle(player_index)).unwrap();
    table.add_state(GlobalMouseClickPos(0.0, 0.0)).unwrap();
    table.add_state(MinionsSpawn(0)).unwrap();

    // renderer::make_tex_map();
}

fn entry_point(table: &mut ecs::Table) {
    // important states
    let running_state = table.read_state::<RunningState>().unwrap();
    let uniform_data = table.read_state::<Uniform>().unwrap();
    let player_index = table.read_state::<PlayerHandle>().unwrap().0;
    let player_sprite = table.read_single::<Sprite>(player_index).unwrap();
    let arrow_key_state = table.read_state::<ArrowKey>().unwrap();
    let global_click_pos = table.read_state::<GlobalMouseClickPos>().unwrap();
    let mod_state = table.read_state::<winit::event::ModifiersState>().unwrap();
    let mouse_state = table.read_state::<MouseState>().unwrap();
    let key_state = table.read_state::<KeyState>().unwrap();

    let ratio = uniform_data.window_height / uniform_data.window_width;
    let relative_click_pos = (
        (mouse_state.x() / (uniform_data.window_width * 0.5) - 1.0)
            * uniform_data.height_resolution
            / ratio,
        (1.0 - mouse_state.y() / (uniform_data.window_height * 0.5))
            * uniform_data.height_resolution,
    );

    let sprite_master = table.read_state::<SpriteMaster3000>().unwrap();

    // quitting
    if key_state.just_clicked(VirtualKeyCode::Q) {
        *running_state = RunningState::Closed;
    }
    // show dtime
    if mouse_state.right_button_pressed() {
        println!("{:?}", uniform_data.delta_time);
    }

    // show uniform data
    if key_state.just_clicked(VirtualKeyCode::Space) {
        // println!("{:?}", uniform_data);
        // println!("{:?}", (mouse_state.x(), mouse_state.y()));
        // println!("{:?}", table.read_column::<Sprite>().unwrap());
    }

    // spawn minions
    if key_state.just_clicked(VirtualKeyCode::C) {
        let minions_spawned = table.read_state::<MinionsSpawn>().unwrap();
        minions_spawned.0 += 1;
        // let mut clone = player_sprite.clone();
        // clone.pos_y += 0.001 * rand::random::<f32>();
        // clone.anim_buffer_index += minions_spawned.0;
        // table.insert_new(clone);
    }

    // check sprite num
    if key_state.just_clicked(VirtualKeyCode::Home) {
        println!("{:?}", table.read_column::<Sprite>().unwrap().len());
    }

    // check sprite master
    if key_state.just_clicked(VirtualKeyCode::Delete) {
        sprite_master.print();
    }

    // drag cam with mouse
    if mouse_state.middle_button_clicked() {
        global_click_pos.0 = relative_click_pos.0 - uniform_data.global_offset_x;
        global_click_pos.1 = relative_click_pos.1 - uniform_data.global_offset_y;
    }
    if mouse_state.middle_button_released() {
        global_click_pos.0 = 0.0;
        global_click_pos.1 = 0.0;
    }
    if mouse_state.middle_button_pressed() {
        uniform_data.global_offset_x = relative_click_pos.0 - global_click_pos.0;
        uniform_data.global_offset_y = relative_click_pos.1 - global_click_pos.1;
        // println!("{:?}", centralized_mouse_pos_in_pixel);
    }

    // get character position
    if key_state.just_clicked(VirtualKeyCode::End) {
        println!("{:?}", (player_sprite.pos_x, player_sprite.pos_y));
    }

    // zooming in and out
    if !mouse_state.middle_button_pressed() {
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
    if key_state.is_pressed(VirtualKeyCode::Left) {
        arrow_key_state.left = true;
        // player_sprite.tex_x = 0.0;
        // player_sprite.tex_y = 32.0;
    } else {
        arrow_key_state.left = false;
    }
    if key_state.is_pressed(VirtualKeyCode::Right) {
        arrow_key_state.right = true;
        // player_sprite.tex_x = 64.0;
        // player_sprite.tex_y = 32.0;
    } else {
        arrow_key_state.right = false;
    }

    // if key_state.is_pressed(VirtualKeyCode::Up) {
    //     arrow_key_state.up = true;
    //     player_sprite.tex_x = 64.0;
    //     player_sprite.tex_y = 0.0;
    // } else {
    //     arrow_key_state.up = false;
    // }
    // if key_state.is_pressed(VirtualKeyCode::Down) {
    //     arrow_key_state.down = true;
    //     player_sprite.tex_x = 0.0;
    //     player_sprite.tex_y = 0.0;
    // } else {
    //     arrow_key_state.down = false;
    // }
    let vec = arrow_key_state.vec(75.0 * uniform_data.delta_time);
    player_sprite.pos_x += vec.0;
    player_sprite.pos_y += vec.1;
}

fn post_func(table: &mut ecs::Table) {
    println!("wrapping up uwu")
}

fn main() {
    renderer::run(
        HEIGHT_RESOLUTION_HALF,
        1024,
        entry_point,
        prep_func,
        post_func,
    );
}
