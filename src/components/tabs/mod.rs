use crate::pages::{
    AddTeamMemberPage, AgentPage, CalendarPage, HistoryPage, NetworkingPage, OrganizationPage,
    OverviewPage, PortfoliosPage, ReportingPage, SettingsPage, TransactionsPage,
};
use crate::stores::{
    create_action, use_app_store, use_notification_store, use_ui_store, use_undo_redo_store,
};
use crate::types::{ActionType, TabType};
use leptos::prelude::*;

#[derive(Clone)]
struct TabListItem {
    tab_type: TabType,
    title: &'static str,
}

/// Per-tab edit mode signal provided as context to child pages.
#[derive(Clone, Copy)]
pub struct TabEditMode(pub Signal<bool>);

pub fn use_tab_edit_mode() -> Signal<bool> {
    use_context::<TabEditMode>()
        .map(|c| c.0)
        .unwrap_or_else(|| Signal::derive(move || false))
}

fn use_tab_toggle(tab_type: TabType) -> Callback<()> {
    let app_store = use_app_store();
    let ui_store = use_ui_store();
    let undo_store = use_undo_redo_store();
    let tab_type_for_scroll = tab_type.clone();
    Callback::new(move |_| {
        let current_tab = tab_type.clone();
        let store = app_store.get();
        let user_id = store.current_user.id;
        let user_name = store.current_user.name.clone();
        let user_role = format!("{:?}", store.current_user.role);
        let org_id = store.current_user.organization_id;
        let already_expanded = store.is_tab_expanded(&current_tab);
        drop(store);

        if already_expanded {
            let tab_id = format!("tab-content-{:?}", tab_type_for_scroll).to_lowercase();
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(el) = document.get_element_by_id(&tab_id) {
                        scroll_el_and_children_to_top(&el);
                    }
                }
            }
            ui_store.update(|ui| ui.close_tabs_drawer());
            return;
        }

        app_store.update(|store| {
            let prev_tab = store.active_tabs.first().cloned();
            let action = create_action(
                ActionType::View,
                "Tab",
                &format!("Opened {:?} tab", current_tab),
                user_id,
                &user_name,
                &user_role,
                org_id,
                None,
            );
            let action = action.with_navigation(
                prev_tab.map(|t| t.as_str().to_string()).unwrap_or_default(),
                current_tab.as_str().to_string(),
            );
            undo_store.update(|u| u.record_action(action));
            store.expand_tab(current_tab);
        });
        ui_store.update(|ui| ui.close_tabs_drawer());
    })
}

fn scroll_el_and_children_to_top(el: &web_sys::Element) {
    use wasm_bindgen::JsCast;
    if let Ok(node) = el.clone().dyn_into::<web_sys::HtmlElement>() {
        node.set_scroll_top(0);
    }
    let children = el.children();
    for i in 0..children.length() {
        if let Some(child) = children.item(i) {
            scroll_el_and_children_to_top(&child);
        }
    }
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
fn TabItem(tab_type: TabType, title: &'static str) -> impl IntoView {
    let app_store = use_app_store();
    let notification_store = use_notification_store();
    let on_toggle = use_tab_toggle(tab_type.clone());
    let tab_type_class = tab_type.clone();
    let tab_type_badge = tab_type.clone();

    view! {
        <div class="tab-item" class:expanded=move || app_store.get().is_tab_expanded(&tab_type_class)>
            <div class="tab-header" on:click=move |_| on_toggle.run(())>
                <span class="tab-title">{title}</span>
                {move || {
                    let count = notification_store.get().notifications_for_tab(&tab_type_badge);
                    if count > 0 {
                        view! { <span class="tab-notif-badge">{count}</span> }.into_any()
                    } else {
                        ().into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn TabContent(tab_type: TabType) -> impl IntoView {
    let app_store = use_app_store();
    let tab_type_for_edit = tab_type.clone();
    let edit_mode = Signal::derive(move || app_store.get().is_tab_edit_mode(&tab_type_for_edit));
    provide_context(TabEditMode(edit_mode));

    let tab_id = format!("tab-content-{:?}", tab_type).to_lowercase();

    view! {
        <div class="tab-content" id=tab_id on:click=|ev| ev.stop_propagation()>
            {render_tab_page(tab_type)}
        </div>
    }
}

#[component]
pub fn TabList() -> impl IntoView {
    let ui_store = use_ui_store();

    let tab_items = move || {
        let mut items = vec![
            TabListItem {
                tab_type: TabType::Portfolios,
                title: "Portfolios",
            },
            TabListItem {
                tab_type: TabType::Networking,
                title: "Networking",
            },
            TabListItem {
                tab_type: TabType::Organization,
                title: "Organization",
            },
            TabListItem {
                tab_type: TabType::Reporting,
                title: "Reporting",
            },
            TabListItem {
                tab_type: TabType::Calendar,
                title: "Calendar",
            },
            TabListItem {
                tab_type: TabType::Transactions,
                title: "Transactions",
            },
            TabListItem {
                tab_type: TabType::History,
                title: "History",
            },
            TabListItem {
                tab_type: TabType::Settings,
                title: "Settings",
            },
            TabListItem {
                tab_type: TabType::Agent,
                title: "Agent",
            },
        ];
        if ui_store.get().networking_add_member_open {
            items.push(TabListItem {
                tab_type: TabType::NetworkingAddMember,
                title: "Add Team",
            });
        }
        items
    };

    view! {
        <div class="tab-list">
            {move || {
                let items = tab_items();
                view! {
                    <For
                        each=move || items.clone()
                        key=|item| item.tab_type.clone()
                        children=move |item| {
                            view! {
                                <TabItem tab_type=item.tab_type title=item.title />
                            }.into_any()
                        }
                    />
                }.into_any()
            }}
        </div>
    }
}

#[component]
pub fn TabsContainer() -> impl IntoView {
    let app_store = use_app_store();

    view! {
        <div class="tabs-container">
            <div class="tabs-viewport">
                {move || {
                    let tabs = app_store.get().active_tabs.clone();
                    if tabs.is_empty() {
                        ().into_any()
                    } else {
                        view! {
                            <For
                                each=move || tabs.clone()
                                key=|tab| tab.clone()
                                children=move |tab| view! { <TabContent tab_type=tab /> }.into_any()
                            />
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
