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
const MAX_SPRITES_NUM: u32 = 16;

struct ArrowKey {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}
impl ArrowKey {
    fn vec(&self, max_speed: f32) -> (f32, f32) {
        let mut x = (self.right as i32 as f32 - self.left as i32 as f32) * max_speed;
        let mut y = (self.up as i32 as f32 - self.down as i32 as f32) * max_speed;
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

struct PlayerIndex(usize);

struct GlobalMouseClickPos(f32, f32);

struct MinionsSpawn(Vec<usize>);

fn prep_func(table: &mut ecs::Table) {
    table.add_state(ArrowKey::new_empty()).unwrap();
    let sprite_master = table.read_state::<SpriteMaster3000>().unwrap();

    let start_cam_offset = (0.0, -32.0);
    let (player_index, player_sprite) = sprite_master
        .add_sprite(
            "idle_right",
            (-16.0 + start_cam_offset.0, 16.0 + start_cam_offset.1),
            // (0.0, 0.0),
            0.5,
        )
        .unwrap();
    // player_sprite.flipped_x = 1;

    let size = 1024.0;
    let (_, bg_sprite) = sprite_master
        .add_sprite(
            "bg_grass",
            (-size / 2.0 + start_cam_offset.0, -16.0 + start_cam_offset.1),
            0.9,
        )
        .unwrap();
    bg_sprite.width = size;
    bg_sprite.height = size;

    table.add_state(PlayerIndex(player_index)).unwrap();
    table.add_state(GlobalMouseClickPos(0.0, 0.0)).unwrap();
    table.add_state(MinionsSpawn(vec![])).unwrap();
}

fn entry_point(table: &mut ecs::Table) {
    // important states
    let running_state = table.read_state::<RunningState>().unwrap();
    let uniform_data = table.read_state::<Uniform>().unwrap();
    let sprite_master = table.read_state::<SpriteMaster3000>().unwrap();
    let player_index = table.read_state::<PlayerIndex>().unwrap().0;
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

    // quitting
    if key_state.just_clicked(VirtualKeyCode::Q) {
        *running_state = RunningState::Closed;
    }

    // update global mouse position everytime a mouse click event is fired
    if mouse_state.middle_button_clicked()
        || mouse_state.left_button_clicked()
        || mouse_state.right_button_clicked()
    {
        global_click_pos.0 = relative_click_pos.0 - uniform_data.global_offset_x;
        global_click_pos.1 = relative_click_pos.1 - uniform_data.global_offset_y;
    }
    if mouse_state.middle_button_released()
        || mouse_state.left_button_released()
        || mouse_state.right_button_released()
    {
        global_click_pos.0 = 0.0;
        global_click_pos.1 = 0.0;
    }

    // drag cam with mouse
    if mouse_state.middle_button_pressed() {
        uniform_data.global_offset_x = relative_click_pos.0 - global_click_pos.0;
        uniform_data.global_offset_y = relative_click_pos.1 - global_click_pos.1;
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

    // show dtime
    if mouse_state.right_button_pressed() {
        println!("{:?}", uniform_data.delta_time);
    }

    // show data
    if key_state.just_clicked(VirtualKeyCode::Space) {
        // println!("{:?}", uniform_data);
        // println!("{:?}", (mouse_state.x(), mouse_state.y()));
        // println!("{:?}", table.read_column::<Sprite>().unwrap());
        println!("{:?}", sprite_master.occupied_indices);
        // println!("{:?}", (player_sprite.pos_x, player_sprite.pos_y));

        // player_sprite.flipped_y = if player_sprite.flipped_y == 0 { 1 } else { 0 };
        // player_sprite.paused = if player_sprite.paused == 0 { 1 } else { 0 };
        // player_sprite.reversed = if player_sprite.reversed == 0 { 1 } else { 0 };
        // player_sprite.duration *= 0.95;
        // println!("{:?}", player_sprite.duration);
        // player_sprite.looping = if player_sprite.looping == 0 { 1 } else { 0 };
    }

    // change state of animation
    // if key_state.just_clicked(VirtualKeyCode::X) {
    //     sprite_master
    //         .change_state(player_index, "jump", false)
    //         .unwrap();
    // }
    // if key_state.just_clicked(VirtualKeyCode::C) {
    //     sprite_master
    //         .change_state(player_index, "test", false)
    //         .unwrap();
    // }

    // spawn minions
    if mouse_state.left_button_clicked() & !mouse_state.middle_button_pressed() {
        let minions_spawned = table.read_state::<MinionsSpawn>().unwrap();
        let (index, new_sprite) = sprite_master.clone_add(player_index).unwrap();
        new_sprite.pos_x = global_click_pos.0 - 16.0;
        new_sprite.pos_y = global_click_pos.1 + 16.0 + 0.001 * (rand::random::<f32>() - 0.5);
        minions_spawned.0.push(index);
    }
    // clear minions
    if key_state.just_clicked(VirtualKeyCode::V) {
        let minions_spawned = table.read_state::<MinionsSpawn>().unwrap();
        for each in &minions_spawned.0 {
            sprite_master.remove_sprite(*each).unwrap();
        }
        println!("{:?}", minions_spawned.0);
        minions_spawned.0.clear();
    }

    // updating sprite position
    // todo add accelaration

    if key_state.is_pressed(VirtualKeyCode::A) {
        arrow_key_state.left = true;
    } else {
        arrow_key_state.left = false;
    }
    if key_state.is_pressed(VirtualKeyCode::D) {
        arrow_key_state.right = true;
    } else {
        arrow_key_state.right = false;
    }

    let vec = arrow_key_state.vec(75.0 * uniform_data.delta_time);
    if vec.0 != 0.0 {
        if vec.0 < 0.0 {
            player_sprite.flipped_x = 1;
        } else {
            player_sprite.flipped_x = 0;
        }
        sprite_master
            .change_state(player_index, "run_right", false)
            .unwrap();
    } else {
        sprite_master
            .change_state(player_index, "idle_right", false)
            .unwrap();
    }
    player_sprite.pos_x += vec.0;
    player_sprite.pos_y += vec.1;
}

fn post_func(table: &mut ecs::Table) {
    println!("wrapping up uwu")
}

fn main() {
    renderer::run(
        HEIGHT_RESOLUTION_HALF,
        MAX_SPRITES_NUM,
        entry_point,
        prep_func,
        post_func,
    );
}
