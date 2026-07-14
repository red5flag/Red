use crate::stores::use_ui_store;
use leptos::prelude::*;

#[component]
pub(crate) fn DisplaySettings() -> impl IntoView {
    let ui_store = use_ui_store();

    view! {
        <div class="data-card">
            <div class="card-header">
                <span class="card-title">"Appearance"</span>
            </div>
            <div class="settings-list">
                <div class="list-item">
                    <div class="list-item-left">
                        <label class="list-item-title" for="settings-theme">"Theme"</label>
                        <div class="list-item-desc">"Choose the colour scheme and visual accessibility mode."</div>
                    </div>
                    <div class="list-item-right">
                        <select
                            id="settings-theme"
                            class="form-select"
                            aria-label="Select theme"
                            prop:value={move || ui_store.get().theme.as_str().to_string()}
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                let theme = match value.as_str() {
                                    "light" => crate::types::Theme::Light,
                                    "dark" => crate::types::Theme::Dark,
                                    "high-contrast" => crate::types::Theme::HighContrast,
                                    "deuteranopia" => crate::types::Theme::Deuteranopia,
                                    "protanopia" => crate::types::Theme::Protanopia,
                                    "tritanopia" => crate::types::Theme::Tritanopia,
                                    "deuteranomaly" => crate::types::Theme::Deuteranomaly,
                                    "protanomaly" => crate::types::Theme::Protanomaly,
                                    "tritanomaly" => crate::types::Theme::Tritanomaly,
                                    "achromatopsia" => crate::types::Theme::Achromatopsia,
                                    "achromatomaly" => crate::types::Theme::Achromatomaly,
                                    "low-vision" => crate::types::Theme::LowVision,
                                    _ => crate::types::Theme::Light,
                                };
                                ui_store.update(|s| s.set_theme(theme));
                            }
                        >
                            <option value="light">"Light"</option>
                            <option value="dark">"Dark"</option>
                            <option value="high-contrast">"High Contrast"</option>
                            <optgroup label="Color blindness">
                                <option value="deuteranopia">"Deuteranopia (green-blind)"</option>
                                <option value="protanopia">"Protanopia (red-blind)"</option>
                                <option value="tritanopia">"Tritanopia (blue-blind)"</option>
                                <option value="deuteranomaly">"Deuteranomaly (green-weak)"</option>
                                <option value="protanomaly">"Protanomaly (red-weak)"</option>
                                <option value="tritanomaly">"Tritanomaly (blue-weak)"</option>
                                <option value="achromatopsia">"Achromatopsia (no colour)"</option>
                                <option value="achromatomaly">"Achromatomaly (reduced colour)"</option>
                            </optgroup>
                            <option value="low-vision">"Low Vision / Legally Blind"</option>
                        </select>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <label class="list-item-title" for="settings-edge-style">"Edge Style"</label>
                        <div class="list-item-desc">"Square, rounded, or pill-shaped controls."</div>
                    </div>
                    <div class="list-item-right">
                        <select
                            id="settings-edge-style"
                            class="form-select"
                            aria-label="Select edge style"
                            prop:value={move || ui_store.get().edge_style.as_str().to_string()}
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                let edge_style = match value.as_str() {
                                    "square" => crate::types::EdgeStyle::Square,
                                    "rounded" => crate::types::EdgeStyle::Rounded,
                                    "pill" => crate::types::EdgeStyle::Pill,
                                    _ => crate::types::EdgeStyle::Square,
                                };
                                ui_store.update(|s| s.set_edge_style(edge_style));
                            }
                        >
                            <option value="square">"Square"</option>
                            <option value="rounded">"Rounded"</option>
                            <option value="pill">"Pill"</option>
                        </select>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <label class="list-item-title" for="settings-button-style">"Button Style"</label>
                        <div class="list-item-desc">"Filled, outline, or ghost buttons."</div>
                    </div>
                    <div class="list-item-right">
                        <select
                            id="settings-button-style"
                            class="form-select"
                            aria-label="Select button style"
                            prop:value={move || ui_store.get().button_style.as_str().to_string()}
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                let button_style = match value.as_str() {
                                    "filled" => crate::types::ButtonStyle::Filled,
                                    "outline" => crate::types::ButtonStyle::Outline,
                                    "ghost" => crate::types::ButtonStyle::Ghost,
                                    _ => crate::types::ButtonStyle::Filled,
                                };
                                ui_store.update(|s| s.set_button_style(button_style));
                            }
                        >
                            <option value="filled">"Filled"</option>
                            <option value="outline">"Outline"</option>
                            <option value="ghost">"Ghost"</option>
                        </select>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <label class="list-item-title" for="settings-density">"Density"</label>
                        <div class="list-item-desc">"Compact, comfortable, or spacious spacing."</div>
                    </div>
                    <div class="list-item-right">
                        <select
                            id="settings-density"
                            class="form-select"
                            aria-label="Select density"
                            prop:value={move || ui_store.get().density.as_str().to_string()}
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                let density = match value.as_str() {
                                    "compact" => crate::types::Density::Compact,
                                    "comfortable" => crate::types::Density::Comfortable,
                                    "spacious" => crate::types::Density::Spacious,
                                    _ => crate::types::Density::Comfortable,
                                };
                                ui_store.update(|s| s.set_density(density));
                            }
                        >
                            <option value="compact">"Compact"</option>
                            <option value="comfortable">"Comfortable"</option>
                            <option value="spacious">"Spacious"</option>
                        </select>
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <label class="list-item-title" for="settings-accent-color">"Accent Color"</label>
                        <div class="list-item-desc">"Choose the primary accent colour used for active and selected states."</div>
                    </div>
                    <div class="list-item-right">
                        <input
                            id="settings-accent-color"
                            type="color"
                            class="form-input settings-color-input"
                            aria-label="Select accent color"
                            prop:value={move || ui_store.get().accent_color.clone()}
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                ui_store.update(|s| s.set_accent_color(value));
                            }
                        />
                    </div>
                </div>
                <div class="list-item">
                    <div class="list-item-left">
                        <div class="list-item-title">"Blind Mode"</div>
                        <div class="list-item-desc">"Show explicit Add, Edit, and Remove buttons for screen readers and accessibility needs."</div>
                    </div>
                    <div class="list-item-right">
                        <label class="settings-toggle">
                            <input
                                type="checkbox"
                                aria-label="Toggle blind mode"
                                prop:checked={move || ui_store.get().blind_mode}
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    ui_store.update(|s| s.blind_mode = checked);
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
