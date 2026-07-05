use crate::models::*;
use crate::stores::{use_app_store, use_organization_store, use_rule_store};
use crate::types::UserRole;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn RuleEngine(org_id: Uuid) -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();
    let rule_store = use_rule_store();
    let (show_builder, set_show_builder) = signal(false);
    let (editing_rule_id, set_editing_rule_id) = signal::<Option<Uuid>>(None);
    let (show_history_for, set_show_history_for) = signal::<Option<Uuid>>(None);
    let (validation_error, set_validation_error) = signal(Option::<String>::None);

    let can_manage = move || {
        let role = organization_store.get().current_user_role_in_org(
            org_id,
            app_store.get().current_user.id,
            app_store.get().current_user.role.clone(),
        );
        matches!(
            role,
            UserRole::Owner | UserRole::Director | UserRole::SeniorManager
        )
    };

    let org_rules = Memo::new(move |_| {
        rule_store
            .get()
            .rules_for_org(org_id)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
    });

    let on_add_rule = move |_| {
        set_editing_rule_id.set(None);
        set_validation_error.set(None);
        set_show_builder.set(true);
    };

    let on_edit_rule = move |rid: Uuid| {
        set_editing_rule_id.set(Some(rid));
        set_validation_error.set(None);
        set_show_builder.set(true);
    };

    let on_delete_rule = move |rid: Uuid| {
        let app = app_store.get();
        let uid = app.current_user.id;
        let name = app.current_user.name.clone();
        rule_store.update(|s| s.delete_rule(rid, uid, name));
    };

    let on_toggle_rule = move |rid: Uuid| {
        let app = app_store.get();
        let uid = app.current_user.id;
        let name = app.current_user.name.clone();
        rule_store.update(|s| s.toggle_rule(rid, uid, name));
    };

    let on_duplicate_rule = move |rid: Uuid| {
        let app = app_store.get();
        let uid = app.current_user.id;
        let name = app.current_user.name.clone();
        rule_store.update(|s| {
            s.duplicate_rule(rid, uid, name);
        });
    };

    view! {
        <div class="rule-engine-section">
            <div class="rule-engine-header">
                <span class="rule-engine-title">"Role Rules & Notifications"</span>
                {if can_manage() {
                    view! {
                        <button class="add-btn-small"
                            aria-label="Add new rule"
                            on:click=on_add_rule>
                            "+ Rule"
                        </button>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            <div class="rule-engine-intro">
                "Rules define what happens when users take actions. "
                "Roles define what users can do, scopes define where, "
                "rules define what happens next, and notifications tell the right people."
            </div>

            // Rule builder / editor
            {move || if show_builder.get() {
                let editing = editing_rule_id.get();
                view! {
                    <RuleBuilder
                        org_id={org_id}
                        editing_rule_id={editing}
                        on_cancel=move |_| set_show_builder.set(false)
                        on_save=move |rule| {
                            if let Err(e) = rule.validate() {
                                set_validation_error.set(Some(e));
                                return;
                            }
                            let app = app_store.get();
                            let uid = app.current_user.id;
                            let name = app.current_user.name.clone();
                            if let Some(_existing) = rule_store.get().rules.iter().find(|r| r.id == rule.id) {
                                rule_store.update(|s| s.update_rule(rule, uid, name));
                            } else {
                                rule_store.update(|s| s.add_rule(rule, name));
                            }
                            set_validation_error.set(None);
                            set_show_builder.set(false);
                        }
                        validation_error={validation_error.get()}
                    />
                }.into_any()
            } else { ().into_any() }}

            // Rule cards
            {move || {
                let rules = org_rules.get();
                if rules.is_empty() {
                    view! {
                        <div class="rule-empty-state">
                            <div class="rule-empty-icon">"📋"</div>
                            <div class="rule-empty-text">"No rules configured yet."</div>
                            <div class="rule-empty-hint">"Create a rule to automate notifications and approvals."</div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="rule-card-list">
                            {rules.into_iter().map(|rule| {
                                let rid = rule.id;
                                let rname = rule.name.clone();
                                let rname_aria = rname.clone();
                                let rname_edit = rule.name.clone();
                                let rname_dup = rule.name.clone();
                                let rname_toggle = rule.name.clone();
                                let rname_history = rule.name.clone();
                                let rname_delete = rule.name.clone();
                                let summary = rule.plain_english_summary();
                                let summary_aria = summary.clone();
                                let enabled = rule.enabled;
                                let trigger_text = rule.trigger.summary();
                                let condition_text = rule.condition_group.summary();
                                let actions_text = rule.actions.iter().map(|a| a.summary()).collect::<Vec<_>>().join("; ");
                                let priority_text = rule.actions.iter()
                                    .map(|a| a.priority.label())
                                    .collect::<std::collections::HashSet<_>>()
                                    .into_iter().collect::<Vec<_>>().join(", ");
                                let priority_aria = priority_text.clone();

                                view! {
                                    <div class="rule-card" class:disabled={!enabled}
                                        role="region"
                                        aria-label={format!("Rule: {}", rname_aria)}>
                                        <div class="rule-card-header">
                                            <div class="rule-card-name-row">
                                                <span class="rule-card-name">{rname}</span>
                                                <span class="rule-card-status" class:enabled={enabled}>
                                                    {if enabled { "Active" } else { "Disabled" }}
                                                </span>
                                            </div>
                                            {if !priority_text.is_empty() {
                                                view! {
                                                    <span class="rule-card-priority" aria-label={format!("Priority: {}", priority_aria)}>
                                                        {priority_text}
                                                    </span>
                                                }.into_any()
                                            } else { ().into_any() }}
                                        </div>

                                        <div class="rule-card-summary" aria-label={format!("Rule summary: {}", summary_aria)}>
                                            {summary}
                                        </div>

                                        <div class="rule-card-details">
                                            <div class="rule-card-detail-row">
                                                <span class="rule-card-detail-label">"Trigger: "</span>
                                                <span class="rule-card-detail-value">{trigger_text}</span>
                                            </div>
                                            {if !rule.condition_group.conditions.is_empty() {
                                                view! {
                                                    <div class="rule-card-detail-row">
                                                        <span class="rule-card-detail-label">"Conditions: "</span>
                                                        <span class="rule-card-detail-value">{condition_text}</span>
                                                    </div>
                                                }.into_any()
                                            } else { ().into_any() }}
                                            <div class="rule-card-detail-row">
                                                <span class="rule-card-detail-label">"Actions: "</span>
                                                <span class="rule-card-detail-value">{actions_text}</span>
                                            </div>
                                        </div>

                                        {if can_manage() {
                                            view! {
                                                <div class="rule-card-actions">
                                                    <button class="rule-card-btn"
                                                        aria-label={format!("Edit rule {}", rname_edit)}
                                                        on:click=move |_| on_edit_rule(rid)>
                                                        "Edit rule"
                                                    </button>
                                                    <button class="rule-card-btn"
                                                        aria-label={format!("{} rule {}", if enabled { "Disable" } else { "Enable" }, rname_toggle)}
                                                        on:click=move |_| on_toggle_rule(rid)>
                                                        {if enabled { "Disable rule" } else { "Enable rule" }}
                                                    </button>
                                                    <button class="rule-card-btn"
                                                        aria-label={format!("Duplicate rule {}", rname_dup)}
                                                        on:click=move |_| on_duplicate_rule(rid)>
                                                        "Duplicate rule"
                                                    </button>
                                                    <button class="rule-card-btn"
                                                        aria-label={format!("View rule history for {}", rname_history)}
                                                        on:click=move |_| set_show_history_for.set(Some(rid))>
                                                        "View rule history"
                                                    </button>
                                                    <button class="rule-card-btn rule-card-btn-danger"
                                                        aria-label={format!("Delete rule {}", rname_delete)}
                                                        on:click=move |_| on_delete_rule(rid)>
                                                        "Delete rule"
                                                    </button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="rule-card-actions">
                                                    <button class="rule-card-btn"
                                                        aria-label={format!("View rule history for {}", rname_history)}
                                                        on:click=move |_| set_show_history_for.set(Some(rid))>
                                                        "View rule history"
                                                    </button>
                                                </div>
                                            }.into_any()
                                        }}
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }
            }}

            // Rule history modal
            {move || show_history_for.get().map(|rid| {
                let history: Vec<_> = rule_store.get().rule_history_for_rule(rid).into_iter().cloned().collect();
                let rule_name = rule_store.get().rules.iter()
                    .find(|r| r.id == rid)
                    .map(|r| r.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                view! {
                    <div class="rule-history-overlay" on:click=move |_| set_show_history_for.set(None)>
                        <div class="rule-history-modal" on:click=|ev| ev.stop_propagation()
                            role="dialog" aria-label={format!("History for rule {}", rule_name)}>
                            <div class="rule-history-header">
                                <span class="rule-history-title">{format!("Rule History: {}", rule_name)}</span>
                                <button class="rule-history-close"
                                    aria-label="Close rule history"
                                    on:click=move |_| set_show_history_for.set(None)>"✕"</button>
                            </div>
                            <div class="rule-history-body">
                                {if history.is_empty() {
                                    view! {
                                        <div class="rule-empty-state">
                                            <div class="rule-empty-text">"No history entries."</div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="rule-history-list">
                                            {history.into_iter().rev().map(|h| {
                                                view! {
                                                    <div class="rule-history-item">
                                                        <div class="rule-history-action">{h.action}</div>
                                                        <div class="rule-history-details">{h.details}</div>
                                                        <div class="rule-history-meta">
                                                            <span>{h.performed_by_name}</span>
                                                            <span>{h.timestamp.format("%b %d, %Y %H:%M").to_string()}</span>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }}
                            </div>
                        </div>
                    </div>
                }.into_any()
            })}
        </div>
    }
}

// ═══════════════════════════════════════════
// Rule Builder — sentence-style builder
// ═══════════════════════════════════════════

#[component]
fn RuleBuilder(
    org_id: Uuid,
    editing_rule_id: Option<Uuid>,
    on_cancel: impl Fn(()) + 'static + Clone,
    on_save: impl Fn(Rule) + 'static,
    validation_error: Option<String>,
) -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();
    let rule_store = use_rule_store();

    // Load existing rule if editing
    let existing = editing_rule_id
        .and_then(|rid| rule_store.get().rules.iter().find(|r| r.id == rid).cloned());

    // Rule name
    let (rule_name, set_rule_name) = signal(
        existing
            .as_ref()
            .map(|r| r.name.clone())
            .unwrap_or_default(),
    );

    // Trigger state
    let (actor_type, set_actor_type) = signal(
        existing
            .as_ref()
            .map(|r| r.trigger.actor_filter.type_label().to_string())
            .unwrap_or_else(|| "Any user".to_string()),
    );
    let (action_sel, set_action_sel) = signal(
        existing
            .as_ref()
            .map(|r| r.trigger.action.label().to_string())
            .unwrap_or_else(|| "Creates".to_string()),
    );
    let (target_sel, set_target_sel) = signal(
        existing
            .as_ref()
            .map(|r| r.trigger.target_type.label().to_string())
            .unwrap_or_else(|| "Document".to_string()),
    );
    let (scope_type, set_scope_type) = signal(
        existing
            .as_ref()
            .map(|r| r.trigger.scope.type_label().to_string())
            .unwrap_or_else(|| "Entire organization".to_string()),
    );

    // Condition group
    let (cond_mode, set_cond_mode) = signal(
        existing
            .as_ref()
            .map(|r| r.condition_group.mode.clone())
            .unwrap_or(ConditionGroupMode::All),
    );
    let (conditions, set_conditions) = signal(
        existing
            .as_ref()
            .map(|r| r.condition_group.conditions.clone())
            .unwrap_or_default(),
    );

    // Actions
    let (rule_actions, set_rule_actions) = signal(
        existing
            .as_ref()
            .map(|r| r.actions.clone())
            .unwrap_or_default(),
    );

    // Advanced mode toggle
    let (advanced_mode, set_advanced_mode) = signal(false);
    let (advanced_expr, set_advanced_expr) = signal(
        existing
            .as_ref()
            .map(|r| r.plain_english_summary())
            .unwrap_or_default(),
    );

    let _org_roles: Vec<(String, String)> = organization_store
        .get()
        .organizations
        .iter()
        .find(|o| o.id == org_id)
        .map(|o| {
            o.roles
                .iter()
                .map(|r| (r.id.to_string(), r.name.clone()))
                .collect()
        })
        .unwrap_or_default();

    let _org_portfolios: Vec<(String, String)> = app_store
        .get()
        .portfolios
        .iter()
        .filter(|p| p.organization_id == Some(org_id))
        .map(|p| (p.id.to_string(), p.name.clone()))
        .collect();

    let on_save_rule = move |_| {
        let name = rule_name.get();
        if name.trim().is_empty() {
            return;
        }

        let actor = match actor_type.get().as_str() {
            "Any user" => ActorFilter::AnyUser,
            "Specific user" => ActorFilter::SpecificUser(Uuid::nil()),
            "User with selected role" => ActorFilter::UserWithRole(Uuid::nil()),
            "User in selected organization" => ActorFilter::UserInOrganization(org_id),
            "User assigned to selected portfolio" => {
                ActorFilter::UserAssignedToPortfolio(Uuid::nil())
            }
            "User with role above selected" => ActorFilter::UserRoleAbove(Uuid::nil()),
            "User with role below selected" => ActorFilter::UserRoleBelow(Uuid::nil()),
            _ => ActorFilter::AnyUser,
        };

        let action = RuleAction::all()
            .iter()
            .find(|a| a.label() == action_sel.get())
            .cloned()
            .unwrap_or(RuleAction::Creates);

        let target = TargetType::all()
            .iter()
            .find(|t| t.label() == target_sel.get())
            .cloned()
            .unwrap_or(TargetType::Document);

        let scope = match scope_type.get().as_str() {
            "Entire organization" => RuleScope::EntireOrganization,
            "Selected organization" => RuleScope::SelectedOrganization(org_id),
            "Direct assets" => RuleScope::DirectAssets,
            "Reporting only" => RuleScope::ReportingOnly,
            "Document controls only" => RuleScope::DocumentControlsOnly,
            "Transactions only" => RuleScope::TransactionsOnly,
            "Networking only" => RuleScope::NetworkingOnly,
            "Calendar only" => RuleScope::CalendarOnly,
            "History/audit only" => RuleScope::HistoryAuditOnly,
            _ => RuleScope::EntireOrganization,
        };

        let trigger = RuleTrigger {
            actor_filter: actor,
            action,
            target_type: target,
            scope,
        };

        let cond_group = ConditionGroup {
            id: Uuid::new_v4(),
            mode: cond_mode.get(),
            conditions: conditions.get(),
        };

        let uid = app_store.get().current_user.id;
        let rule_id = editing_rule_id.unwrap_or_else(Uuid::new_v4);

        let rule = Rule {
            id: rule_id,
            name: name.clone(),
            enabled: existing.as_ref().map(|r| r.enabled).unwrap_or(true),
            organization_id: org_id,
            trigger,
            condition_group: cond_group,
            actions: rule_actions.get(),
            created_by: existing.as_ref().map(|r| r.created_by).unwrap_or(uid),
            created_at: existing
                .as_ref()
                .map(|r| r.created_at)
                .unwrap_or_else(chrono::Utc::now),
            updated_by: uid,
            updated_at: chrono::Utc::now(),
        };

        on_save(rule);
    };

    let add_condition = move |_| {
        set_conditions.update(|c| {
            c.push(Condition::new(
                ConditionLeftValue::DocumentStatus,
                ConditionOperator::Equals,
                ConditionValue::DocumentStatus("Draft".to_string()),
            ));
        });
    };

    let remove_condition = move |cid: Uuid| {
        set_conditions.update(|c| c.retain(|cond| cond.id != cid));
    };

    let add_action = move |_| {
        set_rule_actions.update(|a| {
            a.push(RuleActionEntry::new(
                RuleActionType::NotifyInApp,
                Some(NotificationRecipient::OrganizationOwner),
                ActionPriority::Medium,
            ));
        });
    };

    let remove_action = move |aid: Uuid| {
        set_rule_actions.update(|a| a.retain(|act| act.id != aid));
    };

    view! {
        <div class="rule-builder" role="form" aria-label="Rule builder">
            <div class="rule-builder-header">
                <span class="rule-builder-title">
                    {if editing_rule_id.is_some() { "Edit Rule" } else { "Create New Rule" }}
                </span>
                <button class="rule-builder-close" aria-label="Cancel rule creation" on:click={
                    let on_cancel = on_cancel.clone();
                    move |_: leptos::ev::MouseEvent| on_cancel(())
                }>"✕"</button>
            </div>

            // Rule name
            <div class="rule-builder-field">
                <label class="rule-builder-label" for="rule-name-input">"Rule name"</label>
                <input id="rule-name-input" class="login-input rule-name-input" type="text"
                    placeholder="e.g. Notify document controllers when documents are hidden"
                    prop:value={move || rule_name.get()}
                    on:input=move |ev| set_rule_name.set(event_target_value(&ev)) />
            </div>

            // Sentence-style builder
            <div class="rule-sentence-builder">
                <div class="rule-sentence-section">
                    <div class="rule-sentence-label">"When"</div>
                    <div class="rule-sentence-parts">
                        <select class="login-input rule-select"
                            aria-label="Actor selector"
                            prop:value={move || actor_type.get()}
                            on:change=move |ev| set_actor_type.set(event_target_value(&ev))>
                            <option>"Any user"</option>
                            <option>"Specific user"</option>
                            <option>"User with selected role"</option>
                            <option>"User in selected organization"</option>
                            <option>"User assigned to selected portfolio"</option>
                            <option>"User with role above selected"</option>
                            <option>"User with role below selected"</option>
                        </select>
                        <select class="login-input rule-select"
                            aria-label="Action selector"
                            prop:value={move || action_sel.get()}
                            on:change=move |ev| set_action_sel.set(event_target_value(&ev))>
                            {RuleAction::all().iter().map(|a| view! {
                                <option value={a.label()}>{a.label()}</option>
                            }).collect::<Vec<_>>()}
                        </select>
                        <select class="login-input rule-select"
                            aria-label="Target selector"
                            prop:value={move || target_sel.get()}
                            on:change=move |ev| set_target_sel.set(event_target_value(&ev))>
                            {TargetType::all().iter().map(|t| view! {
                                <option value={t.label()}>{t.label()}</option>
                            }).collect::<Vec<_>>()}
                        </select>
                        <select class="login-input rule-select"
                            aria-label="Scope selector"
                            prop:value={move || scope_type.get()}
                            on:change=move |ev| set_scope_type.set(event_target_value(&ev))>
                            {RuleScope::all_types().iter().map(|s| view! {
                                <option value={*s}>{*s}</option>
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                </div>

                // Conditions section
                <div class="rule-sentence-section">
                    <div class="rule-sentence-label">"And if"</div>
                    <div class="rule-condition-mode">
                        <select class="login-input rule-select"
                            aria-label="Condition group mode"
                            prop:value={move || match cond_mode.get() { ConditionGroupMode::All => "all", ConditionGroupMode::Any => "any" }.to_string()}
                            on:change=move |ev| {
                                set_cond_mode.set(match event_target_value(&ev).as_str() {
                                    "any" => ConditionGroupMode::Any,
                                    _ => ConditionGroupMode::All,
                                });
                            }>
                            <option value="all">"All conditions must match"</option>
                            <option value="any">"Any condition may match"</option>
                        </select>
                    </div>
                    <div class="rule-conditions-list">
                        {move || conditions.get().into_iter().map(|cond| {
                            let cid = cond.id;
                            let summary = cond.summary();
                            let summary_aria = summary.clone();
                            view! {
                                <div class="rule-condition-row" aria-label={format!("Condition: {}", summary_aria)}>
                                    <span class="rule-condition-text">{summary}</span>
                                    <button class="rule-condition-remove"
                                        aria-label={format!("Remove condition: {}", summary_aria)}
                                        on:click=move |_| remove_condition(cid)>"✕"</button>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <button class="rule-add-condition-btn"
                        aria-label="Add condition"
                        on:click=add_condition>
                        "+ Add condition"
                    </button>
                </div>

                // Actions section
                <div class="rule-sentence-section">
                    <div class="rule-sentence-label">"Then"</div>
                    <div class="rule-actions-list">
                        {move || rule_actions.get().into_iter().map(|act| {
                            let aid = act.id;
                            let summary = act.summary();
                            let summary_aria = summary.clone();
                            view! {
                                <div class="rule-action-row" aria-label={format!("Action: {}", summary_aria)}>
                                    <span class="rule-action-text">{summary}</span>
                                    <button class="rule-action-remove"
                                        aria-label={format!("Remove action: {}", summary_aria)}
                                        on:click=move |_| remove_action(aid)>"✕"</button>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <button class="rule-add-action-btn"
                        aria-label="Add action"
                        on:click=add_action>
                        "+ Add action"
                    </button>
                </div>
            </div>

            // Advanced mode toggle
            <div class="rule-advanced-toggle">
                <button class="rule-advanced-btn"
                    aria-label="Toggle advanced mode"
                    on:click=move |_| set_advanced_mode.update(|v| *v = !*v)>
                    {move || if advanced_mode.get() { "▼ Hide advanced expression" } else { "▶ Show advanced expression" }}
                </button>
            </div>
            {move || if advanced_mode.get() {
                view! {
                    <div class="rule-advanced-section">
                        <label class="rule-builder-label" for="rule-advanced-expr">"Advanced expression (readable variables)"</label>
                        <textarea id="rule-advanced-expr" class="login-input rule-advanced-textarea"
                            placeholder="actor.highest_role_rank < target.required_role_rank && event.action == 'modify'"
                            prop:value={move || advanced_expr.get()}
                            on:input=move |ev| set_advanced_expr.set(event_target_value(&ev))>
                        </textarea>
                        <div class="rule-advanced-vars">
                            <div class="rule-advanced-var-group">
                                <div class="rule-advanced-var-title">"Actor variables"</div>
                                <div class="rule-advanced-var">"actor.user_id"</div>
                                <div class="rule-advanced-var">"actor.role_ids"</div>
                                <div class="rule-advanced-var">"actor.highest_role_rank"</div>
                                <div class="rule-advanced-var">"actor.organization_ids"</div>
                            </div>
                            <div class="rule-advanced-var-group">
                                <div class="rule-advanced-var-title">"Event variables"</div>
                                <div class="rule-advanced-var">"event.action"</div>
                                <div class="rule-advanced-var">"event.timestamp"</div>
                            </div>
                            <div class="rule-advanced-var-group">
                                <div class="rule-advanced-var-title">"Target variables"</div>
                                <div class="rule-advanced-var">"target.type"</div>
                                <div class="rule-advanced-var">"target.id"</div>
                                <div class="rule-advanced-var">"target.owner_user_id"</div>
                                <div class="rule-advanced-var">"target.organization_id"</div>
                                <div class="rule-advanced-var">"target.portfolio_id"</div>
                                <div class="rule-advanced-var">"target.status"</div>
                                <div class="rule-advanced-var">"target.locked"</div>
                            </div>
                            <div class="rule-advanced-var-group">
                                <div class="rule-advanced-var-title">"Transaction / Document"</div>
                                <div class="rule-advanced-var">"transaction.amount"</div>
                                <div class="rule-advanced-var">"transaction.currency"</div>
                                <div class="rule-advanced-var">"transaction.risk_level"</div>
                                <div class="rule-advanced-var">"document.status"</div>
                                <div class="rule-advanced-var">"document.owner_user_id"</div>
                                <div class="rule-advanced-var">"document.locked"</div>
                                <div class="rule-advanced-var">"role.rank"</div>
                                <div class="rule-advanced-var">"role.id"</div>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Validation error
            {move || validation_error.as_ref().map(|e| view! {
                <div class="rule-validation-error" role="alert">
                    <span class="rule-validation-error-icon">"⚠"</span>
                    <span>{e.clone()}</span>
                </div>
            }.into_any())}

            // Save / Cancel
            <div class="rule-builder-actions">
                <button class="login-btn rule-save-btn"
                    aria-label="Save rule"
                    on:click=on_save_rule>
                    "Save rule"
                </button>
                <button class="view-btn rule-cancel-btn"
                    aria-label="Cancel rule creation"
                    on:click={
                        let on_cancel = on_cancel.clone();
                        move |_: leptos::ev::MouseEvent| on_cancel(())
                    }>
                    "Cancel"
                </button>
            </div>
        </div>
    }
}
