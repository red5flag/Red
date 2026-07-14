use crate::stores::use_ui_store;
use crate::types::default_presets;
use leptos::prelude::*;

#[component]
pub(crate) fn PresetSelector() -> impl IntoView {
    let ui_store = use_ui_store();
    let default_presets = default_presets();

    let (save_name, set_save_name) = signal(String::new());
    let (show_save, set_show_save) = signal(false);

    let apply_preset = Callback::new(move |preset: crate::types::SettingsPreset| {
        ui_store.update(|s| s.apply_preset(&preset));
    });

    let save_current = Callback::new(move |_: ()| {
        let name = save_name.get();
        let trimmed = name.trim();
        if trimmed.is_empty() {
            set_show_save.set(false);
            return;
        }
        ui_store.update(|s| s.save_current_as_preset(trimmed.to_string()));
        set_save_name.set(String::new());
        set_show_save.set(false);
    });

    view! {
        <div class="data-card">
            <div class="card-header">
                <span class="card-title">"Presets"</span>
            </div>
            <div class="preset-selector">
                <div class="preset-grid">
                    {default_presets.into_iter().map(|preset| {
                        let preset_for_click = preset.clone();
                        let name = preset.name.clone();
                        let edge = preset.edge_style.as_str();
                        let button = preset.button_style.as_str();
                        let density = preset.density.as_str();
                        let color = preset.accent_color.clone();
                        view! {
                            <button
                                class="preset-card"
                                type="button"
                                aria-label={format!("Apply {} preset", name)}
                                on:click=move |_| apply_preset.run(preset_for_click.clone())
                            >
                                <span class="preset-name">{name.clone()}</span>
                                <span class="preset-preview" style={format!("background-color: {}", color)}></span>
                                <span class="preset-meta">{format!("{} · {} · {}", edge, button, density)}</span>
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                {move || {
                    let custom = ui_store.get().custom_presets.clone();
                    if !custom.is_empty() {
                        view! {
                            <div class="preset-section">
                                <div class="preset-section-title">"Custom Presets"</div>
                                <div class="preset-grid">
                                    {custom.into_iter().map(|preset| {
                                        let preset_for_click = preset.clone();
                                        let name = preset.name.clone();
                                        let edge = preset.edge_style.as_str();
                                        let button = preset.button_style.as_str();
                                        let density = preset.density.as_str();
                                        let color = preset.accent_color.clone();
                                        view! {
                                            <button
                                                class="preset-card"
                                                type="button"
                                                aria-label={format!("Apply custom preset {}", name)}
                                                on:click=move |_| apply_preset.run(preset_for_click.clone())
                                            >
                                                <span class="preset-name">{name.clone()}</span>
                                                <span class="preset-preview" style={format!("background-color: {}", color)}></span>
                                                <span class="preset-meta">{format!("{} · {} · {}", edge, button, density)}</span>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}

                <div class="preset-save-row">
                    {move || if show_save.get() {
                        view! {
                            <div class="preset-save-input-row">
                                <input
                                    class="form-input"
                                    type="text"
                                    placeholder="Preset name"
                                    prop:value={move || save_name.get()}
                                    on:input=move |ev| set_save_name.set(event_target_value(&ev))
                                    on:keydown=move |ev| {
                                        if ev.key() == "Enter" {
                                            save_current.run(());
                                        }
                                    }
                                />
                                <button class="settings-action-btn" on:click=move |_| save_current.run(())>"Save"</button>
                                <button class="settings-action-btn" on:click=move |_| set_show_save.set(false)>"Cancel"</button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <button class="settings-action-btn" on:click=move |_| set_show_save.set(true)>
                                "Save current as preset"
                            </button>
                        }.into_any()
                    }}
                </div>
                <p class="preset-persistence-note">"Custom presets are saved in this session only. Backend persistence is deferred."</p>
            </div>
        </div>
    }
}
