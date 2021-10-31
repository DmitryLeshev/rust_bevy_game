use std::f32::consts::PI;

use bevy::{core::FixedTimestep, prelude::*};
use rand::{thread_rng, Rng};

use crate::{
    ActiveEnemies, Enemy, EnemyFrom, Laser, Materials, Speed, WinSize, ENEMY_MAX_COUNT, TIME_STEP,
};

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_system(enemy_laser_movement.system())
            // .add_system(enemy_movement.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.0))
                    .with_system(enemy_spawn.system()),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.9))
                    .with_system(enemy_fire.system()),
            );
    }
}

fn enemy_spawn(
    mut commands: Commands,
    mut active_enemies: ResMut<ActiveEnemies>,
    materials: Res<Materials>,
    win_size: Res<WinSize>,
) {
    if active_enemies.0 < ENEMY_MAX_COUNT {
        let mut rng = thread_rng();
        let width_span = win_size.width / 2. - 100.;
        let height_span = win_size.height / 2. - 100.;

        let x = rng.gen_range(-width_span..width_span) as f32;
        let y = rng.gen_range(-height_span..height_span) as f32;

        commands
            .spawn_bundle(SpriteBundle {
                material: materials.enemy.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y, 0.),
                    scale: Vec3::new(1.0, 1.0, 0.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Enemy)
            .insert(Speed::default());

        active_enemies.0 += 1;
    }
}

fn enemy_movement(mut query: Query<(&Speed, &mut Transform), With<Enemy>>, time: Res<Time>) {
    let now = time.seconds_since_startup() as f32;
    for (speed, mut tf) in query.iter_mut() {
        let max_distance = TIME_STEP * speed.0;
        let x_org = tf.translation.x;
        let y_org = tf.translation.y;

        let (x_offset, y_offset) = (0., 100.);
        let (x_radius, y_radius) = (150., 100.);

        let angle = speed.0 * TIME_STEP * now % 360. / PI;

        let x_dst = x_radius * angle.cos() + x_offset;
        let y_dst = y_radius * angle.sin() + y_offset;

        let dx = x_org - x_dst;
        let dy = y_org - y_dst;

        let distance = (dx * dx + dy * dy).sqrt();
        let distance_ratio = if distance == 0. {
            0.
        } else {
            max_distance / distance
        };

        let x = x_org - dx * distance_ratio;
        let x = if dx > 0. { x.max(x_dst) } else { x.min(x_dst) };

        let y = y_org - dy * distance_ratio;
        let y = if dy > 0. { y.max(y_dst) } else { y.min(y_dst) };

        tf.translation.x = x;
        tf.translation.y = y;
    }
}

fn enemy_fire(
    mut commands: Commands,
    materials: Res<Materials>,
    mut enemy_query: Query<(&Transform), (With<Enemy>)>,
) {
    for transform in enemy_query.iter_mut() {
        let x = transform.translation.x;
        let y = transform.translation.y;

        commands
            .spawn_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(x, y - 15., 0.),
                    scale: Vec3::new(0.02, 0.02, 0.),
                    ..Default::default()
                },
                material: materials.laser.clone(),
                ..Default::default()
            })
            .insert(Laser)
            .insert(EnemyFrom)
            .insert(Speed::default());
    }
}

fn enemy_laser_movement(
    mut commands: Commands,
    mut laser_query: Query<(Entity, &Speed, &mut Transform), (With<Laser>, With<EnemyFrom>)>,
    win_size: Res<WinSize>,
) {
    for (entity, speed, mut transform) in laser_query.iter_mut() {
        transform.translation.y -= speed.0 * TIME_STEP;

        if transform.translation.y < -win_size.height {
            commands.entity(entity).despawn();
        }
    }
}
