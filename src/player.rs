use bevy::{core::FixedTimestep, prelude::*};

use crate::{
    Laser, Materials, Player, PlayerFrom, PlayerReadyFire, PlayerState, Speed, WinSize,
    PLAYER_RESPONSE_DELAY, TIME_STEP,
};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(PlayerState::default())
            .add_startup_stage(
                "game_setup_actor",
                SystemStage::single(player_spawn.system()),
            )
            .add_system(player_movement.system())
            .add_system(player_fire.system())
            .add_system(laser_movement.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.5))
                    .with_system(player_spawn.system()),
            );
    }
}

fn player_spawn(
    mut commands: Commands,
    materials: Res<Materials>,
    win_size: Res<WinSize>,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
) {
    let now = time.seconds_since_startup();
    let last_shot = player_state.last_shot;
    if !player_state.on && (last_shot == 0. || now > last_shot + PLAYER_RESPONSE_DELAY) {
        let bottom = -win_size.height / 2.0;
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.player_materials.clone(),
                transform: Transform {
                    translation: Vec3::new(0., bottom + 25., 0.),
                    scale: Vec3::new(1.0, 1.0, 0.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Player)
            .insert(PlayerReadyFire(true))
            .insert(Speed::default());
        player_state.spawned();
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    _win_size: Res<WinSize>,
    mut query: Query<(&Speed, &mut Transform, With<Player>)>,
) {
    if let Ok((speed, mut transform, _)) = query.single_mut() {
        let direction_x: f32 = if keyboard_input.pressed(KeyCode::Left) {
            -1.
        } else if keyboard_input.pressed(KeyCode::Right) {
            1.
        } else {
            0.
        };
        let direction_y: f32 = if keyboard_input.pressed(KeyCode::Up) {
            1.
        } else if keyboard_input.pressed(KeyCode::Down) {
            -1.
        } else {
            0.
        };
        transform.translation.x += direction_x * speed.0 * TIME_STEP;
        transform.translation.y += direction_y * speed.0 * TIME_STEP;
    }
}

fn player_fire(
    materials: Res<Materials>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut PlayerReadyFire, With<Player>)>,
) {
    if let Ok((player_transform, mut player_ready_fire, _)) = query.single_mut() {
        if player_ready_fire.0 && keyboard_input.pressed(KeyCode::Space) {
            let x = player_transform.translation.x;
            let y = player_transform.translation.y;

            let mut spawn_lasers = |x_offset: f32| {
                commands
                    .spawn_bundle(SpriteBundle {
                        material: materials.laser.clone(),
                        transform: Transform {
                            translation: Vec3::new(x + x_offset, y + 10., 0.),
                            scale: Vec3::new(0.05, 0.05, 0.),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Laser)
                    .insert(PlayerFrom)
                    .insert(Speed::default());
            };

            let x_offset = 10.;
            spawn_lasers(x_offset);
            spawn_lasers(-x_offset);

            // laser_spawn(commands, &materials, x, y);
            // laser_spawn(commands, &materials, x, y);

            player_ready_fire.0 = false;
        }

        if keyboard_input.just_released(KeyCode::Space) {
            player_ready_fire.0 = true;
        }
    }
}

fn laser_movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Speed, &mut Transform), (With<Laser>, With<PlayerFrom>)>,
) {
    for (laser_entity, speed, mut laser_transform) in query.iter_mut() {
        let translation = &mut laser_transform.translation;
        translation.y += speed.0 * TIME_STEP;
        if translation.y > win_size.height {
            commands.entity(laser_entity).despawn();
        }
    }
}
