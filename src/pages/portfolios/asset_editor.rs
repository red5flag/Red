use crate::models::{
    Asset, AssetRelationship, AssetRelationshipPartyType, AssetRelationshipType,
    AvailabilityStatus, CommercialStatus, ConditionStatus, LifecycleStatus, RelatedParty,
};
use crate::pages::portfolios::asset_placeholder_url;
use crate::stores::use_app_store;
use crate::types::AssetType;
use chrono::{DateTime, NaiveDate, Utc};
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

fn date_to_input(d: Option<DateTime<Utc>>) -> String {
    d.map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn input_to_date(s: &str) -> Option<DateTime<Utc>> {
    if s.trim().is_empty() {
        return None;
    }
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|ndt| ndt.and_utc())
}

fn status_options() -> Vec<&'static str> {
    vec!["Active", "Retired", "Disposed", "Archived", "Draft"]
}

fn availability_options() -> Vec<&'static str> {
    vec![
        "Available",
        "Reserved",
        "Booked",
        "Rented",
        "In use",
        "Unavailable",
    ]
}

fn condition_options() -> Vec<&'static str> {
    vec![
        "New",
        "Excellent",
        "Good",
        "Fair",
        "Poor",
        "Damaged",
        "Unsafe",
    ]
}

fn commercial_options() -> Vec<&'static str> {
    vec![
        "Not offered",
        "Internal use only",
        "Listed for sale",
        "Listed for rent",
        "Listed for booking",
        "Sale pending",
        "Rental active",
        "Sold",
        "Withdrawn",
    ]
}

fn asset_class_options() -> Vec<&'static str> {
    vec![
        "RealEstate",
        "Vehicle",
        "Equipment",
        "Stock",
        "Bond",
        "Commodity",
        "Digital",
        "IntellectualProperty",
        "Channel",
    ]
}

fn relationship_type_options() -> Vec<&'static str> {
    vec![
        "Legal owner",
        "Beneficial owner",
        "Managing organization",
        "Responsible department",
        "Asset manager",
        "Current custodian",
        "Assigned employee",
        "Assigned team",
        "Cost centre",
        "Supplier",
        "Manufacturer",
        "Maintenance provider",
    ]
}

fn party_type_options() -> Vec<&'static str> {
    vec![
        "Organization",
        "Team",
        "Member",
        "Supplier",
        "Service provider",
        "External contact",
    ]
}

fn section_header(
    title: &'static str,
    key: &'static str,
    expanded: ReadSignal<HashSet<String>>,
    set_expanded: WriteSignal<HashSet<String>>,
) -> impl IntoView {
    let is_open = Memo::new(move |_| expanded.get().contains(key));
    view! {
        <button
            type="button"
            class="asset-section-header"
            aria-expanded=move || is_open.get().to_string()
            on:click=move |_| {
                set_expanded.update(|set| {
                    if set.contains(key) {
                        set.remove(key);
                    } else {
                        set.insert(key.to_string());
                    }
                });
            }
        >
            <span>{title}</span>
            <span class="asset-section-chevron">{move || if is_open.get() { "▼" } else { "▶" }}</span>
        </button>
    }
}

