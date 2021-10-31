use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::{enemy::EnemyPlugin, player::PlayerPlugin};

mod enemy;
mod player;

// Constants
const PLAYER_SPRITE: &str = "sprites/player.png";
const ENEMY_SPRITE: &str = "sprites/enemy.png";
const PLAYER_LASER: &str = "sprites/laser_1.png";
const EXPLOSION_SHEET: &str = "sprites/explosion_sheet.png";
const TITLE: &str = "BEVY GAME";

const ENEMY_MAX_COUNT: u32 = 3;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

const TIME_STEP: f32 = 1. / 60.;

const PLAYER_RESPONSE_DELAY: f64 = 2.;

// Entityes Componets Systems Resourse

// region: Resourse
pub struct Materials {
    player_materials: Handle<ColorMaterial>,
    laser: Handle<ColorMaterial>,
    enemy: Handle<ColorMaterial>,
    explosion_sheet: Handle<TextureAtlas>,
}

pub struct WinSize {
    #[allow(unused)]
    width: f32,
    height: f32,
}

pub struct ActiveEnemies(u32);

pub struct PlayerState {
    on: bool,
    last_shot: f64,
}
impl Default for PlayerState {
    fn default() -> Self {
        Self {
            on: false,
            last_shot: 0.,
        }
    }
}
impl PlayerState {
    fn shot(&mut self, time: f64) {
        self.last_shot = time;
        self.on = false;
    }
    fn spawned(&mut self) {
        self.last_shot = 0.;
        self.on = true;
    }
}
// endregion: Resourse

// region: Componets
pub struct Laser;
pub struct Player;
pub struct PlayerReadyFire(bool);
pub struct PlayerFrom;
pub struct Enemy;
pub struct EnemyFrom;
pub struct Explosion;
pub struct ExplosionToSpawn(Vec3);

pub struct Speed(f32);
impl Default for Speed {
    fn default() -> Self {
        Self(500.)
    }
}

// endregion: Componets

fn main() {
    println!("Hello, Bevy!");
    let mut app = App::build();

    // Ресурсы
    app.insert_resource(ClearColor(Color::rgba(0.5, 0.2, 0.2, 0.3)))
        .insert_resource(WindowDescriptor {
            title: TITLE.to_string(),
            width: WIDTH,
            height: HEIGHT,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ActiveEnemies(0));
    // .insert_resource(PlayerState::default());
    // Плагины
    app.add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin);

    // Системы
    app.add_startup_system(setup.system())
        .add_system(laser_hit_enemy.system())
        .add_system(laser_hit_player.system())
        .add_system(explosion_to_spawn.system())
        .add_system(animate_explosion.system());
    // .add_startup_stage(
    //     "game_setup_actor",
    //     SystemStage::single(player_spawn.system()),
    // )
    // .add_system(player_movement.system())
    // .add_system(player_fire.system())
    // .add_system(laser_movement.system());

    // Старт
    app.run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    //  create window
    let window = windows.get_primary_mut().unwrap();

    // Камера
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Создаем атлас
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32., 32.), 3, 3);
    // Добавляем ресурсы
    commands.insert_resource(Materials {
        player_materials: materials.add(asset_server.load(PLAYER_SPRITE).into()),
        laser: materials.add(asset_server.load(PLAYER_LASER).into()),
        enemy: materials.add(asset_server.load(ENEMY_SPRITE).into()),
        explosion_sheet: texture_atlases.add(texture_atlas),
    });

    commands.insert_resource(WinSize {
        width: window.width(),
        height: window.height(),
    });

    // window.set_position(IVec2::new(3870, 4830));
}

fn laser_hit_enemy(
    mut commands: Commands,
    mut laser_query: Query<(Entity, &Transform, &Sprite), (With<Laser>, With<PlayerFrom>)>,
    mut enemy_query: Query<(Entity, &Transform, &Sprite, With<Enemy>)>,
    mut active_enemies: ResMut<ActiveEnemies>,
) {
    for (laser_entity, laser_transform, laser_sprite) in laser_query.iter_mut() {
        for (enemy_entity, enemy_transform, enemy_sprite, _) in enemy_query.iter_mut() {
            let laser_scale = Vec2::from(laser_transform.scale);
            let enemy_scale = Vec2::from(enemy_transform.scale);

            let collision = collide(
                laser_transform.translation,
                laser_sprite.size * laser_scale,
                enemy_transform.translation,
                enemy_sprite.size * enemy_scale,
            );

            if let Some(_is_collision) = collision {
                if active_enemies.0 > 0 {
                    // remove the enemy
                    commands.entity(enemy_entity).despawn();
                    active_enemies.0 -= 1;

                    // remove the laser
                    commands.entity(laser_entity).despawn();

                    // spawn exposion
                    commands
                        .spawn()
                        .insert(ExplosionToSpawn(enemy_transform.translation.clone()));
                }
            }
        }
    }
}

fn laser_hit_player(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    mut laser_query: Query<(Entity, &Transform, &Sprite), (With<Laser>, With<EnemyFrom>)>,
    mut player_query: Query<(Entity, &Transform, &Sprite), (With<Player>)>,
) {
    for (laser_entity, laser_transform, laser_sprite) in laser_query.iter() {
        let laser_scale = Vec2::from(laser_transform.scale);
        for (player_entity, player_transform, player_sprite) in player_query.iter() {
            let player_scale = Vec2::from(player_transform.scale);

            let collision = collide(
                player_transform.translation,
                player_sprite.size * player_scale,
                laser_transform.translation,
                laser_sprite.size * laser_scale,
            );
            if let Some(_) = collision {
                player_state.shot(time.seconds_since_startup());
                commands.entity(laser_entity).despawn();
                commands.entity(player_entity).despawn();
                commands
                    .spawn()
                    .insert(ExplosionToSpawn(player_transform.translation.clone()));
            }
        }
    }
}

fn explosion_to_spawn(
    mut commands: Commands,
    mut query: Query<(Entity, &ExplosionToSpawn)>,
    materials: Res<Materials>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter_mut() {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: materials.explosion_sheet.clone(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(Timer::from_seconds(0.05, true));

        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn animate_explosion(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Timer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
        With<Explosion>,
    )>,
) {
    for (
        explosion_entity,
        mut explosion_timer,
        mut explosion_sprite_sheet,
        texture_atlas_handle,
        _,
    ) in query.iter_mut()
    {
        explosion_timer.tick(time.delta());
        if explosion_timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            explosion_sprite_sheet.index += 1;
            if explosion_sprite_sheet.index == texture_atlas.textures.len() as u32 {
                commands.entity(explosion_entity).despawn();
            }
        }
    }
}
