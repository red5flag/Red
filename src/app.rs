use crate::components::footer::Footer;
use crate::components::navbar::Navbar;
use crate::components::tabs::TabsContainer;
use crate::pages::LoginPage;
use crate::stores::{
    AccountStore, AppStore, CalendarStore, MessengerStore, NotificationStore,
    OrganizationStore, RuleStore, SearchStore, TransactionStore, UiStore, UndoRedoStore,
};
use leptos::prelude::*;
use leptos_meta::*;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <link rel="stylesheet" href="/pkg/farley.css"/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context for managing stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Initialize stores as signals
    let app_store = RwSignal::new(AppStore::default());
    let account_store = RwSignal::new(AccountStore::new());
    let calendar_store = RwSignal::new(CalendarStore::new());
    let messenger_store = RwSignal::new(MessengerStore::new());
    let notification_store = RwSignal::new(NotificationStore::default());
    let organization_store = RwSignal::new(OrganizationStore::new());
    let rule_store = RwSignal::new(RuleStore::new());
    let search_store = RwSignal::new(SearchStore::default());
    let transaction_store = RwSignal::new(TransactionStore::new());
    let ui_store = RwSignal::new(UiStore::default());
    let undo_store = RwSignal::new(UndoRedoStore::default());

    // Provide stores to all children
    provide_context(app_store);
    provide_context(account_store);
    provide_context(calendar_store);
    provide_context(messenger_store);
    provide_context(notification_store);
    provide_context(organization_store);
    provide_context(rule_store);
    provide_context(search_store);
    provide_context(transaction_store);
    provide_context(ui_store);
    provide_context(undo_store);

    // Merge credentials from localStorage after hydration to avoid
    // structural mismatch between server-rendered and client-hydrated DOM.
    #[cfg(feature = "hydrate")]
    {
        let app_store = app_store;
        leptos::task::spawn_local(async move {
            app_store.update(|s| s.credentials.merge_from_local_storage());
        });

        // Once authenticated, restore persisted portfolios + channels so
        // marketplace listings, sales profiles, and channel links survive
        // reloads instead of falling back to seed data.
        let (state_loaded, set_state_loaded) = signal(false);
        Effect::new(move |_| {
            if !app_store.with(|s| s.is_authenticated) || state_loaded.get() {
                return;
            }
            leptos::task::spawn_local(async move {
                // Empty result = nothing persisted yet → keep seed data.
                if let Ok(portfolios) = crate::server::get_portfolios().await {
                    if !portfolios.is_empty() {
                        app_store.update(|s| s.portfolios = portfolios);
                    }
                }
                // `None` = channels never saved → keep current list.
                if let Ok(Some(channels)) = crate::server::get_channels().await {
                    app_store.update(|s| s.channels = channels);
                }
                set_state_loaded.set(true);
            });
        });

        // Keep the persisted channel list in sync once the initial
        // server load has resolved (avoids overwriting stored channels
        // with pre-load seed state).
        let channels_memo = Memo::new(move |_| app_store.with(|s| s.channels.clone()));
        Effect::new(move |_| {
            let channels = channels_memo.get();
            if !state_loaded.get() || !app_store.with(|s| s.is_authenticated) {
                return;
            }
            leptos::task::spawn_local(async move {
                let _ = crate::server::save_channels(channels).await;
            });
        });

        // Portfolio autosave: every 8s while authenticated, persist
        // portfolios whose serialized contents changed since the last
        // tick and delete any removed from the store. Complements the
        // explicit save_portfolio calls so every mutation path
        // (create/edit/delete/drag) reaches the server.
        {
            use std::collections::{HashMap, HashSet};
            use std::hash::{Hash, Hasher};
            let known: RwSignal<HashMap<uuid::Uuid, u64>> = RwSignal::new(HashMap::new());
            gloo_timers::callback::Interval::new(8000, move || {
                if !state_loaded.get() || !app_store.with(|s| s.is_authenticated) {
                    return;
                }
                // Fingerprint each portfolio by its serialized form —
                // catches every mutation including metadata-only edits.
                let fps: Vec<(uuid::Uuid, u64)> = app_store.with(|s| {
                    s.portfolios
                        .iter()
                        .map(|p| {
                            let mut h = std::collections::hash_map::DefaultHasher::new();
                            serde_json::to_string(p).unwrap_or_default().hash(&mut h);
                            (p.id, h.finish())
                        })
                        .collect()
                });
                let prev = known.get();
                let mut next = prev.clone();
                let mut current_ids: HashSet<uuid::Uuid> = HashSet::new();
                let mut changed_ids: Vec<uuid::Uuid> = Vec::new();
                for (id, fp) in &fps {
                    current_ids.insert(*id);
                    if next.get(id) != Some(fp) {
                        changed_ids.push(*id);
                        next.insert(*id, *fp);
                    }
                }
                let deleted_ids: Vec<uuid::Uuid> = prev
                    .keys()
                    .filter(|id| !current_ids.contains(*id))
                    .copied()
                    .collect();
                for id in &deleted_ids {
                    next.remove(id);
                }
                known.set(next);
                if changed_ids.is_empty() && deleted_ids.is_empty() {
                    return;
                }
                let to_save: Vec<crate::models::Portfolio> = if changed_ids.is_empty() {
                    Vec::new()
                } else {
                    app_store.with(|s| {
                        s.portfolios
                            .iter()
                            .filter(|p| changed_ids.contains(&p.id))
                            .cloned()
                            .collect()
                    })
                };
                leptos::task::spawn_local(async move {
                    for id in deleted_ids {
                        let _ = crate::server::delete_portfolio(id).await;
                    }
                    for p in to_save {
                        let _ = crate::server::save_portfolio(p).await;
                    }
                });
            })
            .forget();
        }
    }

    let is_authenticated = Memo::new(move |_| app_store.get().is_authenticated);
    let theme_attr = Memo::new(move |_| ui_store.get().theme.as_str().to_string());
    let font_size_attr = Memo::new(move |_| ui_store.get().font_size.clone());
    let reduced_motion_attr = Memo::new(move |_| {
        if ui_store.get().reduced_motion {
            "true".to_string()
        } else {
            "false".to_string()
        }
    });
    let talkback_enabled_attr = Memo::new(move |_| {
        if ui_store.get().talkback_enabled {
            "true".to_string()
        } else {
            "false".to_string()
        }
    });
    let edge_style_attr = Memo::new(move |_| ui_store.get().edge_style.as_str().to_string());
    let button_style_attr = Memo::new(move |_| ui_store.get().button_style.as_str().to_string());
    let density_attr = Memo::new(move |_| ui_store.get().density.as_str().to_string());
    let accent_color_style =
        Memo::new(move |_| format!("--accent-color: {}", ui_store.get().accent_color));

    view! {
        <Show
            when=move || is_authenticated.get()
            fallback=move || view! {
                <div class="app-container" data-theme={move || theme_attr.get()} data-font-size={move || font_size_attr.get()} data-reduced-motion={move || reduced_motion_attr.get()} data-talkback-enabled={move || talkback_enabled_attr.get()} data-edge-style={move || edge_style_attr.get()} data-button-style={move || button_style_attr.get()} data-density={move || density_attr.get()} style={move || accent_color_style.get()}>
                    <LoginPage />
                </div>
            }
        >
            <div class="app-container" data-theme={move || theme_attr.get()} data-font-size={move || font_size_attr.get()} data-reduced-motion={move || reduced_motion_attr.get()} data-talkback-enabled={move || talkback_enabled_attr.get()} data-edge-style={move || edge_style_attr.get()} data-button-style={move || button_style_attr.get()} data-density={move || density_attr.get()} style={move || accent_color_style.get()}>
                // Main content area with collapsible tabs
                <main class="main-content">
                    <TabsContainer />
                </main>

                // Navigation Bar (fixed at top)
                <Navbar />

                // Footer
                <Footer />
            </div>
        </Show>
    }
}
