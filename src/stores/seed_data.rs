use crate::models::{
    Asset, Booking, BookingSource, BookingStatus, CalendarEvent, Channel, ChannelType,
    ConnectionStatus, Organization, Portfolio, SyncDirection, User,
};
use crate::stores::app_store::AppStore;
use crate::stores::notifications::{NotificationStore, NotificationType};
use crate::stores::OrganizationStore;
use crate::types::{AssetType, UserRole};
use uuid::Uuid;

/// Seed the Red family demo organizations and role-based test data.
pub fn seed_red_family_data(
    app_store: &mut AppStore,
    organization_store: &mut OrganizationStore,
    notification_store: &mut NotificationStore,
) {
    let owner_id = app_store.current_user.id;
    let owner_email = app_store.current_user.email.clone();

    // RedOrg is owned by the current user (Red) — primary test company.
    let mut red_org = Organization::new("RedOrg".to_string(), owner_id);
    let red_org_id = red_org.id;
    red_org.description = Some("RedOrg — primary test organization (Company)".to_string());
    red_org.settings.color = Some("#ef4444".to_string());
    red_org.business_type = Some("Company".to_string());
    red_org.abn = Some("12 345 678 901".to_string());
    red_org.lei = Some("12345678901234567890".to_string());
    red_org.business_address = Some("1 Red St, Melbourne VIC 3000, Australia".to_string());
    red_org.business_phone = Some("+61 1 234 567 890".to_string());
    red_org.business_email = Some("contact@redorg.com".to_string());

    let mut red_corp = Organization::new("RedDirector".to_string(), owner_id);
    let red_corp_id = red_corp.id;
    red_corp.description = Some("Red Director - Director role testbed".to_string());
    red_corp.settings.color = Some("#ef4444".to_string());
    red_corp.business_type = Some("Company".to_string());
    red_corp.abn = Some("1".to_string());
    red_corp.lei = Some("1".to_string());
    red_corp.business_address = Some("1".to_string());
    red_corp.business_phone = Some("1".to_string());
    red_corp.business_email = Some("1".to_string());

    let mut red_comp = Organization::new("RedManager".to_string(), owner_id);
    let red_comp_id = red_comp.id;
    red_comp.description = Some("Red Manager - Manager role testbed".to_string());
    red_comp.settings.color = Some("#f97316".to_string());
    red_comp.business_type = Some("Company".to_string());
    red_comp.abn = Some("1".to_string());
    red_comp.lei = Some("1".to_string());
    red_comp.business_address = Some("1".to_string());
    red_comp.business_phone = Some("1".to_string());
    red_comp.business_email = Some("1".to_string());

    let mut red_co = Organization::new("RedWorker".to_string(), owner_id);
    let red_co_id = red_co.id;
    red_co.description = Some("Red Worker - Worker role testbed".to_string());
    red_co.settings.color = Some("#3b82f6".to_string());
    red_co.business_type = Some("Company".to_string());
    red_co.abn = Some("1".to_string());
    red_co.lei = Some("1".to_string());
    red_co.business_address = Some("1".to_string());
    red_co.business_phone = Some("1".to_string());
    red_co.business_email = Some("1".to_string());

    organization_store.organizations.push(red_org);
    organization_store.organizations.push(red_corp);
    organization_store.organizations.push(red_comp);
    organization_store.organizations.push(red_co);
    organization_store.current_organization_id = Some(red_org_id);

    // Red as Owner in RedOrg (same ID as current user).
    let mut red_owner = User::new("Red".to_string(), owner_email.clone(), UserRole::Owner);
    red_owner.id = owner_id;
    red_owner.organization_id = Some(red_org_id);

    // Red seeded into the other orgs with the requested roles.
    let mut red_director = User::new(
        "Red".to_string(),
        "red@reddirector.com".to_string(),
        UserRole::Director,
    );
    red_director.id = owner_id;
    red_director.organization_id = Some(red_corp_id);

    let mut red_manager = User::new(
        "Red".to_string(),
        "red@redmanager.com".to_string(),
        UserRole::Manager,
    );
    red_manager.id = owner_id;
    red_manager.organization_id = Some(red_comp_id);

    let mut red_worker = User::new(
        "Red".to_string(),
        "red@redworker.com".to_string(),
        UserRole::Worker,
    );
    red_worker.id = owner_id;
    red_worker.organization_id = Some(red_co_id);

    organization_store.organization_users.push(red_owner);
    organization_store.organization_users.push(red_director);
    organization_store.organization_users.push(red_manager);
    organization_store.organization_users.push(red_worker);

    // Add Red to every organization's member list.
    for org in &mut organization_store.organizations {
        org.add_member(owner_id);
    }

    // Role-testbed portfolios are assigned to RedOrg so most portfolios live in one org.
    // Only NotRed Portfolio (guest testing) and Mixed Investments (unaffiliated) stay outside.
    app_store.portfolios.push(seed_org_portfolio(
        red_org_id,
        owner_id,
        "RedOrg Portfolio",
        "RedOrg HQ Asset",
    ));
    app_store.portfolios.push(seed_org_portfolio(
        red_org_id,
        owner_id,
        "RedDirector Portfolio",
        "RedDirector Fleet Asset",
    ));
    app_store.portfolios.push(seed_org_portfolio(
        red_org_id,
        owner_id,
        "RedManager Portfolio",
        "RedManager Equipment Asset",
    ));
    app_store.portfolios.push(seed_org_portfolio(
        red_org_id,
        owner_id,
        "RedWorker Portfolio",
        "RedWorker Equipment Asset",
    ));

    seed_notred_data(app_store, organization_store, notification_store);

    seed_red_channel_data(app_store, organization_store);

    // Seed clearly labeled demo Test Channel and booking for the first asset in the first portfolio.
    if let Some(first_portfolio) = app_store.portfolios.first_mut() {
        if let Some(asset) = first_portfolio.assets.first_mut() {
            let asset_id = asset.id;
            let portfolio_id = Some(first_portfolio.id);
            let channel = Channel::new_test_channel(
                "Demo Test Channel".to_string(),
                Some(asset_id),
                portfolio_id,
            );
            let channel_id = channel.id;
            let mut booking = Booking::new(
                asset_id,
                Some(channel_id),
                BookingSource::TestChannel,
                "Demo Guest".to_string(),
                chrono::Utc::now() + chrono::Duration::days(5),
                chrono::Utc::now() + chrono::Duration::days(8),
                150.0,
            );
            booking.status = BookingStatus::Confirmed;

            let event = CalendarEvent::for_booking(
                format!(
                    "{} - {} ({})",
                    booking.guest_name, asset.name, "Test Channel"
                ),
                booking.start_datetime,
                booking.end_datetime,
                portfolio_id,
                asset_id,
                Some(channel_id),
                booking.id,
                "Test Channel",
            );
            booking.calendar_event_ids.push(event.id);

            app_store.channels.push(channel);
            app_store.bookings.push(booking);
            asset.calendar_events.push(event);
        }
    }

    // Start Red as Owner of RedOrg; role updates when switching organizations.
    app_store.current_user.role = UserRole::Owner;
}

