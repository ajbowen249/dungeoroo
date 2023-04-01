use wasm_bindgen::JsCast;
use yew::prelude::*;
use web_sys::{EventTarget, HtmlInputElement};

/**
 * Generates a callback that copies the event value to the given state handle
 *
 * Yoinked from the yew docs
 * https://yew.rs/docs/next/concepts/html/events
 */
pub fn bind_on_input(state: &UseStateHandle<String>) -> Callback<InputEvent> {
    return {
        let state = state.clone();

        Callback::from(move |e: InputEvent| {
            // When events are created the target is undefined, it's only
            // when dispatched does the target get added.
            let target: Option<EventTarget> = e.target();
            // Events can bubble so this listener might catch events from child
            // elements which are not of type HtmlInputElement
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            if let Some(input) = input {
                state.set(input.value());
            }
        })
    };
}

pub fn bind_on_input_with_effect(state: &UseStateHandle<String>, callback: yew::Callback<String>) -> Callback<InputEvent> {
    return {
        let state = state.clone();

        Callback::from(move |e: InputEvent| {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            if let Some(input) = input {
                state.set(input.value());
                callback.emit(input.value());
            }
        })
    };
}

pub const HTML_NBSP: &str = "\u{00a0}";
