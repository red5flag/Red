use crate::types::{
    ButtonStyle, Density, EdgeStyle, ReportSortMode, SettingsPreset, SortMode, Theme, ViewCount,
    ViewMode,
};
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

/// UI-only state: drawers, modals, layout, sorting, and display preferences.
///
/// Keeping this separate from `AppStore` means that changes to UI-only fields
/// do not invalidate reactive consumers that only care about domain data such
/// as portfolios, organizations, or messages.
#[derive(Clone, Debug)]
pub struct UiStore {
    // Search panel
    pub is_search_open: bool,
    // Tabs drawer
    pub tabs_drawer_open: bool,
    // Modal state
    pub active_modal: Option<ModalType>,
    // Open document modals (by entity id) - persisted across re-renders
    pub open_doc_modals: HashSet<Uuid>,
    // Loading state
    pub is_loading: bool,
    // Portfolio layout
    pub portfolio_view_mode: ViewMode,
    pub portfolio_grid_columns: usize,
    pub portfolio_list_view_count: ViewCount,
    pub portfolio_grid_view_count: ViewCount,
    // Portfolio add-form toggles
    pub show_add_portfolio: bool,
    pub show_top_add_group: bool,
    pub show_top_add_asset: bool,
    pub show_add_modal: bool,
    // Portfolio sort state
    pub portfolio_sort_mode: SortMode,
    pub sort_ascending: bool,
    // Reporting sort state
    pub reporting_sort_mode: ReportSortMode,
    pub reporting_sort_ascending: bool,
    // Networking sort state
    pub net_sort_mode: u8,
    pub net_sort_ascending: bool,
    // Networking add member tab
    pub networking_add_member_open: bool,
    // Networking view count
    pub net_view_count: ViewCount,
    // Display/accessibility preferences
    pub theme: Theme,
    pub blind_mode: bool,
    pub font_size: String,
    pub reduced_motion: bool,
    pub language: String,
    pub edge_style: EdgeStyle,
    pub button_style: ButtonStyle,
    pub density: Density,
    pub accent_color: String,
    pub talkback_enabled: bool,
    pub custom_presets: Vec<SettingsPreset>,
}

/// Modal types tracked by the global UI state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModalType {
    CreatePortfolio,
    EditPortfolio(Uuid),
    CreateAssetGroup(Uuid), // portfolio_id
    EditAssetGroup(Uuid),
    CreateAsset(Uuid), // group_id
    EditAsset(Uuid),
    DeleteConfirmation {
        entity_type: String,
        entity_id: Uuid,
        entity_name: String,
    },
    QuickSale(Uuid), // asset_id
    Payout {
        asset_ids: Vec<Uuid>,
        recipients: Vec<Uuid>,
    },
    Notify {
        portfolio_ids: Vec<Uuid>,
        asset_ids: Vec<Uuid>,
    },
    UserDetails(Uuid),
    PaymentSetup(Uuid),
    SettingsEditor,
}

impl Default for UiStore {
    fn default() -> Self {
        Self {
            is_search_open: false,
            tabs_drawer_open: false,
            active_modal: None,
            open_doc_modals: HashSet::new(),
            is_loading: false,
            portfolio_view_mode: ViewMode::List,
            portfolio_grid_columns: 2,
            portfolio_list_view_count: ViewCount::V10,
            portfolio_grid_view_count: ViewCount::V10,
            show_add_portfolio: false,
            show_top_add_group: false,
            show_top_add_asset: false,
            show_add_modal: false,
            portfolio_sort_mode: SortMode::ByOrganization,
            sort_ascending: true,
            reporting_sort_mode: ReportSortMode::Recent,
            reporting_sort_ascending: false,
            net_sort_mode: 0,
            net_sort_ascending: true,
            networking_add_member_open: false,
            net_view_count: ViewCount::V50,
            theme: Theme::default(),
            blind_mode: false,
            font_size: "default".to_string(),
            reduced_motion: false,
            language: "en-AU".to_string(),
            edge_style: EdgeStyle::default(),
            button_style: ButtonStyle::default(),
            density: Density::default(),
            accent_color: "#3b82f6".to_string(),
            talkback_enabled: false,
            custom_presets: Vec::new(),
        }
    }
}

impl UiStore {
    pub fn new() -> Self {
        Self::default()
    }

    // Search panel
    pub fn open_search(&mut self) {
        self.is_search_open = true;
    }

    pub fn close_search(&mut self) {
        self.is_search_open = false;
    }

    /// Toggles search open/closed and returns the new state.
    pub fn toggle_search(&mut self) -> bool {
        self.is_search_open = !self.is_search_open;
        self.is_search_open
    }

    // Tabs drawer
    pub fn toggle_tabs_drawer(&mut self) {
        self.tabs_drawer_open = !self.tabs_drawer_open;
    }

    pub fn close_tabs_drawer(&mut self) {
        self.tabs_drawer_open = false;
    }

    // Modal management
    pub fn open_modal(&mut self, modal_type: ModalType) {
        self.active_modal = Some(modal_type);
    }

    pub fn close_modal(&mut self) {
        self.active_modal = None;
    }

    // Document modal state (persisted across re-renders)
    pub fn is_doc_modal_open(&self, id: Uuid) -> bool {
        self.open_doc_modals.contains(&id)
    }

