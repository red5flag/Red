use crate::components::tabs::use_tab_edit_mode;
use leptos::prelude::*;

/// A text label that becomes an inline input when the current tab is in edit mode.
/// `value` is the current text; `on_commit` is called when the user finishes editing.
#[component]
pub fn EditableText(
    value: Signal<String>,
    #[prop(into)] on_commit: Callback<String>,
) -> impl IntoView {
    let edit_mode = use_tab_edit_mode();
    let (draft, set_draft) = signal(String::new());

    view! {
        {move || {
            if edit_mode.get() {
                let input_view = view! {
                    <input
                        class="editable-text-input"
                        type="text"
                        prop:value=move || value.get()
                        on:input=move |ev| set_draft.set(event_target_value(&ev))
                        on:blur=move |_| {
                            let new_value = draft.get();
                            on_commit.run(new_value);
                        }
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                let new_value = draft.get();
                                on_commit.run(new_value);
                            }
                        }
                    />
                };
                input_view.into_any()
            } else {
                view! { <span class="editable-text">{value.get()}</span> }.into_any()
            }
        }}
    }
}
