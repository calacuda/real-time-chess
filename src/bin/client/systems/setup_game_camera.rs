use crate::client::components::marker_components::GameCamera;
use bevy::{prelude::*, render::camera::ScalingMode};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // Projection::from(OrthographicProjection {
        //     // We can set the scaling mode to FixedVertical to keep the viewport height constant as its aspect ratio changes.
        //     // The viewport height is the height of the camera's view in world units when the scale is 1.
        //     scaling_mode: ScalingMode::FixedVertical {
        //         viewport_height: 100.0,
        //     },
        //     // This is the default value for scale for orthographic projections.
        //     // To zoom in and out, change this value, rather than `ScalingMode` or the camera's position.
        //     scale: 1.0,
        //     // far:
        //     ..OrthographicProjection::default_3d()
        // }),
        // default projection 1
        // Transform::from_xyz(40.0, 40.0, 97.5).looking_at(Vec3::new(40.0, 40.0, 0.0), Vec3::Y),
        // 7.5
        // orthogonal projection 1
        // Transform::from_xyz(-25.0, 120.0, 00.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        Transform::from_xyz(40.0, 40.0, 120.0).looking_at(Vec3::new(65.0, 40.0, 0.0), Vec3::Y),
        GameCamera,
    ));
}
