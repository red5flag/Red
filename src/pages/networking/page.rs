use crate::pages::networking::channels::render_channels;
use crate::pages::networking::contact_list::render_contacts;
use crate::pages::networking::external_orgs::render_external_orgs;
use crate::pages::networking::integrations::render_integrations;
use crate::pages::networking::partners::render_by_type;
use crate::pages::networking::relationship_history::render_relationship_history;
use crate::pages::networking::{
    Channel, ExternalContact, ExternalOrganization, Integration, NetSort, NetTab,
    RelationshipEvent, RelationshipStatus, RiskLevel,
};
use crate::stores::{use_app_store, use_messenger_store, use_search_store, use_ui_store};
use crate::types::{TabType, ViewCount};
use chrono::Utc;
use leptos::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

fn mock_contacts() -> Vec<ExternalContact> {
    vec![
        ExternalContact {
            id: Uuid::new_v4(),
            name: "Sarah Jones".into(),
            title: "Supplier Contact".into(),
            company: "ABC Supplies".into(),
            email: Some("sarah@abcsupplies.com".into()),
            phone: Some("+1-555-0201".into()),
            relationship_type: "Supplier".into(),
            status: RelationshipStatus::Active,
            risk_level: RiskLevel::Medium,
            linked_portfolios: vec!["Commercial Real Estate".into(), "RedOrg Large Scale".into()],
            linked_transactions: vec!["TX-1048".into(), "TX-1032".into()],
            linked_reports: vec!["June Supplier Report".into()],
            last_message: Some("2 days ago".into()),
            last_transaction: Some("Invoice 1048".into()),
            channels: vec!["Email".into(), "Phone".into(), "Supplier Portal".into()],
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=SarahJones".into()),
        },
        ExternalContact {
            id: Uuid::new_v4(),
            name: "Mark Taylor".into(),
            title: "Account Manager".into(),
            company: "ABC Supplies".into(),
            email: Some("mark@abcsupplies.com".into()),
            phone: Some("+1-555-0202".into()),
            relationship_type: "Supplier".into(),
            status: RelationshipStatus::Active,
            risk_level: RiskLevel::Low,
            linked_portfolios: vec!["Commercial Real Estate".into()],
            linked_transactions: vec!["TX-1041".into()],
            linked_reports: vec![],
            last_message: Some("5 days ago".into()),
            last_transaction: Some("Invoice 1041".into()),
            channels: vec!["Email".into(), "Phone".into()],
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=MarkTaylor".into()),
        },
        ExternalContact {
            id: Uuid::new_v4(),
            name: "Emma Wilson".into(),
            title: "Broker".into(),
            company: "ABC Realty".into(),
            email: Some("emma@abcrealty.com".into()),
            phone: Some("+1-555-0301".into()),
            relationship_type: "Partner".into(),
            status: RelationshipStatus::Active,
            risk_level: RiskLevel::Low,
            linked_portfolios: vec!["Commercial Real Estate".into()],
            linked_transactions: vec![],
            linked_reports: vec!["Q2 Market Analysis".into()],
            last_message: Some("1 week ago".into()),
            last_transaction: None,
            channels: vec!["Email".into(), "LinkedIn".into()],
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=EmmaWilson".into()),
        },
        ExternalContact {
            id: Uuid::new_v4(),
            name: "David Chen".into(),
            title: "Client".into(),
            company: "Chen Investments".into(),
            email: Some("david@cheninv.com".into()),
            phone: Some("+1-555-0401".into()),
            relationship_type: "Client".into(),
            status: RelationshipStatus::Active,
            risk_level: RiskLevel::Low,
            linked_portfolios: vec!["RedOrg Large Scale".into()],
            linked_transactions: vec!["TX-1050".into()],
            linked_reports: vec!["Investment Summary".into()],
            last_message: Some("3 days ago".into()),
            last_transaction: Some("Dividend payment".into()),
            channels: vec!["Email".into(), "Investor Portal".into()],
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=DavidChen".into()),
        },
        ExternalContact {
            id: Uuid::new_v4(),
            name: "Lisa Garcia".into(),
            title: "Legal Partner".into(),
            company: "XYZ Legal".into(),
            email: Some("lisa@xyzlegal.com".into()),
            phone: Some("+1-555-0501".into()),
            relationship_type: "Partner".into(),
            status: RelationshipStatus::Pending,
            risk_level: RiskLevel::Low,
            linked_portfolios: vec!["Commercial Real Estate".into()],
            linked_transactions: vec![],
            linked_reports: vec!["Contract Review".into()],
            last_message: None,
            last_transaction: None,
            channels: vec!["Email".into(), "Phone".into()],
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=LisaGarcia".into()),
        },
    ]
}

