use crate::models::User;
use crate::stores::{use_app_store, use_organization_store};
use crate::types::UserRole;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn AddTeamMemberPage() -> impl IntoView {
    let app_store = use_app_store();
    let organization_store = use_organization_store();

    let (search_query, set_search_query) = signal(String::new());
    let (new_name, set_new_name) = signal(String::new());
    let (new_username, set_new_username) = signal(String::new());
    let (new_email, set_new_email) = signal(String::new());
    let (new_role, set_new_role) = signal(UserRole::Worker);

    let add_user = move |name: String, email: String, username: Option<String>, role: UserRole| {
        let name = name.trim().to_string();
        let email = email.trim().to_string();
        if name.is_empty() || email.is_empty() {
            return;
        }
        let username = username
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let avatar = format!(
            "https://api.dicebear.com/7.x/avataaars/svg?seed={}",
            username.as_ref().unwrap_or(&name)
        );
        let mut user = User::new(name, email, role);
        user.username = username;
        user.avatar_url = Some(avatar);
        let app = app_store.get();
        let org = organization_store.get();
        user.organization_id = org
            .current_organization_id
            .or(app.current_user.organization_id);
        drop(app);
        organization_store.update(|s| s.add_organization_user(user));
    };

    let on_add_user = move |_| {
        let username = new_username.get().trim().to_string();
        add_user(
            new_name.get(),
            new_email.get(),
            Some(username),
            new_role.get(),
        );
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
        let app = app_store.get();
        let org = organization_store.get();
        let mut results: Vec<User> = Vec::new();
        let current_org = org
            .current_organization_id
            .or(app.current_user.organization_id);
        let existing_ids: std::collections::HashSet<Uuid> =
            org.organization_users.iter().map(|u| u.id).collect();

        // Local users from credential store
        for cred in app.credentials.credentials.values() {
            let name = cred.display_name.to_lowercase();
            let email = cred.email.to_lowercase();
            let username = cred.username.to_lowercase();
            if name.contains(&query) || email.contains(&query) || username.contains(&query) {
                let mut user = User::new(
                    cred.display_name.clone(),
                    cred.email.clone(),
                    UserRole::Guest,
                );
                user.username = Some(cred.username.clone());
                user.avatar_url = Some(format!(
                    "https://api.dicebear.com/7.x/avataaars/svg?seed={}",
                    cred.username
                ));
                user.organization_id = current_org;
                if !existing_ids.contains(&user.id) {
                    results.push(user);
                }
            }
        }

        // Server/online users already known to the app
        for user in organization_store.get().organization_users.iter() {
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
            User::new(
                "Alice Chen".to_string(),
                "alice@company.com".to_string(),
                UserRole::Manager,
            ),
            User::new(
                "Bob Martinez".to_string(),
                "bob@company.com".to_string(),
                UserRole::Worker,
            ),
            User::new(
                "Carol White".to_string(),
                "carol@company.com".to_string(),
                UserRole::Director,
            ),
            User::new(
                "David Kim".to_string(),
                "david@company.com".to_string(),
                UserRole::Contractor,
            ),
        ];
        for mut user in server_pool {
            let name = user.name.to_lowercase();
            let email = user.email.to_lowercase();
            if name.contains(&query) || email.contains(&query) {
                user.username = Some(format!("{}", user.id.to_string().split_at(8).0));
                user.avatar_url = Some(format!(
                    "https://api.dicebear.com/7.x/avataaars/svg?seed={}",
                    user.name
                ));
                user.organization_id = current_org;
                if !existing_ids.contains(&user.id)
                    && !results.iter().any(|u| u.email == user.email)
                {
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
