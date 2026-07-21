pub mod accessibility_settings;
pub mod account_settings;
pub mod api_test;
pub mod data_settings;
pub mod display_settings;
pub mod notification_settings;
pub mod page;
pub mod preset_selector;
pub mod security_settings;

pub use page::SettingsPage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    Appearance,
    Account,
    Import,
    Accessibility,
    Storage,
    Notifications,
    TwoFactorAuth,
    Data,
    Developer,
    ApiTest,
}

impl SettingsTab {
    pub const fn label(&self) -> &'static str {
        match self {
            SettingsTab::Appearance => "Appearance",
            SettingsTab::Account => "Accounts",
            SettingsTab::Import => "Import",
            SettingsTab::Accessibility => "Accessibility",
            SettingsTab::Storage => "Storage",
            SettingsTab::Notifications => "Notifications",
            SettingsTab::TwoFactorAuth => "2FA",
            SettingsTab::Data => "Data",
            SettingsTab::Developer => "Developer",
            SettingsTab::ApiTest => "API",
        }
    }
}
