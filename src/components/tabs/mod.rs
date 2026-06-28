use crate::pages::{AddTeamMemberPage, AgentPage, CalendarPage, HistoryPage, NetworkingPage, OrganizationPage, OverviewPage, PortfoliosPage, ReportingPage, SettingsPage, TransactionsPage};
use crate::stores::use_app_store;
use crate::types::TabType;
use leptos::prelude::*;

/// Per-tab edit mode signal provided as context to child pages.
#[derive(Clone, Copy)]
pub struct TabEditMode(pub Signal<bool>);

pub fn use_tab_edit_mode() -> Signal<bool> {
    use_context::<TabEditMode>().map(|c| c.0).unwrap_or_else(|| Signal::derive(move || false))
}

fn render_tab_page(tab_type: TabType) -> impl IntoView {
    match tab_type {
        TabType::Overview => view! { <OverviewPage /> }.into_any(),
        TabType::Portfolios => view! { <PortfoliosPage /> }.into_any(),
        TabType::Networking => view! { <NetworkingPage /> }.into_any(),
        TabType::NetworkingAddMember => view! { <AddTeamMemberPage /> }.into_any(),
        TabType::Organization => view! { <OrganizationPage /> }.into_any(),
        TabType::Reporting => view! { <ReportingPage /> }.into_any(),
        TabType::Calendar => view! { <CalendarPage /> }.into_any(),
        TabType::Transactions => view! { <TransactionsPage /> }.into_any(),
        TabType::History => view! { <HistoryPage /> }.into_any(),
        TabType::Settings => view! { <SettingsPage /> }.into_any(),
        TabType::Agent => view! { <AgentPage /> }.into_any(),
    }
}

#[component]
fn TabContent(tab_type: TabType) -> impl IntoView {
    let app_store = use_app_store();
    let tab_type_for_edit = tab_type.clone();
    let edit_mode = Signal::derive(move || app_store.get().is_tab_edit_mode(&tab_type_for_edit));
    provide_context(TabEditMode(edit_mode));

    view! {
        <div class="tab-content" on:click=|ev| ev.stop_propagation()>
            {render_tab_page(tab_type)}
        </div>
    }
}

#[component]
pub fn TabsContainer() -> impl IntoView {
    let app_store = use_app_store();

    view! {
        <div class="tabs-layout">
            <TabDrawer />
            <div class="tabs-viewport" class:drawer-open=move || app_store.get().drawer_open>
                {move || {
                    let tabs = app_store.get().active_tabs.clone();
                    if tabs.is_empty() {
                        ().into_any()
                    } else {
                        tabs.into_iter()
                            .map(|tab| view! { <TabContent tab_type=tab /> }.into_any())
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn TabDrawer() -> impl IntoView {
    let app_store = use_app_store();
    let on_overlay_click = move |_| {
        app_store.update(|s| s.close_drawer());
    };

    view! {
        <>
            <div
                class="tab-drawer-overlay"
                class:drawer-open=move || app_store.get().drawer_open
                on:click=on_overlay_click
            >
                <div class="tab-drawer" on:click=|ev| ev.stop_propagation()>
                    <div class="tab-drawer-header">
                        <span class="tab-drawer-title">"MENU"</span>
                    </div>
                    <div class="tab-drawer-items">
                        <TabDrawerItem tab_type=TabType::Overview title="Overview" />
                        <TabDrawerItem tab_type=TabType::Portfolios title="Portfolios" />
                        <TabDrawerItem tab_type=TabType::Networking title="Networking" />
                        {move || if app_store.get().networking_add_member_open {
                            view! {
                                <TabDrawerItem tab_type=TabType::NetworkingAddMember title="Add Team" />
                            }.into_any()
                        } else { ().into_any() }}
                        <TabDrawerItem tab_type=TabType::Organization title="Organization" />
                        <TabDrawerItem tab_type=TabType::Reporting title="Reporting" />
                        <TabDrawerItem tab_type=TabType::Calendar title="Calendar" />
                        <TabDrawerItem tab_type=TabType::Transactions title="Transactions" />
                        <TabDrawerItem tab_type=TabType::History title="History" />
                        <TabDrawerItem tab_type=TabType::Settings title="Settings" />
                        <TabDrawerItem tab_type=TabType::Agent title="Agent" />
                    </div>
                </div>
            </div>
        </>
    }
}

#[component]
fn TabDrawerItem(tab_type: TabType, title: &'static str) -> impl IntoView {
    let app_store = use_app_store();
    let tab_type_active = tab_type.clone();
    let is_active = move || app_store.get().active_tabs.contains(&tab_type_active);
    let tab_type_click = tab_type.clone();
    let on_click = Callback::new(move |_| {
        let tt = tab_type_click.clone();
        app_store.update(|s| {
            s.expand_tab(tt);
            s.close_drawer();
        });
    });

    view! {
        <div class="tab-drawer-item" class:active=is_active>
            <div class="tab-drawer-row" on:click=move |_| on_click.run(())>
                <span class="tab-drawer-label">{title}</span>
            </div>
        </div>
    }
}