fn mock_external_orgs() -> Vec<ExternalOrganization> {
    vec![
        ExternalOrganization {
            id: Uuid::new_v4(),
            name: "ABC Supplies".into(),
            org_type: "Vendor".into(),
            primary_contact: Some("Sarah Jones".into()),
            status: RelationshipStatus::Active,
            risk_level: RiskLevel::Medium,
            linked_portfolios: vec!["Commercial Real Estate".into(), "RedOrg Large Scale".into()],
            transaction_count: 12,
            document_count: 4,
            channels: vec!["Email".into(), "Supplier Portal".into(), "Phone".into()],
            avatar_url: None,
        },
        ExternalOrganization {
            id: Uuid::new_v4(),
            name: "ABC Realty".into(),
            org_type: "Partner".into(),
            primary_contact: Some("Emma Wilson".into()),
            status: RelationshipStatus::Active,
            risk_level: RiskLevel::Low,
            linked_portfolios: vec!["Commercial Real Estate".into()],
            transaction_count: 3,
            document_count: 2,
            channels: vec!["Email".into(), "LinkedIn".into()],
            avatar_url: None,
        },
        ExternalOrganization {
            id: Uuid::new_v4(),
            name: "Chen Investments".into(),
            org_type: "Client".into(),
            primary_contact: Some("David Chen".into()),
            status: RelationshipStatus::Active,
            risk_level: RiskLevel::Low,
            linked_portfolios: vec!["RedOrg Large Scale".into()],
            transaction_count: 8,
            document_count: 5,
            channels: vec!["Email".into(), "Investor Portal".into()],
            avatar_url: None,
        },
        ExternalOrganization {
            id: Uuid::new_v4(),
            name: "XYZ Legal".into(),
            org_type: "Partner".into(),
            primary_contact: Some("Lisa Garcia".into()),
            status: RelationshipStatus::Pending,
            risk_level: RiskLevel::Low,
            linked_portfolios: vec!["Commercial Real Estate".into()],
            transaction_count: 0,
            document_count: 1,
            channels: vec!["Email".into(), "Phone".into()],
            avatar_url: None,
        },
        ExternalOrganization {
            id: Uuid::new_v4(),
            name: "Gold Coast Maintenance Co".into(),
            org_type: "Vendor".into(),
            primary_contact: None,
            status: RelationshipStatus::Active,
            risk_level: RiskLevel::Medium,
            linked_portfolios: vec!["Commercial Real Estate".into()],
            transaction_count: 6,
            document_count: 3,
            channels: vec!["Phone".into(), "Email".into()],
            avatar_url: None,
        },
    ]
}

#[allow(dead_code)]
fn mock_channels() -> Vec<Channel> {
    vec![
        Channel {
            id: Uuid::new_v4(),
            name: "Email".into(),
            channel_type: "Communication".into(),
            address: Some("contact@redorg.com".into()),
            status: RelationshipStatus::Active,
            linked_contact: Some("Sarah Jones".into()),
        },
        Channel {
            id: Uuid::new_v4(),
            name: "Phone".into(),
            channel_type: "Communication".into(),
            address: Some("+1-555-0100".into()),
            status: RelationshipStatus::Active,
            linked_contact: Some("Mark Taylor".into()),
        },
        Channel {
            id: Uuid::new_v4(),
            name: "Supplier Portal".into(),
            channel_type: "Platform".into(),
            address: Some("portal.abcsupplies.com".into()),
            status: RelationshipStatus::Active,
            linked_contact: Some("Sarah Jones".into()),
        },
        Channel {
            id: Uuid::new_v4(),
            name: "Investor Portal".into(),
            channel_type: "Platform".into(),
            address: Some("invest.redorg.com".into()),
            status: RelationshipStatus::Active,
            linked_contact: Some("David Chen".into()),
        },
        Channel {
            id: Uuid::new_v4(),
            name: "LinkedIn".into(),
            channel_type: "Social".into(),
            address: None,
            status: RelationshipStatus::Active,
            linked_contact: Some("Emma Wilson".into()),
        },
        Channel {
            id: Uuid::new_v4(),
            name: "Banking Channel".into(),
            channel_type: "Financial".into(),
            address: Some("ops.redorg.com".into()),
            status: RelationshipStatus::Active,
            linked_contact: None,
        },
        Channel {
            id: Uuid::new_v4(),
            name: "Real Estate Platform".into(),
            channel_type: "Platform".into(),
            address: Some("realestate.platform.com".into()),
            status: RelationshipStatus::Active,
            linked_contact: Some("Emma Wilson".into()),
        },
        Channel {
            id: Uuid::new_v4(),
            name: "Referral Channel".into(),
            channel_type: "Network".into(),
            address: None,
            status: RelationshipStatus::Pending,
            linked_contact: None,
        },
    ]
}