fn class_specific_fields(
    asset_type: AssetType,
    local: ReadSignal<Asset>,
    set_local: WriteSignal<Asset>,
    can_edit: Signal<bool>,
) -> impl IntoView {
    let disabled = move || !can_edit.get();
    match asset_type {
        AssetType::Vehicle => view! {
            <div class="asset-form-grid">
                <label class="android-label">"VIN"</label>
                <input
                    type="text"
                    class="android-input"
                    prop:value=move || local.get().classification.vin.clone().unwrap_or_default()
                    on:input=move |ev| set_local.update(|a| a.classification.vin = Some(event_target_value(&ev)))
                    disabled=disabled
                />
                <label class="android-label">"Registration number"</label>
                <input
                    type="text"
                    class="android-input"
                    prop:value=move || local.get().classification.registration_number.clone().unwrap_or_default()
                    on:input=move |ev| set_local.update(|a| a.classification.registration_number = Some(event_target_value(&ev)))
                    disabled=disabled
                />
                <label class="android-label">"Model year"</label>
                <input
                    type="text"
                    class="android-input"
                    prop:value=move || local.get().classification.model_year.clone().unwrap_or_default()
                    on:input=move |ev| set_local.update(|a| a.classification.model_year = Some(event_target_value(&ev)))
                    disabled=disabled
                />
            </div>
        }.into_any(),
        AssetType::RealEstate => view! {
            <div class="asset-form-grid">
                <label class="android-label">"Title / deed"</label>
                <input
                    type="text"
                    class="android-input"
                    prop:value=move || local.get().classification.title.clone().unwrap_or_default()
                    on:input=move |ev| set_local.update(|a| a.classification.title = Some(event_target_value(&ev)))
                    disabled=disabled
                />
                <label class="android-label">"Lot"</label>
                <input
                    type="text"
                    class="android-input"
                    prop:value=move || local.get().classification.lot.clone().unwrap_or_default()
                    on:input=move |ev| set_local.update(|a| a.classification.lot = Some(event_target_value(&ev)))
                    disabled=disabled
                />
                <label class="android-label">"Plan"</label>
                <input
                    type="text"
                    class="android-input"
                    prop:value=move || local.get().classification.plan.clone().unwrap_or_default()
                    on:input=move |ev| set_local.update(|a| a.classification.plan = Some(event_target_value(&ev)))
                    disabled=disabled
                />
            </div>
        }.into_any(),
        AssetType::Equipment => view! {
            <div class="asset-form-grid">
                <label class="android-label">"Serial number"</label>
                <input
                    type="text"
                    class="android-input"
                    prop:value=move || local.get().classification.serial_number.clone().unwrap_or_default()
                    on:input=move |ev| set_local.update(|a| a.classification.serial_number = Some(event_target_value(&ev)))
                    disabled=disabled
                />
                <label class="android-label">"SKU"</label>
                <input
                    type="text"
                    class="android-input"
                    prop:value=move || local.get().classification.sku.clone().unwrap_or_default()
                    on:input=move |ev| set_local.update(|a| a.classification.sku = Some(event_target_value(&ev)))
                    disabled=disabled
                />
            </div>
        }.into_any(),
        _ => view! { <div class="asset-form-grid">"No extra build fields for this class."</div> }.into_any(),
    }
}