pub fn seed_org_portfolio(org_id: Uuid, owner_id: Uuid, name: &str, asset_name: &str) -> Portfolio {
    let mut p = Portfolio::new(name.to_string(), owner_id, crate::types::Currency::USD);
    p.organization_id = Some(org_id);
    p.description = Some(format!("{} - role testing portfolio", name));
    let mut asset = Asset::new(asset_name.to_string(), AssetType::Equipment, 10000.0);
    asset.organization_id = Some(org_id);
    p.assets.push(asset);
    p.recalculate_values();
    p
}

/// Seed a separate organization where the current user is a Guest.
/// Used to test that a guest cannot edit organization, portfolio, asset, or document info.
pub fn seed_notred_data(
    app_store: &mut AppStore,
    organization_store: &mut OrganizationStore,
    notification_store: &mut NotificationStore,
) {
    let guest_id = app_store.current_user.id;
    let guest_name = app_store.current_user.name.clone();
    let guest_email = app_store.current_user.email.clone();

    let notred_owner = Uuid::new_v4();
    let notred_id;
    let notred = {
        let mut o = Organization::new("NotRed".to_string(), notred_owner);
        o.description = Some("NotRed - Guest role testbed".to_string());
        o.settings.color = Some("#10b981".to_string());
        o.business_type = Some("Company".to_string());
        o.abn = Some("1".to_string());
        o.lei = Some("1".to_string());
        o.business_address = Some("1".to_string());
        o.business_phone = Some("1".to_string());
        o.business_email = Some("1".to_string());
        o.add_member(guest_id);
        notred_id = o.id;
        o
    };
    organization_store.organizations.push(notred);

    // Current user as a Guest in NotRed.
    let mut notred_guest = User::new(guest_name, guest_email, UserRole::Guest);
    notred_guest.id = guest_id;
    notred_guest.organization_id = Some(notred_id);
    organization_store.organization_users.push(notred_guest);

    // Portfolio assigned to the guest so it is visible to them, but owned by the org owner.
    let mut p = Portfolio::new(
        "NotRed Portfolio".to_string(),
        notred_owner,
        crate::types::Currency::USD,
    );
    p.organization_id = Some(notred_id);
    p.description = Some("NotRed portfolio - Guest view-only testbed".to_string());
    p.assigned_users.push(guest_id);

    let mut asset = Asset::new(
        "NotRed Office Equipment".to_string(),
        AssetType::Equipment,
        5000.0,
    );
    asset.organization_id = Some(notred_id);
    asset.assigned_workers.push(guest_id);
    // Pre-existing documents the guest can read but cannot edit (nil owner = legacy shared).
    asset.documents.push(make_doc("NotRed Welcome", "pdf"));
    asset.documents.push(make_doc("NotRed Policy", "pdf"));

    // Real audit document with @red ping in notes, uploaded by NotRed owner.
    let audit_doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: "NotRed Q3 Audit Report".to_string(),
            file_type: "pdf".to_string(),
            url: "#".to_string(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by: notred_owner,
            content: Some("@red — Please review this audit report for compliance. Notes: financial statements verified, tax filings current, 2 discrepancies flagged for follow-up. @red ping for approval.".to_string()),
        };
    let audit_doc_id = audit_doc.id;
    let notred_pid = p.id;
    let notred_aid = asset.id;
    asset.documents.push(audit_doc);
    p.assets.push(asset);
    p.recalculate_values();
    app_store.portfolios.push(p);

    // Linked document notifications: one on Reporting tab, one on Portfolios tab.
    notification_store.add_document_notification(
            audit_doc_id,
            "NotRed Q3 Audit Report",
            "red",
            "Red1 (NotRed Owner) has listed a new document and requested audit review by Red (Auditor).",
            NotificationType::Warning,
            Some(crate::types::TabType::Reporting),
            Some("Red1".to_string()),
            Some(notred_pid),
            None,
            Some(notred_aid),
        );
}

