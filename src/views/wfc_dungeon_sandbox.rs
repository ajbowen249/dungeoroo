use crate::components::dungeon_cell::*;
use crate::components::dungeon_cell_preview::*;
use crate::generation_fields::dungeon::*;
use crate::wfc::*;
use gloo_console::log;
use gloo_timers::callback::Timeout;
use serde::{Deserialize, Serialize};
use std::f64;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, EventTarget, HtmlInputElement};
use yew::prelude::*;

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
    RenderToCanvas,
    AddRenderWindowOffset(Point2D),
}

type Point2D = (f64, f64);

#[derive(PartialEq, Properties)]
pub struct WFCSandboxProps {}

pub struct WFCDungeonSandbox {
    generator: DungeonGenerator,
    pub selected_set_cell_type: DungeonCellType,
    pub seed_string: String,
    pub is_rendered_mode: bool,
    pub rendered_window_offset: Point2D,
}

fn new_generator() -> DungeonGenerator {
    DungeonGenerator::new(15, 15)
}

const CANVAS_WIDTH: f64 = 1280.0;
const CANVAS_HEIGHT: f64 = 394.0;

impl Component for WFCDungeonSandbox {
    type Message = Msg;
    type Properties = WFCSandboxProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let mut data = WFCDungeonSandbox {
            generator: new_generator(),
            selected_set_cell_type: DungeonCellType::None,
            seed_string: String::from(""),
            is_rendered_mode: true,
            rendered_window_offset: (0.0, 0.0),
        };

        data.seed_string = data.generator.seed.to_string();

        data.generator.generate();

        data
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let step = ctx.link().callback(|_| Msg::Step);
        let step_complete = ctx.link().callback(|_| Msg::StepComplete);
        let generate_instant = ctx.link().callback(|_| Msg::GenerateInstant);
        let select_set_none = ctx
            .link()
            .callback(|_| Msg::SetPaint(DungeonCellType::None));
        let select_set_hall = ctx
            .link()
            .callback(|_| Msg::SetPaint(DungeonCellType::Hall(CellConnections::all())));
        let select_set_room = ctx
            .link()
            .callback(|_| Msg::SetPaint(DungeonCellType::Room(CellConnections::all())));
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

        let shift_left = ctx.link().callback(|_| Msg::AddRenderWindowOffset((-10.0, 0.0)));
        let shift_right = ctx.link().callback(|_| Msg::AddRenderWindowOffset((10.0, 0.0)));
        let shift_up = ctx.link().callback(|_| Msg::AddRenderWindowOffset((0.0, -10.0)));
        let shift_down = ctx.link().callback(|_| Msg::AddRenderWindowOffset((0.0, 10.0)));

        let grid = &self.generator.wfc.get_grid().grid;

        let cell_type_name = match self.selected_set_cell_type {
            DungeonCellType::None => "None",
            DungeonCellType::Hall(_) => "Hall",
            DungeonCellType::Room(_) => "Room",
        };

        let queued_cell_locations = self.generator.wfc.get_queue();
        let can_do_more_work = self.generator.can_do_more_work();
        let mut p_row_index = 0;