fn mock_integrations() -> Vec<Integration> {
    vec![
        Integration {
            id: Uuid::new_v4(),
            name: "QuickBooks".into(),
            integration_type: "Accounting".into(),
            status: RelationshipStatus::Active,
            last_sync: Some("2 hours ago".into()),
            description: "Syncs transactions and invoices".into(),
        },
        Integration {
            id: Uuid::new_v4(),
            name: "DocuSign".into(),
            integration_type: "Document".into(),
            status: RelationshipStatus::Active,
            last_sync: Some("1 day ago".into()),
            description: "Electronic signatures for contracts".into(),
        },
        Integration {
            id: Uuid::new_v4(),
            name: "Stripe".into(),
            integration_type: "Payment".into(),
            status: RelationshipStatus::Active,
            last_sync: Some("5 minutes ago".into()),
            description: "Payment processing for client invoices".into(),
        },
        Integration {
            id: Uuid::new_v4(),
            name: "Google Calendar".into(),
            integration_type: "Calendar".into(),
            status: RelationshipStatus::Active,
            last_sync: Some("1 hour ago".into()),
            description: "Syncs calendar events and reminders".into(),
        },
        Integration {
            id: Uuid::new_v4(),
            name: "Slack".into(),
            integration_type: "Communication".into(),
            status: RelationshipStatus::Suspended,
            last_sync: Some("3 days ago".into()),
            description: "Team notifications and alerts".into(),
        },
    ]
}

fn mock_relationship_events() -> Vec<RelationshipEvent> {
    let now = Utc::now();
    vec![
        RelationshipEvent {
            id: Uuid::new_v4(),
            entity_name: "ABC Supplies".into(),
            event_description: "Sarah Jones sent a message".into(),
            timestamp: now,
            event_type: "Message".into(),
        },
        RelationshipEvent {
            id: Uuid::new_v4(),
            entity_name: "ABC Supplies".into(),
            event_description: "Transaction TX-1048 was approved".into(),
            timestamp: now,
            event_type: "Transaction".into(),
        },
        RelationshipEvent {
            id: Uuid::new_v4(),
            entity_name: "ABC Supplies".into(),
            event_description: "Payee bank details were updated".into(),
            timestamp: now - chrono::Duration::days(1),
            event_type: "Banking".into(),
        },
        RelationshipEvent {
            id: Uuid::new_v4(),
            entity_name: "ABC Supplies".into(),
            event_description: "Linked to Commercial Real Estate Portfolio".into(),
            timestamp: now - chrono::Duration::days(1),
            event_type: "Portfolio".into(),
        },
        RelationshipEvent {
            id: Uuid::new_v4(),
            entity_name: "XYZ Legal".into(),
            event_description: "Supplier contract document was locked".into(),
            timestamp: now - chrono::Duration::days(7),
            event_type: "Document".into(),
        },
        RelationshipEvent {
            id: Uuid::new_v4(),
            entity_name: "Chen Investments".into(),
            event_description: "New external organization added".into(),
            timestamp: now - chrono::Duration::days(3),
            event_type: "Organization".into(),
        },
        RelationshipEvent {
            id: Uuid::new_v4(),
            entity_name: "Slack".into(),
            event_description: "Integration suspended - authentication failed".into(),
            timestamp: now - chrono::Duration::days(3),
            event_type: "Integration".into(),
        },
    ]
}

