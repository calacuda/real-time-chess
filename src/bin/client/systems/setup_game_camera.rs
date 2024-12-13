use crate::client::components::marker_components::GameCamera;
use bevy::{prelude::*, render::camera::ScalingMode};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            // We can set the scaling mode to FixedVertical to keep the viewport height constant as its aspect ratio changes.
            // The viewport height is the height of the camera's view in world units when the scale is 1.
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 8.0,
            },
            // This is the default value for scale for orthographic projections.
            // To zoom in and out, change this value, rather than `ScalingMode` or the camera's position.
            scale: 1.,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0., 8.0, 2.5).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        GameCamera,
    ));
}
