use yew::prelude::*;
use gloo_timers::callback::Timeout;
use crate::wfc::*;
use crate::components::dungeon_cell_preview::*;
use crate::components::dungeon_cell::*;
use crate::generation_fields::dungeon::*;
use web_sys::{EventTarget, HtmlInputElement};
use wasm_bindgen::JsCast;

pub enum Msg {
    None,
    Reset,
    Step,
    StepComplete,
    GenerateInstant,
    SetPaint(DungeonCellType),
    SetCell(GridLocation),
    SeedInputChanged(u64),
    ToggleIsRenderedMode,
}


#[derive(PartialEq, Properties)]
pub struct WFCSandboxProps {
}

pub struct WFCDungeonSandbox {
    generator: DungeonGenerator,
    pub selected_set_cell_type: DungeonCellType,
    pub seed_string: String,
    pub is_rendered_mode: bool,
}

fn new_generator() -> DungeonGenerator {
    DungeonGenerator::new(15, 15)
}

impl Component for WFCDungeonSandbox {
    type Message = Msg;
    type Properties = WFCSandboxProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let mut data = WFCDungeonSandbox {
            generator: new_generator(),
            selected_set_cell_type: DungeonCellType::None,
            seed_string: String::from(""),
            is_rendered_mode: true,
        };

        data.seed_string = data.generator.seed.to_string();

        data.generator.generate();

        data
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let step = ctx.link().callback(|_| Msg::Step);
        let step_complete = ctx.link().callback(|_| Msg::StepComplete);
        let generate_instant = ctx.link().callback(|_| Msg::GenerateInstant);
        let select_set_none = ctx.link().callback(|_| Msg::SetPaint(DungeonCellType::None));
        let select_set_hall = ctx.link().callback(|_| Msg::SetPaint(DungeonCellType::Hall(CellConnections::all())));
        let select_set_room = ctx.link().callback(|_| Msg::SetPaint(DungeonCellType::Room(CellConnections::all())));
        let toggle_rendered = ctx.link().callback(|_| Msg::ToggleIsRenderedMode);
        let reset = ctx.link().callback(|_| Msg::Reset);
        let seed_changed = {
            let on_seed_changed = ctx.link().callback(|val: u64| Msg::SeedInputChanged(val));
            Callback::from(move |e: InputEvent| {
                let target: Option<EventTarget> = e.target();
                let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

                if let Some(input) = input {
                    on_seed_changed.emit(input.value().parse::<u64>().unwrap());
                }
            })
        };

        let grid = &self.generator.wfc.get_grid().grid;

        let cell_type_name = match self.selected_set_cell_type {
            DungeonCellType::None => "None",
            DungeonCellType::Hall(_) => "Hall",
            DungeonCellType::Room(_) => "Room",
        };

        let queued_cell_locations = self.generator.wfc.get_queue();
        let can_do_more_work = self.generator.can_do_more_work();
        let mut p_row_index = 0;