fn make_doc(name: &str, ext: &str) -> crate::models::Document {
    crate::models::Document {
        id: Uuid::new_v4(),
        name: name.to_string(),
        file_type: ext.to_string(),
        url: "#".to_string(),
        uploaded_at: chrono::Utc::now(),
        uploaded_by: Uuid::nil(),
        content: None,
    }
}

fn make_asset(
    name: &str,
    desc: &str,
    location: &str,
    purchase: f64,
    current: f64,
    docs: Vec<crate::models::Document>,
) -> crate::models::Asset {
    use crate::types::AssetType;
    let mut a = crate::models::Asset::new(name.to_string(), AssetType::RealEstate, purchase);
    a.description = Some(desc.to_string());
    a.location = Some(location.to_string());
    a.update_value(current);
    a.documents = docs;
    a
}

pub fn seed_default_portfolio(owner_id: Uuid) -> Portfolio {
    let mut p = Portfolio::new(
        "Commercial Real Estate".to_string(),
        owner_id,
        crate::types::Currency::USD,
    );
    p.description = Some("Office buildings and retail spaces".to_string());
    p.tags = vec!["real-estate".to_string(), "commercial".to_string()];
    p.documents = vec![
        make_doc("Portfolio Overview", "pdf"),
        make_doc("Annual Report 2024", "xlsx"),
        make_doc("Investment Strategy", "docx"),
    ];

    let mut hq = make_asset(
        "Headquarters",
        "Main corporate headquarters building, 8 floors, 4200 sqm.",
        "123 Collins St, Melbourne VIC 3000",
        5_000_000.0,
        6_200_000.0,
        vec![
            make_doc("Title Deed", "pdf"),
            make_doc("Valuation Report", "pdf"),
            make_doc("Insurance Certificate", "pdf"),
        ],
    );
    hq.images = vec![format!("https://placehold.co/400x400/2d3748/FFF?text=HQ")];
    p.assets.push(hq);

    // Downtown Properties group
    let mut group1 = crate::models::AssetGroup::new("Downtown Properties".to_string());
    group1.description = Some("Central business district commercial properties".to_string());
    group1.documents = vec![
        make_doc("Group Overview", "pdf"),
        make_doc("CBD Market Analysis", "xlsx"),
    ];

    let mut a1 = make_asset(
        "Main Office Building",
        "12-storey premium grade-A office tower, fully leased to blue-chip tenants.",
        "1 George St, Sydney NSW 2000",
        2_500_000.0,
        3_200_000.0,
        vec![
            make_doc("Title Deed", "pdf"),
            make_doc("Lease Agreements", "docx"),
            make_doc("Floor Plans", "pdf"),
            make_doc("Valuation 2024", "xlsx"),
        ],
    );
    a1.images = vec!["https://placehold.co/400x400/1a365d/FFF?text=Main+Office".to_string()];

    let mut a2 = make_asset(
        "Retail Plaza",
        "Street-level retail complex with 14 tenancies, high foot traffic corner site.",
        "88 Queen St, Brisbane QLD 4000",
        1_200_000.0,
        1_450_000.0,
        vec![
            make_doc("Title Deed", "pdf"),
            make_doc("Tenant Schedule", "xlsx"),
            make_doc("Inspection Report", "pdf"),
        ],
    );
    a2.images = vec!["https://placehold.co/400x400/2a4365/FFF?text=Retail+Plaza".to_string()];

    let mut a3 = make_asset(
        "Meridian Tower Suite 9",
        "Premium sublease office suite on level 9, panoramic harbour views, 420 sqm.",
        "100 Barangaroo Ave, Sydney NSW 2000",
        980_000.0,
        1_150_000.0,
        vec![
            make_doc("Sublease Agreement", "docx"),
            make_doc("Fit-Out Schedule", "pdf"),
            make_doc("Building Compliance", "pdf"),
        ],
    );
    a3.images = vec!["https://placehold.co/400x400/2c5282/FFF?text=Meridian".to_string()];

    let mut a4 = make_asset(
        "Exchange Court Carpark",
        "Multi-deck 280-bay commercial carpark adjacent to main office tower.",
        "3 Exchange Court, Sydney NSW 2000",
        750_000.0,
        820_000.0,
        vec![
            make_doc("Carpark Licence", "pdf"),
            make_doc("Revenue Report", "xlsx"),
        ],
    );
    a4.images = vec!["https://placehold.co/400x400/2d3748/FFF?text=Carpark".to_string()];

    let mut a5 = make_asset(
        "12345 Tan St Residence",
        "Residential property on Tan St, Gold Coast. 4 bed, 2 bath, double garage.",
        "12345 Tan St, Gold Coast, QLD 4000",
        850_000.0,
        920_000.0,
        vec![
            make_doc("Contract of Sale", "pdf"),
            make_doc("Building & Pest Inspection", "pdf"),
            make_doc("Title Search", "pdf"),
            make_doc("Rental Appraisal", "docx"),
        ],
    );
    a5.images = vec!["https://placehold.co/400x400/744210/FFF?text=Tan+St".to_string()];

    let mut a6 = make_asset(
        "567 Modl Ct Residence",
        "Residential property on Modl Ct, Gold Coast. 3 bed, 2 bath, single garage.",
        "567 Modl Ct, Gold Coast, QLD 4001",
        720_000.0,
        780_000.0,
        vec![
            make_doc("Contract of Sale", "pdf"),
            make_doc("Strata Report", "pdf"),
            make_doc("Tenant Lease", "docx"),
            make_doc("Depreciation Schedule", "xlsx"),
        ],
    );
    a6.images = vec!["https://placehold.co/400x400/7b341e/FFF?text=Modl+Ct".to_string()];

    group1.assets = vec![a1, a2, a3, a4, a5, a6];
    group1.recalculate_values();

    // Suburban Offices group
    let mut group2 = crate::models::AssetGroup::new("Suburban Offices".to_string());
    group2.description = Some("Technology park and suburban office campus holdings".to_string());
    group2.documents = vec![
        make_doc("Campus Master Plan", "pdf"),
        make_doc("Occupancy Report Q4", "xlsx"),
    ];

    let mut b1 = make_asset(
        "Tech Park Building A",
        "Modern 4-storey office building, open-plan, 2800 sqm NLA, NBN connected.",
        "15 Innovation Dr, Macquarie Park NSW 2113",
        1_800_000.0,
        2_100_000.0,
        vec![
            make_doc("Title Deed", "pdf"),
            make_doc("Lease Roll", "xlsx"),
            make_doc("Energy Audit", "pdf"),
            make_doc("Fitout Specs", "docx"),
        ],
    );
    b1.images = vec!["https://placehold.co/400x400/276749/FFF?text=Tech+Park+A".to_string()];

    let mut b2 = make_asset(
        "Tech Park Building B",
        "Companion building to Building A, shared amenities, 2400 sqm NLA.",
        "17 Innovation Dr, Macquarie Park NSW 2113",
        1_600_000.0,
        1_850_000.0,
        vec![
            make_doc("Title Deed", "pdf"),
            make_doc("Lease Roll", "xlsx"),
            make_doc("NABERS Rating", "pdf"),
        ],
    );
    b2.images = vec!["https://placehold.co/400x400/2f855a/FFF?text=Tech+Park+B".to_string()];

    let mut b3 = make_asset(
        "Parkside Annex",
        "Single-storey annex building used as a training centre and boardroom facility.",
        "19 Innovation Dr, Macquarie Park NSW 2113",
        620_000.0,
        710_000.0,
        vec![
            make_doc("Building Survey", "pdf"),
            make_doc("Maintenance Schedule", "docx"),
        ],
    );
    b3.images = vec!["https://placehold.co/400x400/285e61/FFF?text=Parkside".to_string()];

    let mut b4 = make_asset(
        "North Business Hub",
        "Boutique 6-suite business centre, fully serviced, short-term leases.",
        "7 Rosebery Ave, Rosebery NSW 2018",
        890_000.0,
        975_000.0,
        vec![
            make_doc("Lease Summary", "xlsx"),
            make_doc("Services Agreement", "docx"),
            make_doc("Insurance", "pdf"),
        ],
    );
    b4.images = vec!["https://placehold.co/400x400/322659/FFF?text=North+Hub".to_string()];

    let mut b5 = make_asset(
        "5454 Matter St Commercial",
        "Commercial real estate on Matter St, Gold Coast. Ground-floor retail with office above.",
        "5454 Matter St, Gold Coast, QLD 4000",
        1_250_000.0,
        1_380_000.0,
        vec![
            make_doc("Title Deed", "pdf"),
            make_doc("Commercial Lease Agreement", "docx"),
            make_doc("Council Zoning Certificate", "pdf"),
            make_doc("Outgoings Schedule", "xlsx"),
        ],
    );
    b5.images = vec!["https://placehold.co/400x400/2c3e50/FFF?text=Matter+St".to_string()];

    let mut b6 = make_asset(
        "321 Porks Crescent Residence",
        "Residential property on Porks Crescent, Gold Coast. 4 bed, 3 bath, pool.",
        "321 Porks Crescent, Gold Coast, QLD 4001",
        980_000.0,
        1_050_000.0,
        vec![
            make_doc("Contract of Sale", "pdf"),
            make_doc("Pool Compliance Certificate", "pdf"),
            make_doc("Body Corporate Disclosure", "docx"),
            make_doc("Rental Income History", "xlsx"),
        ],
    );
    b6.images = vec!["https://placehold.co/400x400/6b2737/FFF?text=Porks+Crsct".to_string()];

    group2.assets = vec![b1, b2, b3, b4, b5, b6];
    group2.recalculate_values();

    p.asset_groups = vec![group1, group2];
    p.recalculate_values();
    p
}