#[component]
pub(crate) fn AssetDetailView(
    asset: Asset,
    #[prop(default = None)] portfolio_id: Option<Uuid>,
    #[prop(into)] can_edit: Signal<bool>,
    on_close: Callback<()>,
) -> impl IntoView {
    let app_store = use_app_store();
    let user_id = app_store.get().current_user.id;

    let (local, set_local) = signal(asset);
    let (errors, set_errors) = signal(Vec::<String>::new());
    let (expanded, set_expanded) = signal(HashSet::from_iter([
        "overview".to_string(),
        "identity".to_string(),
        "classification".to_string(),
        "status".to_string(),
        "lifecycle".to_string(),
        "ownership".to_string(),
        "history".to_string(),
    ]));

    let save = move || {
        let updated = local.get_untracked();
        let validation_errors = updated.validate();
        if !validation_errors.is_empty() {
            set_errors.set(validation_errors);
            return;
        }
        set_errors.set(Vec::new());
        app_store.update(|s| {
            let target = if let Some(pid) = portfolio_id {
                s.portfolios.iter_mut().find(|p| p.id == pid)
            } else {
                s.portfolios.first_mut()
            };
            if let Some(p) = target {
                let all: Vec<_> = p
                    .assets
                    .iter_mut()
                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                    .collect();
                for a in all {
                    if a.id == updated.id {
                        *a = updated.clone();
                        break;
                    }
                }
            }
        });
        on_close.run(());
    };

    let disabled = move || !can_edit.get();

    let (new_rel_type, set_new_rel_type) = signal("Asset manager".to_string());
    let (new_rel_party, set_new_rel_party) = signal("".to_string());
    let (new_rel_contact, set_new_rel_contact) = signal("".to_string());
    let (new_rel_party_type, set_new_rel_party_type) = signal("Member".to_string());

    let add_relationship = move || {
        let rel_type_str = new_rel_type.get_untracked();
        let rel_type = AssetRelationshipType::from_str(&rel_type_str).unwrap_or_default();
        let party_type = AssetRelationshipPartyType::from_str(&new_rel_party_type.get_untracked())
            .unwrap_or_default();
        let name = new_rel_party.get_untracked().trim().to_string();
        let contact = new_rel_contact.get_untracked().trim().to_string();
        if name.is_empty() {
            return;
        }
        let user_name = Some(app_store.get().current_user.name.clone());
        let party = RelatedParty {
            name,
            contact: if contact.is_empty() {
                None
            } else {
                Some(contact)
            },
            party_type,
            party_id: None,
        };
        let rel = AssetRelationship {
            id: Uuid::new_v4(),
            relationship_type: rel_type.clone(),
            related_party: party,
            start_date: Some(Utc::now()),
            end_date: None,
            primary_contact: None,
            notes: None,
            active: true,
        };
        set_local.update(|a| {
            let party_name = a
                .relationships
                .iter()
                .find(|r| r.relationship_type == rel_type && r.active)
                .map(|r| r.related_party.name.clone())
                .unwrap_or_default();
            for existing in a.relationships.iter_mut() {
                if existing.relationship_type == rel_type && existing.active {
                    existing.active = false;
                    existing.end_date = Some(Utc::now());
                }
            }
            let previous = if party_name.is_empty() {
                None
            } else {
                Some(party_name.as_str())
            };
            let new_name = rel.related_party.name.as_str();
            if rel_type == AssetRelationshipType::CurrentCustodian {
                a.record_custodian_change(
                    user_id,
                    user_name.clone(),
                    previous,
                    Some(new_name),
                    None,
                );
            } else {
                a.record_ownership_change(
                    user_id,
                    user_name.clone(),
                    rel_type.as_str(),
                    previous,
                    Some(new_name),
                    None,
                );
            }
            a.relationships.push(rel);
        });
        set_new_rel_party.set(String::new());
        set_new_rel_contact.set(String::new());
    };

    view! {
        <div class="asset-detail-view">
            <div class="asset-detail-toolbar">
                <button class="asset-detail-close" type="button" on:click=move |_| on_close.run(())>"Close"</button>
                <span class="asset-detail-title">"Asset editor"</span>
                <button class="asset-detail-save" type="button" on:click=move |_| save() disabled=disabled>"Save"</button>
            </div>

            {move || {
                let errs = errors.get();
                if errs.is_empty() {
                    return view! { <div></div> }.into_any();
                }
                view! {
                    <div class="asset-errors" role="alert" aria-live="polite">
                        {errs.into_iter().map(|e| view! { <div class="asset-error">{e}</div> }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}

            <div class="asset-detail-scroll">
                {section_header("Overview", "overview", expanded, set_expanded)}
                {move || {
                    if !expanded.get().contains("overview") { return view! { <div></div> }.into_any(); }
                    let a = local.get();
                    let image = a.images.first().cloned().unwrap_or_else(|| asset_placeholder_url(&a.asset_type, &a.name));
                    let next_date = a.next_important_date().map(|d| d.format("%d %b %Y").to_string()).unwrap_or_else(|| "—".to_string());
                    let name = a.name.clone();
                    let reference = a.reference_code.clone();
                    let location = a.location.clone();
                    let current_value = a.current_value;
                    let manager = a.current_manager().unwrap_or("—").to_string();
                    let custodian = a.current_custodian().unwrap_or("—").to_string();
                    let badges = a.status_badges();
                    let class_label = a.class_label();
                    view! {
                        <section class="asset-overview">
                            <img src=image alt=move || { let a = local.get(); format!("{} preview", a.name) } class="asset-overview-image" />
                            <div class="asset-overview-meta">
                                <h2 class="asset-overview-name">{name.clone()}</h2>
                                <div class="asset-overview-sub">{reference.unwrap_or_else(|| "No reference".to_string())} " · " {class_label}</div>
                                <div class="asset-overview-statuses" role="list">
                                    {badges.into_iter().map(|(label, value)| view! {
                                        <div class="asset-status-badge" role="listitem">
                                            <span class="asset-status-label">{label} ": "</span>
                                            <span class="asset-status-value">{value}</span>
                                        </div>
                                    }).collect::<Vec<_>>()}
                                </div>
                                <div class="asset-overview-row">
                                    <span>"Location: "</span>
                                    <span>{location.unwrap_or_else(|| "—".to_string())}</span>
                                </div>
                                <div class="asset-overview-row">
                                    <span>"Manager: "</span>
                                    <span>{manager}</span>
                                </div>
                                <div class="asset-overview-row">
                                    <span>"Custodian: "</span>
                                    <span>{custodian}</span>
                                </div>
                                <div class="asset-overview-row">
                                    <span>"Current value: "</span>
                                    <span>{format!("${:.2}", current_value)}</span>
                                </div>
                                <div class="asset-overview-row">
                                    <span>"Next important date: "</span>
                                    <span>{next_date}</span>
                                </div>
                            </div>
                        </section>
                    }.into_any()
                }}

                {section_header("Identity", "identity", expanded, set_expanded)}
                {move || {
                    if !expanded.get().contains("identity") { return view! { <div></div> }.into_any(); }
                    view! {
                        <section class="asset-form-grid">
                            <label class="android-label" for="asset-name">"Asset name"</label>
                            <input id="asset-name" type="text" class="android-input" prop:value=move || local.get().name on:input=move |ev| set_local.update(|a| a.name = event_target_value(&ev)) disabled=disabled />

                            <label class="android-label" for="asset-ref">"Reference code"</label>
                            <input id="asset-ref" type="text" class="android-input" prop:value=move || local.get().reference_code.clone().unwrap_or_default() on:input=move |ev| set_local.update(|a| a.reference_code = Some(event_target_value(&ev))) disabled=disabled />

                            <label class="android-label" for="asset-desc">"Description"</label>
                            <textarea id="asset-desc" class="android-input" rows="3" prop:value=move || local.get().description.clone().unwrap_or_default() on:input=move |ev| set_local.update(|a| a.description = Some(event_target_value(&ev))) disabled=disabled></textarea>

                            <label class="android-label" for="asset-loc">"Location"</label>
                            <input id="asset-loc" type="text" class="android-input" prop:value=move || local.get().location.clone().unwrap_or_default() on:input=move |ev| set_local.update(|a| a.location = Some(event_target_value(&ev))) disabled=disabled />
                        </section>
                    }.into_any()
                }}

                {section_header("Classification & build", "classification", expanded, set_expanded)}
                {move || {
                    if !expanded.get().contains("classification") { return view! { <div></div> }.into_any(); }
                    let asset_type = local.get().asset_type.clone();
                    view! {
                        <section class="asset-form-grid">
                            <label class="android-label" for="asset-class">"Asset class"</label>
                            <select id="asset-class" class="android-select" prop:value=move || local.get().asset_type.to_input_string() on:change=move |ev| set_local.update(|a| a.asset_type = AssetType::from_input(&event_target_value(&ev))) disabled=disabled>
                                {asset_class_options().into_iter().map(|o| view! { <option value=o selected=move || local.get().asset_type.to_input_string() == o>{o}</option> }).collect::<Vec<_>>()}
                            </select>

                            <label class="android-label" for="asset-subtype">"Subtype"</label>
                            <input id="asset-subtype" type="text" class="android-input" prop:value=move || local.get().asset_subtype.clone().unwrap_or_default() on:input=move |ev| set_local.update(|a| a.asset_subtype = Some(event_target_value(&ev))) disabled=disabled />

                            <label class="android-label" for="asset-manufacturer">"Manufacturer / brand"</label>
                            <input id="asset-manufacturer" type="text" class="android-input" prop:value=move || local.get().classification.manufacturer.clone().unwrap_or_default() on:input=move |ev| set_local.update(|a| a.classification.manufacturer = Some(event_target_value(&ev))) disabled=disabled />

                            <label class="android-label" for="asset-model">"Model"</label>
                            <input id="asset-model" type="text" class="android-input" prop:value=move || local.get().classification.model.clone().unwrap_or_default() on:input=move |ev| set_local.update(|a| a.classification.model = Some(event_target_value(&ev))) disabled=disabled />

                            {class_specific_fields(asset_type, local, set_local, can_edit)}
                        </section>
                    }.into_any()
                }}

                {section_header("Status controls", "status", expanded, set_expanded)}
                {move || {
                    if !expanded.get().contains("status") { return view! { <div></div> }.into_any(); }
                    view! {
                        <section class="asset-form-grid">
                            <label class="android-label" for="lifecycle-status">"Lifecycle status"</label>
                            <select id="lifecycle-status" class="android-select" prop:value=move || local.get().lifecycle_status.as_str() on:change=move |ev| {
                                let s = event_target_value(&ev);
                                let user_name = Some(app_store.get().current_user.name.clone());
                                set_local.update(|a| a.set_lifecycle_status(user_id, user_name, LifecycleStatus::from_str(&s).unwrap_or_default(), None));
                            } disabled=disabled>
                                {status_options().into_iter().map(|o| view! { <option value=o selected=move || local.get().lifecycle_status.as_str() == o>{o}</option> }).collect::<Vec<_>>()}
                            </select>

                            <label class="android-label" for="availability-status">"Availability status"</label>
                            <select id="availability-status" class="android-select" prop:value=move || local.get().availability_status.as_str() on:change=move |ev| {
                                let s = event_target_value(&ev);
                                let user_name = Some(app_store.get().current_user.name.clone());
                                set_local.update(|a| a.set_availability_status(user_id, user_name, AvailabilityStatus::from_str(&s).unwrap_or_default(), None));
                            } disabled=disabled>
                                {availability_options().into_iter().map(|o| view! { <option value=o selected=move || local.get().availability_status.as_str() == o>{o}</option> }).collect::<Vec<_>>()}
                            </select>

                            <label class="android-label" for="condition-status">"Condition status"</label>
                            <select id="condition-status" class="android-select" prop:value=move || local.get().condition_status.as_str() on:change=move |ev| {
                                let s = event_target_value(&ev);
                                let user_name = Some(app_store.get().current_user.name.clone());
                                set_local.update(|a| a.set_condition_status(user_id, user_name, ConditionStatus::from_str(&s).unwrap_or_default(), None));
                            } disabled=disabled>
                                {condition_options().into_iter().map(|o| view! { <option value=o selected=move || local.get().condition_status.as_str() == o>{o}</option> }).collect::<Vec<_>>()}
                            </select>

                            <label class="android-label" for="commercial-status">"Commercial status"</label>
                            <select id="commercial-status" class="android-select" prop:value=move || local.get().commercial_status.as_str() on:change=move |ev| {
                                let s = event_target_value(&ev);
                                let user_name = Some(app_store.get().current_user.name.clone());
                                set_local.update(|a| a.set_commercial_status(user_id, user_name, CommercialStatus::from_str(&s).unwrap_or_default(), None));
                            } disabled=disabled>
                                {commercial_options().into_iter().map(|o| view! { <option value=o selected=move || local.get().commercial_status.as_str() == o>{o}</option> }).collect::<Vec<_>>()}
                            </select>

                            <label class="android-label" for="status-reason">"Status reason"</label>
                            <textarea id="status-reason" class="android-input" rows="2" prop:value=move || local.get().status_reason.clone().unwrap_or_default() on:input=move |ev| set_local.update(|a| a.status_reason = Some(event_target_value(&ev))) disabled=disabled></textarea>
                        </section>
                    }.into_any()
                }}

                {section_header("Lifecycle", "lifecycle", expanded, set_expanded)}
                {move || {
                    if !expanded.get().contains("lifecycle") { return view! { <div></div> }.into_any(); }
                    view! {
                        <section class="asset-form-grid">
                            <label class="android-label" for="purchase-date">"Purchase date"</label>
                            <input id="purchase-date" type="date" class="android-input" prop:value=move || date_to_input(Some(local.get().purchase_date)) on:change=move |ev| set_local.update(|a| a.purchase_date = input_to_date(&event_target_value(&ev)).unwrap_or(Utc::now())) disabled=disabled />

                            <label class="android-label" for="commissioning-date">"Commissioning date"</label>
                            <input id="commissioning-date" type="date" class="android-input" prop:value=move || date_to_input(local.get().lifecycle.commissioning_date) on:change=move |ev| set_local.update(|a| a.lifecycle.commissioning_date = input_to_date(&event_target_value(&ev))) disabled=disabled />

                            <label class="android-label" for="warranty-start">"Warranty start"</label>
                            <input id="warranty-start" type="date" class="android-input" prop:value=move || date_to_input(local.get().lifecycle.warranty_start_date) on:change=move |ev| set_local.update(|a| a.lifecycle.warranty_start_date = input_to_date(&event_target_value(&ev))) disabled=disabled />

                            <label class="android-label" for="warranty-expiry">"Warranty expiry"</label>
                            <input id="warranty-expiry" type="date" class="android-input" prop:value=move || date_to_input(local.get().lifecycle.warranty_expiry_date) on:change=move |ev| set_local.update(|a| a.lifecycle.warranty_expiry_date = input_to_date(&event_target_value(&ev))) disabled=disabled />

                            <label class="android-label" for="retirement-date">"Expected retirement"</label>
                            <input id="retirement-date" type="date" class="android-input" prop:value=move || date_to_input(local.get().lifecycle.expected_retirement_date) on:change=move |ev| set_local.update(|a| a.lifecycle.expected_retirement_date = input_to_date(&event_target_value(&ev))) disabled=disabled />

                            <label class="android-label" for="disposal-date">"Disposal date"</label>
                            <input id="disposal-date" type="date" class="android-input" prop:value=move || date_to_input(local.get().lifecycle.disposal_date) on:change=move |ev| set_local.update(|a| a.lifecycle.disposal_date = input_to_date(&event_target_value(&ev))) disabled=disabled />

                            <label class="android-label" for="disposal-method">"Disposal method"</label>
                            <input id="disposal-method" type="text" class="android-input" prop:value=move || local.get().lifecycle.disposal_method.clone().unwrap_or_default() on:input=move |ev| set_local.update(|a| a.lifecycle.disposal_method = Some(event_target_value(&ev))) disabled=disabled />

                            <label class="android-label" for="disposal-value">"Disposal / residual value"</label>
                            <input id="disposal-value" type="number" class="android-input" prop:value=move || local.get().lifecycle.disposal_value.map(|v| v.to_string()).unwrap_or_default() on:change=move |ev| set_local.update(|a| a.lifecycle.disposal_value = event_target_value(&ev).parse::<f64>().ok()) disabled=disabled />
                        </section>
                    }.into_any()
                }}

                {section_header("Ownership & responsibility", "ownership", expanded, set_expanded)}
                {move || {
                    if !expanded.get().contains("ownership") { return view! { <div></div> }.into_any(); }
                    view! {
                        <section>
                            <div class="asset-relationships">
                                {local.get().relationships.iter().filter(|r| r.active).cloned().map(|r| view! {
                                    <div class="asset-relationship-chip">
                                        <span class="asset-relationship-type">{r.relationship_type.as_str()}</span>
                                        <span class="asset-relationship-party">{r.related_party.name.clone()}</span>
                                    </div>
                                }).collect::<Vec<_>>()}
                            </div>

                            <Show when=move || can_edit.get() fallback=|| view! { <div></div> }>
                                <div class="asset-form-grid add-relationship">
                                    <label class="android-label">"Relationship type"</label>
                                    <select class="android-select" prop:value=move || new_rel_type.get() on:change=move |ev| set_new_rel_type.set(event_target_value(&ev))>
                                        {relationship_type_options().into_iter().map(|o| view! { <option value=o selected=move || new_rel_type.get() == o>{o}</option> }).collect::<Vec<_>>()}
                                    </select>
                                    <label class="android-label">"Party type"</label>
                                    <select class="android-select" prop:value=move || new_rel_party_type.get() on:change=move |ev| set_new_rel_party_type.set(event_target_value(&ev))>
                                        {party_type_options().into_iter().map(|o| view! { <option value=o selected=move || new_rel_party_type.get() == o>{o}</option> }).collect::<Vec<_>>()}
                                    </select>
                                    <label class="android-label">"Name / identifier"</label>
                                    <input type="text" class="android-input" prop:value=move || new_rel_party.get() on:input=move |ev| set_new_rel_party.set(event_target_value(&ev)) />
                                    <label class="android-label">"Contact / email"</label>
                                    <input type="text" class="android-input" prop:value=move || new_rel_contact.get() on:input=move |ev| set_new_rel_contact.set(event_target_value(&ev)) />
                                    <button class="android-button" type="button" on:click=move |_| add_relationship()>"Add relationship"</button>
                                </div>
                            </Show>
                        </section>
                    }.into_any()
                }}

                {section_header("History", "history", expanded, set_expanded)}
                {move || {
                    if !expanded.get().contains("history") { return view! { <div></div> }.into_any(); }
                    let mut events = local.get().history.clone();
                    events.reverse();
                    view! {
                        <section class="asset-history-list">
                            {if events.is_empty() {
                                view! { <div class="asset-empty">"No history events yet."</div> }.into_any()
                            } else {
                                events.into_iter().map(|e| view! {
                                    <div class="asset-history-item">
                                        <div class="asset-history-meta">
                                            <span class="asset-history-type">{e.event_type.as_str()}</span>
                                            <span class="asset-history-time">{e.timestamp.format("%d %b %Y %H:%M").to_string()}</span>
                                        </div>
                                        <div class="asset-history-body">
                                            <span class="asset-history-field">{e.field.clone().unwrap_or_default()}{if e.field.is_some() { ": " } else { "" }}</span>
                                            <span class="asset-history-value">{e.new_value.clone().unwrap_or_else(|| "—".to_string())}</span>
                                        </div>
                                        <div class="asset-history-reason">{e.reason.clone().unwrap_or_default()}</div>
                                    </div>
                                }).collect::<Vec<_>>().into_any()
                            }}
                        </section>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
