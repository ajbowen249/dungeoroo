use yew::prelude::*;
use gloo_timers::callback::Timeout;
use crate::wfc::*;
use crate::components::basic_hex_cell::*;

pub enum Msg {
    IterateQueue,
    IterateQueueComplete,
    IterateQueueCompleteInstant,
    SetPaint(CellType),
    PaintCell(GridLocation),
}

#[derive(Debug, Eq, Copy, Clone, PartialEq)]
pub enum CellType {
    Beach,
    Sea,
    Land,
}

#[derive(PartialEq, Properties)]
pub struct WFCSandboxProps {
}

pub struct WFCSandbox {
    wfc: WaveFunctionCollapseContext<CellType>,
    pub selected_paint_color: CellType,
}

impl Component for WFCSandbox {
    type Message = Msg;
    type Properties = WFCSandboxProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let all_types = vec![CellType::Beach, CellType::Land, CellType::Sea];

        WFCSandbox {
            wfc: WaveFunctionCollapseContext::<CellType>::new(40, 40, &all_types),
            selected_paint_color: CellType::Land,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let iterate_queue = ctx.link().callback(|_| Msg::IterateQueue);
        let iterate_queue_complete = ctx.link().callback(|_| Msg::IterateQueueComplete);
        let iterate_queue_complete_instant = ctx.link().callback(|_| Msg::IterateQueueCompleteInstant);
        let select_sea = ctx.link().callback(|_| Msg::SetPaint(CellType::Sea));
        let select_land = ctx.link().callback(|_| Msg::SetPaint(CellType::Land));

        let grid = &self.wfc.get_grid().grid;

        let paint_color = match self.selected_paint_color {
            CellType::Beach => "tan",
            CellType::Sea => "blue",
            CellType::Land => "green",
        };

        let queued_cell_locations = self.wfc.get_queue();
        let queue_is_empty = queued_cell_locations.is_empty();
        let mut row_index = 0;

        html! {
            <div>
                <div class={classes!("config-panel")}>
                    <div style={format!("background-color: {}", paint_color)}>{crate::util::HTML_NBSP}</div>
                    <button onclick={select_sea}>{"Sea"}</button>
                    <button onclick={select_land}>{"Land"}</button>
                    <button disabled={queue_is_empty} onclick={iterate_queue}>{"Iterate Queue"}</button>
                    <button disabled={queue_is_empty} onclick={iterate_queue_complete}>{"Iterate To End"}</button>
                    <button disabled={queue_is_empty} onclick={iterate_queue_complete_instant}>{"Iterate To End (Instant)"}</button>
                </div>
                <div class={classes!("wfc-sandbox-hex-grid")}>
                {
                    grid.iter().map(|row| {
                        // "odd" is even-numbered index since they start at 0
                        let is_odd = row_index % 2 == 0;

                        let row_class = classes!(
                            "wfc-sandbox-grid-row",
                            if is_odd { "wfc-sandbox-grid-row-odd" } else { "wfc-sandbox-grid-row-even" }
                        );

                        let mut col_index = 0;

                        let html = html! {
                            <div class={row_class}>
                            {
                                row.iter().map(|cell| {
                                    let mut color = "purple";
                                    let cell = cell.borrow();
                                    if cell.possible_types.len() == 0 {
                                        color = "black";
                                    } else if cell.possible_types.len() == 1 {
                                        color = match cell.possible_types[0] {
                                            CellType::Beach => "tan",
                                            CellType::Sea => "blue",
                                            CellType::Land => "green",
                                        }
                                    }

                                    let cell_is_queued = queued_cell_locations.iter().any(|loc| {
                                        loc.row == row_index && loc.col == col_index
                                    });


                                    let paint_cell = ctx.link().callback(move |_| Msg::PaintCell(GridLocation::new(row_index, col_index)));

                                    let ihtml = html! {
                                        <div class={classes!("wfc-sandbox-grid-cell-container")}>
                                            <div class={classes!("wfc-sandbox-grid-cell-container-outer")}>
                                                <div onclick={paint_cell} class={classes!("wfc-sandbox-grid-cell-container-inner")}>
                                                    <BasicHexCell color={color} />
                                                </div>
                                                if cell_is_queued {
                                                    <div class={classes!("wfc-sandbox-grid-cell-container-flag")}>
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

                        row_index += 1;

                        html
                    }).collect::<Html>()
                }
                </div>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let rules: WafeFunctionReducer<CellType> = |loc, cell, grid| {
            let mut cell = cell.borrow_mut();
            let mut changed = false;

            // We're attempting go to a basic land-sea-beach thing here.
            // Land can only touch land and beach
            // Sea can only touch sea and beach
            // Beach can touch anything, unless surrounded by land (surrounded by sea can be a small island, sand bar, etc.)

            let cell2 = cell.clone();
            let cell_can_be = move |cell_type: CellType| cell2.possible_types.contains(&cell_type);

            let neighbors: Vec<MaybeCell<CellType>> = loc.get_neighbors().iter().map(|location| grid.get_cell(location)).collect();

            let cell_has_neighbor_of_type = |cell_type: CellType| {
                neighbors.iter().any(|neighbor| match neighbor { None => false, Some(neighbor) => {
                    let neighbor = neighbor.borrow();
                    neighbor.possible_types.len() == 1 && neighbor.possible_types.contains(&cell_type)
                }} )
            };

            let all_neighbors_are = |cell_type: CellType| {
                neighbors.iter().all(|neighbor| match neighbor { None => false, Some(neighbor) => {
                    let neighbor = neighbor.borrow();
                    neighbor.possible_types.len() == 1 && neighbor.possible_types.contains(&cell_type)
                }} )
            };

            if cell_can_be(CellType::Land) && cell_has_neighbor_of_type(CellType::Sea) {
                cell.possible_types = cell.possible_types.clone().into_iter().filter(|t| *t != CellType::Land).collect();
                changed = true;
            }

            if cell_can_be(CellType::Sea) && cell_has_neighbor_of_type(CellType::Land) {
                cell.possible_types = cell.possible_types.clone().into_iter().filter(|t| *t != CellType::Sea).collect();
                changed = true;
            }

            if cell_can_be(CellType::Beach) && all_neighbors_are(CellType::Land) {
                cell.possible_types = vec![CellType::Land];
                changed = true;
            }

            // This might not technically belong here...re-evaluate for non-toy application

            // We generally want land tiles to generate more land and sea tiles to generate more sea.
            // If we were just narrowed down, possibly just keep building on that.
            if changed && cell.possible_types.len() == 2 {
                if cell_can_be(CellType::Land) && cell_has_neighbor_of_type(CellType::Land) {
                    cell.possible_types = vec![CellType::Land];
                } else if cell_can_be(CellType::Sea) && cell_has_neighbor_of_type(CellType::Sea) {
                    cell.possible_types = vec![CellType::Sea];
                }
            }

            changed
        };

        match msg {
            Msg::IterateQueue => self.wfc.iterate_queue(rules),
            Msg::IterateQueueComplete => {
                let requeue = ctx.link().callback(|_: ()| Msg::IterateQueueComplete);

                self.wfc.iterate_queue(rules);
                if !self.wfc.get_queue().is_empty() {
                    let timer = Timeout::new(1, move || {
                        requeue.emit(());
                    });
                    timer.forget();
                }
            },
            Msg::IterateQueueCompleteInstant => self.wfc.iterate_queue_complete(rules),
            Msg::SetPaint(cell_type) => self.selected_paint_color = cell_type,
            Msg::PaintCell(location) => self.wfc.apply_types(vec![(location, vec![self.selected_paint_color])]),
        };

        true
    }
}
