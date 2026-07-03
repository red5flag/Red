use crate::models::{default_permissions_for_role, PaymentSettings, User, UserActivity, UserAssignment};
use crate::stores::use_app_store;
use crate::types::{PaymentInterval, PaymentMethod, TabType, UserRole};
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
enum NetTab {
    Contacts,
    ExternalOrgs,
    Channels,
    Partners,
    Clients,
    Suppliers,
    Integrations,
    RelationshipMap,
    RelationshipHistory,
}

impl NetTab {
    fn label(&self) -> &'static str {
        match self {
            NetTab::Contacts => "Contacts",
            NetTab::ExternalOrgs => "External Organizations",
            NetTab::Channels => "Channels",
            NetTab::Partners => "Partners",
            NetTab::Clients => "Clients",
            NetTab::Suppliers => "Suppliers / Vendors",
            NetTab::Integrations => "Integrations",
            NetTab::RelationshipMap => "Relationship Map",
            NetTab::RelationshipHistory => "Relationship History",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum RelationshipStatus {
    Active,
    Inactive,
    Pending,
    Suspended,
    Archived,
}

impl RelationshipStatus {
    fn label(&self) -> &'static str {
        match self {
            RelationshipStatus::Active => "Active",
            RelationshipStatus::Inactive => "Inactive",
            RelationshipStatus::Pending => "Pending",
            RelationshipStatus::Suspended => "Suspended",
            RelationshipStatus::Archived => "Archived",
        }
    }
    fn css_class(&self) -> &'static str {
        match self {
            RelationshipStatus::Active => "net-status-active",
            RelationshipStatus::Inactive => "net-status-inactive",
            RelationshipStatus::Pending => "net-status-pending",
            RelationshipStatus::Suspended => "net-status-suspended",
            RelationshipStatus::Archived => "net-status-archived",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum RiskLevel { Low, Medium, High, Unknown }

impl RiskLevel {
    fn label(&self) -> &'static str {
        match self { RiskLevel::Low => "Low", RiskLevel::Medium => "Medium", RiskLevel::High => "High", RiskLevel::Unknown => "Unknown" }
    }
    fn css_class(&self) -> &'static str {
        match self { RiskLevel::Low => "net-risk-low", RiskLevel::Medium => "net-risk-medium", RiskLevel::High => "net-risk-high", RiskLevel::Unknown => "net-risk-unknown" }
    }
}

#[derive(Clone, Debug)]
struct Channel {
    id: Uuid,
    name: String,
    channel_type: String,
    address: Option<String>,
    status: RelationshipStatus,
    linked_contact: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalContact {
    id: Uuid,
    name: String,
    title: String,
    company: String,
    email: Option<String>,
    phone: Option<String>,
    relationship_type: String,
    status: RelationshipStatus,
    risk_level: RiskLevel,
    linked_portfolios: Vec<String>,
    linked_transactions: Vec<String>,
    linked_reports: Vec<String>,
    last_message: Option<String>,
    last_transaction: Option<String>,
    channels: Vec<String>,
    avatar_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalOrganization {
    id: Uuid,
    name: String,
    org_type: String,
    primary_contact: Option<String>,
    status: RelationshipStatus,
    risk_level: RiskLevel,
    linked_portfolios: Vec<String>,
    transaction_count: usize,
    document_count: usize,
    channels: Vec<String>,
    avatar_url: Option<String>,
}

#[derive(Clone, Debug)]
struct Integration {
    id: Uuid,
    name: String,
    integration_type: String,
    status: RelationshipStatus,
    last_sync: Option<String>,
    description: String,
}

#[derive(Clone, Debug)]
struct RelationshipEvent {
    id: Uuid,
    entity_name: String,
    event_description: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    event_type: String,
}

fn mock_contacts() -> Vec<ExternalContact> {
    vec![
        ExternalContact { id: Uuid::new_v4(), name: "Sarah Jones".into(), title: "Supplier Contact".into(), company: "ABC Supplies".into(), email: Some("sarah@abcsupplies.com".into()), phone: Some("+1-555-0201".into()), relationship_type: "Supplier".into(), status: RelationshipStatus::Active, risk_level: RiskLevel::Medium, linked_portfolios: vec!["Commercial Real Estate".into(), "RedOrg Large Scale".into()], linked_transactions: vec!["TX-1048".into(), "TX-1032".into()], linked_reports: vec!["June Supplier Report".into()], last_message: Some("2 days ago".into()), last_transaction: Some("Invoice 1048".into()), channels: vec!["Email".into(), "Phone".into(), "Supplier Portal".into()], avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=SarahJones".into()) },
        ExternalContact { id: Uuid::new_v4(), name: "Mark Taylor".into(), title: "Account Manager".into(), company: "ABC Supplies".into(), email: Some("mark@abcsupplies.com".into()), phone: Some("+1-555-0202".into()), relationship_type: "Supplier".into(), status: RelationshipStatus::Active, risk_level: RiskLevel::Low, linked_portfolios: vec!["Commercial Real Estate".into()], linked_transactions: vec!["TX-1041".into()], linked_reports: vec![], last_message: Some("5 days ago".into()), last_transaction: Some("Invoice 1041".into()), channels: vec!["Email".into(), "Phone".into()], avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=MarkTaylor".into()) },
        ExternalContact { id: Uuid::new_v4(), name: "Emma Wilson".into(), title: "Broker".into(), company: "ABC Realty".into(), email: Some("emma@abcrealty.com".into()), phone: Some("+1-555-0301".into()), relationship_type: "Partner".into(), status: RelationshipStatus::Active, risk_level: RiskLevel::Low, linked_portfolios: vec!["Commercial Real Estate".into()], linked_transactions: vec![], linked_reports: vec!["Q2 Market Analysis".into()], last_message: Some("1 week ago".into()), last_transaction: None, channels: vec!["Email".into(), "LinkedIn".into()], avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=EmmaWilson".into()) },
        ExternalContact { id: Uuid::new_v4(), name: "David Chen".into(), title: "Client".into(), company: "Chen Investments".into(), email: Some("david@cheninv.com".into()), phone: Some("+1-555-0401".into()), relationship_type: "Client".into(), status: RelationshipStatus::Active, risk_level: RiskLevel::Low, linked_portfolios: vec!["RedOrg Large Scale".into()], linked_transactions: vec!["TX-1050".into()], linked_reports: vec!["Investment Summary".into()], last_message: Some("3 days ago".into()), last_transaction: Some("Dividend payment".into()), channels: vec!["Email".into(), "Investor Portal".into()], avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=DavidChen".into()) },
        ExternalContact { id: Uuid::new_v4(), name: "Lisa Garcia".into(), title: "Legal Partner".into(), company: "XYZ Legal".into(), email: Some("lisa@xyzlegal.com".into()), phone: Some("+1-555-0501".into()), relationship_type: "Partner".into(), status: RelationshipStatus::Pending, risk_level: RiskLevel::Low, linked_portfolios: vec!["Commercial Real Estate".into()], linked_transactions: vec![], linked_reports: vec!["Contract Review".into()], last_message: None, last_transaction: None, channels: vec!["Email".into(), "Phone".into()], avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=LisaGarcia".into()) },
    ]
}

fn mock_external_orgs() -> Vec<ExternalOrganization> {
    vec![
        ExternalOrganization { id: Uuid::new_v4(), name: "ABC Supplies".into(), org_type: "Vendor".into(), primary_contact: Some("Sarah Jones".into()), status: RelationshipStatus::Active, risk_level: RiskLevel::Medium, linked_portfolios: vec!["Commercial Real Estate".into(), "RedOrg Large Scale".into()], transaction_count: 12, document_count: 4, channels: vec!["Email".into(), "Supplier Portal".into(), "Phone".into()], avatar_url: None },
        ExternalOrganization { id: Uuid::new_v4(), name: "ABC Realty".into(), org_type: "Partner".into(), primary_contact: Some("Emma Wilson".into()), status: RelationshipStatus::Active, risk_level: RiskLevel::Low, linked_portfolios: vec!["Commercial Real Estate".into()], transaction_count: 3, document_count: 2, channels: vec!["Email".into(), "LinkedIn".into()], avatar_url: None },
        ExternalOrganization { id: Uuid::new_v4(), name: "Chen Investments".into(), org_type: "Client".into(), primary_contact: Some("David Chen".into()), status: RelationshipStatus::Active, risk_level: RiskLevel::Low, linked_portfolios: vec!["RedOrg Large Scale".into()], transaction_count: 8, document_count: 5, channels: vec!["Email".into(), "Investor Portal".into()], avatar_url: None },
        ExternalOrganization { id: Uuid::new_v4(), name: "XYZ Legal".into(), org_type: "Partner".into(), primary_contact: Some("Lisa Garcia".into()), status: RelationshipStatus::Pending, risk_level: RiskLevel::Low, linked_portfolios: vec!["Commercial Real Estate".into()], transaction_count: 0, document_count: 1, channels: vec!["Email".into(), "Phone".into()], avatar_url: None },
        ExternalOrganization { id: Uuid::new_v4(), name: "Gold Coast Maintenance Co".into(), org_type: "Vendor".into(), primary_contact: None, status: RelationshipStatus::Active, risk_level: RiskLevel::Medium, linked_portfolios: vec!["Commercial Real Estate".into()], transaction_count: 6, document_count: 3, channels: vec!["Phone".into(), "Email".into()], avatar_url: None },
    ]
}

fn mock_channels() -> Vec<Channel> {
    vec![
        Channel { id: Uuid::new_v4(), name: "Email".into(), channel_type: "Communication".into(), address: Some("contact@redorg.com".into()), status: RelationshipStatus::Active, linked_contact: Some("Sarah Jones".into()) },
        Channel { id: Uuid::new_v4(), name: "Phone".into(), channel_type: "Communication".into(), address: Some("+1-555-0100".into()), status: RelationshipStatus::Active, linked_contact: Some("Mark Taylor".into()) },
        Channel { id: Uuid::new_v4(), name: "Supplier Portal".into(), channel_type: "Platform".into(), address: Some("portal.abcsupplies.com".into()), status: RelationshipStatus::Active, linked_contact: Some("Sarah Jones".into()) },
        Channel { id: Uuid::new_v4(), name: "Investor Portal".into(), channel_type: "Platform".into(), address: Some("invest.redorg.com".into()), status: RelationshipStatus::Active, linked_contact: Some("David Chen".into()) },
        Channel { id: Uuid::new_v4(), name: "LinkedIn".into(), channel_type: "Social".into(), address: None, status: RelationshipStatus::Active, linked_contact: Some("Emma Wilson".into()) },
        Channel { id: Uuid::new_v4(), name: "Banking Channel".into(), channel_type: "Financial".into(), address: Some("ops.redorg.com".into()), status: RelationshipStatus::Active, linked_contact: None },
        Channel { id: Uuid::new_v4(), name: "Real Estate Platform".into(), channel_type: "Platform".into(), address: Some("realestate.platform.com".into()), status: RelationshipStatus::Active, linked_contact: Some("Emma Wilson".into()) },
        Channel { id: Uuid::new_v4(), name: "Referral Channel".into(), channel_type: "Network".into(), address: None, status: RelationshipStatus::Pending, linked_contact: None },
    ]
}

fn mock_integrations() -> Vec<Integration> {
    vec![
        Integration { id: Uuid::new_v4(), name: "QuickBooks".into(), integration_type: "Accounting".into(), status: RelationshipStatus::Active, last_sync: Some("2 hours ago".into()), description: "Syncs transactions and invoices".into() },
        Integration { id: Uuid::new_v4(), name: "DocuSign".into(), integration_type: "Document".into(), status: RelationshipStatus::Active, last_sync: Some("1 day ago".into()), description: "Electronic signatures for contracts".into() },
        Integration { id: Uuid::new_v4(), name: "Stripe".into(), integration_type: "Payment".into(), status: RelationshipStatus::Active, last_sync: Some("5 minutes ago".into()), description: "Payment processing for client invoices".into() },
        Integration { id: Uuid::new_v4(), name: "Google Calendar".into(), integration_type: "Calendar".into(), status: RelationshipStatus::Active, last_sync: Some("1 hour ago".into()), description: "Syncs calendar events and reminders".into() },
        Integration { id: Uuid::new_v4(), name: "Slack".into(), integration_type: "Communication".into(), status: RelationshipStatus::Suspended, last_sync: Some("3 days ago".into()), description: "Team notifications and alerts".into() },
    ]
}

fn mock_relationship_events() -> Vec<RelationshipEvent> {
    let now = Utc::now();
    vec![
        RelationshipEvent { id: Uuid::new_v4(), entity_name: "ABC Supplies".into(), event_description: "Sarah Jones sent a message".into(), timestamp: now, event_type: "Message".into() },
        RelationshipEvent { id: Uuid::new_v4(), entity_name: "ABC Supplies".into(), event_description: "Transaction TX-1048 was approved".into(), timestamp: now, event_type: "Transaction".into() },
        RelationshipEvent { id: Uuid::new_v4(), entity_name: "ABC Supplies".into(), event_description: "Payee bank details were updated".into(), timestamp: now - chrono::Duration::days(1), event_type: "Banking".into() },
        RelationshipEvent { id: Uuid::new_v4(), entity_name: "ABC Supplies".into(), event_description: "Linked to Commercial Real Estate Portfolio".into(), timestamp: now - chrono::Duration::days(1), event_type: "Portfolio".into() },
        RelationshipEvent { id: Uuid::new_v4(), entity_name: "XYZ Legal".into(), event_description: "Supplier contract document was locked".into(), timestamp: now - chrono::Duration::days(7), event_type: "Document".into() },
        RelationshipEvent { id: Uuid::new_v4(), entity_name: "Chen Investments".into(), event_description: "New external organization added".into(), timestamp: now - chrono::Duration::days(3), event_type: "Organization".into() },
        RelationshipEvent { id: Uuid::new_v4(), entity_name: "Slack".into(), event_description: "Integration suspended - authentication failed".into(), timestamp: now - chrono::Duration::days(3), event_type: "Integration".into() },
    ]
}

#[component]
pub fn NetworkingPage() -> impl IntoView {
    let app_store = use_app_store();
    let (active_tab, set_active_tab) = signal(NetTab::Contacts);
    let (search_query, set_search_query) = signal(String::new());
    let (selected_contact, set_selected_contact) = signal::<Option<ExternalContact>>(None);
    let (selected_org, set_selected_org) = signal::<Option<ExternalOrganization>>(None);
    let (_edit_mode, _set_edit_mode) = signal(false);

    let contacts = StoredValue::new(mock_contacts());
    let external_orgs = StoredValue::new(mock_external_orgs());
    let channels = StoredValue::new(mock_channels());
    let integrations = StoredValue::new(mock_integrations());
    let events = StoredValue::new(mock_relationship_events());

    let contacts_count = contacts.get_value().len();
    let orgs_count = external_orgs.get_value().len();
    let channels_count = channels.get_value().len();
    let integrations_count = integrations.get_value().len();

    let filtered_contacts = Memo::new(move |_| {
        let q = search_query.get().to_lowercase();
        contacts.get_value().iter().filter(|c| {
            q.is_empty() || c.name.to_lowercase().contains(&q) || c.company.to_lowercase().contains(&q)
        }).cloned().collect::<Vec<_>>()
    });

    let filtered_orgs = Memo::new(move |_| {
        let q = search_query.get().to_lowercase();
        external_orgs.get_value().iter().filter(|o| {
            q.is_empty() || o.name.to_lowercase().contains(&q) || o.org_type.to_lowercase().contains(&q)
        }).cloned().collect::<Vec<_>>()
    });

    let all_tabs = [
        NetTab::Contacts, NetTab::ExternalOrgs, NetTab::Channels,
        NetTab::Partners, NetTab::Clients, NetTab::Suppliers,
        NetTab::Integrations, NetTab::RelationshipMap, NetTab::RelationshipHistory,
    ];

    let render_tab_content = move || {
        let tab = active_tab.get();
        match tab {
            NetTab::Contacts => render_contacts(filtered_contacts.get(), set_selected_contact).into_any(),
            NetTab::ExternalOrgs => render_external_orgs(filtered_orgs.get(), set_selected_org).into_any(),
            NetTab::Channels => render_channels(&channels.get_value()).into_any(),
            NetTab::Partners => render_by_type(&contacts.get_value(), &external_orgs.get_value(), "Partner").into_any(),
            NetTab::Clients => render_by_type(&contacts.get_value(), &external_orgs.get_value(), "Client").into_any(),
            NetTab::Suppliers => render_by_type(&contacts.get_value(), &external_orgs.get_value(), "Supplier").into_any(),
            NetTab::Integrations => render_integrations(&integrations.get_value()).into_any(),
            NetTab::RelationshipMap => render_relationship_map(&contacts.get_value(), &external_orgs.get_value()).into_any(),
            NetTab::RelationshipHistory => render_relationship_history(&events.get_value()).into_any(),
        }
    };

    view! {
        <div class="home-screen">
            // Search bar
            <div class="net-search-bar">
                <input
                    class="net-search-input"
                    type="text"
                    placeholder="Search contacts, companies, channels..."
                    prop:value=move || search_query.get()
                    on:input=move |ev| set_search_query.set(event_target_value(&ev))
                />
            </div>

            // Quick filter tabs
            <div class="net-quick-tabs">
                {all_tabs.iter().map(|t| {
                    let tab = t.clone();
                    let tab_for_click = t.clone();
                    let label = t.label().to_string();
                    let label_for_aria = label.clone();
                    view! {
                        <button
                            class="net-quick-tab"
                            class:active={move || active_tab.get() == tab}
                            on:click=move |_| set_active_tab.set(tab_for_click.clone())
                            aria-label={label_for_aria}
                        >
                            {label}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Metrics bar
            <div class="org-metrics-bar">
                <div class="org-metric">
                    <div class="org-metric-value">{contacts_count}</div>
                    <div class="org-metric-label">"Contacts"</div>
                </div>
                <div class="org-metric">
                    <div class="org-metric-value">{orgs_count}</div>
                    <div class="org-metric-label">"Organizations"</div>
                </div>
                <div class="org-metric">
                    <div class="org-metric-value">{channels_count}</div>
                    <div class="org-metric-label">"Channels"</div>
                </div>
                <div class="org-metric">
                    <div class="org-metric-value">{integrations_count}</div>
                    <div class="org-metric-label">"Integrations"</div>
                </div>
            </div>

            // Action bar
            <div class="net-action-bar">
                <button class="net-action-btn" on:click=move |_| app_store.update(|s| s.toggle_networking_add_member())>"Add Contact"</button>
                <button class="net-action-btn" on:click=move |_| app_store.update(|s| s.open_search())>"Search"</button>
                <button class="net-action-btn" on:click=move |_| app_store.update(|s| s.set_message_drawer(true))>"Messages"</button>
            </div>

            // Tab content
            {render_tab_content}

            // Contact detail modal
            {move || selected_contact.get().map(|c| {
                let on_close = move |_| set_selected_contact.set(None);
                let on_message = {
                    let _name = c.name.clone();
                    move |_| { app_store.update(|s| s.set_message_drawer(true)); }
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

fn render_contacts(
    contacts: Vec<ExternalContact>,
    set_selected: WriteSignal<Option<ExternalContact>>,
) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            {if contacts.is_empty() {
                view! {
                    <div class="data-card">
                        <div class="empty-state"><div class="empty-text">"No contacts found"</div></div>
                    </div>
                }.into_any()
            } else {
                contacts.into_iter().map(|c| {
                    let c_for_click = c.clone();
                    let c_for_msg = c.clone();
                    let status_cls = c.status.css_class();
                    let risk_cls = c.risk_level.css_class();
                    view! {
                        <div class="net-relationship-card" on:click=move |_| set_selected.set(Some(c_for_click.clone()))>
                            <img class="net-rel-avatar" src={c.avatar_url.clone().unwrap_or_default()} alt={c.name.clone()} />
                            <div class="net-rel-body">
                                <div class="net-rel-name">{c.name.clone()}</div>
                                <div class="net-rel-type">{format!("{} • {}", c.title, c.company)}</div>
                                <div class="net-rel-meta">
                                    <span class={format!("net-rel-status {}", status_cls)}>{c.status.label()}</span>
                                    <span class={format!("net-rel-risk {}", risk_cls)}>{format!("Risk: {}", c.risk_level.label())}</span>
                                </div>
                                <div class="net-rel-linked">
                                    {format!("Portfolios: {} • Transactions: {} • Reports: {}",
                                        c.linked_portfolios.len(), c.linked_transactions.len(), c.linked_reports.len())}
                                </div>
                                {c.last_message.as_ref().map(|m| view! {
                                    <div class="net-rel-activity">{format!("Last message: {}", m)}</div>
                                }.into_any()).unwrap_or_else(|| ().into_any())}
                            </div>
                            <div class="net-rel-actions" on:click=|ev| ev.stop_propagation()>
                                <button class="net-rel-btn" on:click=move |_| {
                                    use_app_store().update(|s| s.set_message_drawer(true));
                                }>"Message"</button>
                                <button class="net-rel-btn" on:click=move |_| set_selected.set(Some(c_for_msg.clone()))>"View"</button>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>().into_any()
            }}
        </div>
    }
}

fn render_external_orgs(
    orgs: Vec<ExternalOrganization>,
    set_selected: WriteSignal<Option<ExternalOrganization>>,
) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            {if orgs.is_empty() {
                view! {
                    <div class="data-card">
                        <div class="empty-state"><div class="empty-text">"No external organizations found"</div></div>
                    </div>
                }.into_any()
            } else {
                orgs.into_iter().map(|o| {
                    let o_for_click = o.clone();
                    let o_for_btn = o.clone();
                    let status_cls = o.status.css_class();
                    let risk_cls = o.risk_level.css_class();
                    let initial = o.name.chars().next().unwrap_or('A');
                    view! {
                        <div class="net-relationship-card" on:click=move |_| set_selected.set(Some(o_for_click.clone()))>
                            <div class="net-rel-avatar net-rel-avatar-org">{initial}</div>
                            <div class="net-rel-body">
                                <div class="net-rel-name">{o.name.clone()}</div>
                                <div class="net-rel-type">{o.org_type.clone()}</div>
                                <div class="net-rel-meta">
                                    <span class={format!("net-rel-status {}", status_cls)}>{o.status.label()}</span>
                                    <span class={format!("net-rel-risk {}", risk_cls)}>{format!("Risk: {}", o.risk_level.label())}</span>
                                </div>
                                <div class="net-rel-linked">
                                    {format!("Portfolios: {} • Transactions: {} • Documents: {}",
                                        o.linked_portfolios.len(), o.transaction_count, o.document_count)}
                                </div>
                                {o.primary_contact.as_ref().map(|p| view! {
                                    <div class="net-rel-activity">{format!("Primary contact: {}", p)}</div>
                                }.into_any()).unwrap_or_else(|| ().into_any())}
                            </div>
                            <div class="net-rel-actions" on:click=|ev| ev.stop_propagation()>
                                <button class="net-rel-btn" on:click=move |_| set_selected.set(Some(o_for_btn.clone()))>"View"</button>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>().into_any()
            }}
        </div>
    }
}

fn render_channels(channels: &[Channel]) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            <div class="net-channels-grid">
                {channels.iter().map(|ch| {
                    let status_cls = ch.status.css_class();
                    let icon = match ch.channel_type.as_str() {
                        "Communication" => "📧",
                        "Platform" => "🌐",
                        "Social" => "👤",
                        "Financial" => "🏦",
                        "Network" => "🔗",
                        _ => "📡",
                    };
                    view! {
                        <div class="net-channel-card">
                            <div class="net-channel-icon">{icon}</div>
                            <div class="net-channel-name">{ch.name.clone()}</div>
                            <div class="net-channel-type">{ch.channel_type.clone()}</div>
                            {ch.address.as_ref().map(|a| view! {
                                <div class="net-channel-addr">{a.clone()}</div>
                            }.into_any()).unwrap_or_else(|| ().into_any())}
                            <span class={format!("net-rel-status {}", status_cls)}>{ch.status.label()}</span>
                            {ch.linked_contact.as_ref().map(|c| view! {
                                <div class="net-channel-contact">{format!("Linked: {}", c)}</div>
                            }.into_any()).unwrap_or_else(|| ().into_any())}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

fn render_by_type(
    contacts: &[ExternalContact],
    orgs: &[ExternalOrganization],
    rel_type: &str,
) -> impl IntoView {
    let filtered_contacts: Vec<_> = contacts.iter().filter(|c| c.relationship_type == rel_type).cloned().collect();
    let filtered_orgs: Vec<_> = orgs.iter().filter(|o| {
        let ot = o.org_type.to_lowercase();
        match rel_type {
            "Supplier" => ot == "vendor" || ot == "supplier",
            "Partner" => ot == "partner",
            "Client" => ot == "client",
            _ => false,
        }
    }).cloned().collect();

    view! {
        <div class="net-tab-content">
            {if filtered_contacts.is_empty() && filtered_orgs.is_empty() {
                view! {
                    <div class="data-card">
                        <div class="empty-state"><div class="empty-text">{format!("No {} found", rel_type)}</div></div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div>
                        {if !filtered_orgs.is_empty() {
                            view! {
                                <div class="net-section-title">"Organizations"</div>
                                <div class="net-cards-list">
                                    {filtered_orgs.iter().map(|o| {
                                        let status_cls = o.status.css_class();
                                        let initial = o.name.chars().next().unwrap_or('A');
                                        view! {
                                            <div class="net-relationship-card">
                                                <div class="net-rel-avatar net-rel-avatar-org">{initial}</div>
                                                <div class="net-rel-body">
                                                    <div class="net-rel-name">{o.name.clone()}</div>
                                                    <div class="net-rel-type">{o.org_type.clone()}</div>
                                                    <div class="net-rel-meta">
                                                        <span class={format!("net-rel-status {}", status_cls)}>{o.status.label()}</span>
                                                    </div>
                                                    <div class="net-rel-linked">
                                                        {format!("Portfolios: {} • Transactions: {}",
                                                            o.linked_portfolios.len(), o.transaction_count)}
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        } else { ().into_any() }}
                        {if !filtered_contacts.is_empty() {
                            view! {
                                <div class="net-section-title">"Contacts"</div>
                                <div class="net-cards-list">
                                    {filtered_contacts.iter().map(|c| {
                                        let status_cls = c.status.css_class();
                                        view! {
                                            <div class="net-relationship-card">
                                                <img class="net-rel-avatar" src={c.avatar_url.clone().unwrap_or_default()} alt={c.name.clone()} />
                                                <div class="net-rel-body">
                                                    <div class="net-rel-name">{c.name.clone()}</div>
                                                    <div class="net-rel-type">{format!("{} • {}", c.title, c.company)}</div>
                                                    <div class="net-rel-meta">
                                                        <span class={format!("net-rel-status {}", status_cls)}>{c.status.label()}</span>
                                                    </div>
                                                    <div class="net-rel-linked">
                                                        {format!("Portfolios: {} • Transactions: {}",
                                                            c.linked_portfolios.len(), c.linked_transactions.len())}
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

fn render_integrations(integrations: &[Integration]) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            <div class="net-cards-list">
                {integrations.iter().map(|i| {
                    let status_cls = i.status.css_class();
                    let icon = match i.integration_type.as_str() {
                        "Accounting" => "📊",
                        "Document" => "📄",
                        "Payment" => "💳",
                        "Calendar" => "📅",
                        "Communication" => "💬",
                        _ => "⚙️",
                    };
                    view! {
                        <div class="net-relationship-card">
                            <div class="net-rel-avatar net-rel-avatar-int">{icon}</div>
                            <div class="net-rel-body">
                                <div class="net-rel-name">{i.name.clone()}</div>
                                <div class="net-rel-type">{i.integration_type.clone()}</div>
                                <div class="net-rel-meta">
                                    <span class={format!("net-rel-status {}", status_cls)}>{i.status.label()}</span>
                                </div>
                                <div class="net-rel-linked">{i.description.clone()}</div>
                                {i.last_sync.as_ref().map(|s| view! {
                                    <div class="net-rel-activity">{format!("Last sync: {}", s)}</div>
                                }.into_any()).unwrap_or_else(|| ().into_any())}
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

fn render_relationship_map(
    contacts: &[ExternalContact],
    orgs: &[ExternalOrganization],
) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Relationship Map"</span>
                </div>
                <div class="net-rel-map">
                    <div class="net-rel-map-center">
                        <div class="net-rel-map-core">"RedOrg"</div>
                    </div>
                    {orgs.iter().map(|o| {
                        let status_cls = o.status.css_class();
                        view! {
                            <div class="net-rel-map-node">
                                <div class="net-rel-map-node-name">{o.name.clone()}</div>
                                <div class="net-rel-map-node-type">{o.org_type.clone()}</div>
                                <span class={format!("net-rel-status {}", status_cls)}>{o.status.label()}</span>
                                <div class="net-rel-map-contacts">
                                    {contacts.iter().filter(|c| c.company == o.name).map(|c| {
                                        view! { <div class="net-rel-map-contact">{c.name.clone()}</div> }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}

fn render_relationship_history(events: &[RelationshipEvent]) -> impl IntoView {
    let now = Utc::now();
    let today: Vec<_> = events.iter().filter(|e| (now - e.timestamp).num_hours() < 24).cloned().collect();
    let earlier: Vec<_> = events.iter().filter(|e| (now - e.timestamp).num_hours() >= 24).cloned().collect();

    let render_event = |e: &RelationshipEvent| {
        let icon = match e.event_type.as_str() {
            "Message" => "💬",
            "Transaction" => "💰",
            "Banking" => "🏦",
            "Portfolio" => "📊",
            "Document" => "📄",
            "Organization" => "🏢",
            "Integration" => "⚙️",
            _ => "📌",
        };
        view! {
            <div class="net-history-item">
                <span class="net-history-icon">{icon}</span>
                <div class="net-history-body">
                    <div class="net-history-desc">{e.event_description.clone()}</div>
                    <div class="net-history-meta">
                                        {format!("{} • {}", e.entity_name, e.timestamp.format("%d %b %Y"))}
                    </div>
                </div>
            </div>
        }
    };

    view! {
        <div class="net-tab-content">
            {if !today.is_empty() {
                view! {
                    <div class="net-history-section">
                        <div class="net-history-day">"Today"</div>
                        {today.iter().map(render_event).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else { ().into_any() }}
            {if !earlier.is_empty() {
                view! {
                    <div class="net-history-section">
                        <div class="net-history-day">"Earlier"</div>
                        {earlier.iter().map(render_event).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else { ().into_any() }}
            {if today.is_empty() && earlier.is_empty() {
                view! {
                    <div class="data-card">
                        <div class="empty-state"><div class="empty-text">"No relationship history"</div></div>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}

#[component]
pub fn AddTeamMemberPage() -> impl IntoView {
    let app_store = use_app_store();

    let (search_query, set_search_query) = signal(String::new());
    let (new_name, set_new_name) = signal(String::new());
    let (new_username, set_new_username) = signal(String::new());
    let (new_email, set_new_email) = signal(String::new());
    let (new_role, set_new_role) = signal(UserRole::Worker);

    let add_user = move |name: String, email: String, username: Option<String>, role: UserRole| {
        let name = name.trim().to_string();
        let email = email.trim().to_string();
        if name.is_empty() || email.is_empty() { return; }
        let username = username.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let avatar = format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", username.as_ref().unwrap_or(&name));
        let mut user = User::new(name, email, role);
        user.username = username;
        user.avatar_url = Some(avatar);
        let store = app_store.get();
        user.organization_id = store.current_organization_id.or(store.current_user.organization_id);
        drop(store);
        app_store.update(|s| s.add_organization_user(user));
    };

    let on_add_user = move |_| {
        let username = new_username.get().trim().to_string();
        add_user(new_name.get(), new_email.get(), Some(username), new_role.get());
        set_new_name.set(String::new());
        set_new_username.set(String::new());
        set_new_email.set(String::new());
        set_new_role.set(UserRole::Worker);
    };

    let on_add_found = move |name: String, email: String, username: Option<String>| {
        add_user(name, email, username, new_role.get());
    };

    let search_results = Memo::new(move |_| {
        let query = search_query.get().trim().to_lowercase();
        if query.len() < 2 {
            return Vec::<User>::new();
        }
        let store = app_store.get();
        let mut results: Vec<User> = Vec::new();
        let current_org = store.current_organization_id.or(store.current_user.organization_id);
        let existing_ids: std::collections::HashSet<Uuid> = store.organization_users.iter().map(|u| u.id).collect();

        // Local users from credential store
        for cred in store.credentials.credentials.values() {
            let name = cred.display_name.to_lowercase();
            let email = cred.email.to_lowercase();
            let username = cred.username.to_lowercase();
            if name.contains(&query) || email.contains(&query) || username.contains(&query) {
                let mut user = User::new(cred.display_name.clone(), cred.email.clone(), UserRole::Guest);
                user.username = Some(cred.username.clone());
                user.avatar_url = Some(format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", cred.username));
                user.organization_id = current_org;
                if !existing_ids.contains(&user.id) {
                    results.push(user);
                }
            }
        }

        // Server/online users already known to the app
        for user in store.organization_users.iter() {
            let name = user.name.to_lowercase();
            let email = user.email.to_lowercase();
            let username = user.username.clone().unwrap_or_default().to_lowercase();
            if name.contains(&query) || email.contains(&query) || username.contains(&query) {
                if !results.iter().any(|u| u.email == user.email) {
                    results.push(user.clone());
                }
            }
        }

        // Mock server users representing people available on the server but not yet in the org
        let server_pool = vec![
            User::new("Alice Chen".to_string(), "alice@company.com".to_string(), UserRole::Manager),
            User::new("Bob Martinez".to_string(), "bob@company.com".to_string(), UserRole::Worker),
            User::new("Carol White".to_string(), "carol@company.com".to_string(), UserRole::Director),
            User::new("David Kim".to_string(), "david@company.com".to_string(), UserRole::Contractor),
        ];
        for mut user in server_pool {
            let name = user.name.to_lowercase();
            let email = user.email.to_lowercase();
            if name.contains(&query) || email.contains(&query) {
                user.username = Some(format!("{}", user.id.to_string().split_at(8).0));
                user.avatar_url = Some(format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", user.name));
                user.organization_id = current_org;
                if !existing_ids.contains(&user.id) && !results.iter().any(|u| u.email == user.email) {
                    results.push(user);
                }
            }
        }

        results
    });

    view! {
        <div class="home-screen">
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Find Team Member"</span>
                </div>
                <div class="form-group">
                    <label class="form-label">"Search"</label>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="Search by name, email, or username"
                        prop:value={move || search_query.get()}
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                </div>
                {move || {
                    let results = search_results.get();
                    if results.is_empty() {
                        if search_query.get().trim().len() >= 2 {
                            view! { <div class="list-item"><div class="list-item-left"><div class="list-item-subtitle">"No matching users found"</div></div></div> }.into_any()
                        } else {
                            ().into_any()
                        }
                    } else {
                        view! {
                            <div>
                                <div class="net-filter-label">"Results from local + server"</div>
                                {results.into_iter().map(|u| {
                                    let name = u.name.clone();
                                    let email = u.email.clone();
                                    let username = u.username.clone();
                                    let role = format!("{:?}", u.role);
                                    view! {
                                        <div class="list-item">
                                            <div class="list-item-left">
                                                <div class="list-item-title">{name.clone()}</div>
                                                <div class="list-item-subtitle">{format!("{} • {}", email.clone(), role)}</div>
                                            </div>
                                            <div class="list-item-right">
                                                <button class="net-action-btn" on:click=move |_| on_add_found(name.clone(), email.clone(), username.clone())>"Add"</button>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Add Manually"</span>
                </div>
                <div class="form-group">
                    <label class="form-label">"Name"</label>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="Full name"
                        prop:value=new_name
                        on:input=move |ev| set_new_name.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label">"Username"</label>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="Username"
                        prop:value=new_username
                        on:input=move |ev| set_new_username.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label">"Email"</label>
                    <input
                        class="form-input"
                        type="email"
                        placeholder="Email address"
                        prop:value=new_email
                        on:input=move |ev| set_new_email.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label">"Role"</label>
                    <select
                        class="form-select"
                        prop:value={move || format!("{:?}", new_role.get())}
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_new_role.set(match value.as_str() {
                                "Owner" => UserRole::Owner,
                                "Director" => UserRole::Director,
                                "SeniorManager" => UserRole::SeniorManager,
                                "Manager" => UserRole::Manager,
                                "Worker" => UserRole::Worker,
                                "DocumentWorker" => UserRole::DocumentWorker,
                                "Contractor" => UserRole::Contractor,
                                _ => UserRole::Guest,
                            });
                        }
                    >
                        <option value="Owner">"Owner"</option>
                        <option value="Director">"Director"</option>
                        <option value="SeniorManager">"Senior Manager"</option>
                        <option value="Manager">"Manager"</option>
                        <option value="Worker">"Worker"</option>
                        <option value="DocumentWorker">"Document Worker"</option>
                        <option value="Contractor">"Contractor"</option>
                        <option value="Guest">"Guest"</option>
                    </select>
                </div>
                <button class="card-btn" on:click=on_add_user>"Add Member"</button>
            </div>
        </div>
    }
}

fn default_mock_users() -> Vec<User> {
    let org_id = Uuid::new_v4();
    vec![
        User {
            id: Uuid::new_v4(),
            name: "John Smith".to_string(),
            username: Some("jsmith".to_string()),
            email: "john@company.com".to_string(),
            role: UserRole::Owner,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=John".to_string()),
            assignments: vec![UserAssignment {
                target_type: "Portfolio".to_string(),
                target_id: Uuid::new_v4(),
                target_name: "Downtown Properties".to_string(),
                assigned_at: Utc::now(),
                duration_days: Some(365),
                reason: Some("Property oversight".to_string()),
            }],
            activity_log: vec![UserActivity {
                action: "Created".to_string(),
                target_type: "Asset".to_string(),
                target_name: "Downtown Office".to_string(),
                timestamp: Utc::now(),
                reason: Some("New acquisition".to_string()),
            }],
            department: Some("Executive".to_string()),
            phone: Some("+1-555-0100".to_string()),
            address: Some("123 Main St".to_string()),
            hire_date: Some(Utc::now()),
            base_salary: Some(200000.0),
            payment_settings: PaymentSettings {
                payment_method: PaymentMethod::BankTransfer,
                account_details: "****1234".to_string(),
                payment_interval: PaymentInterval::Monthly,
                currency: crate::types::Currency::USD,
                automatic_payout: true,
                payout_threshold: None,
            },
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Owner),
            documents: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: Some(Utc::now()),
            is_active: true,
        },
        User {
            id: Uuid::new_v4(),
            name: "Sarah Johnson".to_string(),
            username: Some("sjohnson".to_string()),
            email: "sarah@company.com".to_string(),
            role: UserRole::Manager,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=Sarah".to_string()),
            assignments: vec![UserAssignment {
                target_type: "Asset Group".to_string(),
                target_id: Uuid::new_v4(),
                target_name: "Fleet Vehicles".to_string(),
                assigned_at: Utc::now(),
                duration_days: Some(180),
                reason: Some("Fleet coordinator".to_string()),
            }],
            activity_log: vec![UserActivity {
                action: "Modified".to_string(),
                target_type: "Asset".to_string(),
                target_name: "Fleet Van #3".to_string(),
                timestamp: Utc::now(),
                reason: Some("Value update".to_string()),
            }],
            department: Some("Operations".to_string()),
            phone: Some("+1-555-0101".to_string()),
            address: None,
            hire_date: Some(Utc::now()),
            base_salary: Some(120000.0),
            payment_settings: PaymentSettings {
                payment_method: PaymentMethod::DirectDeposit,
                account_details: "****5678".to_string(),
                payment_interval: PaymentInterval::BiWeekly,
                currency: crate::types::Currency::USD,
                automatic_payout: true,
                payout_threshold: None,
            },
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Manager),
            documents: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: Some(Utc::now()),
            is_active: true,
        },
        User {
            id: Uuid::new_v4(),
            name: "Mike Williams".to_string(),
            username: Some("mwilliams".to_string()),
            email: "mike@company.com".to_string(),
            role: UserRole::Worker,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=Mike".to_string()),
            assignments: vec![UserAssignment {
                target_type: "Asset".to_string(),
                target_id: Uuid::new_v4(),
                target_name: "Warehouse A".to_string(),
                assigned_at: Utc::now(),
                duration_days: Some(90),
                reason: Some("Maintenance rotation".to_string()),
            }],
            activity_log: vec![UserActivity {
                action: "Updated".to_string(),
                target_type: "Task".to_string(),
                target_name: "Roof repair".to_string(),
                timestamp: Utc::now(),
                reason: Some("Routine maintenance".to_string()),
            }],
            department: Some("Field Operations".to_string()),
            phone: Some("+1-555-0102".to_string()),
            address: None,
            hire_date: Some(Utc::now()),
            base_salary: Some(65000.0),
            payment_settings: PaymentSettings {
                payment_method: PaymentMethod::BankTransfer,
                account_details: "****9012".to_string(),
                payment_interval: PaymentInterval::Weekly,
                currency: crate::types::Currency::USD,
                automatic_payout: true,
                payout_threshold: None,
            },
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Worker),
            documents: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: Some(Utc::now()),
            is_active: true,
        },
        User {
            id: Uuid::new_v4(),
            name: "Guest User".to_string(),
            username: Some("guest".to_string()),
            email: "guest@company.com".to_string(),
            role: UserRole::Guest,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=Guest".to_string()),
            assignments: vec![],
            activity_log: vec![],
            department: Some("External".to_string()),
            phone: None,
            address: None,
            hire_date: None,
            base_salary: None,
            payment_settings: PaymentSettings::default(),
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Guest),
            documents: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            is_active: true,
        },
    ]
}
