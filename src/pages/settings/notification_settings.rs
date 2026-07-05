use crate::stores::use_notification_store;
use leptos::prelude::*;

#[component]
pub(crate) fn NotificationSettings() -> impl IntoView {
    let notification_store = use_notification_store();

    view! {
        <div class="data-card">
            <div class="card-header">
                <span class="card-title">"Notifications"</span>
            </div>
            <div class="settings-list">
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Email Notifications"</div>
                        <div class="list-item-desc">"Receive security alerts and summaries by email."</div>
                    </div>
                    <div class="list-item-right">
                        <label class="settings-toggle">
                            <input
                                type="checkbox"
                                aria-label="Toggle email notifications"
                                prop:checked={move || notification_store.get().email_notifications}
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    notification_store.update(|s| s.set_email_notifications(checked));
                                }
                            />
                            <span class="settings-toggle-slider"></span>
                        </label>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Push Notifications"</div>
                        <div class="list-item-desc">"Show in-app notifications and alerts."</div>
                    </div>
                    <div class="list-item-right">
                        <label class="settings-toggle">
                            <input
                                type="checkbox"
                                aria-label="Toggle push notifications"
                                prop:checked={move || notification_store.get().push_notifications}
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    notification_store.update(|s| s.set_push_notifications(checked));
                                }
                            />
                            <span class="settings-toggle-slider"></span>
                        </label>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Sound Effects"</div>
                        <div class="list-item-desc">"Play a sound for alerts and actions."</div>
                    </div>
                    <div class="list-item-right">
                        <label class="settings-toggle">
                            <input
                                type="checkbox"
                                aria-label="Toggle sound effects"
                                prop:checked={move || notification_store.get().sound_enabled}
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    notification_store.update(|s| s.set_sound_enabled(checked));
                                }
                            />
                            <span class="settings-toggle-slider"></span>
                        </label>
                    </div>
                </div>
            </div>
        </div>
    }
}
