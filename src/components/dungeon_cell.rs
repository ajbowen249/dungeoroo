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

fn hall_cell(connections: &CellConnections) -> Html {
    directional_cell(connections, "dungeon-cell-hall", "dungeon-cell-hall-connection")
}

fn room_cell(connections: &CellConnections) -> Html {
    directional_cell(connections, "dungeon-cell-room", "dungeon-cell-room-door")
}

fn indeterminate_cell(connections: &CellConnections) -> Html {
    directional_cell(connections, "dungeon-cell-indeterminate", "dungeon-cell-indeterminate-directions")
}

fn directional_cell(connections: &CellConnections, main_class: &str, dir_class: &str) -> Html {
    html! {
        <div class={format!("{}", main_class)}>
        if connections.top_left {
            <div class={format!("{} dungeon-cell-rotate-top-left", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
        if connections.top_right {
            <div class={format!("{} dungeon-cell-rotate-top-right", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
        if connections.right {
            <div class={format!("{} dungeon-cell-rotate-right", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
        if connections.bottom_right {
            <div class={format!("{} dungeon-cell-rotate-bottom-right", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
        if connections.bottom_left {
            <div class={format!("{} dungeon-cell-rotate-bottom-left", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
        if connections.left {
            <div class={format!("{} dungeon-cell-rotate-left", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
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
        if num_cell_types > 1 { <div class={"dungeon-cell-remaining-types"}>{num_cell_types}</div> }
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
