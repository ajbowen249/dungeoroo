use yew::prelude::*;
use yew::html::*;
use crate::generation_fields::dungeon::*;

#[derive(Debug, PartialEq)]
pub struct DungeonCellUIProps {
    pub possible_types: Vec<DungeonCellType>,
    pub is_start_location: bool,
    pub is_goal_location: bool,
    pub is_goal_entrance_location: bool,
}

impl IntoPropValue<DungeonCellProps> for DungeonCellUIProps {
    fn into_prop_value(self) -> DungeonCellProps {
        DungeonCellProps {
            ui_props: self,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct DungeonCellProps {
    pub ui_props: DungeonCellUIProps,
}

type CartesianPoint2D = (f64, f64);

const CELL_WIDTH_PX: f64 = 1280.0;
const CELL_HEIGHT_PX: f64 = 394.0;
const UNIT_CUBE_ELEMENT_WIDTH: f64 = 128.0;
const UNIT_CUBE_ELEMENT_HEIGHT: f64 = 132.0;

const UNIT_CUBE_Z_PX: f64 = 68.0;
const POSITIVE_X: CartesianPoint2D = (64.0, 32.0);
const POSITIVE_Y: CartesianPoint2D = (POSITIVE_X.0 * -1.0, POSITIVE_X.1);

/// Transforms 3D coordinates into a 2D location in the bounding box of the containing cell.
/// (0, 0, 0) is 29px below the lower left of the cell element. This allows the iso grid to have easy alignment with the bounding hexagon. The left point of
/// the cell-local hexagon is (3, 3, 0).
/// Positive X is diagonally up right. Positive Y is diagonally up left. Z is up.
fn volume_loc_to_cell_loc(x: f64, y: f64, z: f64) -> CartesianPoint2D {
    let mut x2d: f64 = 0.0;
    let mut y2d: f64 = CELL_HEIGHT_PX + 29.0;

    x2d += x * POSITIVE_X.0;
    y2d -= x * POSITIVE_X.1;

    x2d += y * POSITIVE_Y.0;
    y2d -= y * POSITIVE_Y.1;

    y2d += z * UNIT_CUBE_Z_PX;

    (x2d, y2d)
}

fn to_css_px(point: CartesianPoint2D) -> (String, String) {
    (
        format!("{}px", point.0),
        format!("{}px", point.1),
    )
}

fn unit_cube(bottom_left: &CartesianPoint2D, color: Option<&str>) -> Html {
    let offset_point = to_css_px((
        bottom_left.0.floor(),
        bottom_left.1 .floor(),
    ));

    html! {
        <div
            style={format!(
                "margin-left: {}; margin-top: {};{}",
                offset_point.0,
                offset_point.1,
                match color { Some(color) => format!(" --color: {}", color), None => String::from("") }
            )}
            class={classes!("dungeon-cell-unit-cube")}
        >
            <div class={classes!("dungeon-cell-unit-cube-inner")}>
                <div class={classes!("dungeon-cell-unit-cube-face", "dungeon-cell-unit-cube-top-face")}>
                    <svg><path /></svg>
                </div>
                <div class={classes!("dungeon-cell-unit-cube-face", "dungeon-cell-unit-cube-left-face")}>
                    <svg><path /></svg>
                </div>
                <div class={classes!("dungeon-cell-unit-cube-face", "dungeon-cell-unit-cube-right-face")}>
                    <svg><path /></svg>
                </div>
            </div>
        </div>
    }
}

// Edge of the outline of the local cell. So top left is as viewed, not taking into account the rotating underlying data.
enum OutlineEdge {
    TopLeft,
    Top,
    TopRight,
    BottomRight,
    Bottom,
    BottomLeft,
}

fn get_outline_edge_points(edge: OutlineEdge, z: f64) -> Vec<CartesianPoint2D> {
    // The rev()s here are the painter's algorithm
    match edge {
        OutlineEdge::TopLeft => (3..9).rev().map(|loc| volume_loc_to_cell_loc(loc as f64, 3.0, z)).collect::<Vec<CartesianPoint2D>>(),
        OutlineEdge::Top => {
            vec![
                // outer
                (0..4).rev().map(|loc| volume_loc_to_cell_loc(12.0 - (loc as f64), 0.0 + (loc as f64), z)).collect::<Vec<CartesianPoint2D>>(),
                // inner
                (0..3).rev().map(|loc| volume_loc_to_cell_loc(11.0 - (loc as f64), 0.0 + (loc as f64), z)).collect::<Vec<CartesianPoint2D>>(),
            ].into_iter().flatten().collect::<Vec<CartesianPoint2D>>()
        },
        OutlineEdge::TopRight => (-5..0).rev().map(|loc| volume_loc_to_cell_loc(12.0, loc as f64, z)).collect::<Vec<CartesianPoint2D>>(),
        OutlineEdge::BottomRight => (7..13).rev().map(|loc| volume_loc_to_cell_loc(loc as f64, -6.0, z)).collect::<Vec<CartesianPoint2D>>(),
        OutlineEdge::Bottom => {
            vec![
                // inner
                (0..3).rev().map(|loc| volume_loc_to_cell_loc(6.0 - (loc as f64), -5.0 + (loc as f64), z)).collect::<Vec<CartesianPoint2D>>(),
                // outer
                (0..4).rev().map(|loc| volume_loc_to_cell_loc(6.0 - (loc as f64), -6.0 + (loc as f64), z)).collect::<Vec<CartesianPoint2D>>(),
            ].into_iter().flatten().collect::<Vec<CartesianPoint2D>>()
        },
        OutlineEdge::BottomLeft => (-2..3).rev().map(|loc| volume_loc_to_cell_loc(3.0, loc as f64, z)).collect::<Vec<CartesianPoint2D>>(),
    }
}

fn get_hex_grid_floor(is_filled: bool) -> Vec<CartesianPoint2D> {
    vec![
        get_outline_edge_points(OutlineEdge::Top, -1.0),
        get_outline_edge_points(OutlineEdge::TopRight, -1.0),
        get_outline_edge_points(OutlineEdge::TopLeft, -1.0),

        // Fill Spans
        (if is_filled { (0..4).rev().map(|loc| volume_loc_to_cell_loc(11.0 - (loc as f64), -1.0 + (loc as f64), -1.0)).collect::<Vec<CartesianPoint2D>>() } else { vec![]} ),
        (if is_filled { (0..5).rev().map(|loc| volume_loc_to_cell_loc(11.0 - (loc as f64), -2.0 + (loc as f64), -1.0)).collect::<Vec<CartesianPoint2D>>() } else { vec![]} ),
        (if is_filled { (0..6).rev().map(|loc| volume_loc_to_cell_loc(11.0 - (loc as f64), -3.0 + (loc as f64), -1.0)).collect::<Vec<CartesianPoint2D>>() } else { vec![]} ),
        (if is_filled { (0..7).rev().map(|loc| volume_loc_to_cell_loc(11.0 - (loc as f64), -4.0 + (loc as f64), -1.0)).collect::<Vec<CartesianPoint2D>>() } else { vec![]} ),
        (if is_filled { (0..8).rev().map(|loc| volume_loc_to_cell_loc(11.0 - (loc as f64), -5.0 + (loc as f64), -1.0)).collect::<Vec<CartesianPoint2D>>() } else { vec![]} ),
        (if is_filled { (0..7).rev().map(|loc| volume_loc_to_cell_loc(10.0 - (loc as f64), -5.0 + (loc as f64), -1.0)).collect::<Vec<CartesianPoint2D>>() } else { vec![]} ),
        (if is_filled { (0..6).rev().map(|loc| volume_loc_to_cell_loc( 9.0 - (loc as f64), -5.0 + (loc as f64), -1.0)).collect::<Vec<CartesianPoint2D>>() } else { vec![]} ),
        (if is_filled { (0..5).rev().map(|loc| volume_loc_to_cell_loc( 8.0 - (loc as f64), -5.0 + (loc as f64), -1.0)).collect::<Vec<CartesianPoint2D>>() } else { vec![]} ),
        (if is_filled { (0..4).rev().map(|loc| volume_loc_to_cell_loc( 7.0 - (loc as f64), -5.0 + (loc as f64), -1.0)).collect::<Vec<CartesianPoint2D>>() } else { vec![]} ),

        get_outline_edge_points(OutlineEdge::BottomRight, -1.0),
        get_outline_edge_points(OutlineEdge::BottomLeft, -1.0),
        get_outline_edge_points(OutlineEdge::Bottom, -1.0),
    ].into_iter().flatten().collect::<Vec<CartesianPoint2D>>()
}

fn hall_cell(connections: &CellConnections) -> Html {
    let cubes = get_hex_grid_floor(true).iter()
        .map(|cube| unit_cube(cube, Some("darkcyan")))
        .collect::<Vec<Html>>();

    html! {
        <div class={classes!("dungeon-cell-hall")}>
            {
                cubes
            }
        </div>
    }
}

fn room_cell(connections: &CellConnections) -> Html {
    let cubes = get_hex_grid_floor(true).iter()
        .map(|cube| unit_cube(cube, Some("blue")))
        .collect::<Vec<Html>>();

    html! {
        <div class={classes!("dungeon-cell-hall")}>
            {
                cubes
            }
        </div>
    }
}

fn indeterminate_cell(connections: &CellConnections) -> Html {
    html! {
        <div class={classes!("dungeon-cell-indeterminate")}>
        </div>
    }
}

#[function_component]
pub fn DungeonCell(props: &DungeonCellProps) -> Html {
    let cell_types = &props.ui_props.possible_types;
    let num_cell_types = cell_types.len();
    let has_determinate_type = num_cell_types == 1;
    let is_valid_cell = cell_types.len() != 0;

    let mut outer_classes = classes!("dungeon-cell");
    if !is_valid_cell {
        outer_classes.push("dungeon-cell-invalid");
    }

    let maybe_cell_type: Option<DungeonCellType> = if is_valid_cell && has_determinate_type {
        Some(cell_types[0])
    } else {
        None
    };

    let possible_connections: Vec<bool> = cell_types.iter()
        .filter(|cell_type| match cell_type {
            DungeonCellType::None => false,
            _ => true,
        }).map(|cell_type| match cell_type {
            DungeonCellType::Hall(connections) => connections.to_vec(),
            DungeonCellType::Room(connections) => connections.to_vec(),
            _ => vec![],
        }).fold(CellConnections::none().to_vec(), |total_connections, other_conn|
            CellConnections::or(&total_connections, &other_conn)
        ).into_iter().collect();

    html! {
        <div class={outer_classes}>
        if props.ui_props.is_start_location { <div class={"dungeon-cell-start-location"}>{"S"}</div> }
        if props.ui_props.is_goal_location { <div class={"dungeon-cell-goal-location"}>{"G"}</div> }
        if props.ui_props.is_goal_entrance_location { <div class={"dungeon-cell-goal-location"}>{"GE"}</div> }
        if is_valid_cell {
            {
                match maybe_cell_type {
                    None => {
                        html! {
                            if num_cell_types == *NUM_POSSIBLE_DUNGEON_CELLS {
                                <div class={"dungeon-cell-untouched"} />
                            } else {
                                { indeterminate_cell(&CellConnections::from_vec(possible_connections)) }
                            }
                        }
                    },
                    Some(cell_type) => match cell_type {
                        DungeonCellType::None => html! { <div class={"dungeon-cell-none"} /> },
                        DungeonCellType::Hall(connections) => hall_cell(&connections),
                        DungeonCellType::Room(connections) => room_cell(&connections),
                    },
                }
            }
        }
        </div>
    }
}
