use leptos::prelude::*;

#[component]
pub(crate) fn UndoRedoDropdown(
    dropdown: ReadSignal<Option<(i32, i32, Vec<crate::models::Action>, bool)>>,
    close_dropdown: Callback<()>,
    on_dropdown_action: Callback<(uuid::Uuid, bool)>,
) -> impl IntoView {
    view! {
        {move || if let Some((x, y, actions, is_redo)) = dropdown.get() {
            let title = if is_redo { "Redo" } else { "Undo" };
            view! {
                <div class="nav-dropdown-overlay" on:click=move |_| close_dropdown.run(())></div>
                <div class="nav-dropdown-menu" style={format!("left:{}px;top:{}px;", x, y)}>
                    <div class="nav-dropdown-title">{title}" actions"</div>
                    {if actions.is_empty() {
                        view! { <div class="nav-dropdown-empty">"No actions"</div> }.into_any()
                    } else {
                        actions.into_iter().map(|action| {
                            let action_id = action.id;
                            let desc = format!("{} {}", action.action_type_badge(), action.description);
                            let is_redo = is_redo;
                            view! {
                                <div class="nav-dropdown-item"
                                    on:click=move |_| on_dropdown_action.run((action_id, is_redo))>
                                    {desc}
                                </div>
                            }
                        }).collect::<Vec<_>>().into_any()
                    }}
                </div>
            }.into_any()
        } else { ().into_any() }}
    }
}
