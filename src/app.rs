use crate::components::footer::Footer;
use crate::components::navbar::Navbar;
use crate::components::tabs::TabsContainer;
use crate::pages::LoginPage;
use crate::stores::{
    AppStore, CalendarStore, MessengerStore, NotificationStore, OrganizationStore, RuleStore,
    SearchStore, TransactionStore, UiStore, UndoRedoStore,
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
    provide_context(calendar_store);
    provide_context(messenger_store);
    provide_context(notification_store);
    provide_context(organization_store);
    provide_context(rule_store);
    provide_context(search_store);
    provide_context(transaction_store);
    provide_context(ui_store);
    provide_context(undo_store);

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

    view! {
        <Show
            when=move || is_authenticated.get()
            fallback=move || view! {
                <div class="app-container" data-theme={move || theme_attr.get()} data-font-size={move || font_size_attr.get()} data-reduced-motion={move || reduced_motion_attr.get()}>
                    <LoginPage />
                </div>
            }
        >
            <div class="app-container" data-theme={move || theme_attr.get()} data-font-size={move || font_size_attr.get()} data-reduced-motion={move || reduced_motion_attr.get()}>
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