        if self.is_rendered_mode {
            let render_to_canvas = ctx.link().callback(|_: ()| Msg::RenderToCanvas);
            let timer = Timeout::new(1, move || {
                render_to_canvas.emit(());
            });
            timer.forget();
        }

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
                    if self.is_rendered_mode {
                        <div>
                            <button onclick={shift_left}>{"<"}</button>
                            <button onclick={shift_up}>{"^"}</button>
                            <button onclick={shift_right}>{">"}</button>
                            <button onclick={shift_down}>{"v"}</button>
                        </div>
                    }
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
                    <div class={classes!("wfc-ds-canvas-container")}>
                        <canvas id={"canvas"} width={format!("{}px", CANVAS_WIDTH)} height={format!("{}px", CANVAS_HEIGHT)} class={classes!("wfc-ds-canvas")} />
                    </div>
                }
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::None => {}
            Msg::Reset => {
                self.generator = new_generator();
                self.generator.seed = self.seed_string.parse::<u64>().unwrap();
            }
            Msg::Step => self.generator.step(),
            Msg::StepComplete => {
                let requeue = ctx.link().callback(|_: ()| Msg::StepComplete);

                for _ in 0..if self.generator.state == DungeonGeneratorState::Fill {
                    20
                } else {
                    1
                } {
                    self.generator.step();
                }

                if self.generator.can_do_more_work() {
                    let timer = Timeout::new(1, move || {
                        requeue.emit(());
                    });
                    timer.forget();
                }
            }
            Msg::GenerateInstant => self.generator.generate(),
            Msg::SetPaint(cell_type) => self.selected_set_cell_type = cell_type,
            Msg::SetCell(location) => self
                .generator
                .wfc
                .apply_types(vec![(location, vec![self.selected_set_cell_type])]),
            Msg::SeedInputChanged(seed) => {
                self.generator.seed = seed;
                self.seed_string = seed.to_string();
            }
            Msg::ToggleIsRenderedMode => {
                self.is_rendered_mode = !self.is_rendered_mode;
            }
            Msg::RenderToCanvas => {
                log!("render request");
                let document = web_sys::window().unwrap().document().unwrap();
                let canvas = document.get_element_by_id("canvas").unwrap();
                let canvas: web_sys::HtmlCanvasElement = canvas
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .map_err(|_| ())
                    .unwrap();

                let context = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .unwrap();

                self.clear_canvas(&context);

                self.draw_cube(&context, &(0.0, 0.0));
                self.draw_cube(&context, &(64.0, 32.0));
                // We don't want the view to re-render, we just drew to the canvas
                return false;
            },
            Msg::AddRenderWindowOffset(offset) => {
                self.rendered_window_offset = (self.rendered_window_offset.0 + offset.0, self.rendered_window_offset.1 + offset.1);
                ctx.link().callback(|_: ()| Msg::RenderToCanvas).emit(());
                // We don't want the view to re-render, we want the canvas to re-render.
                return false;
            },
        };

        true
    }
}

impl WFCDungeonSandbox {
    fn point_to_canvas_point(&self, point: &Point2D) -> Point2D {
        (point.0 - self.rendered_window_offset.0, point.1 - self.rendered_window_offset.1)
    }

    fn clear_canvas(&self, context: &CanvasRenderingContext2d) {
        context.clear_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);
    }

    fn draw_cube(&self, context: &CanvasRenderingContext2d, upper_left: &Point2D) {
        context.set_stroke_style(&JsValue::from_str("#000000ff"));
        context.set_fill_style(&JsValue::from_str("#565656ff"));
        let upper_left = self.point_to_canvas_point(upper_left);

        // Top
        context.set_fill_style(&JsValue::from_str("#565656ff"));
        context.begin_path();
        context.move_to(upper_left.0, upper_left.1 + 31.0);
        context.line_to(upper_left.0 + 63.0, upper_left.1);
        context.line_to(upper_left.0 + 64.0, upper_left.1);
        context.line_to(upper_left.0 + 127.0, upper_left.1 + 31.0);
        context.line_to(upper_left.0 + 127.0, upper_left.1 + 32.0);
        context.line_to(upper_left.0 + 64.0, upper_left.1 + 63.0);
        context.line_to(upper_left.0 + 63.0, upper_left.1 + 63.0);
        context.line_to(upper_left.0 + 0.0, upper_left.1 + 32.0);
        context.close_path();
        context.fill();
        context.stroke();

        // Left
        context.set_fill_style(&JsValue::from_str("#424242ff"));
        context.begin_path();
        context.move_to(upper_left.0, upper_left.1 + 32.0);
        context.line_to(upper_left.0 + 63.0, upper_left.1 + 63.0);
        context.line_to(upper_left.0 + 63.0, upper_left.1 + 131.0);
        context.line_to(upper_left.0 + 0.0, upper_left.1 + 100.0);
        context.close_path();
        context.fill();
        context.stroke();

        // Right
        context.set_fill_style(&JsValue::from_str("#888888ff"));
        context.begin_path();
        context.move_to(upper_left.0 + 127.0, upper_left.1 + 31.0);
        context.line_to(upper_left.0 + 127.0, upper_left.1 + 100.0);
        context.line_to(upper_left.0 + 64.0, upper_left.1 + 131.0);
        context.line_to(upper_left.0 + 64.0, upper_left.1 + 63.0);
        context.close_path();
        context.fill();
        context.stroke();

    }

    fn draw_hex_cell_floor() {
        
    }
}
