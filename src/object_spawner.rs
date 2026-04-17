use crate::cube::CubeBundle;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component)]
pub struct SpawnerContext;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ShowGhostAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct SpawnObject;

// fn spawner_context(mut commands: Commands) {
//     commands.spawn(actions!(
//         SpawnerContext[(Action::<ShowGhost>::new(), bindings![MouseButton::Right])]
//     ));
// }
//
// pub struct SpawnerPlugin;
//
// impl Plugin for SpawnerPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_input_context::<SpawnerContext>();
//
//         app.add_systems(Startup, spawner_context)
//             .add_observer(handle_spawn_ghost_object)
//             .add_observer(handle_move_ghost_object)
//             .add_observer(handle_despawn_ghost_object);
//     }
// }

fn out_front(transform: &Transform) -> Transform {
    let mut position = *transform;

    position.translation += transform.forward().normalize() * 10.0;
    position.rotation.y = 0.0;

    position
}

pub fn handle_toggle_ghost_object(
    _on: On<Start<ShowGhostAction>>,
    query_ghost: Query<Entity, With<SpawnerContext>>,
    query_spawn: Query<(&Transform, Entity), With<Camera3d>>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(ghost) = query_ghost.single() {
        commands.entity(ghost).despawn();
    } else {
        handle_spawn_ghost_object(query_spawn, commands, meshes, materials);
    }
}

pub fn handle_spawn_ghost_object(
    query: Query<(&Transform, Entity), With<Camera3d>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((transform, entity)) = query.single() else {
        error!("No player transform found for spawn");
        return;
    };

    let color = Srgba::hex("#FF000041").expect("Failed to build hex color");

    let ghost_object = (
        SpawnerContext,
        RigidBody::Static,
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(color))),
        RigidBodyDisabled,
        out_front(transform),
    );

    commands.entity(entity).with_child(ghost_object);
}

// pub fn handle_move_ghost_object(
//     _on: On<Fire<ShowGhostAction>>,
//     show_ghost: Res<ShowGhost>,
//     mut transforms: ParamSet<(
//         Query<&Transform, With<Player>>,
//         Query<&mut Transform, With<SpawnerContext>>,
//     )>,
// ) {
//     if !show_ghost.0 {
//         info!("Show ghost is {}", show_ghost.0);
//         return;
//     }
//
//     let transform = if let Ok(transform) = transforms.p0().single() {
//         *transform
//     } else {
//         error!("No player transform found for move");
//         return;
//     };
//
//     if let Ok(mut ghost) = transforms.p1().single_mut() {
//         ghost.translation = out_front(&transform).translation;
//     } else {
//         error!("No ghost transform found for move");
//     };
// }

pub fn handle_cube_spawn(
    _on: On<Start<SpawnObject>>,
    query: Query<Entity, With<SpawnerContext>>,
    query_transform: Query<&GlobalTransform>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(entity) = query.single() else {
        return;
    };

    let Ok(transform) = query_transform.get(entity) else {
        error!("Failed to get spawner transform");
        return;
    };

    commands.spawn(CubeBundle::new(
        meshes,
        materials,
        transform.compute_transform(),
    ));
}
