use crate::stores::use_ui_store;
use leptos::prelude::*;

#[component]
pub(crate) fn AccessibilitySettings() -> impl IntoView {
    let ui_store = use_ui_store();

    view! {
        <div class="data-card">
            <div class="card-header">
                <span class="card-title">"Accessibility"</span>
            </div>
            <div class="settings-list">
                <div class="list-item">
                    <div class="list-item-left">
                        <label class="list-item-title" for="settings-font-size">"Font Size"</label>
                        <div class="list-item-desc">"Adjust the text size across the application."</div>
                    </div>
                    <div class="list-item-right">
                        <select
                            id="settings-font-size"
                            class="form-select"
                            aria-label="Select font size"
                            prop:value={move || ui_store.get().font_size.clone()}
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                ui_store.update(|s| s.font_size = value);
                            }
                        >
                            <option value="small">"Small"</option>
                            <option value="default">"Default"</option>
                            <option value="large">"Large"</option>
                            <option value="extra-large">"Extra Large"</option>
                        </select>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Reduced Motion"</div>
                        <div class="list-item-desc">"Minimise animations and transitions for vestibular and motion sensitivity."</div>
                    </div>
                    <div class="list-item-right">
                        <label class="settings-toggle">
                            <input
                                type="checkbox"
                                aria-label="Toggle reduced motion"
                                prop:checked={move || ui_store.get().reduced_motion}
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    ui_store.update(|s| s.reduced_motion = checked);
                                }
                            />
                            <span class="settings-toggle-slider"></span>
                        </label>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"TalkBack / Accessibility Hints"</div>
                        <div class="list-item-desc">"Increase screen-reader labelling and visible focus hints across the interface."</div>
                    </div>
                    <div class="list-item-right">
                        <label class="settings-toggle">
                            <input
                                type="checkbox"
                                aria-label="Toggle TalkBack accessibility hints"
                                prop:checked={move || ui_store.get().talkback_enabled}
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    ui_store.update(|s| s.set_talkback_enabled(checked));
                                }
                            />
                            <span class="settings-toggle-slider"></span>
                        </label>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <label class="list-item-title" for="settings-language">"Language"</label>
                        <div class="list-item-desc">"Choose the display language for the interface."</div>
                    </div>
                    <div class="list-item-right">
                        <select
                            id="settings-language"
                            class="form-select"
                            aria-label="Select language"
                            prop:value={move || ui_store.get().language.clone()}
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                ui_store.update(|s| s.language = value);
                            }
                        >
                            <option value="en-AU">"English (Australia)"</option>
                            <option value="en-US">"English (United States)"</option>
                            <option value="en-GB">"English (United Kingdom)"</option>
                        </select>
                    </div>
                </div>
            </div>
        </div>
    }
}
