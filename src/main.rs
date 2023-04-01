use yew_router::prelude::*;
use yew::prelude::*;

mod components;
mod context;
mod generation_fields;
mod router;
mod util;
mod views;
mod wfc;

#[function_component]
fn App() -> Html {
    html! {
        <context::GameContextProvider>
            <BrowserRouter>
                <Switch<router::Route> render={router::router} />
            </BrowserRouter>
        </context::GameContextProvider>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