#[component]
pub fn NetworkingPage() -> impl IntoView {
    let app_store = use_app_store();
    let messenger_store = use_messenger_store();
    let search_store = use_search_store();
    let ui_store = use_ui_store();
    let (active_tab, set_active_tab) = signal(NetTab::Contacts);
    let (selected_contact, set_selected_contact) = signal::<Option<ExternalContact>>(None);
    let (selected_org, set_selected_org) = signal::<Option<ExternalOrganization>>(None);
    let (_edit_mode, _set_edit_mode) = signal(false);

    // Sort options horizontal scrollbar state
    let sort_scroll_ref = NodeRef::<leptos::html::Div>::new();
    let (sort_can_scroll_left, set_sort_can_scroll_left) = signal(false);
    let (sort_can_scroll_right, set_sort_can_scroll_right) = signal(true);

    let update_sort_scroll_state = move || {
        if let Some(el) = sort_scroll_ref.get() {
            let left = el.scroll_left() as f64;
            let client = el.client_width() as f64;
            let width = el.scroll_width() as f64;
            set_sort_can_scroll_left.set(left > 0.0);
            set_sort_can_scroll_right.set(left + client < width - 1.0);
        }
    };

    let scroll_sort_left = move || {
        if let Some(el) = sort_scroll_ref.get() {
            let _ = el.scroll_by_with_x_and_y(-120.0, 0.0);
        }
        update_sort_scroll_state();
    };

    let scroll_sort_right = move || {
        if let Some(el) = sort_scroll_ref.get() {
            let _ = el.scroll_by_with_x_and_y(120.0, 0.0);
        }
        update_sort_scroll_state();
    };

    Effect::new(move |_| {
        let _ = ui_store.get().net_sort_mode;
        let _ = sort_scroll_ref.get();
        update_sort_scroll_state();
    });

    let sort_mode = move || match ui_store.get().net_sort_mode {
        0 => NetSort::Name,
        1 => NetSort::Company,
        2 => NetSort::Status,
        3 => NetSort::Risk,
        4 => NetSort::Type,
        _ => NetSort::Transactions,
    };
    let sort_ascending = move || ui_store.get().net_sort_ascending;

    let contacts = StoredValue::new(mock_contacts());
    let external_orgs = StoredValue::new(mock_external_orgs());
    let integrations = StoredValue::new(mock_integrations());
    let events = StoredValue::new(mock_relationship_events());

    let (expand_counts, set_expand_counts) = signal(HashMap::<NetTab, usize>::new());
    let page_size = move || ui_store.get().net_view_count().as_usize();
    let visible_for = move |tab: NetTab| {
        let expands = expand_counts.get().get(&tab).copied().unwrap_or(0);
        page_size() * (expands + 1)
    };
    let expand_scope = Callback::new(move |tab: NetTab| {
        set_expand_counts.update(|map| {
            *map.entry(tab).or_insert(0) += 1;
        });
    });

    let filtered_contacts = Memo::new(move |_| {
        let q = search_store.get().query.to_lowercase();
        let sm = sort_mode();
        let asc = sort_ascending();
        let mut list: Vec<_> = contacts
            .get_value()
            .iter()
            .filter(|c| {
                q.is_empty()
                    || c.name.to_lowercase().contains(&q)
                    || c.company.to_lowercase().contains(&q)
            })
            .cloned()
            .collect();
        list.sort_by(|a, b| {
            let ord = match sm {
                NetSort::Name => a.name.cmp(&b.name),
                NetSort::Company => a.company.cmp(&b.company),
                NetSort::Status => a.status.label().cmp(b.status.label()),
                NetSort::Risk => a.risk_level.label().cmp(b.risk_level.label()),
                NetSort::Type => a.relationship_type.cmp(&b.relationship_type),
                NetSort::Transactions => a
                    .linked_transactions
                    .len()
                    .cmp(&b.linked_transactions.len()),
            };
            if asc {
                ord
            } else {
                ord.reverse()
            }
        });
        list
    });

    let filtered_orgs = Memo::new(move |_| {
        let q = search_store.get().query.to_lowercase();
        let sm = sort_mode();
        let asc = sort_ascending();
        let mut list: Vec<_> = external_orgs
            .get_value()
            .iter()
            .filter(|o| {
                q.is_empty()
                    || o.name.to_lowercase().contains(&q)
                    || o.org_type.to_lowercase().contains(&q)
            })
            .cloned()
            .collect();
        list.sort_by(|a, b| {
            let ord = match sm {
                NetSort::Name => a.name.cmp(&b.name),
                NetSort::Company => a.org_type.cmp(&b.org_type),
                NetSort::Status => a.status.label().cmp(b.status.label()),
                NetSort::Risk => a.risk_level.label().cmp(b.risk_level.label()),
                NetSort::Type => a.org_type.cmp(&b.org_type),
                NetSort::Transactions => a.transaction_count.cmp(&b.transaction_count),
            };
            if asc {
                ord
            } else {
                ord.reverse()
            }
        });
        list
    });

    let primary_tabs = [
        NetTab::Contacts,
        NetTab::ExternalOrgs,
        NetTab::Channels,
        NetTab::Integrations,
    ];
    let secondary_tabs = [
        NetTab::Partners,
        NetTab::Clients,
        NetTab::Suppliers,
        NetTab::RelationshipMap,
        NetTab::RelationshipHistory,
    ];

    let render_tab_content = move || {
        let tab = active_tab.get();
        match tab {
            NetTab::Contacts => {
                let visible_count = Signal::derive(move || visible_for(NetTab::Contacts));
                render_contacts(
                    filtered_contacts.get(),
                    set_selected_contact,
                    visible_count,
                    expand_scope,
                )
                .into_any()
            }
            NetTab::ExternalOrgs => {
                let visible_count = Signal::derive(move || visible_for(NetTab::ExternalOrgs));
                render_external_orgs(
                    filtered_orgs.get(),
                    set_selected_org,
                    visible_count,
                    expand_scope,
                )
                .into_any()
            }
            NetTab::Channels => render_channels().into_any(),
            NetTab::Partners => {
                let visible_count = Signal::derive(move || visible_for(NetTab::Partners));
                render_by_type(
                    &contacts.get_value(),
                    &external_orgs.get_value(),
                    "Partner",
                    NetTab::Partners,
                    visible_count,
                    expand_scope,
                )
                .into_any()
            }
            NetTab::Clients => {
                let visible_count = Signal::derive(move || visible_for(NetTab::Clients));
                render_by_type(
                    &contacts.get_value(),
                    &external_orgs.get_value(),
                    "Client",
                    NetTab::Clients,
                    visible_count,
                    expand_scope,
                )
                .into_any()
            }
            NetTab::Suppliers => {
                let visible_count = Signal::derive(move || visible_for(NetTab::Suppliers));
                render_by_type(
                    &contacts.get_value(),
                    &external_orgs.get_value(),
                    "Supplier",
                    NetTab::Suppliers,
                    visible_count,
                    expand_scope,
                )
                .into_any()
            }
            NetTab::Integrations => render_integrations(&integrations.get_value()).into_any(),
            NetTab::RelationshipMap => {
                crate::pages::networking::relationship_history::render_relationship_map(
                    &contacts.get_value(),
                    &external_orgs.get_value(),
                )
                .into_any()
            }
            NetTab::RelationshipHistory => {
                render_relationship_history(&events.get_value()).into_any()
            }
        }
    };

    // Sort option dropdown state
    let (sort_dropdown_open, set_sort_dropdown_open) = signal(false);
    let (view_dropdown_open, set_view_dropdown_open) = signal(false);

    view! {
        <div class="home-screen net-page">
            // Networking controls bar attached below the navbar
            <div class="networking-controls-bar">
                <div class="net-action-bar">
                    <button class="net-action-btn" on:click=move |_| ui_store.update(|s| s.toggle_networking_add_member())>"Add Contact"</button>
                    // View dropdown: contains the view tabs and the number-of-view control
                    {move || {
                        let view_open = view_dropdown_open.get();
                        view! {
                            <div class="net-sort-btn-wrap net-view-btn-wrap">
                                <button
                                    class="net-action-btn net-view-btn"
                                    aria-haspopup="menu"
                                    aria-expanded={view_open}
                                    on:click=move |_| set_view_dropdown_open.update(|v| *v = !*v)
                                >
                                    {move || format!("View: {}", page_size())}
                                </button>
                                {if view_open {
                                    view! {
                                        <div class="net-sort-dropdown net-view-dropdown" on:click=|ev| ev.stop_propagation()>
                                            <div class="net-sort-dropdown-section">
                                                <div class="net-sort-dropdown-section-title">"VIEW"</div>
                                                <div class="net-sort-vertical-list" role="tablist" aria-label="Networking views">
                                                    {primary_tabs.iter().chain(secondary_tabs.iter()).map(|t| {
                                                        let tab = *t;
                                                        let label = t.label();
                                                        view! {
                                                            <button
                                                                class="net-sort-vertical-tab"
                                                                class:net-sort-vertical-tab-active={move || active_tab.get() == tab}
                                                                on:click=move |_| {
                                                                    set_active_tab.set(tab);
                                                                    set_view_dropdown_open.set(false);
                                                                }
                                                            >
                                                                {label}
                                                            </button>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                            <div class="net-sort-dropdown-section">
                                                <div class="net-sort-dropdown-section-title">"VIEW COUNT"</div>
                                                <div class="net-view-count-row">
                                                    <select
                                                        class="net-view-select"
                                                        aria-label="View amount"
                                                        prop:value={move || {
                                                            match ui_store.get().net_view_count() {
                                                                ViewCount::V1 => "view_1",
                                                                ViewCount::V10 => "view_10",
                                                                ViewCount::V20 => "view_20",
                                                                ViewCount::V50 => "view_50",
                                                                ViewCount::V100 => "view_100",
                                                                ViewCount::Custom(_) => "view_custom",
                                                            }.to_string()
                                                        }}
                                                        on:change=move |ev| {
                                                            let v = event_target_value(&ev);
                                                            let vc = match v.as_str() {
                                                                "view_1" => ViewCount::V1,
                                                                "view_10" => ViewCount::V10,
                                                                "view_20" => ViewCount::V20,
                                                                "view_50" => ViewCount::V50,
                                                                "view_100" => ViewCount::V100,
                                                                "view_custom" => ViewCount::Custom(50),
                                                                _ => ViewCount::V50,
                                                            };
                                                            ui_store.update(|s| s.set_net_view_count(vc));
                                                        }
                                                    >
                                                        <option value="view_1">"1"</option>
                                                        <option value="view_10">"10"</option>
                                                        <option value="view_20">"20"</option>
                                                        <option value="view_50">"50"</option>
                                                        <option value="view_100">"100"</option>
                                                        <option value="view_custom">"..."</option>
                                                    </select>
                                                    {if matches!(ui_store.get().net_view_count(), ViewCount::Custom(_)) {
                                                        view! {
                                                            <input
                                                                class="net-view-input"
                                                                type="number"
                                                                min="1"
                                                                step="1"
                                                                aria-label="Custom view count"
                                                                prop:value={move || match ui_store.get().net_view_count() {
                                                                    ViewCount::Custom(n) => n.to_string(),
                                                                    _ => "50".to_string(),
                                                                }}
                                                                on:input=move |ev| {
                                                                    let val = event_target_value(&ev);
                                                                    if let Ok(n) = val.parse::<usize>() {
                                                                        let n = n.max(1);
                                                                        ui_store.update(|s| s.set_net_view_count(ViewCount::Custom(n)));
                                                                    }
                                                                }
                                                            />
                                                        }.into_any()
                                                    } else { ().into_any() }}
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }}
                            </div>
                        }.into_any()
                    }}
                    // Sort option dropdown with horizontal scrollable sort options
                    {move || {
                        let is_open = sort_dropdown_open.get();
                        view! {
                            <div class="net-sort-btn-wrap">
                                <button
                                    class="net-action-btn net-sort-btn"
                                    aria-haspopup="menu"
                                    aria-expanded={is_open}
                                    on:click=move |_| set_sort_dropdown_open.update(|v| *v = !*v)
                                >
                                    <span class="net-sort-label">{format!("Sort: {}", sort_mode().label())}</span>
                                    <span class="net-sort-arrow">{if sort_ascending() { "↑" } else { "↓" }}</span>
                                </button>
                                {if is_open {
                                    view! {
                                        <div class="net-sort-dropdown" on:click=|ev| ev.stop_propagation()>
                                            <div class="net-sort-dropdown-section">
                                                <div class="net-sort-dropdown-section-title">
                                                    "SORT BY"
                                                    <button class="net-sort-toggle" on:click=move |_| ui_store.update(|s| s.net_sort_ascending = !s.net_sort_ascending)>
                                                        {if ui_store.get().net_sort_ascending { "↑ Asc" } else { "↓ Desc" }}
                                                    </button>
                                                </div>
                                                <div class="net-sort-tabs-outer net-sort-tabs-dropdown">
                                                    <button
                                                        class="net-scroll-arrow net-scroll-arrow-left"
                                                        class:hidden={move || !sort_can_scroll_left.get()}
                                                        title="Scroll sort options left"
                                                        aria-label="Scroll sort options left"
                                                        on:click=move |_| scroll_sort_left()
                                                    >"<"</button>
                                                <div
                                                    class="net-sort-tabs"
                                                    role="radiogroup"
                                                    aria-label="Sort by"
                                                    node_ref=sort_scroll_ref
                                                    on:scroll=move |_| update_sort_scroll_state()
                                                >
                                                    {[
                                                        (0, "Name"),
                                                        (1, "Company"),
                                                        (2, "Status"),
                                                        (3, "Risk"),
                                                        (4, "Type"),
                                                        (5, "Transactions"),
                                                    ].iter().map(|(idx, label)| {
                                                        let label_text = *label;
                                                        let idx_u8 = *idx as u8;
                                                        let idx_for_class = idx_u8;
                                                        let idx_for_click = idx_u8;
                                                        view! {
                                                            <button
                                                                class="net-sort-tab"
                                                                class:net-sort-tab-active={move || ui_store.get().net_sort_mode == idx_for_class}
                                                                aria-pressed={move || ui_store.get().net_sort_mode == idx_for_class}
                                                                on:click=move |_| {
                                                                    ui_store.update(|s| {
                                                                        if s.net_sort_mode == idx_for_click {
                                                                            s.net_sort_ascending = !s.net_sort_ascending;
                                                                        } else {
                                                                            s.net_sort_mode = idx_for_click;
                                                                        }
                                                                    });
                                                                    set_sort_dropdown_open.set(false);
                                                                }
                                                            >
                                                                {label_text}
                                                                {move || if ui_store.get().net_sort_mode == idx_for_class {
                                                                    Some(view! { <span class="net-sort-tab-arrow">{if ui_store.get().net_sort_ascending { " ↑" } else { " ↓" }}</span> })
                                                                } else { None }}
                                                            </button>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                                <button
                                                    class="net-scroll-arrow net-scroll-arrow-right"
                                                    class:hidden={move || !sort_can_scroll_right.get()}
                                                    title="Scroll sort options right"
                                                    aria-label="Scroll sort options right"
                                                    on:click=move |_| scroll_sort_right()
                                                >">"</button>
                                            </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }}
                            </div>
                        }.into_any()
                    }}
                </div>
            </div>

            // Tab content
            {render_tab_content}

            // Contact detail modal
            {move || selected_contact.get().map(|c| {
                let on_close = move |_| set_selected_contact.set(None);
                let on_message = {
                    let _name = c.name.clone();
                    move |_| { messenger_store.update(|s| s.set_message_drawer(true)); }
                };
                view! {
                    <div class="net-detail-overlay" on:click=on_close>
                        <div class="net-detail-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="net-detail-header">
                                <img class="net-detail-avatar" src={c.avatar_url.clone().unwrap_or_default()} alt={c.name.clone()} />
                                <div class="net-detail-title-block">
                                    <div class="net-detail-name">{c.name.clone()}</div>
                                    <div class="net-detail-subtitle">{format!("{} • {}", c.title, c.company)}</div>
                                </div>
                                <button class="net-detail-close" on:click=move |_| set_selected_contact.set(None)>"✕"</button>
                            </div>
                            <div class="net-detail-body">
                                <div class="net-detail-section">
                                    <div class="net-detail-row"><span class="net-detail-label">"Type"</span><span class="net-detail-value">{c.relationship_type.clone()}</span></div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Status"</span><span class={format!("net-detail-value {}", c.status.css_class())}>{c.status.label()}</span></div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Risk"</span><span class={format!("net-detail-value {}", c.risk_level.css_class())}>{c.risk_level.label()}</span></div>
                                    {c.email.as_ref().map(|e| view! { <div class="net-detail-row"><span class="net-detail-label">"Email"</span><span class="net-detail-value">{e.clone()}</span></div> }.into_any()).unwrap_or_else(|| ().into_any())}
                                    {c.phone.as_ref().map(|p| view! { <div class="net-detail-row"><span class="net-detail-label">"Phone"</span><span class="net-detail-value">{p.clone()}</span></div> }.into_any()).unwrap_or_else(|| ().into_any())}
                                </div>
                                <div class="net-detail-section">
                                    <div class="net-detail-section-title">"Linked Items"</div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Portfolios"</span><span class="net-detail-value">{c.linked_portfolios.join(", ")}</span></div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Transactions"</span><span class="net-detail-value">{c.linked_transactions.join(", ")}</span></div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Reports"</span><span class="net-detail-value">{if c.linked_reports.is_empty() { "—".into() } else { c.linked_reports.join(", ") }}</span></div>
                                </div>
                                <div class="net-detail-section">
                                    <div class="net-detail-section-title">"Channels"</div>
                                    <div class="net-detail-channels">
                                        {c.channels.iter().map(|ch| view! { <span class="net-channel-tag">{ch.clone()}</span> }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                                <div class="net-detail-section">
                                    <div class="net-detail-section-title">"Recent Activity"</div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Last message"</span><span class="net-detail-value">{c.last_message.clone().unwrap_or_else(|| "—".into())}</span></div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Last transaction"</span><span class="net-detail-value">{c.last_transaction.clone().unwrap_or_else(|| "—".into())}</span></div>
                                </div>
                                <div class="net-detail-actions">
                                    <button class="net-detail-btn net-detail-btn-primary" on:click=on_message>"Message"</button>
                                    <button class="net-detail-btn" on:click=move |_| app_store.update(|s| s.expand_tab(TabType::Transactions))>"View Transactions"</button>
                                    <button class="net-detail-btn" on:click=move |_| app_store.update(|s| s.expand_tab(TabType::Reporting))>"View Reports"</button>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            })}

            // Organization detail modal
            {move || selected_org.get().map(|o| {
                view! {
                    <div class="net-detail-overlay" on:click=move |_| set_selected_org.set(None)>
                        <div class="net-detail-modal" on:click=|ev| ev.stop_propagation()>
                            <div class="net-detail-header">
                                <div class="net-detail-avatar net-detail-avatar-org">{o.name.chars().next().unwrap_or('A')}</div>
                                <div class="net-detail-title-block">
                                    <div class="net-detail-name">{o.name.clone()}</div>
                                    <div class="net-detail-subtitle">{o.org_type.clone()}</div>
                                </div>
                                <button class="net-detail-close" on:click=move |_| set_selected_org.set(None)>"✕"</button>
                            </div>
                            <div class="net-detail-body">
                                <div class="net-detail-section">
                                    <div class="net-detail-row"><span class="net-detail-label">"Type"</span><span class="net-detail-value">{o.org_type.clone()}</span></div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Status"</span><span class={format!("net-detail-value {}", o.status.css_class())}>{o.status.label()}</span></div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Risk"</span><span class={format!("net-detail-value {}", o.risk_level.css_class())}>{o.risk_level.label()}</span></div>
                                    {o.primary_contact.as_ref().map(|p| view! { <div class="net-detail-row"><span class="net-detail-label">"Primary Contact"</span><span class="net-detail-value">{p.clone()}</span></div> }.into_any()).unwrap_or_else(|| ().into_any())}
                                </div>
                                <div class="net-detail-section">
                                    <div class="net-detail-section-title">"Linked Items"</div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Portfolios"</span><span class="net-detail-value">{o.linked_portfolios.join(", ")}</span></div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Transactions"</span><span class="net-detail-value">{format!("{}", o.transaction_count)}</span></div>
                                    <div class="net-detail-row"><span class="net-detail-label">"Documents"</span><span class="net-detail-value">{format!("{}", o.document_count)}</span></div>
                                </div>
                                <div class="net-detail-section">
                                    <div class="net-detail-section-title">"Channels"</div>
                                    <div class="net-detail-channels">
                                        {o.channels.iter().map(|ch| view! { <span class="net-channel-tag">{ch.clone()}</span> }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                                <div class="net-detail-actions">
                                    <button class="net-detail-btn" on:click=move |_| app_store.update(|s| s.expand_tab(TabType::Transactions))>"View Transactions"</button>
                                    <button class="net-detail-btn" on:click=move |_| app_store.update(|s| s.expand_tab(TabType::Reporting))>"View Reports"</button>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
