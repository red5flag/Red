use leptos::prelude::*;

#[component]
pub(crate) fn PrintRecordButton() -> impl IntoView {
    let on_print = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.print();
        }
    };
    view! {
        <button class="tx-print-btn" on:click=on_print>"🖨 Print Record"</button>
    }
}
