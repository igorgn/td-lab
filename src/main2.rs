use bevy::camera::ScalingMode;
use bevy::feathers::{
    FeathersPlugins,
    controls::*,
    dark_theme::create_dark_theme,
    display::label,
    theme::{ThemeBackgroundColor, ThemeToken, UiTheme},
    tokens,
};
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::ui_widgets::ValueChange;
use bevy::window::PrimaryWindow;

/// Fraction of a cell covered by its tile; the rest is a visible gap between tiles.
const TILE_FILL: f32 = 0.95;
/// Building width as a fraction of the tile; height as a fraction of the cell size.
const BUILDING_FOOTPRINT: f32 = 0.7;
const BUILDING_HEIGHT: f32 = 0.5;
/// World units visible top-to-bottom at zoom scale 1.0.
const CAMERA_VIEW_HEIGHT: f32 = 5.0;
const ZOOM_SCALE_RANGE: (f32, f32) = (0.1, 10.0);
/// Fraction of zoom change per scroll line (applied exponentially).
const ZOOM_SPEED: f32 = 0.1;
const CAMERA_START: Vec3 = Vec3::new(11.0, 10.0, 11.0);
const CAMERA_TARGET_START: Vec3 = Vec3::new(1.0, 0.0, 1.0);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Tile;

#[derive(Component)]
struct Building;

/// Logical grid coordinates of a tile.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Cell(UVec2);

/// What can be done with a cell. Every flag is independent.
#[derive(Component, Clone, Copy)]
struct CellState {
    selectable: bool,
    walkable: bool,
    buildable: bool,
}

impl Default for CellState {
    fn default() -> Self {
        Self {
            selectable: true,
            walkable: true,
            buildable: true,
        }
    }
}

/// The cell under the cursor and the cell last clicked. Only selectable cells can be either.
#[derive(Resource, Default)]
struct CellSelection {
    hovered: Option<UVec2>,
    selected: Option<UVec2>,
}

/// Shared tile materials; a tile's look is chosen from these each frame.
#[derive(Resource)]
struct TileMaterials {
    normal: Handle<StandardMaterial>,
    hovered: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
    building: Handle<StandardMaterial>,
}

/// `size` cells per side; each cell is `cell_size` world units wide.
/// Logical cell (x, z) sits at world (x * cell_size, 0, z * cell_size).
#[derive(Resource)]
struct GridConfig {
    size: u32,
    cell_size: f32,
}

/// Where the camera is and what point it looks at. The camera's `Transform` is derived from this.
#[derive(Resource)]
struct CameraRig {
    position: Vec3,
    target: Vec3,
}

/// Which value a number input edits: the position or the target, and which axis (0 = x, 1 = y, 2 = z).
#[derive(Component, Default, Clone)]
struct CameraControl {
    target: bool,
    axis: usize,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FeathersPlugins))
        .insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(CameraRig {
            position: CAMERA_START,
            target: CAMERA_TARGET_START,
        })
        .insert_resource(GridConfig {
            size: 3,
            cell_size: 1.0,
        })
        .init_resource::<CellSelection>()
        .add_systems(Startup, (spawn_camera, setup_tile_materials, ui.spawn()))
        .add_systems(
            Update,
            (
                apply_camera_rig.run_if(resource_changed::<CameraRig>),
                sync_camera_inputs.run_if(resource_changed::<CameraRig>),
                pan_camera,
                zoom_camera,
                rebuild_grid.run_if(resource_changed::<GridConfig>),
                (
                    update_hovered_cell,
                    select_cell,
                    place_building,
                    highlight_cells,
                )
                    .chain(),
            ),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: CAMERA_VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(CAMERA_START).looking_at(CAMERA_TARGET_START, Vec3::Y),
    ));
}

fn apply_camera_rig(rig: Res<CameraRig>, mut camera: Single<&mut Transform, With<MainCamera>>) {
    **camera = Transform::from_translation(rig.position).looking_at(rig.target, Vec3::Y);
}

/// Keeps the position/target inputs showing the rig's values when something else (e.g. panning) moves it.
fn sync_camera_inputs(
    rig: Res<CameraRig>,
    inputs: Query<(Entity, &CameraControl)>,
    mut commands: Commands,
) {
    for (entity, control) in &inputs {
        let v = if control.target {
            rig.target[control.axis]
        } else {
            rig.position[control.axis]
        };
        commands.entity(entity).insert(NumberInputValue::F32(v));
    }
}

