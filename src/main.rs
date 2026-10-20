use bevy::prelude::*;

const NORMAL_GREEN: Color = Color::srgb(0.3, 0.6, 0.4);
const HOVERED: Color = Color::srgb(0.6, 0.65, 0.85);
const NOT_BUILDABLE_RED: Color = Color::srgb(0.8, 0.1, 0.1);
const SELECTED_BLUE: Color = Color::srgb(0.1, 0.2, 0.7);
const CELL_SIZE: f32 = 0.5;
const TILE_FILL: f32 = 0.95;
const GRID_SIZE: u32 = 6;
const VIEWPORT_HEIGHT: f32 = 5.;
fn main() {
    App::new()
        .insert_resource(HoveredCell(None))
        .insert_resource(SelectedCell(None))
        .insert_resource(GridConfig {
            size: GRID_SIZE,
            cell_size: CELL_SIZE,
        })
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (
                setup_camera,
                setup_tiles,
                setup_light,
                inspect_cells.after(setup_tiles),
            ),
        )
        .add_systems(
            PostUpdate,
            (cell_material_manager, detect_hovered_cell, mouse_selection).chain(),
        )
        .run();
}
#[derive(Component, Debug)]
struct Cell(UVec2);

#[derive(Component, Debug)]
struct CellState {
    buildable: bool,
    walkable: bool,
    selectable: bool,
}

impl Default for CellState {
    fn default() -> Self {
        Self {
            buildable: true,
            walkable: true,
            selectable: true,
        }
    }
}

#[derive(Resource, Default)]
struct HoveredCell(Option<UVec2>);

#[derive(Resource, Default)]
struct SelectedCell(Option<UVec2>);

#[derive(Resource)]
struct GridConfig {
    size: u32,
    cell_size: f32,
}
fn inspect_cells(cells: Query<&CellState>) {
    for cell in cells {
        dbg!(cell);
    }
}

fn setup_camera(mut commands: Commands, grid_config: Res<GridConfig>) {
    let middle = (grid_config.size - 1) as f32 * (grid_config.cell_size / 2.);
    let center = Vec3::new(middle, 0., middle);

    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: VIEWPORT_HEIGHT,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(middle + 10., 10., middle + 10.).looking_at(center, Vec3::Y),
        // Transform::from_xyz(0., 10., 0.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn setup_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Resource)]
struct TileMaterials {
    normal: Handle<StandardMaterial>,
    hovered: Handle<StandardMaterial>,
    not_buildable: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
}

fn setup_tiles(
    grid_config: Res<GridConfig>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Plane3d::default().mesh().size(
        grid_config.cell_size * TILE_FILL,
        grid_config.cell_size * TILE_FILL,
    ));

    let normal = materials.add(NORMAL_GREEN);
    let hovered = materials.add(HOVERED);
    let not_buildable = materials.add(NOT_BUILDABLE_RED);
    let selected = materials.add(SELECTED_BLUE);

    commands.insert_resource(TileMaterials {
        normal: normal.clone(),
        hovered: hovered.clone(),
        not_buildable: not_buildable.clone(),
        selected: selected.clone(),
    });

    for x in 0..grid_config.size {
        for z in 0..grid_config.size {
            let cell_coordinates = Vec3::new(
                x as f32 * grid_config.cell_size,
                0.,
                z as f32 * grid_config.cell_size,
            );
            commands.spawn((
                Cell(UVec2::new(x, z)),
                // This is for testing
                if x % 2 == 0 {
                    CellState::default()
                } else {
                    CellState {
                        buildable: false,
                        ..default()
                    }
                },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(normal.clone()),
                Transform::from_translation(cell_coordinates),
            ));
        }
    }
}

fn cell_material_manager(
    mut cells: Query<(&Cell, &CellState, &mut MeshMaterial3d<StandardMaterial>)>,
    mut previous_hover: Local<Option<UVec2>>,
    mut previous_selected: Local<Option<UVec2>>,
    current_hover: Res<HoveredCell>,
    current_selected: Res<SelectedCell>,
    materials: Res<TileMaterials>,
) {
    for (cell, _, mut cell_material) in &mut cells {
        if Some(cell.0) == current_selected.0 {
            cell_material.0 = materials.selected.clone();
        } else if Some(cell.0) == *previous_selected {
            cell_material.0 = materials.normal.clone();
        }
    }

    *previous_selected = current_selected.0;

    if current_hover.0 == *previous_hover {
        return;
    }

    for (cell, cell_state, mut cell_material) in &mut cells {
        if !cell_state.selectable {
            continue;
        }

        if Some(cell.0) == current_hover.0 && current_selected.0 != current_hover.0 {
            cell_material.0 = if cell_state.buildable {
                materials.hovered.clone()
            } else {
                materials.not_buildable.clone()
            };
        } else if Some(cell.0) == *previous_hover && current_selected.0 != *previous_hover {
            cell_material.0 = materials.normal.clone();
        }
    }

    *previous_hover = current_hover.0;
}

fn neighbours(cell: UVec2, grid_size: u32) -> [Option<UVec2>; 4] {
    let north = if cell.y == 0 {
        None
    } else {
        Some(UVec2::new(cell.x, cell.y - 1))
    };
    let south = if cell.y + 1 >= grid_size {
        None
    } else {
        Some(UVec2::new(cell.x, cell.y + 1))
    };
    let west = if cell.x == 0 {
        None
    } else {
        Some(UVec2::new(cell.x - 1, cell.y))
    };
    let east = if cell.x + 1 >= grid_size {
        None
    } else {
        Some(UVec2::new(cell.x + 1, cell.y))
    };

    println!("{:?}",[north, south, west, east]);
    [north, south, west, east]
}

fn detect_hovered_cell(
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    grid_config: Res<GridConfig>,
    mut hovered_cell: ResMut<HoveredCell>,
) {
    let (camera, camera_transform) = *camera;
    if let Some(cursor_position) = window.cursor_position()
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
        && let Some(ip) =
            ray.plane_intersection_point(Vec3::ZERO, InfinitePlane3d { normal: Dir3::Y })
    {
        let cell_x = ((ip.x + grid_config.cell_size / 2.) / grid_config.cell_size).floor() as i32;
        let cell_z = ((ip.z + grid_config.cell_size / 2.) / grid_config.cell_size).floor() as i32;
        if (0..grid_config.size as i32).contains(&cell_x)
            && (0..grid_config.size as i32).contains(&cell_z)
        {
            hovered_cell.0 = Some(UVec2::new(cell_x as u32, cell_z as u32));
            neighbours(hovered_cell.0.unwrap(), grid_config.size);
        } else {
            hovered_cell.0 = None;
        }
        
    }
}

fn mouse_selection(
    input: Res<ButtonInput<MouseButton>>,
    hovered: Res<HoveredCell>,
    mut selected_cell: ResMut<SelectedCell>,
) {
    if input.just_pressed(MouseButton::Left) {
        selected_cell.0 = hovered.0
    }
    if input.just_pressed(MouseButton::Right) {
        selected_cell.0 = None
    }
}