/// Generate a simple asset with a random-ish name and value.
fn gen_asset(idx: usize, prefix: &str, base_value: f64) -> crate::models::Asset {
    use crate::types::AssetType;
    let asset_types = [
        AssetType::RealEstate,
        AssetType::Vehicle,
        AssetType::Equipment,
        AssetType::Stock,
        AssetType::Bond,
        AssetType::Commodity,
        AssetType::Digital,
        AssetType::IntellectualProperty,
    ];
    let at = asset_types[idx % asset_types.len()].clone();
    let purchase = base_value + (idx as f64 * 10_000.0);
    let current = purchase * (1.0 + ((idx % 7) as f64 * 0.03));
    let mut a = crate::models::Asset::new(format!("{} #{}", prefix, idx + 1), at, purchase);
    a.description = Some(format!("Test asset {} for portfolio testing.", idx + 1));
    a.location = Some(format!("Test Location {}", idx + 1));
    a.documents = vec![
        make_doc(&format!("{} #{} Title Deed", prefix, idx + 1), "pdf"),
        make_doc(&format!("{} #{} Valuation", prefix, idx + 1), "xlsx"),
        make_doc(&format!("{} #{} Inspection", prefix, idx + 1), "pdf"),
    ];
    a.update_value(current);
    a
}

