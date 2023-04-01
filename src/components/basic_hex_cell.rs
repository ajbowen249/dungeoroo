use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BasicHexCellProps {
    pub color: AttrValue,
}

#[function_component]
pub fn BasicHexCell(props: &BasicHexCellProps) -> Html {
    html! {
        <div class={classes!("basic-hex-cell")}>
            <div class={classes!("basic-hex-cell-inner")}>
                // Color is background-color for mid but the top and bottom use a border trick
                // Yes, top and bottom look flipped.
                <div style={format!("border-bottom-color: {}", props.color)} class={classes!("basic-hex-cell-top")}></div>
                <div style={format!("background: {}", props.color)} class={classes!("basic-hex-cell-mid")}></div>
                <div style={format!("border-top-color: {}", props.color)} class={classes!("basic-hex-cell-bottom")}></div>
            </div>
        </div>
    }
}
