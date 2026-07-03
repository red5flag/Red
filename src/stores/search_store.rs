use crate::models::{Asset, Organization, Portfolio, User};
use crate::stores::AppStore;
use crate::types::{SearchFilters, SortMode, TabType};
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

// Search store with Meilisearch integration
#[derive(Clone, Debug)]
pub struct SearchStore {
    // Current search query
    pub query: String,
    // Active search filters
    pub filters: SearchFilters,
    // Search results
    pub results: SearchResults,
    // Is search loading
    pub is_loading: bool,
    // Recent searches
    pub recent_searches: Vec<String>,
    // Search suggestions
    pub suggestions: Vec<String>,
    // Context-aware priority
    pub current_tab: Option<TabType>,
    // Whether the advanced search panel is open
    pub is_advanced_search_open: bool,
    // Available filter tags
    pub available_tags: Vec<String>,
    // Selected filter tags
    pub selected_tags: Vec<String>,
    // Search history for current session
    pub search_history: Vec<SearchQuery>,
    // Sort mode for search results
    pub sort_mode: SortMode,
    // Whether the user has submitted a search query (typed at least 1 char)
    pub has_searched: bool,
}

#[derive(Clone, Debug)]
pub struct SearchQuery {
    pub query: String,
    pub filters: SearchFilters,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub result_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct SearchResults {
    pub portfolios: Vec<Portfolio>,
    pub assets: Vec<Asset>,
    pub organizations: Vec<Organization>,
    pub users: Vec<User>,
    pub total_count: usize,
    pub search_time_ms: u64,
}

/// A single, mixed search result row.
#[derive(Clone, Debug)]
pub enum SearchResultItem {
    Portfolio(Portfolio),
    Asset { asset: Asset, portfolio_id: Uuid, portfolio_name: String },
    Organization(Organization),
    User(User),
}

impl SearchResultItem {
    pub fn name(&self) -> String {
        match self {
            SearchResultItem::Portfolio(p) => p.name.clone(),
            SearchResultItem::Asset { asset, .. } => asset.name.clone(),
            SearchResultItem::Organization(o) => o.name.clone(),
            SearchResultItem::User(u) => u.name.clone(),
        }
    }

    pub fn entity_type(&self) -> &'static str {
        match self {
            SearchResultItem::Portfolio(_) => "Portfolio",
            SearchResultItem::Asset { .. } => "Asset",
            SearchResultItem::Organization(_) => "Organization",
            SearchResultItem::User(_) => "Member",
        }
    }

    pub fn last_accessed(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            SearchResultItem::Portfolio(p) => p.last_accessed_at,
            SearchResultItem::Asset { asset, .. } => asset.last_accessed_at,
            SearchResultItem::Organization(o) => o.updated_at,
            SearchResultItem::User(u) => u.updated_at,
        }
    }

    pub fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            SearchResultItem::Portfolio(p) => p.updated_at,
            SearchResultItem::Asset { asset, .. } => asset.last_accessed_at,
            SearchResultItem::Organization(o) => o.updated_at,
            SearchResultItem::User(u) => u.updated_at,
        }
    }

    pub fn productivity_score(&self) -> f64 {
        match self {
            SearchResultItem::Portfolio(p) => {
                let asset_count = p.get_all_assets().len() as f64;
                p.total_value + p.revenue + asset_count * 1000.0 + p.profit_loss.max(0.0)
            }
            SearchResultItem::Asset { asset, .. } => {
                asset.current_value + asset.revenue + asset.profit_loss.max(0.0)
            }
            SearchResultItem::Organization(o) => {
                o.members.len() as f64 * 500.0 + o.roles.len() as f64 * 200.0
            }
            SearchResultItem::User(_) => 100.0,
        }
    }

    /// Whether this item belongs to the given tab context.
    pub fn matches_tab(&self, tab: &TabType) -> bool {
        match (self, tab) {
            (SearchResultItem::Portfolio(_), TabType::Portfolios) => true,
            (SearchResultItem::Asset { .. }, TabType::Portfolios) => true,
            (SearchResultItem::Organization(_), TabType::Organization) => true,
            (SearchResultItem::User(_), TabType::Networking) => true,
            (SearchResultItem::User(_), TabType::NetworkingAddMember) => true,
            (SearchResultItem::User(_), TabType::Organization) => true,
            _ => false,
        }
    }
}

