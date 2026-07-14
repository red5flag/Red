use crate::pages::reporting::{
    scroll_tabs_node_ref, start_scroll_interval, stop_scroll_interval, ReportTab,
};
use crate::stores::{use_app_store, use_transaction_store, use_ui_store};
use crate::types::TransactionType;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn ReportingPage() -> impl IntoView {
    let app_store = use_app_store();
    let transaction_store = use_transaction_store();
    let _ui_store = use_ui_store();
    let (active_tab, set_active_tab) = signal(ReportTab::Sales);

    let seed_demo = move |_| {
        let user = app_store.get().current_user.id;
        transaction_store.update(|s| {
            let t = crate::models::Transaction::new(
                TransactionType::Sale,
                1_250_000.0,
                crate::types::Currency::AUD,
                crate::models::EntityReference {
                    entity_type: crate::models::EntityType::External,
                    entity_id: Uuid::new_v4(),
                    name: "Buyer".to_string(),
                },
                crate::models::EntityReference {
                    entity_type: crate::models::EntityType::Organization,
                    entity_id: Uuid::new_v4(),
                    name: "Red".to_string(),
                },
                user,
            );
            s.add_transaction(t);
        });
    };

    let tabs = [
        ReportTab::CamScan,
        ReportTab::Sales,
        ReportTab::Purchases,
        ReportTab::Bills,
        ReportTab::Invoices,
        ReportTab::Notices,
        ReportTab::Documents,
        ReportTab::Statements,
        ReportTab::Summaries,
        ReportTab::Assets,
        ReportTab::Compliance,
        ReportTab::Transactions,
        ReportTab::ExportedRecords,
        ReportTab::Bookings,
    ];

    let tabs_container_ref = NodeRef::<leptos::html::Div>::new();
    let scroll_interval = StoredValue::new(None::<i32>);

    let start_scroll = {
        let tabs_container_ref = tabs_container_ref.clone();
        let scroll_interval = scroll_interval.clone();
        move |delta: i32| {
            stop_scroll_interval(scroll_interval.get_value());
            let id = start_scroll_interval(&tabs_container_ref, delta);
            scroll_interval.set_value(id);
        }
    };

    let stop_scroll = {
        let scroll_interval = scroll_interval.clone();
        move |_| {
            stop_scroll_interval(scroll_interval.get_value());
            scroll_interval.set_value(None);
        }
    };

    let scroll_tabs_left = move |_| scroll_tabs_node_ref(&tabs_container_ref, -150);
    let scroll_tabs_left_hold_mouse = {
        let start_scroll = start_scroll.clone();
        move |_ev: leptos::ev::MouseEvent| start_scroll(-30)
    };
    let scroll_tabs_left_hold_touch = {
        let start_scroll = start_scroll.clone();
        move |_ev: leptos::ev::TouchEvent| start_scroll(-30)
    };

    let scroll_tabs_right = move |_| scroll_tabs_node_ref(&tabs_container_ref, 150);
    let scroll_tabs_right_hold_mouse = {
        let start_scroll = start_scroll.clone();
        move |_ev: leptos::ev::MouseEvent| start_scroll(30)
    };
    let scroll_tabs_right_hold_touch = {
        let start_scroll = start_scroll.clone();
        move |_ev: leptos::ev::TouchEvent| start_scroll(30)
    };

    let stop_scroll_mouse = {
        let stop_scroll = stop_scroll.clone();
        move |_ev: leptos::ev::MouseEvent| stop_scroll(())
    };
    let stop_scroll_touch = {
        let stop_scroll = stop_scroll.clone();
        move |_ev: leptos::ev::TouchEvent| stop_scroll(())
    };

    view! {
        <div class="reporting-page">
            <div class="reporting-tabs-outer reporting-tabs-top">
                <button
                    class="reporting-tab-arrow"
                    title="Scroll left"
                    on:click=scroll_tabs_left
                    on:mousedown=scroll_tabs_left_hold_mouse
                    on:mouseup=stop_scroll_mouse
                    on:mouseleave=stop_scroll_mouse
                    on:touchstart=scroll_tabs_left_hold_touch
                    on:touchend=stop_scroll_touch
                >"←"</button>
                <div class="reporting-tabs" node_ref=tabs_container_ref>
                    {tabs.iter().map(|tab| {
                        let t = *tab;
                        let is_camscan = t == ReportTab::CamScan;
                        view! {
                            <button
                                class="reporting-tab"
                                class:reporting-tab-camscan={is_camscan}
                                class:active={move || active_tab.get() == t}
                                on:click=move |_| set_active_tab.set(t)
                                aria-label={t.label()}
                            >
                                {t.label()}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
                <button
                    class="reporting-tab-arrow"
                    title="Scroll right"
                    on:click=scroll_tabs_right
                    on:mousedown=scroll_tabs_right_hold_mouse
                    on:mouseup=stop_scroll_mouse
                    on:mouseleave=stop_scroll_mouse
                    on:touchstart=scroll_tabs_right_hold_touch
                    on:touchend=stop_scroll_touch
                >"→"</button>
            </div>

            <div class="reporting-actions">
                {move || if app_store.get().developer_mode {
                    view! {
                        <button class="reporting-btn" on:click=seed_demo>"+ Seed Demo Sale"</button>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            <div class="reporting-controls-bar">
                {super::report_filters::report_filters()}
            </div>

            <div class="reporting-body">
                {move || match active_tab.get() {
                    ReportTab::Sales => super::report_list::sales_view(&app_store).into_any(),
                    ReportTab::Purchases => super::report_list::purchases_view(&app_store).into_any(),
                    ReportTab::Bills => super::report_list::bills_view(&app_store).into_any(),
                    ReportTab::Invoices => super::report_list::invoices_view(&app_store).into_any(),
                    ReportTab::Notices => super::report_list::notices_view(&app_store).into_any(),
                    ReportTab::Documents => super::report_list::documents_view(&app_store).into_any(),
                    ReportTab::Statements => super::report_list::statements_view(&app_store).into_any(),
                    ReportTab::Summaries => super::report_summary::summaries_view(&app_store).into_any(),
                    ReportTab::Assets => super::report_list::assets_view(&app_store).into_any(),
                    ReportTab::Compliance => super::report_list::compliance_view(&app_store).into_any(),
                    ReportTab::Transactions => super::report_list::transactions_view(&app_store).into_any(),
                    ReportTab::ExportedRecords => super::report_summary::exported_records_view(&app_store).into_any(),
                    ReportTab::Bookings => super::booking_report::bookings_view().into_any(),
                    ReportTab::CamScan => view! {
                        <div class="reporting-camscan-wrap">
                            <crate::pages::camscan::CamScanView app_store />
                        </div>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}
