use crate::components::navbar::Navbar;
use crate::components::tabs::TabsContainer;
use crate::pages::{AgentPage, HistoryPage, NetworkingPage, OverviewPage, PortfoliosPage, SettingsPage};
use crate::stores::{AppStore, SearchStore, UndoRedoStore};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Router, Routes, Route};
use leptos_router::path;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
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
    let search_store = RwSignal::new(SearchStore::default());
    let undo_store = RwSignal::new(UndoRedoStore::default());

    // Provide stores to all children
    provide_context(app_store);
    provide_context(search_store);
    provide_context(undo_store);

    view! {
        <Router>
            <div class="app-container">
                // Main content area with routes
                <main class="main-content">
                    <Routes fallback=|| "Page not found">
                        <Route path=path!("/") view=OverviewPage />
                        <Route path=path!("/overview") view=OverviewPage />
                        <Route path=path!("/portfolios") view=PortfoliosPage />
                        <Route path=path!("/networking") view=NetworkingPage />
                        <Route path=path!("/history") view=HistoryPage />
                        <Route path=path!("/settings") view=SettingsPage />
                        <Route path=path!("/agent") view=AgentPage />
                    </Routes>
                </main>

                // Tabs Container
                <TabsContainer />

                // Navigation Bar (fixed at bottom)
                <Navbar />
            </div>
        </Router>
    }
}