/// Right-drag pans: the ground under the cursor follows the mouse.
fn pan_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Transform, &Projection), With<MainCamera>>,
    mut rig: ResMut<CameraRig>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    let cursor = window.cursor_position();
    let previous = std::mem::replace(&mut *last_cursor, cursor);

    if !buttons.pressed(MouseButton::Right) {
        return;
    }
    let (Some(cursor), Some(previous)) = (cursor, previous) else {
        return;
    };
    let delta = cursor - previous;
    if delta == Vec2::ZERO {
        return;
    }

    let (transform, projection) = camera.into_inner();
    let Projection::Orthographic(ortho) = projection else {
        return;
    };
    let world_per_pixel = CAMERA_VIEW_HEIGHT * ortho.scale / window.height();

    // Move the camera opposite to the drag, within the view plane...
    let mut shift = Vec3::from(transform.right()) * (-delta.x * world_per_pixel)
        + Vec3::from(transform.up()) * (delta.y * world_per_pixel);
    // ...then slide along the view direction until the shift lies flat on the ground.
    // Sliding along the view direction doesn't change what an orthographic camera sees.
    let forward = Vec3::from(transform.forward());
    if forward.y.abs() < 1e-4 {
        return;
    }
    shift += forward * (-shift.y / forward.y);

    rig.position += shift;
    rig.target += shift;
}

fn zoom_camera(
    scroll: Res<AccumulatedMouseScroll>,
    mut projection: Single<&mut Projection, With<MainCamera>>,
) {
    if scroll.delta.y == 0.0 {
        return;
    }
    let lines = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / 40.0,
    };
    if let Projection::Orthographic(ortho) = &mut **projection {
        // Scrolling up zooms in, which means a smaller projection area.
        ortho.scale = (ortho.scale * (-ZOOM_SPEED * lines).exp())
            .clamp(ZOOM_SCALE_RANGE.0, ZOOM_SCALE_RANGE.1);
    }
}

fn setup_tile_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mut unlit = |color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })
    };
    commands.insert_resource(TileMaterials {
        normal: unlit(Color::srgb(0.3, 0.6, 0.4)),
        hovered: unlit(Color::srgb(0.5, 0.8, 0.6)),
        selected: unlit(Color::srgb(0.9, 0.8, 0.3)),
        building: unlit(Color::srgb(0.6, 0.4, 0.3)),
    });
}

/// Finds the cell under the cursor by intersecting the cursor ray with the ground plane (y = 0).
fn update_hovered_cell(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    grid: Res<GridConfig>,
    cells: Query<(&Cell, &CellState)>,
    mut selection: ResMut<CellSelection>,
) {
    let (camera, camera_transform) = *camera;
    let hovered = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .and_then(|ray| {
            let distance = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
            let point = ray.get_point(distance);
            // Tile centers sit on multiples of cell_size, so rounding gives the nearest cell.
            let x = (point.x / grid.cell_size).round();
            let z = (point.z / grid.cell_size).round();
            if x < 0.0 || z < 0.0 || x >= grid.size as f32 || z >= grid.size as f32 {
                return None;
            }
            Some(UVec2::new(x as u32, z as u32))
        })
        .filter(|coords| {
            cells
                .iter()
                .any(|(cell, state)| cell.0 == *coords && state.selectable)
        });

    if selection.hovered != hovered {
        selection.hovered = hovered;
    }
}

/// Left click selects the hovered cell; clicking anywhere else clears the selection.
fn select_cell(buttons: Res<ButtonInput<MouseButton>>, mut selection: ResMut<CellSelection>) {
    if buttons.just_pressed(MouseButton::Left) {
        selection.selected = selection.hovered;
    }
}

/// Left click on a buildable cell places a cuboid on it and marks the cell as no longer buildable.
fn place_building(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    selection: Res<CellSelection>,
    grid: Res<GridConfig>,
    materials: Res<TileMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cells: Query<(&Cell, &mut CellState)>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(hovered) = selection.hovered else {
        return;
    };
    let Some((_, mut state)) = cells
        .iter_mut()
        .find(|(cell, state)| cell.0 == hovered && state.buildable)
    else {
        return;
    };
    state.buildable = false;

    let footprint = grid.cell_size * TILE_FILL * BUILDING_FOOTPRINT;
    let height = grid.cell_size * BUILDING_HEIGHT;
    commands.spawn((
        Building,
        Mesh3d(meshes.add(Cuboid::new(footprint, height, footprint))),
        MeshMaterial3d(materials.building.clone()),
        // Cuboids are centered, so lift by half the height to rest on the tile.
        Transform::from_xyz(
            hovered.x as f32 * grid.cell_size,
            height / 2.0,
            hovered.y as f32 * grid.cell_size,
        ),
    ));
}