/// Portfolio 2: mixed assets — 3 direct + 8 assets across 2 groups
pub fn seed_portfolio_2(owner_id: Uuid) -> Portfolio {
    let mut p = Portfolio::new(
        "Mixed Investments".to_string(),
        owner_id,
        crate::types::Currency::USD,
    );
    p.description = Some("Diverse asset collection for testing".to_string());
    p.tags = vec!["mixed".to_string(), "test".to_string()];
    p.documents = vec![
        make_doc("Portfolio Summary", "pdf"),
        make_doc("Asset Allocation Report", "xlsx"),
        make_doc("Investment Policy Statement", "docx"),
    ];

    // 3 direct assets
    for i in 0..3 {
        p.assets.push(gen_asset(i, "Direct Asset", 500_000.0));
    }

    // Group 1: 4 assets
    let mut g1 = crate::models::AssetGroup::new("Group Alpha".to_string());
    g1.description = Some("First test group".to_string());
    g1.documents = vec![
        make_doc("Group Alpha Overview", "pdf"),
        make_doc("Alpha Performance Report", "xlsx"),
    ];
    for i in 0..4 {
        g1.assets.push(gen_asset(i, "Alpha Asset", 300_000.0));
    }
    g1.recalculate_values();

    // Group 2: 4 assets
    let mut g2 = crate::models::AssetGroup::new("Group Beta".to_string());
    g2.description = Some("Second test group".to_string());
    g2.documents = vec![
        make_doc("Group Beta Overview", "pdf"),
        make_doc("Beta Performance Report", "xlsx"),
    ];
    for i in 0..4 {
        g2.assets.push(gen_asset(i + 4, "Beta Asset", 250_000.0));
    }
    g2.recalculate_values();

    p.asset_groups = vec![g1, g2];
    p.recalculate_values();
    p
}