impl Default for SearchStore {
    fn default() -> Self {
        Self {
            query: String::new(),
            filters: SearchFilters::default(),
            results: SearchResults::default(),
            is_loading: false,
            recent_searches: Vec::new(),
            suggestions: Vec::new(),
            current_tab: None,
            is_advanced_search_open: false,
            available_tags: vec![
                "Real Estate".to_string(),
                "Vehicle".to_string(),
                "Equipment".to_string(),
                "Stock".to_string(),
                "Active".to_string(),
                "For Sale".to_string(),
                "High Value".to_string(),
                "Trending".to_string(),
            ],
            selected_tags: Vec::new(),
            search_history: Vec::new(),
            sort_mode: SortMode::Recent,
            has_searched: false,
        }
    }
}

impl SearchStore {
    pub fn new() -> Self {
        Self::default()
    }

    // Set search query
    pub fn set_query(&mut self, query: String) {
        self.has_searched = !query.trim().is_empty();
        self.query = query;
        self.update_suggestions();
    }

    // Clear search
    pub fn clear(&mut self) {
        self.query.clear();
        self.filters = SearchFilters::default();
        self.results = SearchResults::default();
        self.selected_tags.clear();
        self.sort_mode = SortMode::Recent;
        self.has_searched = false;
    }

    // Set sort mode
    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
    }

    // Add tag filter
    pub fn toggle_tag(&mut self, tag: String) {
        if self.selected_tags.contains(&tag) {
            self.selected_tags.retain(|t| t != &tag);
        } else {
            self.selected_tags.push(tag);
        }
        self.update_filters_from_tags();
    }

    // Update filters based on selected tags
    fn update_filters_from_tags(&mut self) {
        // Convert tags to filters
        self.filters.tags = self.selected_tags.clone();

        // Map common tags to asset types
        self.filters.asset_types = self
            .selected_tags
            .iter()
            .filter_map(|tag| match tag.as_str() {
                "Real Estate" => Some(crate::types::AssetType::RealEstate),
                "Vehicle" => Some(crate::types::AssetType::Vehicle),
                "Equipment" => Some(crate::types::AssetType::Equipment),
                "Stock" => Some(crate::types::AssetType::Stock),
                _ => None,
            })
            .collect();
    }

    // Update suggestions based on query
    fn update_suggestions(&mut self) {
        if self.query.len() < 2 {
            self.suggestions.clear();
            return;
        }

        // Generate suggestions based on query and context
        let query_lower = self.query.to_lowercase();
        self.suggestions = self
            .available_tags
            .iter()
            .filter(|tag| tag.to_lowercase().contains(&query_lower))
            .take(5)
            .cloned()
            .collect();
    }

    /// Perform search across the current AppStore data.
    /// Prioritizes results matching the current tab context first, then includes
    /// other results by relevance. If the query is empty, returns 10 relevant
    /// results based on recent access, recent modification, and productivity.
    pub fn perform_search(&mut self, app_store: &AppStore) {
        let start = std::time::SystemTime::now();
        let user_id = app_store.current_user.id;
        let org_ids: std::collections::HashSet<Uuid> = app_store.organizations.iter().map(|o| o.id).collect();
        let can_view_all = app_store.current_user.role == crate::types::UserRole::DocumentWorker
            || app_store.current_user.role.level() >= crate::types::UserRole::Manager.level();
        let query_lower = self.query.to_lowercase();
        let empty_query = self.query.trim().is_empty();
        let current_tab = self.current_tab.clone();

        // Collect all visible items across data types.
        let mut items: Vec<SearchResultItem> = Vec::new();

        // Portfolios and assets
        for p in &app_store.portfolios {
            let visible = can_view_all
                || p.owner_id == user_id
                || p.organization_id.map_or(false, |oid| org_ids.contains(&oid))
                || p.assigned_users.contains(&user_id);
            if !visible {
                continue;
            }

            if empty_query || Self::portfolio_matches(p, &query_lower, &self.filters) {
                items.push(SearchResultItem::Portfolio(p.clone()));
            }

            for asset in p.get_all_assets() {
                let asset_visible = can_view_all
                    || asset.organization_id.map_or(false, |oid| org_ids.contains(&oid))
                    || asset.assigned_workers.contains(&user_id);
                if !asset_visible {
                    continue;
                }
                if empty_query || Self::asset_matches(asset, &query_lower, &self.filters) {
                    items.push(SearchResultItem::Asset {
                        asset: asset.clone(),
                        portfolio_id: p.id,
                        portfolio_name: p.name.clone(),
                    });
                }
            }
        }

        // Organizations
        for o in &app_store.organizations {
            if empty_query || Self::org_matches(o, &query_lower) {
                items.push(SearchResultItem::Organization(o.clone()));
            }
        }

        // Users (organization_users)
        for u in &app_store.organization_users {
            if empty_query || Self::user_matches(u, &query_lower) {
                items.push(SearchResultItem::User(u.clone()));
            }
        }

        // Partition: tab-matching items first, then the rest.
        let (mut tab_items, mut other_items): (Vec<_>, Vec<_>) = if let Some(ref tab) = current_tab {
            items.into_iter().partition(|item| item.matches_tab(tab))
        } else {
            (Vec::new(), items)
        };

        // Sort both partitions by relevance score.
        tab_items.sort_by(|a, b| {
            let score_a = Self::relevance_score(a);
            let score_b = Self::relevance_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        other_items.sort_by(|a, b| {
            let score_a = Self::relevance_score(a);
            let score_b = Self::relevance_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // For empty query, truncate to keep results manageable.
        if empty_query {
            tab_items.truncate(7);
            other_items.truncate(5);
        } else {
            tab_items.truncate(15);
            other_items.truncate(10);
        }

        // Combine: tab items first, then other items.
        let mut combined: Vec<SearchResultItem> = tab_items;
        combined.extend(other_items);

        let mut portfolios = Vec::new();
        let mut assets = Vec::new();
        let mut organizations = Vec::new();
        let mut users = Vec::new();
        for item in combined {
            match item {
                SearchResultItem::Portfolio(p) => portfolios.push(p),
                SearchResultItem::Asset { asset, .. } => assets.push(asset),
                SearchResultItem::Organization(o) => organizations.push(o),
                SearchResultItem::User(u) => users.push(u),
            }
        }

        let total = portfolios.len() + assets.len() + organizations.len() + users.len();
        let elapsed = start
            .elapsed()
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        self.results = SearchResults {
            portfolios,
            assets,
            organizations,
            users,
            total_count: total,
            search_time_ms: elapsed,
        };

        // Record search in history
        self.search_history.push(SearchQuery {
            query: self.query.clone(),
            filters: self.filters.clone(),
            timestamp: chrono::Utc::now(),
            result_count: self.results.total_count,
        });

        // Keep only last 20 searches
        if self.search_history.len() > 20 {
            self.search_history.remove(0);
        }
    }

    fn org_matches(o: &Organization, query: &str) -> bool {
        o.name.to_lowercase().contains(query)
            || o.description.as_ref().map_or(false, |d| d.to_lowercase().contains(query))
    }

    fn user_matches(u: &User, query: &str) -> bool {
        u.name.to_lowercase().contains(query)
            || u.email.to_lowercase().contains(query)
    }

    fn portfolio_matches(p: &Portfolio, query: &str, filters: &SearchFilters) -> bool {
        let status_str = format!("{:?}", p.status).to_lowercase();
        let matches_query = query.is_empty()
            || p.name.to_lowercase().contains(query)
            || p.description.as_ref().map_or(false, |d| d.to_lowercase().contains(query))
            || p.tags.iter().any(|t| t.to_lowercase().contains(query))
            || status_str.contains(query);

        let matches_type = filters.asset_types.is_empty();
        let matches_value = filters.value_range.map_or(true, |(min, max)| {
            p.total_value >= min && p.total_value <= max
        });
        let matches_date = filters.date_range.map_or(true, |(from, to)| {
            p.updated_at >= from && p.updated_at <= to
        });
        let matches_status = filters.status.as_ref().map_or(true, |s| {
            status_str == s.to_lowercase()
        });

        matches_query && matches_type && matches_value && matches_date && matches_status
    }

    fn asset_matches(a: &Asset, query: &str, filters: &SearchFilters) -> bool {
        let type_str = format!("{:?}", a.asset_type).to_lowercase();
        let status_str = format!("{:?}", a.status).to_lowercase();
        let matches_query = query.is_empty()
            || a.name.to_lowercase().contains(query)
            || a.description.as_ref().map_or(false, |d| d.to_lowercase().contains(query))
            || a.tags.iter().any(|t| t.to_lowercase().contains(query))
            || type_str.contains(query)
            || a.location.as_ref().map_or(false, |l| l.to_lowercase().contains(query));

        let matches_type = filters.asset_types.is_empty()
            || filters.asset_types.contains(&a.asset_type);
        let matches_value = filters.value_range.map_or(true, |(min, max)| {
            a.current_value >= min && a.current_value <= max
        });
        let matches_date = filters.date_range.map_or(true, |(from, to)| {
            a.last_accessed_at >= from && a.last_accessed_at <= to
        });
        let matches_status = filters.status.as_ref().map_or(true, |s| {
            status_str == s.to_lowercase()
        });

        matches_query && matches_type && matches_value && matches_date && matches_status
    }

    /// Relevance score for empty-query recommendations.
    /// Combines recent access, recent modification, and productivity.
    fn relevance_score(item: &SearchResultItem) -> f64 {
        let now = Utc::now();
        let days_since_access = (now - item.last_accessed()).num_seconds().max(0) as f64 / 86400.0;
        let days_since_update = (now - item.updated_at()).num_seconds().max(0) as f64 / 86400.0;
        let recency_score = (1.0 / (1.0 + days_since_access)) * 100.0;
        let modification_score = (1.0 / (1.0 + days_since_update)) * 80.0;
        let productivity_score = item.productivity_score() / 1000.0;
        recency_score + modification_score + productivity_score
    }

    // Toggle advanced search panel
    pub fn toggle_advanced_search(&mut self) {
        self.is_advanced_search_open = !self.is_advanced_search_open;
    }

    pub fn open_advanced_search(&mut self) {
        self.is_advanced_search_open = true;
    }

    pub fn close_advanced_search(&mut self) {
        self.is_advanced_search_open = false;
    }

    // Set context tab for priority searching
    pub fn set_context_tab(&mut self, tab: TabType) {
        self.current_tab = Some(tab.clone());

        // Adjust search priorities based on tab
        match tab {
            TabType::Overview => {
                // Prioritize trending/recent
            }
            TabType::Portfolios => {
                // Prioritize portfolios and assets
            }
            TabType::Networking | TabType::NetworkingAddMember => {
                // Prioritize users
            }
            TabType::Organization => {
                // Prioritize organizations and members
            }
            TabType::Reporting => {
                // Prioritize documents, payslips, hours, deeds
            }
            TabType::Transactions => {
                // Prioritize transactions
            }
            TabType::Calendar => {
                // Prioritize bookings and calendar events
            }
            TabType::History => {
                // Prioritize actions
            }
            TabType::Settings => {
                // Prioritize settings
            }
            TabType::Agent => {
                // No specific priority
            }
        }
    }

    // Get search suggestions with context
    pub fn get_contextual_suggestions(&self) -> Vec<String> {
        let mut suggestions = self.suggestions.clone();

        // Add context-specific suggestions
        if let Some(ref tab) = self.current_tab {
            match tab {
                TabType::Portfolios => {
                    suggestions.push("portfolio:".to_string());
                    suggestions.push("asset:".to_string());
                }
                TabType::Networking | TabType::NetworkingAddMember => {
                    suggestions.push("user:".to_string());
                    suggestions.push("role:".to_string());
                }
                TabType::Organization => {
                    suggestions.push("org:".to_string());
                    suggestions.push("member:".to_string());
                    suggestions.push("department:".to_string());
                }
                TabType::Reporting => {
                    suggestions.push("doc:".to_string());
                    suggestions.push("payslip:".to_string());
                    suggestions.push("deed:".to_string());
                    suggestions.push("registration:".to_string());
                }
                TabType::Transactions => {
                    suggestions.push("txn:".to_string());
                    suggestions.push("status:".to_string());
                }
                TabType::History => {
                    suggestions.push("action:".to_string());
                    suggestions.push("date:".to_string());
                }
                _ => {}
            }
        }

        suggestions.truncate(8);
        suggestions
    }

    // Export search as URL query string
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if !self.query.is_empty() {
            params.push(format!("q={}", self.query));
        }

        for tag in &self.selected_tags {
            params.push(format!("tag={}", tag));
        }

        params.join("&")
    }
}

// Create a signal-based store for Leptos
pub fn create_search_store() -> RwSignal<SearchStore> {
    RwSignal::new(SearchStore::new())
}

/// Perform an async search via Meilisearch, falling back to local in-memory search on failure.
/// Updates `search_store.results`, `is_loading`, and `has_searched`.
pub async fn perform_meilisearch(app_store: &AppStore, search_store: &mut SearchStore) {
    let query = search_store.query.clone();
    let filters = search_store.filters.clone();
    let config = MeilisearchConfig::default();

    search_store.is_loading = true;

    let meili_result = search_meilisearch(&config, &query, &filters).await;

    match meili_result {
        Ok(results) => {
            search_store.results = results;
        }
        Err(err) => {
            leptos::logging::log!("Meilisearch search failed, falling back to local search: {}", err);
            search_store.perform_search(app_store);
        }
    }

    search_store.is_loading = false;
    search_store.has_searched = !query.trim().is_empty();

    // Record search history
    let result_count = search_store.results.total_count;
    search_store.search_history.push(SearchQuery {
        query: query.clone(),
        filters: filters.clone(),
        timestamp: chrono::Utc::now(),
        result_count,
    });
    if search_store.search_history.len() > 20 {
        search_store.search_history.remove(0);
    }
}

async fn search_meilisearch(
    config: &MeilisearchConfig,
    query: &str,
    filters: &SearchFilters,
) -> Result<SearchResults, String> {
    let url = format!("{}/indexes/{}/search", config.host, config.index_name);
    let body = serde_json::json!({
        "q": query,
        "limit": 20,
        "filter": build_meilisearch_filter(filters),
        "attributesToHighlight": ["name", "description"],
    });

    let response_json: serde_json::Value;

    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let client = reqwest::Client::new();
            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", config.api_key))
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            response_json = response.json().await.map_err(|e| e.to_string())?;
        } else {
            let request = gloo_net::http::Request::post(&url)
                .header("Authorization", &format!("Bearer {}", config.api_key))
                .json(&body)
                .map_err(|e| e.to_string())?;
            let response = request.send().await.map_err(|e| e.to_string())?;
            response_json = response.json().await.map_err(|e| e.to_string())?;
        }
    }

    parse_meilisearch_response(response_json)
}

