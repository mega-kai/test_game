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
const MAX_SPRITES_NUM: u32 = 512;

struct TwoWayArrowKey {
    left: bool,
    right: bool,

    max_speed: f32,
    min_speed: f32,
    acc: f32,
    deacc: f32,
    current_speed: f32,

    vertical_speed: f32,
    gravity_constant: f32,

    in_air: bool,
    double_jumped: bool,
}
impl TwoWayArrowKey {
    fn new(max_speed: f32, min_speed: f32, acc: f32, deacc: f32, gravity_constant: f32) -> Self {
        Self {
            left: false,
            right: false,

            max_speed,
            min_speed,
            acc,
            deacc,

            current_speed: 0.0,

            gravity_constant,
            vertical_speed: 0.0,

            in_air: false,
            double_jumped: false,
        }
    }

    fn update_speed(&mut self, delta_time: f32) {
        if !self.in_air {
            if self.left ^ self.right {
                if self.left {
                    self.current_speed -= self.acc * delta_time;
                    self.current_speed = self.current_speed.min(-self.min_speed);
                } else {
                    self.current_speed += self.acc * delta_time;
                    self.current_speed = self.current_speed.max(self.min_speed);
                }
            } else {
                if self.current_speed > 0.0 {
                    self.current_speed -= self.deacc * delta_time;
                } else if self.current_speed < 0.0 {
                    self.current_speed += self.deacc * delta_time;
                }
                if self.current_speed.abs() < 0.1 {
                    self.current_speed = 0.0;
                }
            }
        } else {
            if self.left ^ self.right {
                if self.left {
                    self.current_speed -= self.acc * delta_time;
                } else {
                    self.current_speed += self.acc * delta_time;
                }
            }
        }

        self.vertical_speed -= self.gravity_constant * delta_time;
        self.current_speed = self.current_speed.clamp(-self.max_speed, self.max_speed);

        // println!("{:?}", self.current_speed);
    }
}

struct PlayerIndex(usize);
struct FloorCollisionIndex(usize);

struct GlobalMouseClickPos(f32, f32);

struct MinionsSpawn(Vec<usize>);

fn prep_func(table: &mut ecs::Table) {
    let sprite_master = table.read_state::<SpriteMaster3000>().unwrap();
    let collision_manager = table.read_state::<CollisionManager>().unwrap();

    let start_cam_offset = (0.0, -32.0);
    let (player_index, player_sprite) = sprite_master
        .add_sprite(
            "idle_right",
            (-16.0 + start_cam_offset.0, 16.0 + start_cam_offset.1),
            // (0.0, 0.0),
            0.5,
        )
        .unwrap();
    // player_sprite.duration = 1.0;
    player_sprite.looping = 1;

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

    // add collision shapes
    collision_manager
        .insert_collision_rect(
            player_index,
            (-16.0 + start_cam_offset.0, 16.0 + start_cam_offset.1),
            (32.0, 32.0),
        )
        .unwrap();
    // test collision shape
    let floor = collision_manager.add_collision_rect(
        (-size / 2.0 + start_cam_offset.0, -16.0 + start_cam_offset.1),
        (size, size),
    );

    table
        .add_state(TwoWayArrowKey::new(3.0, 2.0, 5.0, 10.0, 9.8))
        .unwrap();
    table.add_state(PlayerIndex(player_index)).unwrap();
    table.add_state(GlobalMouseClickPos(0.0, 0.0)).unwrap();
    table.add_state(MinionsSpawn(vec![])).unwrap();
    table.add_state(FloorCollisionIndex(floor)).unwrap();
}

