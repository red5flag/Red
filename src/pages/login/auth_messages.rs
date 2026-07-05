use leptos::prelude::*;

#[component]
pub(crate) fn AuthMessages(
    error: ReadSignal<String>,
    changelog_open: ReadSignal<bool>,
    commits: ReadSignal<Vec<(String, String)>>,
    commits_loading: ReadSignal<bool>,
    on_toggle_changelog: Callback<leptos::ev::MouseEvent>,
) -> impl IntoView {
    view! {
        // ── ERROR ──
        {move || {
            let err = error.get();
            if err.is_empty() { ().into_any() } else {
                view! { <div class="lp-error">{err}</div> }.into_any()
            }
        }}

        // ── CHANGELOG ACCORDION ──
        <div class="lp-changelog">
            <div class="lp-changelog-header">
                <a
                    class="lp-changelog-link"
                    href="https://github.com/red5flag/Carly"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    "CHANGELOG"
                </a>
                <span
                    class="lp-changelog-toggle"
                    on:click=move |ev| on_toggle_changelog.run(ev)
                >
                    {move || if changelog_open.get() { "▲" } else { "▼" }}
                </span>
            </div>
            {move || if changelog_open.get() {
                view! {
                    <div class="lp-changelog-body">
                        <div class="lp-changelog-summary">
                            {move || if commits_loading.get() {
                                "Loading recent commits from GitHub…"
                            } else {
                                "Latest commits from red5flag/Carly on GitHub."
                            }}
                        </div>
                        <ul class="lp-changelog-list">
                            {move || commits.get().iter().map(|(sha, msg)| {
                                view! {
                                    <li>
                                        <a
                                            href={format!("https://github.com/red5flag/Carly/commit/{}", sha)}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="lp-changelog-commit"
                                        >
                                            <span class="lp-commit-sha">{sha.clone()}</span>
                                            <span class="lp-commit-msg">{msg.clone()}</span>
                                        </a>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                        {move || if commits.get().is_empty() && !commits_loading.get() {
                            view! {
                                <div class="lp-changelog-empty">
                                    "Unable to load commits. "
                                    <a
                                        href="https://github.com/red5flag/Carly/commits/main"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                    >"View on GitHub →"</a>
                                </div>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}
