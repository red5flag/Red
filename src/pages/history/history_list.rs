use crate::pages::history::HistoryCard;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub(crate) fn HistoryList(
    #[prop(into)] actions: Signal<Vec<crate::models::Action>>,
    current_user_id: Uuid,
    on_history_undo: Callback<Uuid>,
) -> impl IntoView {
    view! {
        {move || {
            let actions_vec = actions.get();
            if actions_vec.is_empty() {
                view! {
                    <div class="history-empty">
                        <p>"No matching actions"</p>
                        <div class="history-empty-icon">"📜"</div>
                    </div>
                }
                    .into_any()
            } else {
                view! {
                    <div class="timeline">
                        <For
                            each=move || actions_vec.clone()
                            key=|action| action.id
                            children=move |action| {
                                view! {
                                    <HistoryCard
                                        action={action}
                                        current_user_id={current_user_id}
                                        on_undo={Callback::new(move |id| on_history_undo.run(id))}
                                    />
                                }
                            }
                        />
                    </div>
                }
                    .into_any()
            }
        }}
    }
}