fn entry_point(table: &mut ecs::Table) {
    // important states
    let running_state = table.read_state::<RunningState>().unwrap();
    let uniform_data = table.read_state::<Uniform>().unwrap();
    let sprite_master = table.read_state::<SpriteMaster3000>().unwrap();
    let player_index = table.read_state::<PlayerIndex>().unwrap().0;
    let player_sprite = table.read_single::<Sprite>(player_index).unwrap();
    let collision_manager = table.read_state::<CollisionManager>().unwrap();
    let player_collision_rect = table.read_single::<CollisionRect>(player_index).unwrap();

    let global_click_pos = table.read_state::<GlobalMouseClickPos>().unwrap();
    let mod_state = table.read_state::<winit::event::ModifiersState>().unwrap();
    let mouse_state = table.read_state::<MouseState>().unwrap();
    let key_state = table.read_state::<KeyState>().unwrap();

    let speed_resolver = table.read_state::<TwoWayArrowKey>().unwrap();

    let ratio = uniform_data.window_height / uniform_data.window_width;
    let relative_click_pos = (
        (mouse_state.x() / (uniform_data.window_width * 0.5) - 1.0)
            * uniform_data.height_resolution
            / ratio,
        (1.0 - mouse_state.y() / (uniform_data.window_height * 0.5))
            * uniform_data.height_resolution,
    );

    let floor_index = table.read_state::<FloorCollisionIndex>().unwrap().0;
    let floor_rect = table.read_single::<CollisionRect>(floor_index).unwrap();

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
        // println!("{:?}", minions_spawned.0);
        minions_spawned.0.clear();
    }

    if key_state.is_pressed(VirtualKeyCode::Home) {
        uniform_data.global_offset_x += 2.0 * (rand::random::<f32>() - 0.5);
        uniform_data.global_offset_y += 2.0 * (rand::random::<f32>() - 0.5);
    }

    // give speed input
    if key_state.just_clicked(VirtualKeyCode::Space) {
        if !speed_resolver.in_air {
            speed_resolver.vertical_speed += 5.0;
        } else if !speed_resolver.double_jumped {
            speed_resolver.double_jumped = true;
            speed_resolver.vertical_speed += 3.0;
        }
    }
    // println!("{:?}", speed_resolver.vertical_speed);
    if key_state.is_pressed(VirtualKeyCode::A) {
        speed_resolver.left = true;
    } else {
        speed_resolver.left = false;
    }
    if key_state.is_pressed(VirtualKeyCode::D) {
        speed_resolver.right = true;
    } else {
        speed_resolver.right = false;
    }

    speed_resolver.update_speed(uniform_data.delta_time);

    player_sprite.pos_x += speed_resolver.current_speed;
    player_sprite.pos_y += speed_resolver.vertical_speed;
    player_collision_rect.sync_all(player_sprite);

    if player_collision_rect.pos_y > -16.0 {
        speed_resolver.in_air = true;
    }

    // falling on the ground
    if collision_manager.check_if_colliding(player_index, floor_index)
        && player_collision_rect.pos_y < -16.0
    {
        player_sprite.pos_y = -16.0;
        speed_resolver.vertical_speed = 0.0;
        speed_resolver.in_air = false;
        speed_resolver.double_jumped = false;
    }
    println!("{:?}", speed_resolver.double_jumped);

    // update animation according the speed data
    if speed_resolver.in_air {
        if speed_resolver.vertical_speed >= 0.0 {
            if speed_resolver.vertical_speed <= 0.5 {
                sprite_master
                    .change_state(player_index, "jump_mid_air")
                    .unwrap();
            } else {
                sprite_master
                    .change_state(player_index, "jump_start")
                    .unwrap();
            }
        } else {
            sprite_master
                .change_state(player_index, "jump_fall")
                .unwrap();
        }
    } else {
        if speed_resolver.current_speed != 0.0 {
            sprite_master
                .change_state(player_index, "run_right")
                .unwrap();
        } else {
            sprite_master
                .change_state(player_index, "idle_right")
                .unwrap();
        }
    }
    if speed_resolver.current_speed < 0.0 {
        player_sprite.flipped_x = 1;
    } else if speed_resolver.current_speed > 0.0 {
        player_sprite.flipped_x = 0;
    }
    player_collision_rect.sync_all(player_sprite);
}

fn post_func(table: &mut ecs::Table) {
    // println!("wrapping up uwu")
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
