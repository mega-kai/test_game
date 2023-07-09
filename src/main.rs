#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    unused_assignments,
    unreachable_code
)]
#![windows_subsystem = "windows"]

use renderer::{ecs::Access, *};

const HEIGHT_RESOLUTION_HALF: f32 = 64.0;
const MAX_RESOLUTION_HALF: f32 = 256.0;
const MAX_SPRITES_NUM: u32 = 2048 * 2;

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

struct PlayerAccess((ecs::Access<Sprite>, ecs::Access<CollisionRect>));
struct FloorCollisionAccess(Access<CollisionRect>);

struct GlobalMouseClickPos(f32, f32);

struct MinionsSpawn(Vec<Access<Sprite>>);

fn prep_func(table: &mut ecs::Table) {
    let mut sprite_master = table.read_state::<SpriteMaster3000>().unwrap();
    let mut collision_manager = table.read_state::<CollisionManager>().unwrap();

    let start_cam_offset = (0.0, -32.0);
    let mut player = sprite_master
        .add_sprite(
            "idle_right",
            (-16.0 + start_cam_offset.0, 16.0 + start_cam_offset.1),
            0.0,
        )
        .unwrap();
    player.looping = 1;

    let size = 1024.0;
    let mut background = sprite_master
        .add_sprite(
            "bg_grass",
            (-size / 2.0 + start_cam_offset.0, -16.0 + start_cam_offset.1),
            0.9,
        )
        .unwrap();
    background.width = size;
    background.height = size;

    // add collision shapes
    let player_collision = collision_manager
        .insert_collision_rect(
            player.get_sparse_index().unwrap(),
            (-16.0 + start_cam_offset.0, 16.0 + start_cam_offset.1),
            (32.0, 32.0),
            0.0,
            0.1,
            0,
        )
        .unwrap();
    // test collision shape
    let floor = collision_manager.add_collision_rect(
        (-size / 2.0 + start_cam_offset.0, -16.0 + start_cam_offset.1),
        (size, size),
        0.0,
        0.1,
        0,
    );

    table
        .add_state(TwoWayArrowKey::new(3.0, 2.0, 5.0, 10.0, 9.8))
        .unwrap();
    table
        .add_state(PlayerAccess((player, player_collision)))
        .unwrap();
    table.add_state(GlobalMouseClickPos(0.0, 0.0)).unwrap();
    table.add_state(MinionsSpawn(vec![])).unwrap();
    table.add_state(FloorCollisionAccess(floor)).unwrap();
}