/// Portfolio 3: 5 asset groups with 3, 10, 19, 37, 98 assets
pub fn seed_portfolio_3(owner_id: Uuid) -> Portfolio {
    let mut p = Portfolio::new(
        "Large Scale Portfolio".to_string(),
        owner_id,
        crate::types::Currency::USD,
    );
    p.description = Some("Stress test portfolio with large asset groups".to_string());
    p.tags = vec!["large-scale".to_string(), "stress-test".to_string()];
    p.documents = vec![
        make_doc("Portfolio Master Plan", "pdf"),
        make_doc("Risk Assessment Report", "pdf"),
        make_doc("Quarterly Performance Summary", "xlsx"),
        make_doc("Compliance Certificate", "docx"),
    ];

    let group_specs: [(usize, &str); 5] = [
        (3, "Mini Group"),
        (10, "Small Group"),
        (19, "Medium Group"),
        (37, "Large Group"),
        (98, "Mega Group"),
    ];

    let mut groups = Vec::new();
    for (count, name) in group_specs {
        let mut g = crate::models::AssetGroup::new(name.to_string());
        g.description = Some(format!("{} with {} assets", name, count));
        g.documents = vec![
            make_doc(&format!("{} Overview", name), "pdf"),
            make_doc(&format!("{} Asset Register", name), "xlsx"),
        ];
        for i in 0..count {
            g.assets.push(gen_asset(i, name, 100_000.0));
        }
        g.recalculate_values();
        groups.push(g);
    }

    p.asset_groups = groups;
    p.recalculate_values();
    p
}

/// Portfolio: only direct assets (4 assets, no groups).
pub fn seed_direct_portfolio(owner_id: Uuid, org_id: Option<Uuid>) -> Portfolio {
    let mut p = Portfolio::new(
        "Single Assets Investments".to_string(),
        owner_id,
        crate::types::Currency::USD,
    );
    p.organization_id = org_id;
    p.description = Some("Portfolio with only direct assets for single-asset viewport testing".to_string());
    p.tags = vec!["direct-only".to_string(), "test".to_string()];
    p.documents = vec![make_doc("Direct Portfolio Overview", "pdf")];

    for i in 0..4 {
        let mut a = gen_asset(i, "Direct Asset", 500_000.0);
        a.organization_id = org_id;
        p.assets.push(a);
    }

    p.assigned_users.push(owner_id);
    p.recalculate_values();
    p
}

/// Portfolio: only asset groups (2 groups, 4 assets each).
pub fn seed_groups_only_portfolio(owner_id: Uuid, org_id: Option<Uuid>) -> Portfolio {
    let mut p = Portfolio::new(
        "Asset Groups Investments".to_string(),
        owner_id,
        crate::types::Currency::USD,
    );
    p.organization_id = org_id;
    p.description = Some("Portfolio with only two asset groups for group viewport testing".to_string());
    p.tags = vec!["groups-only".to_string(), "test".to_string()];
    p.documents = vec![make_doc("Groups Portfolio Overview", "pdf")];

    let group_specs: [(usize, &str, f64); 2] = [
        (0, "Group Alpha", 300_000.0),
        (1, "Group Beta", 350_000.0),
    ];

    let mut groups = Vec::new();
    for (group_idx, name, base_value) in group_specs {
        let mut g = crate::models::AssetGroup::new(name.to_string());
        g.description = Some(format!("{} with 4 assets", name));
        g.documents = vec![make_doc(&format!("{} Overview", name), "pdf")];
        for i in 0..4 {
            let idx = group_idx * 4 + i;
            let mut a = gen_asset(idx, &format!("{} Asset", name), base_value);
            a.organization_id = org_id;
            g.assets.push(a);
        }
        g.recalculate_values();
        groups.push(g);
    }

    p.asset_groups = groups;
    p.assigned_users.push(owner_id);
    p.recalculate_values();
    p
}

fn make_channel(
    name: &str,
    channel_type: ChannelType,
    asset_id: Uuid,
    portfolio_id: Uuid,
    nightly_rate: f64,
) -> Channel {
    let now = chrono::Utc::now();
    Channel {
        id: Uuid::new_v4(),
        name: name.to_string(),
        channel_type,
        linked_asset_id: Some(asset_id),
        portfolio_id: Some(portfolio_id),
        connection_status: ConnectionStatus::Connected,
        sync_direction: SyncDirection::TwoWay,
        nightly_rate_override: Some(nightly_rate),
        minimum_nights: Some(2),
        maximum_nights: None,
        commission_percent: Some(10.0),
        currency: crate::types::Currency::USD,
        last_sync_at: Some(now),
        last_sync_status: Some("Connected (seeded)".to_string()),
        sync_errors: Vec::new(),
        enabled: true,
        created_at: now,
        updated_at: now,
    }
}