fn highlight_cells(
    selection: Res<CellSelection>,
    materials: Res<TileMaterials>,
    mut tiles: Query<(&Cell, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    for (cell, mut material) in &mut tiles {
        let wanted = if selection.selected == Some(cell.0) {
            &materials.selected
        } else if selection.hovered == Some(cell.0) {
            &materials.hovered
        } else {
            &materials.normal
        };
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }
}

fn rebuild_grid(
    mut commands: Commands,
    grid: Res<GridConfig>,
    old_tiles: Query<Entity, Or<(With<Tile>, With<Building>)>>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TileMaterials>,
    mut selection: ResMut<CellSelection>,
) {
    for tile in &old_tiles {
        commands.entity(tile).despawn();
    }
    *selection = CellSelection::default();

    let tile_size = grid.cell_size * TILE_FILL;
    let mesh = meshes.add(Plane3d::default().mesh().size(tile_size, tile_size));

    for x in 0..grid.size {
        for z in 0..grid.size {
            commands.spawn((
                Tile,
                Cell(UVec2::new(x, z)),
                CellState::default(),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(materials.normal.clone()),
                Transform::from_xyz(x as f32 * grid.cell_size, 0.0, z as f32 * grid.cell_size),
            ));
        }
    }
}

fn ui() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(8),
            left: px(8),
            width: px(160),
            padding: UiRect::all(px(8)),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            @label("Camera position")
            --
            @camera_input(false, 0, CAMERA_START.x, tokens::TEXT_INPUT_X_AXIS, "X")
            --
            @camera_input(false, 1, CAMERA_START.y, tokens::TEXT_INPUT_Y_AXIS, "Y")
            --
            @camera_input(false, 2, CAMERA_START.z, tokens::TEXT_INPUT_Z_AXIS, "Z")
            --
            @label("Camera looks at")
            --
            @camera_input(true, 0, CAMERA_TARGET_START.x, tokens::TEXT_INPUT_X_AXIS, "X")
            --
            @camera_input(true, 1, CAMERA_TARGET_START.y, tokens::TEXT_INPUT_Y_AXIS, "Y")
            --
            @camera_input(true, 2, CAMERA_TARGET_START.z, tokens::TEXT_INPUT_Z_AXIS, "Z")
            --
            @label("Grid size (cells per side)")
            --
            @FeathersNumberInput
            NumberInputValue::I32(3)
            HardLimit(NumberInputRange::I32(1..=50))
            on(
                |change: On<ValueChange<i32>>, mut grid: ResMut<GridConfig>, mut commands: Commands| {
                    grid.size = change.value.max(1) as u32;
                    commands.entity(change.event_target()).insert(NumberInputValue::I32(change.value));
                }
            )
            --
            @label("Cell size (world units)")
            --
            @FeathersNumberInput
            NumberInputValue::F32(1.0)
            NumberInputPrecision(2)
            HardLimit(NumberInputRange::F32(0.1..=5.0))
            on(
                |change: On<ValueChange<f32>>, mut grid: ResMut<GridConfig>, mut commands: Commands| {
                    grid.cell_size = change.value.max(0.1);
                    commands.entity(change.event_target()).insert(NumberInputValue::F32(change.value));
                }
            )
        ]
    }
}

fn camera_input(
    target: bool,
    axis: usize,
    value: f32,
    sigil_color: ThemeToken,
    label_text: &'static str,
) -> impl Scene {
    bsn! {
        @FeathersNumberInput {
            @sigil_color: sigil_color,
            @label_text: label_text,
        }
        CameraControl { target: target, axis: axis }
        NumberInputValue::F32(value)
        on(
            |change: On<ValueChange<f32>>,
             controls: Query<&CameraControl>,
             mut rig: ResMut<CameraRig>,
             mut commands: Commands| {
                let entity = change.event_target();
                if let Ok(control) = controls.get(entity) {
                    if control.target {
                        rig.target[control.axis] = change.value;
                    } else {
                        rig.position[control.axis] = change.value;
                    }
                }
                commands.entity(entity).insert(NumberInputValue::F32(change.value));
            }
        )
    }
}
