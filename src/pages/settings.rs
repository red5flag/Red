use crate::stores::use_app_store;
use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let _app_store = use_app_store();

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"Settings"</h1>
                <p>"Customize your experience"</p>
            </div>
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Appearance"</span>
                </div>
                <div class="settings-list">
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">"Theme"</div>
                        </div>
                        <div class="list-item-right">
                            <select class="form-select">
                                <option value="dark">"Dark"</option>
                                <option value="light">"Light"</option>
                                <option value="high-contrast">"High Contrast"</option>
                            </select>
                        </div>
                    </div>
                </div>
            </div>
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Notifications"</span>
                </div>
                <div class="settings-list">
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">"Email Notifications"</div>
                        </div>
                        <div class="list-item-right">
                            <input type="checkbox" checked />
                        </div>
                    </div>
                    <div class="list-item">
                        <div class="list-item-left">
                            <div class="list-item-title">"Push Notifications"</div>
                        </div>
                        <div class="list-item-right">
                            <input type="checkbox" checked />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