fn entry_point(table: &mut ecs::Table) {
    // important states
    let mut running_state = table.read_state::<RunningState>().unwrap();
    let mut uniform_data = table.read_state::<Uniform>().unwrap();
    let mut sprite_master = table.read_state::<SpriteMaster3000>().unwrap();
    let mut collision_manager = table.read_state::<CollisionManager>().unwrap();

    let (player, player_collision_rect) = &mut table.read_state::<PlayerAccess>().unwrap().0;
    let floor_rect = &table.read_state::<FloorCollisionAccess>().unwrap().0;

    let mod_state = table
        .read_state::<winit::keyboard::ModifiersState>()
        .unwrap();
    let mouse_state = table.read_state::<MouseState>().unwrap();
    let key_state = table.read_state::<KeyState>().unwrap();

    let mut speed_resolver = table.read_state::<TwoWayArrowKey>().unwrap();

    let mut global_click_pos = table.read_state::<GlobalMouseClickPos>().unwrap();
    let ratio = uniform_data.window_height / uniform_data.window_width;
    let relative_click_pos = (
        (mouse_state.x() / (uniform_data.window_width * 0.5) - 1.0)
            * uniform_data.height_resolution
            / ratio,
        (1.0 - mouse_state.y() / (uniform_data.window_height * 0.5))
            * uniform_data.height_resolution,
    );

    // quitting
    if key_state.just_clicked(winit::keyboard::KeyCode::KeyQ) {
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
    if mouse_state.right_button_clicked() {
        // println!("{:?}", uniform_data.delta_time);
        // println!("{:?}", unsafe {
        //     table.read_column::<Sprite>().unwrap().len()
        // });
        // println!("{:?}", sprite_master.read_anim_data(player));
    }

    // spawn minions
    if mouse_state.left_button_clicked() & !mouse_state.middle_button_pressed() {
        let mut minions_spawned = table.read_state::<MinionsSpawn>().unwrap();
        let mut new_sprite = sprite_master.clone_add(player).unwrap();
        new_sprite.base_depth = 0.5;
        new_sprite.pos_x = global_click_pos.0 - 16.0;
        new_sprite.pos_y = global_click_pos.1 + 16.0 + 0.001 * (rand::random::<f32>() - 0.5);
        minions_spawned.0.push(new_sprite);
    }

    // clear minions
    if key_state.just_clicked(winit::keyboard::KeyCode::KeyV) {
        let mut minions_spawned = table.remove_state::<MinionsSpawn>().unwrap();
        for each in minions_spawned.0 {
            sprite_master.remove_sprite(each).unwrap();
        }
        // println!("{:?}", minions_spawned.0);
        table
            .add_state::<MinionsSpawn>(MinionsSpawn(vec![]))
            .unwrap();
    }

    if key_state.is_pressed(winit::keyboard::KeyCode::Home) {
        uniform_data.global_offset_x += 2.0 * (rand::random::<f32>() - 0.5);
        uniform_data.global_offset_y += 2.0 * (rand::random::<f32>() - 0.5);
    }

    // give speed input
    if key_state.just_clicked(winit::keyboard::KeyCode::Space) {
        if !speed_resolver.in_air {
            speed_resolver.vertical_speed += 5.0;
        } else if !speed_resolver.double_jumped {
            speed_resolver.double_jumped = true;
            speed_resolver.vertical_speed = 5.0;
        }
    }
    // println!("{:?}", speed_resolver.vertical_speed);
    if key_state.is_pressed(winit::keyboard::KeyCode::KeyA) {
        speed_resolver.left = true;
    } else {
        speed_resolver.left = false;
    }
    if key_state.is_pressed(winit::keyboard::KeyCode::KeyD) {
        speed_resolver.right = true;
    } else {
        speed_resolver.right = false;
    }

    speed_resolver.update_speed(uniform_data.delta_time);

    player.pos_x += speed_resolver.current_speed;
    player.pos_y += speed_resolver.vertical_speed;
    player_collision_rect.sync_size_and_pos(&player);

    if player_collision_rect.pos_y > -16.0 {
        speed_resolver.in_air = true;
    }

    // falling on the ground
    if collision_manager.check_if_colliding(
        player.get_sparse_index().unwrap(),
        floor_rect.get_sparse_index().unwrap(),
    ) && player_collision_rect.pos_y < -16.0
    {
        player.pos_y = -16.0;
        speed_resolver.vertical_speed = 0.0;
        speed_resolver.in_air = false;
        speed_resolver.double_jumped = false;
    }

    // update animation according the speed data
    if speed_resolver.in_air {
        if speed_resolver.vertical_speed >= 0.0 {
            if speed_resolver.vertical_speed <= 0.5 {
                sprite_master.change_state(player, "jump_mid_air").unwrap();
            } else {
                sprite_master.change_state(player, "jump_start").unwrap();
            }
        } else {
            sprite_master.change_state(player, "jump_fall").unwrap();
        }
    } else {
        if speed_resolver.current_speed != 0.0 {
            sprite_master.change_state(player, "run_right").unwrap();
        } else {
            sprite_master.change_state(player, "idle_right").unwrap();
        }
    }
    if speed_resolver.current_speed < 0.0 {
        player.flipped_x = 1;
    } else if speed_resolver.current_speed > 0.0 {
        player.flipped_x = 0;
    }
    player_collision_rect.sync_size_and_pos(&player);
}

fn post_func(table: &mut ecs::Table) {
    // println!("wrapping up uwu")
}

fn main() {
    // std::env::set_var("RUST_BACKTRACE", "1");
    renderer::run(
        HEIGHT_RESOLUTION_HALF,
        MAX_SPRITES_NUM,
        entry_point,
        prep_func,
        post_func,
    );
}
