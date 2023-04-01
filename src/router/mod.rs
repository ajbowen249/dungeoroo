use yew_router::prelude::*;
use yew::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    WFCDungeonSandbox,
    #[at("/terrain")]
    WFCSandbox,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn router(routes: Route) -> Html {
    match routes {
        Route::WFCDungeonSandbox => html! { <crate::views::wfc_dungeon_sandbox::WFCDungeonSandbox /> },
        Route::WFCSandbox => html! { <crate::views::wfc_sandbox::WFCSandbox /> },
        Route::NotFound => html! { <crate::views::not_found::NotFound /> },
    }
}
