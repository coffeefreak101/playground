use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
struct Cube;

#[derive(Bundle)]
pub struct CubeBundle {
    cube: Cube,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub mesh3d: Mesh3d,
    pub mesh_material3d: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub mass: Mass,
}

impl CubeBundle {
    pub fn new(
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        transform: Transform,
    ) -> Self {
        Self {
            cube: Cube,
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cuboid(1.0, 1.0, 1.0),
            mesh3d: Mesh3d(meshes.add(Cuboid::from_length(1.0))),
            mesh_material3d: MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            transform,
            mass: Mass(0.1),
        }
    }
}
