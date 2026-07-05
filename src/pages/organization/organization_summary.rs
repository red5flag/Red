use leptos::prelude::*;

#[component]
pub(crate) fn OrganizationSummary(
    #[prop(into)] owner_name: String,
    portfolio_count: usize,
    member_count: usize,
    role_count: usize,
    document_count: usize,
) -> impl IntoView {
    view! {
        <div class="org-overview">
            <div class="org-overview-row">
                <span class="org-overview-label">"Owner"</span>
                <span class="org-overview-value">{owner_name}</span>
            </div>
            <div class="org-overview-row">
                <span class="org-overview-label">"Portfolios"</span>
                <span class="org-overview-value">{portfolio_count.to_string()}</span>
            </div>
            <div class="org-overview-row">
                <span class="org-overview-label">"Members"</span>
                <span class="org-overview-value">{member_count.to_string()}</span>
            </div>
            <div class="org-overview-row">
                <span class="org-overview-label">"Roles"</span>
                <span class="org-overview-value">{role_count.to_string()}</span>
            </div>
            <div class="org-overview-row">
                <span class="org-overview-label">"Documents"</span>
                <span class="org-overview-value">{document_count.to_string()}</span>
            </div>
        </div>
    }
}