fn make_channel_asset(
    name: &str,
    desc: &str,
    location: &str,
    purchase: f64,
    current: f64,
    org_id: Uuid,
    portfolio_id: Uuid,
    channels: Vec<(ChannelType, f64, &str)>,
) -> (Asset, Vec<Channel>) {
    let mut asset = make_asset(
        name,
        desc,
        location,
        purchase,
        current,
        vec![make_doc("Title Deed", "pdf")],
    );
    asset.organization_id = Some(org_id);
    asset.asset_type = AssetType::RealEstate;
    asset.images = vec![format!(
        "https://placehold.co/400x400/2d3748/FFF?text={}",
        name.replace(' ', "+")
    )];

    let mut channel_objects = Vec::new();
    for (channel_type, rate, suffix) in channels {
        let channel_name = format!("{} - {}", name, suffix);
        let channel = make_channel(&channel_name, channel_type, asset.id, portfolio_id, rate);
        asset.channel_ids.push(channel.id);
        channel_objects.push(channel);
    }

    (asset, channel_objects)
}

/// Seed the RedChannel organization with Red as owner, two channel-test portfolios, and multiple assets.
pub fn seed_red_channel_data(
    app_store: &mut AppStore,
    organization_store: &mut OrganizationStore,
) {
    let owner_id = app_store.current_user.id;
    let owner_email = app_store.current_user.email.clone();

    let mut red_channel = Organization::new("RedChannel".to_string(), owner_id);
    red_channel.description = Some("RedChannel — channel management test organization".to_string());
    red_channel.settings.color = Some("#ef4444".to_string());
    red_channel.business_type = Some("Company".to_string());
    red_channel.abn = Some("1".to_string());
    red_channel.lei = Some("1".to_string());
    red_channel.business_address = Some("1 RedChannel Way, Melbourne VIC 3000, Australia".to_string());
    red_channel.business_phone = Some("+61 1 234 567 890".to_string());
    red_channel.business_email = Some("contact@redchannel.com".to_string());
    red_channel.add_member(owner_id);
    let red_channel_id = red_channel.id;
    organization_store.organizations.push(red_channel);

    let mut red_channel_owner = User::new("Red".to_string(), owner_email, UserRole::Owner);
    red_channel_owner.id = owner_id;
    red_channel_owner.organization_id = Some(red_channel_id);
    organization_store.organization_users.push(red_channel_owner);

    // Portfolio 1: short-stay rentals
    let mut p1 = Portfolio::new(
        "RedChannel Short Stays".to_string(),
        owner_id,
        crate::types::Currency::USD,
    );
    p1.organization_id = Some(red_channel_id);
    p1.description = Some("Short-stay rental portfolio for channel management testing".to_string());
    p1.tags = vec!["channel".to_string(), "short-stay".to_string()];
    p1.assigned_users.push(owner_id);

    let p1_id = p1.id;
    let short_stay_assets: Vec<(_, _, _, f64, f64, Vec<(ChannelType, f64, &str)>)> = vec![
        (
            "Studio Loft A",
            "Downtown studio apartment with balcony and city views.",
            "101 Collins St, Melbourne VIC 3000",
            350_000.0,
            420_000.0,
            vec![
                (ChannelType::Airbnb, 145.0, "Airbnb"),
                (ChannelType::BookingCom, 140.0, "Booking.com"),
                (ChannelType::Expedia, 138.0, "Expedia"),
            ],
        ),
        (
            "City Apartment B",
            "Modern 2-bedroom CBD apartment with high-rise views.",
            "200 Spencer St, Melbourne VIC 3000",
            580_000.0,
            650_000.0,
            vec![
                (ChannelType::Airbnb, 210.0, "Airbnb"),
                (ChannelType::BookingCom, 205.0, "Booking.com"),
                (ChannelType::Expedia, 200.0, "Expedia"),
            ],
        ),
        (
            "Suburban Cottage C",
            "3-bedroom family cottage with garden and quiet street.",
            "12 Rose St, Richmond VIC 3121",
            720_000.0,
            780_000.0,
            vec![
                (ChannelType::Airbnb, 185.0, "Airbnb"),
                (ChannelType::Vrbo, 180.0, "Vrbo"),
                (ChannelType::BookingCom, 182.0, "Booking.com"),
            ],
        ),
        (
            "Beach House D",
            "4-bedroom beachfront house with pool and deck.",
            "1 Beach Rd, St Kilda VIC 3182",
            1_200_000.0,
            1_350_000.0,
            vec![
                (ChannelType::Airbnb, 350.0, "Airbnb"),
                (ChannelType::Vrbo, 340.0, "Vrbo"),
                (ChannelType::Expedia, 345.0, "Expedia"),
                (ChannelType::Other("Luxico".to_string()), 360.0, "Luxico"),
            ],
        ),
    ];

    for (name, desc, loc, purchase, current, channels) in short_stay_assets {
        let (mut asset, channel_objects) = make_channel_asset(
            name,
            desc,
            loc,
            purchase,
            current,
            red_channel_id,
            p1_id,
            channels,
        );
        if let Some(first_channel) = channel_objects.first() {
            let channel_id = first_channel.id;
            let start = chrono::Utc::now() + chrono::Duration::days(3);
            let end = start + chrono::Duration::days(5);
            let mut booking = Booking::new(
                asset.id,
                Some(channel_id),
                BookingSource::TestChannel,
                "Demo Guest".to_string(),
                start,
                end,
                first_channel.nightly_rate_override.unwrap_or(150.0),
            );
            booking.status = BookingStatus::Confirmed;
            let event = CalendarEvent::for_booking(
                format!("{} - {}", booking.guest_name, asset.name),
                start,
                end,
                Some(p1_id),
                asset.id,
                Some(channel_id),
                booking.id,
                "Test Channel",
            );
            booking.calendar_event_ids.push(event.id);
            asset.calendar_events.push(event);
            app_store.bookings.push(booking);
        }
        p1.assets.push(asset);
        for c in channel_objects {
            app_store.add_channel(c);
        }
    }
    p1.recalculate_values();
    app_store.portfolios.push(p1);

    // Portfolio 2: long-stay / corporate rentals
    let mut p2 = Portfolio::new(
        "RedChannel Long Stays".to_string(),
        owner_id,
        crate::types::Currency::USD,
    );
    p2.organization_id = Some(red_channel_id);
    p2.description = Some("Long-stay and corporate rental portfolio for channel management testing".to_string());
    p2.tags = vec!["channel".to_string(), "long-stay".to_string()];
    p2.assigned_users.push(owner_id);

    let p2_id = p2.id;
    let long_stay_assets: Vec<(_, _, _, f64, f64, Vec<(ChannelType, f64, &str)>)> = vec![
        (
            "Executive Suite 1",
            "Premium 1-bedroom executive suite, fully serviced.",
            "50 Collins St, Melbourne VIC 3000",
            450_000.0,
            510_000.0,
            vec![
                (ChannelType::Test, 125.0, "Test Channel"),
                (ChannelType::Vrbo, 120.0, "Vrbo"),
                (ChannelType::Other("Corporate Portal".to_string()), 130.0, "Corporate Portal"),
            ],
        ),
        (
            "Corporate Unit 2",
            "2-bedroom corporate apartment, minimum 28-night leases.",
            "80 Lonsdale St, Melbourne VIC 3000",
            620_000.0,
            700_000.0,
            vec![
                (ChannelType::Test, 150.0, "Test Channel"),
                (ChannelType::LinkedIn, 145.0, "LinkedIn"),
                (ChannelType::Other("Corporate Portal".to_string()), 155.0, "Corporate Portal"),
            ],
        ),
        (
            "Manager Residence 3",
            "3-bedroom townhouse for on-site managers and long-stay guests.",
            "15 High St, Kew VIC 3101",
            850_000.0,
            920_000.0,
            vec![
                (ChannelType::Test, 165.0, "Test Channel"),
                (ChannelType::Vrbo, 160.0, "Vrbo"),
                (ChannelType::BookingCom, 162.0, "Booking.com"),
            ],
        ),
    ];

    for (name, desc, loc, purchase, current, channels) in long_stay_assets {
        let (mut asset, channel_objects) = make_channel_asset(
            name,
            desc,
            loc,
            purchase,
            current,
            red_channel_id,
            p2_id,
            channels,
        );
        if let Some(first_channel) = channel_objects.first() {
            let channel_id = first_channel.id;
            let start = chrono::Utc::now() + chrono::Duration::days(10);
            let end = start + chrono::Duration::days(30);
            let mut booking = Booking::new(
                asset.id,
                Some(channel_id),
                BookingSource::TestChannel,
                "Corp Guest".to_string(),
                start,
                end,
                first_channel.nightly_rate_override.unwrap_or(150.0),
            );
            booking.status = BookingStatus::Confirmed;
            let event = CalendarEvent::for_booking(
                format!("{} - {}", booking.guest_name, asset.name),
                start,
                end,
                Some(p2_id),
                asset.id,
                Some(channel_id),
                booking.id,
                "Test Channel",
            );
            booking.calendar_event_ids.push(event.id);
            asset.calendar_events.push(event);
            app_store.bookings.push(booking);
        }
        p2.assets.push(asset);
        for c in channel_objects {
            app_store.add_channel(c);
        }
    }
    p2.recalculate_values();
    app_store.portfolios.push(p2);
}
