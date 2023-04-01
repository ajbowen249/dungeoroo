use std::rc::Rc;
use yew::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GameContext {
    pub placeholder: String,
}

pub const DEFAULT_PLACEHOLDER: &str = "placeholder";

pub enum ContextAction {
    SetPlaceholder(String),
}

impl Reducible for GameContext {
    type Action = ContextAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut state = (*self).clone();

        match action {
            ContextAction::SetPlaceholder(value) => {
                state.placeholder = value;
            },
        }

        state.into()
    }
}

pub type GameContextHandle = UseReducerHandle<GameContext>;

#[derive(Properties, Debug, PartialEq)]
pub struct GameContextProviderProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component]
pub fn GameContextProvider(props: &GameContextProviderProps) -> Html {
    let mut context = GameContext {
        placeholder: String::from(DEFAULT_PLACEHOLDER),
    };

    let context = use_reducer(|| context);

    html! {
        <ContextProvider<GameContextHandle> context={context}>
            {props.children.clone()}
        </ContextProvider<GameContextHandle>>
    }
}
