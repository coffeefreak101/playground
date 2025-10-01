use avian3d::prelude::*;
use bevy::prelude::*;
use chrono::{DateTime, Duration, Utc};

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct DespawnAfter(DateTime<Utc>);

#[derive(Bundle)]
pub struct BallBundle {
    ball: Ball,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub mesh3d: Mesh3d,
    pub mesh_material3d: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub linear_velocity: LinearVelocity,
    pub despawn_after: DespawnAfter,
}

impl BallBundle {
    pub fn new(
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        transform: Transform,
    ) -> Self {
        let size = 0.1;
        let despawn_after = DespawnAfter(Utc::now() + Duration::seconds(3));
        let mut velocity = transform.forward().normalize() * 100.0;
        // Aim slightly upward so the ball doesn't immediately start to fall after thrown
        velocity.y += 0.1;

        Self {
            ball: Ball,
            rigid_body: RigidBody::Dynamic,
            collider: Collider::sphere(size),
            mesh3d: Mesh3d(meshes.add(Sphere::new(size))),
            mesh_material3d: MeshMaterial3d(materials.add(Color::BLACK)),
            linear_velocity: LinearVelocity(velocity),
            despawn_after,
            transform,
        }
    }
}

pub fn handle_despawn_after(mut commands: Commands, query: Query<(Entity, &DespawnAfter)>) {
    for (entity, despawn_after) in query {
        if despawn_after.0 <= Utc::now() {
            commands.entity(entity).despawn();
        }
    }
}
