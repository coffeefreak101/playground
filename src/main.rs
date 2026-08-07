mod ball;
mod cube;
mod object_spawner;
mod player_movement;

use crate::ball::handle_despawn_after;
use crate::object_spawner::{ShowGhostAction, SpawnObject};
use crate::player_movement::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowMode};
use bevy_enhanced_input::prelude::*;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window: Single<&mut Window>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;
    window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Primary);

    // Static physics object with a collision shape
    commands.spawn((
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(128.0, 128.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));

    // Dynamic physics object with a collision shape and initial angular velocity
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(-1.0, 4.0, -1.0),
        Mass(0.1),
    ));

    // Light
    // commands.spawn((
    //     PointLight { ..default() },
    //     Transform::from_xyz(4.0, 8.0, 4.0),
    // ));

    let light = AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        ..default()
    };

    // Player
    commands
        .spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.4, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            Transform::from_xyz(0.0, 0.0, 0.0),
            PlayerBundle::new(Collider::capsule(0.4, 1.0)),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            GravityScale(1.0),
            actions!(Player[
                (
                    Action::<PlayerJump>::new(),
                    bindings![KeyCode::Space],
                ),
                (
                    Action::<PlayerMove>::new(),
                    DeadZone::default(),
                    SmoothNudge::default(),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick(),
                    ))
                ),
                (
                    Action::<PlayerAction>::new(),
                    bindings![MouseButton::Left],
                ),
                (
                    Action::<ShowGhostAction>::new(),
                    bindings![MouseButton::Right],
                ),
                (
                    Action::<PlayerSprint>::new(),
                    bindings![KeyCode::ShiftLeft]
                ),
                (
                    Action::<SpawnObject>::new(),
                    bindings![KeyCode::KeyE]
                )
            ]),
        ))
        .with_child((Camera3d::default(), Transform::from_xyz(0.0, 1.0, 0.0), light));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EnhancedInputPlugin,
            PhysicsPlugins::default().set(
                PhysicsInterpolationPlugin::interpolate_all()
            ),
            PlayerPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, handle_despawn_after)
        .run();
}