fn build_meilisearch_filter(filters: &SearchFilters) -> String {
    let mut parts = Vec::new();

    if !filters.asset_types.is_empty() {
        let types: Vec<String> = filters
            .asset_types
            .iter()
            .map(|t| format!("asset_type = {:?}", t))
            .collect();
        parts.push(format!("({})", types.join(" OR ")));
    }

    if let Some((min, max)) = filters.value_range {
        parts.push(format!("current_value >= {} AND current_value <= {}", min, max));
    }

    if let Some((from, to)) = filters.date_range {
        parts.push(format!(
            "updated_at >= {} AND updated_at <= {}",
            from.to_rfc3339(),
            to.to_rfc3339()
        ));
    }

    if let Some(ref status) = filters.status {
        parts.push(format!("status = {}", status));
    }

    if let Some(owner) = filters.owner {
        parts.push(format!("owner_id = {}", owner));
    }

    if let Some(portfolio) = filters.portfolio {
        parts.push(format!("portfolio_id = {}", portfolio));
    }

    if !filters.tags.is_empty() {
        let tags: Vec<String> = filters
            .tags
            .iter()
            .map(|t| format!("tags = {}", t))
            .collect();
        parts.push(format!("({})", tags.join(" OR ")));
    }

    parts.join(" AND ")
}

