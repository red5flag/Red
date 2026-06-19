use crate::stores::use_app_store;
use crate::types::TabType;
use leptos::prelude::*;

// Overview Tab Component
#[component]
fn OverviewTab(on_click: Callback<()>) -> impl IntoView {
    view! {
        <div
            class="tab-item"
            on:click=move |_| on_click.run(())
        >
            <span class="tab-title">"Overview"</span>
            <div class="tab-hot-options">
                <button class="hot-btn" title="Refresh">"↻"</button>
                <button class="hot-btn" title="Settings">"⚙"</button>
            </div>
        </div>
    }
}

// Portfolios Tab Component
#[component]
fn PortfoliosTab(on_click: Callback<()>) -> impl IntoView {
    view! {
        <div
            class="tab-item"
            on:click=move |_| on_click.run(())
        >
            <span class="tab-title">"Portfolios"</span>
            <div class="tab-hot-options">
                <button
                    class="hot-btn"
                    title="Modify"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        // Open modify modal
                    }
                >
                    "✎"
                </button>
                <button
                    class="hot-btn"
                    title="Payout"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        // Open payout modal
                    }
                >
                    "$"
                </button>
                <button
                    class="hot-btn"
                    title="Notify"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        // Open notify modal
                    }
                >
                    "🔔"
                </button>
            </div>
        </div>
    }
}

// Networking Tab Component
#[component]
fn NetworkingTab(on_click: Callback<()>) -> impl IntoView {
    view! {
        <div
            class="tab-item"
            on:click=move |_| on_click.run(())
        >
            <span class="tab-title">"Networking"</span>
            <div class="tab-hot-options">
                <button
                    class="hot-btn"
                    title="Add User"
                    on:click=move |ev| {
                        ev.stop_propagation();
                    }
                >
                    "+"
                </button>
                <button
                    class="hot-btn"
                    title="Payment Setup"
                    on:click=move |ev| {
                        ev.stop_propagation();
                    }
                >
                    "$"
                </button>
            </div>
        </div>
    }
}

// History Tab Component
#[component]
fn HistoryTab(on_click: Callback<()>) -> impl IntoView {
    view! {
        <div
            class="tab-item"
            on:click=move |_| on_click.run(())
        >
            <span class="tab-title">"History"</span>
            <div class="tab-hot-options">
                <button
                    class="hot-btn"
                    title="Filter"
                    on:click=move |ev| {
                        ev.stop_propagation();
                    }
                >
                    "⚙"
                </button>
            </div>
        </div>
    }
}

// Settings Tab Component
#[component]
fn SettingsTab(on_click: Callback<()>) -> impl IntoView {
    view! {
        <div
            class="tab-item"
            on:click=move |_| on_click.run(())
        >
            <span class="tab-title">"Settings"</span>
            <div class="tab-hot-options">
                <button
                    class="hot-btn"
                    title="Editor"
                    on:click=move |ev| {
                        ev.stop_propagation();
                    }
                >
                    "✎"
                </button>
            </div>
        </div>
    }
}

// Agent Tab Component
#[component]
fn AgentTab(on_click: Callback<()>) -> impl IntoView {
    view! {
        <div
            class="tab-item"
            on:click=move |_| on_click.run(())
        >
            <span class="tab-title">"Agent"</span>
            <div class="tab-hot-options">
                <button
                    class="hot-btn"
                    title="Clear Chat"
                    on:click=move |ev| {
                        ev.stop_propagation();
                    }
                >
                    "✕"
                </button>
            </div>
        </div>
    }
}

// Main Tabs Container
#[component]
pub fn TabsContainer() -> impl IntoView {
    let app_store = use_app_store();

    let on_tab_click = move |tab: TabType| {
        app_store.update(|store| {
            if store.expanded_tab.as_ref() == Some(&tab) {
                // Collapse if already expanded
                store.collapse_tab();
            } else {
                // Expand the tab
                store.expand_tab(tab);
            }
        });
    };

    view! {
        <div class="tabs-container">
            <OverviewTab
                on_click={
                    let tab = TabType::Overview;
                    Callback::new(move |_| on_tab_click(tab.clone()))
                }
            />
            <PortfoliosTab
                on_click={
                    let tab = TabType::Portfolios;
                    Callback::new(move |_| on_tab_click(tab.clone()))
                }
            />
            <NetworkingTab
                on_click={
                    let tab = TabType::Networking;
                    Callback::new(move |_| on_tab_click(tab.clone()))
                }
            />
            <HistoryTab
                on_click={
                    let tab = TabType::History;
                    Callback::new(move |_| on_tab_click(tab.clone()))
                }
            />
            <SettingsTab
                on_click={
                    let tab = TabType::Settings;
                    Callback::new(move |_| on_tab_click(tab.clone()))
                }
            />
            <AgentTab
                on_click={
                    let tab = TabType::Agent;
                    Callback::new(move |_| on_tab_click(tab.clone()))
                }
            />
        </div>
    }
}
