use yew::prelude::*;
use yew::html::*;
use crate::generation_fields::dungeon::*;

#[derive(Debug, PartialEq)]
pub struct DungeonCellPreviewUIProps {
    pub possible_types: Vec<DungeonCellType>,
    pub is_start_location: bool,
    pub is_goal_location: bool,
    pub is_goal_entrance_location: bool,
}

impl IntoPropValue<DungeonCellPreviewProps> for DungeonCellPreviewUIProps {
    fn into_prop_value(self) -> DungeonCellPreviewProps {
        DungeonCellPreviewProps {
            ui_props: self,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct DungeonCellPreviewProps {
    pub ui_props: DungeonCellPreviewUIProps,
}

fn hall_cell(connections: &CellConnections) -> Html {
    directional_cell(connections, "dungeon-cell-preview-hall", "dungeon-cell-preview-hall-connection")
}

fn room_cell(connections: &CellConnections) -> Html {
    directional_cell(connections, "dungeon-cell-preview-room", "dungeon-cell-preview-room-door")
}

fn indeterminate_cell(connections: &CellConnections) -> Html {
    directional_cell(connections, "dungeon-cell-preview-indeterminate", "dungeon-cell-preview-indeterminate-directions")
}

fn directional_cell(connections: &CellConnections, main_class: &str, dir_class: &str) -> Html {
    html! {
        <div class={format!("{}", main_class)}>
        if connections.top_left {
            <div class={format!("{} dungeon-cell-preview-rotate-top-left", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
        if connections.top_right {
            <div class={format!("{} dungeon-cell-preview-rotate-top-right", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
        if connections.right {
            <div class={format!("{} dungeon-cell-preview-rotate-right", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
        if connections.bottom_right {
            <div class={format!("{} dungeon-cell-preview-rotate-bottom-right", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
        if connections.bottom_left {
            <div class={format!("{} dungeon-cell-preview-rotate-bottom-left", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
        if connections.left {
            <div class={format!("{} dungeon-cell-preview-rotate-left", dir_class)}>{crate::util::HTML_NBSP}</div>
        }
        </div>
    }
}

#[function_component]
pub fn DungeonCellPreview(props: &DungeonCellPreviewProps) -> Html {
    let cell_types = &props.ui_props.possible_types;
    let num_cell_types = cell_types.len();
    let has_determinate_type = num_cell_types == 1;
    let is_valid_cell = cell_types.len() != 0;

    let mut outer_classes = classes!("dungeon-cell-preview");
    if !is_valid_cell {
        outer_classes.push("dungeon-cell-preview-invalid");
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
        if num_cell_types > 1 { <div class={"dungeon-cell-preview-remaining-types"}>{num_cell_types}</div> }
        if props.ui_props.is_start_location { <div class={"dungeon-cell-preview-start-location"}>{"S"}</div> }
        if props.ui_props.is_goal_location { <div class={"dungeon-cell-preview-goal-location"}>{"G"}</div> }
        if props.ui_props.is_goal_entrance_location { <div class={"dungeon-cell-preview-goal-location"}>{"GE"}</div> }
        if is_valid_cell {
            {
                match maybe_cell_type {
                    None => {
                        html! {
                            if num_cell_types == *NUM_POSSIBLE_DUNGEON_CELLS {
                                <div class={"dungeon-cell-preview-untouched"} />
                            } else {
                                { indeterminate_cell(&CellConnections::from_vec(possible_connections)) }
                            }
                        }
                    },
                    Some(cell_type) => match cell_type {
                        DungeonCellType::None => html! { <div class={"dungeon-cell-preview-none"} /> },
                        DungeonCellType::Hall(connections) => hall_cell(&connections),
                        DungeonCellType::Room(connections) => room_cell(&connections),
                    },
                }
            }
        }
        </div>
    }
}
