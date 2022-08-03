use bevy::{
    math::{Quat, Vec2},
    prelude::*,
    time::Time,
};
use iyes_loopless::prelude::*;

use crate::{
    consts::{self, LEFT_BOUNDARY_MAX_DISTANCE},
    enemy::SpawnLocationX,
    metadata::{GameMeta, LevelMeta},
    GameState, Player,
};

/// Plugin handling movement and rotation through velocities and torques.
pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::Last,
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(
                    // Here we add a chain of systems that act as constraints on movements, ending
                    // the chain with the velocity system itself which applies the velocities to the
                    // entities.
                    update_left_movement_boundary
                        .chain(constrain_player_movement)
                        .chain(velocity_system),
                )
                .with_system(torque_system)
                .into(),
        );
    }
}

/// An entity's linear velocity.
///
/// This is similar to the velocity you would set in a physics simulation, but in our case we use a
/// simple constraints system instead of actual physics simulation.
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Deref, DerefMut, Default)]
pub struct Velocity(pub Vec2);

/// System that updates translations based on entity velocities.
pub fn velocity_system(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, dir) in &mut query.iter_mut() {
        transform.translation += dir.0.extend(0.) * time.delta_seconds();
    }
}

/// An entity's angular velocity.
///
/// A positive value means a clockwise rotation and a negative value means couter-clockwise.
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Deref, DerefMut, Default)]
pub struct Torque(pub f32);

/// System that applies rotations based on entity torques.
pub fn torque_system(mut query: Query<(&mut Transform, &Torque)>, time: Res<Time>) {
    for (mut transform, torque) in &mut query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(**torque * time.delta_seconds());
    }
}

// (Moving) bondary before which, the players can't go back.
#[derive(Component)]
pub struct LeftMovementBoundary(f32);

impl Default for LeftMovementBoundary {
    fn default() -> Self {
        Self(-LEFT_BOUNDARY_MAX_DISTANCE)
    }
}

/// Updates player left movement boundary
pub fn update_left_movement_boundary(
    query: Query<&Transform, With<Player>>,
    mut boundary: ResMut<LeftMovementBoundary>,
    game_meta: Res<GameMeta>,
) {
    let max_player_x = query
        .iter()
        .map(|transform| transform.translation.x)
        .max_by(|ax, bx| ax.total_cmp(bx));

    if let Some(max_player_x) = max_player_x {
        boundary.0 = boundary
            .0
            .max(max_player_x - game_meta.camera_move_right_boundary - LEFT_BOUNDARY_MAX_DISTANCE);
    }
}

/// Constrains player movement based on multiple factors
fn constrain_player_movement(
    enemy_spawn_locations_query: Query<&'static SpawnLocationX>,
    level_meta: Res<LevelMeta>,
    game_meta: Res<GameMeta>,
    left_movement_boundary: Res<LeftMovementBoundary>,
    mut players: Query<(&Transform, &mut Velocity), With<Player>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    // Collect player positions and velocities
    let mut player_velocities = players
        .iter_mut()
        .map(|(transform, vel)| (transform.translation, vel))
        .collect::<Vec<_>>();

    // Identify the current stop poing
    let current_stop_point = level_meta.stop_points.iter().find(|point_x| {
        player_velocities
            .iter()
            .any(|(location, dir)| location.x < **point_x && **point_x <= location.x + dir.x)
    });

    // If there is a current stop point
    if let Some(current_stop_point) = current_stop_point {
        let any_enemy_behind_stop_point = enemy_spawn_locations_query
            .iter()
            .any(|SpawnLocationX(spawn_x)| spawn_x <= current_stop_point);

        // Prevent movement beyond the stop point if there are enemies not yet defeated behind the
        // stop point.
        if any_enemy_behind_stop_point {
            for (location, velocity) in player_velocities.iter_mut() {
                // Can be simplified, but it's harder to understand.
                if location.x + velocity.x * dt > *current_stop_point {
                    velocity.x = 0.;
                }
            }
        }
    }

    // Then, we perform the absolute clamping (screen top/left/bottom), and we collect the data
    // required for the relative clamping.

    let mut min_new_player_x = f32::MAX;

    #[allow(clippy::needless_collect)] // False alarm
    let velocities = player_velocities
        .into_iter()
        .map(|(location, mut velocity)| {
            let new_x = location.x + velocity.x * dt;

            if new_x < left_movement_boundary.0 {
                velocity.x = 0.;
            }

            //Restrict player to the ground
            let new_y = location.y + velocity.y * dt + consts::GROUND_OFFSET;

            if new_y >= consts::MAX_Y || new_y <= consts::MIN_Y {
                velocity.y = 0.;
            }

            let new_velocity = (velocity, new_x);

            min_new_player_x = min_new_player_x.min(new_x);

            (location, new_velocity)
        })
        .collect::<Vec<_>>();

    // Then, we perform the clamping of the players relative to each other.
    let max_players_x_distance = LEFT_BOUNDARY_MAX_DISTANCE + game_meta.camera_move_right_boundary;

    velocities
        .into_iter()
        .for_each(|(_, (mut velocity, new_player_x))| {
            if new_player_x > min_new_player_x + max_players_x_distance {
                **velocity = Vec2::ZERO
            }
        });
}