fn parse_meilisearch_response(json: serde_json::Value) -> Result<SearchResults, String> {
    let hits = json
        .get("hits")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut portfolios = Vec::new();
    let mut assets = Vec::new();
    let mut organizations = Vec::new();
    let mut users = Vec::new();

    for hit in hits {
        let kind = hit.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        match kind {
            "portfolio" => {
                if let Ok(p) = serde_json::from_value::<Portfolio>(hit.clone()) {
                    portfolios.push(p);
                }
            }
            "asset" => {
                if let Ok(a) = serde_json::from_value::<Asset>(hit.clone()) {
                    assets.push(a);
                }
            }
            "organization" => {
                if let Ok(o) = serde_json::from_value::<Organization>(hit.clone()) {
                    organizations.push(o);
                }
            }
            "user" | "member" => {
                if let Ok(u) = serde_json::from_value::<User>(hit.clone()) {
                    users.push(u);
                }
            }
            _ => {}
        }
    }

    let total_count = json
        .get("estimatedTotalHits")
        .or_else(|| json.get("totalHits"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let search_time_ms = json
        .get("processingTimeMs")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    Ok(SearchResults {
        portfolios,
        assets,
        organizations,
        users,
        total_count,
        search_time_ms,
    })
}

// Context provider for the search store
pub fn provide_search_store() -> RwSignal<SearchStore> {
    let store = create_search_store();
    provide_context(store);
    store
}

// Hook to use the search store
pub fn use_search_store() -> RwSignal<SearchStore> {
    expect_context::<RwSignal<SearchStore>>()
}

// Meilisearch client configuration
#[derive(Clone, Debug)]
pub struct MeilisearchConfig {
    pub host: String,
    pub api_key: String,
    pub index_name: String,
}

impl Default for MeilisearchConfig {
    fn default() -> Self {
        Self {
            host: "http://localhost:7700".to_string(),
            api_key: String::new(),
            index_name: "farley".to_string(),
        }
    }
}

// Meilisearch search client (SSR only - uses reqwest)
#[cfg(feature = "ssr")]
pub struct MeilisearchClient {
    config: MeilisearchConfig,
    client: reqwest::Client,
}

#[cfg(feature = "ssr")]
impl MeilisearchClient {
    pub fn new(config: MeilisearchConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn search(
        &self,
        query: &str,
        filters: &SearchFilters,
    ) -> Result<SearchResults, reqwest::Error> {
        let url = format!(
            "{}/indexes/{}/search",
            self.config.host, self.config.index_name
        );

        let search_request = serde_json::json!({
            "q": query,
            "filter": self.build_filter_string(filters),
            "limit": 20,
            "attributesToHighlight": ["name", "description"],
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&search_request)
            .send()
            .await?;

        // Parse response and convert to SearchResults
        // This is simplified - full implementation would parse the Meilisearch response
        let _search_response: serde_json::Value = response.json().await?;

        Ok(SearchResults::default())
    }

    fn build_filter_string(&self, filters: &SearchFilters) -> String {
        let mut filter_parts = Vec::new();

        if !filters.asset_types.is_empty() {
            let types: Vec<String> = filters
                .asset_types
                .iter()
                .map(|t| format!("asset_type = {:?}", t))
                .collect();
            filter_parts.push(format!("({})", types.join(" OR ")));
        }

        if let Some((min, max)) = filters.value_range {
            filter_parts.push(format!("current_value >= {} AND current_value <= {}", min, max));
        }

        filter_parts.join(" AND ")
    }

    pub async fn add_document(&self, document: &serde_json::Value) -> Result<(), reqwest::Error> {
        let url = format!(
            "{}/indexes/{}/documents",
            self.config.host, self.config.index_name
        );

        self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(document)
            .send()
            .await?;

        Ok(())
    }

    pub async fn update_document(
        &self,
        document: &serde_json::Value,
    ) -> Result<(), reqwest::Error> {
        // Same as add - Meilisearch upserts
        self.add_document(document).await
    }

    pub async fn delete_document(&self, document_id: &str) -> Result<(), reqwest::Error> {
        let url = format!(
            "{}/indexes/{}/documents/{}",
            self.config.host, self.config.index_name, document_id
        );

        self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;

        Ok(())
    }
}
