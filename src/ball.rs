use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
struct Ball;

#[derive(Bundle)]
pub struct BallBundle {
    ball: Ball,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub mesh3d: Mesh3d,
    pub mesh_material3d: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub linear_velocity: LinearVelocity,
}

impl BallBundle {
    pub fn new(
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        transform: Transform,
    ) -> Self {
        let size = 0.1;

        Self {
            ball: Ball,
            rigid_body: RigidBody::Dynamic,
            collider: Collider::sphere(size),
            mesh3d: Mesh3d(meshes.add(Sphere::new(size))),
            mesh_material3d: MeshMaterial3d(materials.add(Color::BLACK)),
            linear_velocity: LinearVelocity(transform.forward() * 100.0),
            transform,
        }
    }
}
