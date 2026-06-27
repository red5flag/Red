use crate::models::Portfolio;
use leptos::prelude::*;

#[server(SavePortfolio, "/api")]
pub async fn save_portfolio(portfolio: Portfolio) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::storage::portfolio_store;
        let store = portfolio_store();
        store
            .save_portfolio(&portfolio)
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    }
    Ok(())
}