    pub fn open_doc_modal(&mut self, id: Uuid) {
        self.open_doc_modals.insert(id);
    }

    pub fn close_doc_modal(&mut self, id: Uuid) {
        self.open_doc_modals.remove(&id);
    }

    pub fn toggle_doc_modal(&mut self, id: Uuid) {
        if self.open_doc_modals.contains(&id) {
            self.open_doc_modals.remove(&id);
        } else {
            self.open_doc_modals.insert(id);
        }
    }

    // Display preferences
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_edge_style(&mut self, edge_style: EdgeStyle) {
        self.edge_style = edge_style;
    }

    pub fn set_button_style(&mut self, button_style: ButtonStyle) {
        self.button_style = button_style;
    }

    pub fn set_density(&mut self, density: Density) {
        self.density = density;
    }

    pub fn set_accent_color(&mut self, accent_color: String) {
        self.accent_color = accent_color;
    }

    pub fn set_talkback_enabled(&mut self, talkback_enabled: bool) {
        self.talkback_enabled = talkback_enabled;
    }

    pub fn apply_preset(&mut self, preset: &SettingsPreset) {
        self.theme = preset.theme.clone();
        self.edge_style = preset.edge_style.clone();
        self.button_style = preset.button_style.clone();
        self.density = preset.density.clone();
        self.accent_color = preset.accent_color.clone();
    }

    pub fn save_current_as_preset(&mut self, name: String) {
        let preset = SettingsPreset {
            name,
            theme: self.theme.clone(),
            edge_style: self.edge_style.clone(),
            button_style: self.button_style.clone(),
            density: self.density.clone(),
            accent_color: self.accent_color.clone(),
        };
        self.custom_presets.push(preset);
    }

    pub fn reset_display_preferences(&mut self) {
        self.theme = Theme::default();
        self.edge_style = EdgeStyle::default();
        self.button_style = ButtonStyle::default();
        self.density = Density::default();
        self.accent_color = "#3b82f6".to_string();
        self.blind_mode = false;
        self.font_size = "default".to_string();
        self.reduced_motion = false;
        self.language = "en-AU".to_string();
        self.talkback_enabled = false;
    }

    // Portfolio grid column count
    pub fn set_portfolio_grid_columns(&mut self, columns: usize) {
        let allowed = [1, 2, 3, 4, 6, 8, 12];
        self.portfolio_grid_columns = allowed.iter().copied().find(|&c| c == columns).unwrap_or(2);
    }

    // Portfolio view count per mode (List / Grid)
    pub fn portfolio_view_count(&self, mode: ViewMode) -> ViewCount {
        match mode {
            ViewMode::Grid => self.portfolio_grid_view_count,
            _ => self.portfolio_list_view_count,
        }
    }

    pub fn set_portfolio_view_count(&mut self, mode: ViewMode, count: ViewCount) {
        match mode {
            ViewMode::Grid => self.portfolio_grid_view_count = count,
            _ => self.portfolio_list_view_count = count,
        }
    }

    pub fn net_view_count(&self) -> ViewCount {
        self.net_view_count
    }

    pub fn set_net_view_count(&mut self, count: ViewCount) {
        self.net_view_count = count;
    }

    // Portfolio sort direction
    pub fn toggle_sort_direction(&mut self) {
        self.sort_ascending = !self.sort_ascending;
    }

    pub fn reversed_sort_mode(&self) -> SortMode {
        use crate::types::SortMode;
        match self.portfolio_sort_mode {
            SortMode::Recent => SortMode::Oldest,
            SortMode::Oldest => SortMode::Recent,
            SortMode::HighestValue => SortMode::LowestValue,
            SortMode::LowestValue => SortMode::HighestValue,
            SortMode::HighestProfit => SortMode::LowestProfit,
            SortMode::LowestProfit => SortMode::HighestProfit,
            SortMode::HighestRevenue => SortMode::LowestRevenue,
            SortMode::LowestRevenue => SortMode::HighestRevenue,
            SortMode::ByOrganization => SortMode::ByOrganization,
        }
    }

    // Reporting sort helpers
    pub fn toggle_reporting_sort_direction(&mut self) {
        self.reporting_sort_ascending = !self.reporting_sort_ascending;
    }

    pub fn effective_reporting_sort_mode(&self) -> ReportSortMode {
        use crate::types::ReportSortMode;
        if self.reporting_sort_ascending {
            match &self.reporting_sort_mode {
                ReportSortMode::Recent => ReportSortMode::Oldest,
                ReportSortMode::Oldest => ReportSortMode::Recent,
                ReportSortMode::HighestValue => ReportSortMode::LowestValue,
                ReportSortMode::LowestValue => ReportSortMode::HighestValue,
                other => other.clone(),
            }
        } else {
            self.reporting_sort_mode.clone()
        }
    }

    // Networking add member
    pub fn toggle_networking_add_member(&mut self) {
        self.networking_add_member_open = !self.networking_add_member_open;
    }
}

// Context provider for the UI store
pub fn provide_ui_store() -> RwSignal<UiStore> {
    let store = RwSignal::new(UiStore::default());
    provide_context(store);
    store
}

// Hook to use the UI store
pub fn use_ui_store() -> RwSignal<UiStore> {
    expect_context::<RwSignal<UiStore>>()
}