        html! {
            <div>
                <div class={classes!("config-panel")}>
                    <div>{format!("Selected: {}", cell_type_name)}</div>
                    <button onclick={select_set_none}>{"None"}</button>
                    <button onclick={select_set_hall}>{"Hall"}</button>
                    <button onclick={select_set_room}>{"Room"}</button>
                    <button disabled={!can_do_more_work} onclick={step}>{"Step"}</button>
                    <button disabled={!can_do_more_work} onclick={step_complete}>{"Step To End"}</button>
                    <button disabled={!can_do_more_work} onclick={generate_instant}>{"Generate (Instant)"}</button>
                    <button disabled={can_do_more_work} onclick={reset}>{"Reset"}</button><br />
                    <div>{format!("State: {}", self.generator.debug_state())}</div>
                    <input type={"number"} min={0} value={self.seed_string.clone()} oninput={seed_changed} />
                    <label for={"is_rendered_checkbox"}>{"Rendered"}</label>
                    <input type={"checkbox"} id={"is_rendered_checkbox"} checked={self.is_rendered_mode} onclick={toggle_rendered} />
                </div>
                if !self.is_rendered_mode {
                    <div class={classes!("wfc-ds-preview-grid")}>
                    {
                        grid.iter().map(|row| {
                            // "odd" is even-numbered index since they start at 0
                            let is_odd = p_row_index % 2 == 0;

                            let row_class = classes!(
                                "wfc-ds-preview-grid-row",
                                if is_odd { "wfc-ds-preview-grid-row-odd" } else { "wfc-ds-preview-grid-row-even" }
                            );

                            let mut col_index = 0;

                            let html = html! {
                                <div class={row_class}>
                                {
                                    row.iter().map(|cell| {
                                        let cell_is_queued = queued_cell_locations.iter().any(|loc| {
                                            loc.row == p_row_index && loc.col == col_index
                                        });

                                        let location = GridLocation::new(p_row_index, col_index);

                                        let set_cell = ctx.link().callback(move |_| Msg::SetCell(location));

                                        let ihtml = html! {
                                            <div class={classes!("wfc-ds-preview-grid-cell-container")}>
                                                <div class={classes!("wfc-ds-preview-grid-cell-container-outer")}>
                                                    <div onclick={set_cell} class={classes!("wfc-ds-preview-grid-cell-container-inner")}>
                                                            <DungeonCellPreview ui_props={DungeonCellPreviewUIProps {
                                                                possible_types: cell.borrow().possible_types.clone(),
                                                                is_start_location: location == self.generator.start_location,
                                                                is_goal_location: location == self.generator.goal_location,
                                                                is_goal_entrance_location: location == self.generator.goal_entrance_location,
                                                            }} />
                                                    </div>
                                                    if cell_is_queued {
                                                        <div class={classes!("wfc-ds-preview-grid-cell-container-flag")}>
                                                            { "Q" }
                                                        </div>
                                                    }
                                                </div>
                                            </div>
                                        };

                                        col_index += 1;

                                        ihtml
                                    }).collect::<Html>()
                                }
                                </div>
                            };

                            p_row_index += 1;

                            html
                        }).collect::<Html>()
                    }
                    </div>
                } else {
                    // I'm iterating the grid 90 degrees off, since the rendered hexagons need to go the other way around, and I don't want to refactor
                    // everything for flat-bottom hexagons when I just got it working.

                    // The leftmost cell of the last row is the upper left column. The last row going right is the first column going down.

                    // This still isn't quite 100% correct. Right-to-left is correct, but we need top-to-bottom as well
                    <div class={classes!("wfc-ds-rendered-grid")}>
                    {
                        (0..self.generator.rows).map(|row_index| {
                            let is_odd = row_index % 2 == 0;
                            html! {
                                <div class={classes!(
                                    "wfc-ds-rendered-grid-column",
                                    if is_odd { "" } else { "wfc-ds-rendered-grid-column-even" }
                                    )}>
                                {
                                    (0..self.generator.cols).map(|col_index| {
                                        let location = GridLocation::new(row_index as i64, col_index as i64);
                                        let cell = self.generator.wfc.get_grid().get_cell(&location).unwrap();

                                        html! {
                                            <div class={classes!("wfc-dungeon-sandbox-rendered-cell-container")}>
                                                <DungeonCell ui_props={DungeonCellUIProps {
                                                    possible_types: cell.borrow().possible_types.clone(),
                                                    is_start_location: location == self.generator.start_location,
                                                    is_goal_location: location == self.generator.goal_location,
                                                    is_goal_entrance_location: location == self.generator.goal_entrance_location,
                                                }} />
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                                </div>
                            }
                        }).collect::<Html>()
                    }
                    </div>
                }
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::None => {},
            Msg::Reset => {
                self.generator = new_generator();
                self.generator.seed = self.seed_string.parse::<u64>().unwrap();
            },
            Msg::Step => self.generator.step(),
            Msg::StepComplete => {
                let requeue = ctx.link().callback(|_: ()| Msg::StepComplete);

                for _ in 0..if self.generator.state == DungeonGeneratorState::Fill { 20 } else { 1 } {
                    self.generator.step();
                }

                if self.generator.can_do_more_work() {
                    let timer = Timeout::new(1, move || {
                        requeue.emit(());
                    });
                    timer.forget();
                }
            },
            Msg::GenerateInstant => self.generator.generate(),
            Msg::SetPaint(cell_type) => self.selected_set_cell_type = cell_type,
            Msg::SetCell(location) => self.generator.wfc.apply_types(vec![(location, vec![self.selected_set_cell_type])]),
            Msg::SeedInputChanged(seed) => {
                self.generator.seed = seed;
                self.seed_string = seed.to_string();
            },
            Msg::ToggleIsRenderedMode => {
                self.is_rendered_mode = !self.is_rendered_mode;
            },
        };

        true
    }
}
