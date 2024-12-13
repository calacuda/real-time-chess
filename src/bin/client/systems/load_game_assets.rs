use bevy::prelude::*;

pub fn load_game_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor = Mesh3d(
        meshes.add(
            Plane3d::from_points(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(80.0, 0.0, 0.0),
                Vec3::new(80.0, 80.0, 0.0),
            )
            .0
            // Plane3d::new(Vec3::Z, Vec2::splat(40.0))
            .mesh()
            .size(80.0, 80.0)
            .subdivisions(10),
        ),
    );
    // let floor_material = MeshMaterial3d(materials.add(Color::from(LinearRgba::GREEN)));
    let floor_pos = Transform::from_xyz(40.0, 40.0, 0.0);

    commands
        .spawn((floor, floor_pos))
        .with_children(|commands| {
            let square = Mesh3d(
                meshes.add(
                    Plane3d::from_points(
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(10.0, 0.0, 0.0),
                        Vec3::new(10.0, 10.0, 0.0),
                    )
                    .0
                    // Plane3d::new(Vec3::Z, Vec2::splat(40.0))
                    .mesh()
                    .size(10.0, 10.0)
                    .subdivisions(1),
                ),
            );

            let offset = 5.0;

            for i in 0..(8 * 8) {
                let material =
                    if (i % 2 == 1 && (i / 8) % 2 == 0) || (i % 2 == 0 && (i / 8) % 2 == 1) {
                        Color::from(LinearRgba::GREEN)
                    } else {
                        Color::from(LinearRgba::BLACK)
                    };
                let x = (10 * (i % 8)) as f32 + offset - 40.0;
                let y = (10 * (i / 8)) as f32 + offset - 40.0;

                info!("square at ({x}, {y}) is {:?}", material);

                // TODO: figure out a way to uniquely identify every square.
                commands.spawn((
                    square.clone(),
                    MeshMaterial3d(materials.add(material)),
                    Transform::from_xyz(x, y, 0.0),
                ));
            }
        });
}
