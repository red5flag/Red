use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ═══════════════════════════════════════════
// Actor filter — who triggers the rule
// ═══════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorFilter {
    SpecificUser(Uuid),
    AnyUser,
    UserWithRole(Uuid),
    UserInOrganization(Uuid),
    UserAssignedToPortfolio(Uuid),
    UserRoleAbove(Uuid),
    UserRoleBelow(Uuid),
}

impl ActorFilter {
    pub fn label(&self) -> String {
        match self {
            ActorFilter::SpecificUser(id) => format!("User {}", &id.to_string()[..8]),
            ActorFilter::AnyUser => "Any user".to_string(),
            ActorFilter::UserWithRole(id) => format!("User with role {}", &id.to_string()[..8]),
            ActorFilter::UserInOrganization(id) => {
                format!("User in organization {}", &id.to_string()[..8])
            }
            ActorFilter::UserAssignedToPortfolio(id) => {
                format!("User assigned to portfolio {}", &id.to_string()[..8])
            }
            ActorFilter::UserRoleAbove(id) => {
                format!("User with role above {}", &id.to_string()[..8])
            }
            ActorFilter::UserRoleBelow(id) => {
                format!("User with role below {}", &id.to_string()[..8])
            }
        }
    }

    pub fn type_label(&self) -> &'static str {
        match self {
            ActorFilter::SpecificUser(_) => "Specific user",
            ActorFilter::AnyUser => "Any user",
            ActorFilter::UserWithRole(_) => "User with selected role",
            ActorFilter::UserInOrganization(_) => "User in selected organization",
            ActorFilter::UserAssignedToPortfolio(_) => "User assigned to selected portfolio",
            ActorFilter::UserRoleAbove(_) => "User with role above selected",
            ActorFilter::UserRoleBelow(_) => "User with role below selected",
        }
    }
}

// ═══════════════════════════════════════════
// Rule action — what action triggers the rule
// ═══════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleAction {
    Creates,
    Edits,
    Deletes,
    Hides,
    Unhides,
    Archives,
    Restores,
    Locks,
    Unlocks,
    Approves,
    Rejects,
    Submits,
    Assigns,
    Transfers,
    Exports,
    ChangesRole,
    ChangesPermissions,
    ChangesDocumentStatus,
    ChangesTransactionStatus,
}

impl RuleAction {
    pub fn label(&self) -> &'static str {
        match self {
            RuleAction::Creates => "Creates",
            RuleAction::Edits => "Edits",
            RuleAction::Deletes => "Deletes",
            RuleAction::Hides => "Hides",
            RuleAction::Unhides => "Unhides",
            RuleAction::Archives => "Archives",
            RuleAction::Restores => "Restores",
            RuleAction::Locks => "Locks",
            RuleAction::Unlocks => "Unlocks",
            RuleAction::Approves => "Approves",
            RuleAction::Rejects => "Rejects",
            RuleAction::Submits => "Submits",
            RuleAction::Assigns => "Assigns",
            RuleAction::Transfers => "Transfers",
            RuleAction::Exports => "Exports",
            RuleAction::ChangesRole => "Changes role",
            RuleAction::ChangesPermissions => "Changes permissions",
            RuleAction::ChangesDocumentStatus => "Changes document status",
            RuleAction::ChangesTransactionStatus => "Changes transaction status",
        }
    }

    pub fn all() -> Vec<RuleAction> {
        vec![
            RuleAction::Creates,
            RuleAction::Edits,
            RuleAction::Deletes,
            RuleAction::Hides,
            RuleAction::Unhides,
            RuleAction::Archives,
            RuleAction::Restores,
            RuleAction::Locks,
            RuleAction::Unlocks,
            RuleAction::Approves,
            RuleAction::Rejects,
            RuleAction::Submits,
            RuleAction::Assigns,
            RuleAction::Transfers,
            RuleAction::Exports,
            RuleAction::ChangesRole,
            RuleAction::ChangesPermissions,
            RuleAction::ChangesDocumentStatus,
            RuleAction::ChangesTransactionStatus,
        ]
    }
}

// ═══════════════════════════════════════════
// Target type — what resource is affected
// ═══════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetType {
    Organization,
    Portfolio,
    AssetGroup,
    Asset,
    DirectAsset,
    Document,
    Report,
    Transaction,
    CalendarEvent,
    Contact,
    NetworkingRecord,
    Member,
    Role,
    PermissionGroup,
}

impl TargetType {
    pub fn label(&self) -> &'static str {
        match self {
            TargetType::Organization => "Organization",
            TargetType::Portfolio => "Portfolio",
            TargetType::AssetGroup => "Asset group",
            TargetType::Asset => "Asset",
            TargetType::DirectAsset => "Direct asset",
            TargetType::Document => "Document",
            TargetType::Report => "Report",
            TargetType::Transaction => "Transaction",
            TargetType::CalendarEvent => "Calendar event",
            TargetType::Contact => "Contact",
            TargetType::NetworkingRecord => "Networking record",
            TargetType::Member => "Member",
            TargetType::Role => "Role",
            TargetType::PermissionGroup => "Permission group",
        }
    }

    pub fn all() -> Vec<TargetType> {
        vec![
            TargetType::Organization,
            TargetType::Portfolio,
            TargetType::AssetGroup,
            TargetType::Asset,
            TargetType::DirectAsset,
            TargetType::Document,
            TargetType::Report,
            TargetType::Transaction,
            TargetType::CalendarEvent,
            TargetType::Contact,
            TargetType::NetworkingRecord,
            TargetType::Member,
            TargetType::Role,
            TargetType::PermissionGroup,
        ]
    }
}

// ═══════════════════════════════════════════
// Scope — where the rule applies
// ═══════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleScope {
    EntireOrganization,
    SelectedOrganization(Uuid),
    MultipleOrganizations(Vec<Uuid>),
    SelectedPortfolio(Uuid),
    MultiplePortfolios(Vec<Uuid>),
    SelectedAssetGroup(Uuid),
    SelectedAsset(Uuid),
    DirectAssets,
    ReportingOnly,
    DocumentControlsOnly,
    TransactionsOnly,
    NetworkingOnly,
    CalendarOnly,
    HistoryAuditOnly,
}

impl RuleScope {
    pub fn label(&self) -> String {
        match self {
            RuleScope::EntireOrganization => "Entire organization".to_string(),
            RuleScope::SelectedOrganization(id) => format!("Organization {}", &id.to_string()[..8]),
            RuleScope::MultipleOrganizations(ids) => format!("{} organizations", ids.len()),
            RuleScope::SelectedPortfolio(id) => format!("Portfolio {}", &id.to_string()[..8]),
            RuleScope::MultiplePortfolios(ids) => format!("{} portfolios", ids.len()),
            RuleScope::SelectedAssetGroup(id) => format!("Asset group {}", &id.to_string()[..8]),
            RuleScope::SelectedAsset(id) => format!("Asset {}", &id.to_string()[..8]),
            RuleScope::DirectAssets => "Direct assets".to_string(),
            RuleScope::ReportingOnly => "Reporting only".to_string(),
            RuleScope::DocumentControlsOnly => "Document controls only".to_string(),
            RuleScope::TransactionsOnly => "Transactions only".to_string(),
            RuleScope::NetworkingOnly => "Networking only".to_string(),
            RuleScope::CalendarOnly => "Calendar only".to_string(),
            RuleScope::HistoryAuditOnly => "History/audit only".to_string(),
        }
    }

    pub fn type_label(&self) -> &'static str {
        match self {
            RuleScope::EntireOrganization => "Entire organization",
            RuleScope::SelectedOrganization(_) => "Selected organization",
            RuleScope::MultipleOrganizations(_) => "Multiple organizations",
            RuleScope::SelectedPortfolio(_) => "Selected portfolio",
            RuleScope::MultiplePortfolios(_) => "Multiple portfolios",
            RuleScope::SelectedAssetGroup(_) => "Selected asset group",
            RuleScope::SelectedAsset(_) => "Selected asset",
            RuleScope::DirectAssets => "Direct assets",
            RuleScope::ReportingOnly => "Reporting only",
            RuleScope::DocumentControlsOnly => "Document controls only",
            RuleScope::TransactionsOnly => "Transactions only",
            RuleScope::NetworkingOnly => "Networking only",
            RuleScope::CalendarOnly => "Calendar only",
            RuleScope::HistoryAuditOnly => "History/audit only",
        }
    }

    pub fn all_types() -> Vec<&'static str> {
        vec![
            "Entire organization",
            "Selected organization",
            "Multiple organizations",
            "Selected portfolio",
            "Multiple portfolios",
            "Selected asset group",
            "Selected asset",
            "Direct assets",
            "Reporting only",
            "Document controls only",
            "Transactions only",
            "Networking only",
            "Calendar only",
            "History/audit only",
        ]
    }
}

// ═══════════════════════════════════════════
// Recipient — who gets notified
// ═══════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationRecipient {
    ActingUser,
    ResourceOwner,
    OrganizationOwner,
    SpecificUser(Uuid),
    SpecificRole(Uuid),
    SpecificTeam(Uuid),
    MembersAboveActorRole,
    MembersBelowActorRole,
    MembersWithApprovePermission,
    MembersAssignedToPortfolio,
    MembersWatchingItem,
}

impl NotificationRecipient {
    pub fn label(&self) -> String {
        match self {
            NotificationRecipient::ActingUser => "Acting user".to_string(),
            NotificationRecipient::ResourceOwner => "Resource owner".to_string(),
            NotificationRecipient::OrganizationOwner => "Organization owner".to_string(),
            NotificationRecipient::SpecificUser(id) => format!("User {}", &id.to_string()[..8]),
            NotificationRecipient::SpecificRole(id) => format!("Role {}", &id.to_string()[..8]),
            NotificationRecipient::SpecificTeam(id) => format!("Team {}", &id.to_string()[..8]),
            NotificationRecipient::MembersAboveActorRole => {
                "Members above actor's role".to_string()
            }
            NotificationRecipient::MembersBelowActorRole => {
                "Members below actor's role".to_string()
            }
            NotificationRecipient::MembersWithApprovePermission => {
                "Members with permission to approve".to_string()
            }
            NotificationRecipient::MembersAssignedToPortfolio => {
                "Members assigned to affected portfolio".to_string()
            }
            NotificationRecipient::MembersWatchingItem => {
                "Members watching the affected item".to_string()
            }
        }
    }

    pub fn type_label(&self) -> &'static str {
        match self {
            NotificationRecipient::ActingUser => "Acting user",
            NotificationRecipient::ResourceOwner => "Resource owner",
            NotificationRecipient::OrganizationOwner => "Organization owner",
            NotificationRecipient::SpecificUser(_) => "Specific user",
            NotificationRecipient::SpecificRole(_) => "Specific role",
            NotificationRecipient::SpecificTeam(_) => "Specific team",
            NotificationRecipient::MembersAboveActorRole => "Members above actor's role",
            NotificationRecipient::MembersBelowActorRole => "Members below actor's role",
            NotificationRecipient::MembersWithApprovePermission => {
                "Members with permission to approve"
            }
            NotificationRecipient::MembersAssignedToPortfolio => {
                "Members assigned to affected portfolio"
            }
            NotificationRecipient::MembersWatchingItem => "Members watching the affected item",
        }
    }

    pub fn all_types() -> Vec<&'static str> {
        vec![
            "Acting user",
            "Resource owner",
            "Organization owner",
            "Specific user",
            "Specific role",
            "Specific team",
            "Members above actor's role",
            "Members below actor's role",
            "Members with permission to approve",
            "Members assigned to affected portfolio",
            "Members watching the affected item",
        ]
    }
}

// ═══════════════════════════════════════════
// Condition — individual condition
// ═══════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Is,
    IsNot,
    Contains,
    DoesNotContain,
}

impl ConditionOperator {
    pub fn label(&self) -> &'static str {
        match self {
            ConditionOperator::Equals => "equals",
            ConditionOperator::NotEquals => "does not equal",
            ConditionOperator::GreaterThan => "is greater than",
            ConditionOperator::LessThan => "is less than",
            ConditionOperator::GreaterThanOrEqual => "is greater than or equal to",
            ConditionOperator::LessThanOrEqual => "is less than or equal to",
            ConditionOperator::Is => "is",
            ConditionOperator::IsNot => "is not",
            ConditionOperator::Contains => "contains",
            ConditionOperator::DoesNotContain => "does not contain",
        }
    }

    pub fn all() -> Vec<ConditionOperator> {
        vec![
            ConditionOperator::Equals,
            ConditionOperator::NotEquals,
            ConditionOperator::GreaterThan,
            ConditionOperator::LessThan,
            ConditionOperator::GreaterThanOrEqual,
            ConditionOperator::LessThanOrEqual,
            ConditionOperator::Is,
            ConditionOperator::IsNot,
            ConditionOperator::Contains,
            ConditionOperator::DoesNotContain,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ConditionValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    RoleId(Uuid),
    OrgId(Uuid),
    PortfolioId(Uuid),
    Currency(String),
    DocumentStatus(String),
    TransactionType(String),
    RiskLevel(String),
}

impl ConditionValue {
    pub fn display(&self) -> String {
        match self {
            ConditionValue::Text(s) => s.clone(),
            ConditionValue::Number(n) => format!("{}", n),
            ConditionValue::Boolean(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            ConditionValue::RoleId(id) => format!("role {}", &id.to_string()[..8]),
            ConditionValue::OrgId(id) => format!("organization {}", &id.to_string()[..8]),
            ConditionValue::PortfolioId(id) => format!("portfolio {}", &id.to_string()[..8]),
            ConditionValue::Currency(c) => c.clone(),
            ConditionValue::DocumentStatus(s) => s.clone(),
            ConditionValue::TransactionType(s) => s.clone(),
            ConditionValue::RiskLevel(s) => s.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionLeftValue {
    ActorIsUser,
    ActorHasRole,
    ActorRoleRankAbove,
    ActorRoleRankBelow,
    ActorRoleRankEqual,
    TargetBelongsToOrganization,
    TargetBelongsToPortfolio,
    TargetIsLocked,
    DocumentStatus,
    TransactionAmount,
    TransactionCurrency,
    TransactionIsFiat,
    TransactionIsCrypto,
    CryptoNetwork,
    PayeeIsNew,
    PayeeIsHighRisk,
    ActionOutsideBusinessHours,
    ActionAffectsFinalLockedRecords,
    ActionAffectsRoleEqualOrAboveActor,
    ActionAffectsDocumentNotOwnedByActor,
    ActionAffectsPortfolioNotOwnedByActor,
}

impl ConditionLeftValue {
    pub fn label(&self) -> &'static str {
        match self {
            ConditionLeftValue::ActorIsUser => "Actor is user",
            ConditionLeftValue::ActorHasRole => "Actor has role",
            ConditionLeftValue::ActorRoleRankAbove => "Actor role rank is above",
            ConditionLeftValue::ActorRoleRankBelow => "Actor role rank is below",
            ConditionLeftValue::ActorRoleRankEqual => "Actor role rank is equal to",
            ConditionLeftValue::TargetBelongsToOrganization => "Target belongs to organization",
            ConditionLeftValue::TargetBelongsToPortfolio => "Target belongs to portfolio",
            ConditionLeftValue::TargetIsLocked => "Target is locked",
            ConditionLeftValue::DocumentStatus => "Document status",
            ConditionLeftValue::TransactionAmount => "Transaction amount",
            ConditionLeftValue::TransactionCurrency => "Transaction currency",
            ConditionLeftValue::TransactionIsFiat => "Transaction is fiat",
            ConditionLeftValue::TransactionIsCrypto => "Transaction is crypto",
            ConditionLeftValue::CryptoNetwork => "Crypto network",
            ConditionLeftValue::PayeeIsNew => "Payee is new",
            ConditionLeftValue::PayeeIsHighRisk => "Payee is high-risk",
            ConditionLeftValue::ActionOutsideBusinessHours => {
                "Action occurs outside business hours"
            }
            ConditionLeftValue::ActionAffectsFinalLockedRecords => {
                "Action affects final/locked records"
            }
            ConditionLeftValue::ActionAffectsRoleEqualOrAboveActor => {
                "Action affects a role equal to or above the actor's role"
            }
            ConditionLeftValue::ActionAffectsDocumentNotOwnedByActor => {
                "Action affects a document not owned by the actor"
            }
            ConditionLeftValue::ActionAffectsPortfolioNotOwnedByActor => {
                "Action affects a portfolio the actor does not own"
            }
        }
    }

    pub fn all() -> Vec<ConditionLeftValue> {
        vec![
            ConditionLeftValue::ActorIsUser,
            ConditionLeftValue::ActorHasRole,
            ConditionLeftValue::ActorRoleRankAbove,
            ConditionLeftValue::ActorRoleRankBelow,
            ConditionLeftValue::ActorRoleRankEqual,
            ConditionLeftValue::TargetBelongsToOrganization,
            ConditionLeftValue::TargetBelongsToPortfolio,
            ConditionLeftValue::TargetIsLocked,
            ConditionLeftValue::DocumentStatus,
            ConditionLeftValue::TransactionAmount,
            ConditionLeftValue::TransactionCurrency,
            ConditionLeftValue::TransactionIsFiat,
            ConditionLeftValue::TransactionIsCrypto,
            ConditionLeftValue::CryptoNetwork,
            ConditionLeftValue::PayeeIsNew,
            ConditionLeftValue::PayeeIsHighRisk,
            ConditionLeftValue::ActionOutsideBusinessHours,
            ConditionLeftValue::ActionAffectsFinalLockedRecords,
            ConditionLeftValue::ActionAffectsRoleEqualOrAboveActor,
            ConditionLeftValue::ActionAffectsDocumentNotOwnedByActor,
            ConditionLeftValue::ActionAffectsPortfolioNotOwnedByActor,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub id: Uuid,
    pub left_value: ConditionLeftValue,
    pub operator: ConditionOperator,
    pub right_value: ConditionValue,
}

impl Condition {
    pub fn new(
        left_value: ConditionLeftValue,
        operator: ConditionOperator,
        right_value: ConditionValue,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            left_value,
            operator,
            right_value,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "{} {} {}",
            self.left_value.label(),
            self.operator.label(),
            self.right_value.display()
        )
    }
}

// ═══════════════════════════════════════════
// Condition group — AND/OR logic
// ═══════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionGroupMode {
    All, // AND
    Any, // OR
}

impl ConditionGroupMode {
    pub fn label(&self) -> &'static str {
        match self {
            ConditionGroupMode::All => "All conditions must match",
            ConditionGroupMode::Any => "Any condition may match",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConditionGroup {
    pub id: Uuid,
    pub mode: ConditionGroupMode,
    pub conditions: Vec<Condition>,
}

impl ConditionGroup {
    pub fn new(mode: ConditionGroupMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            mode,
            conditions: Vec::new(),
        }
    }

    pub fn summary(&self) -> String {
        if self.conditions.is_empty() {
            return "No conditions".to_string();
        }
        let connector = match self.mode {
            ConditionGroupMode::All => " AND ",
            ConditionGroupMode::Any => " OR ",
        };
        self.conditions
            .iter()
            .map(|c| c.summary())
            .collect::<Vec<_>>()
            .join(connector)
    }
}

// ═══════════════════════════════════════════
// Rule action — what the rule does when triggered
// ═══════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleActionType {
    NotifyInApp,
    SendMessage,
    SendEmail,
    AddAuditEntry,
    RequireApproval,
    EscalateToHigherRole,
    BlockAction,
    MarkAsHighRisk,
    CreateReviewTask,
    LockAffectedRecord,
    RequestConfirmationReason,
}

impl RuleActionType {
    pub fn label(&self) -> &'static str {
        match self {
            RuleActionType::NotifyInApp => "Notify in-app",
            RuleActionType::SendMessage => "Send message",
            RuleActionType::SendEmail => "Send email (if enabled)",
            RuleActionType::AddAuditEntry => "Add audit log entry",
            RuleActionType::RequireApproval => "Require approval",
            RuleActionType::EscalateToHigherRole => "Escalate to higher role",
            RuleActionType::BlockAction => "Block action",
            RuleActionType::MarkAsHighRisk => "Mark as high-risk",
            RuleActionType::CreateReviewTask => "Create review task",
            RuleActionType::LockAffectedRecord => "Lock affected record",
            RuleActionType::RequestConfirmationReason => "Request confirmation reason",
        }
    }

    pub fn all() -> Vec<RuleActionType> {
        vec![
            RuleActionType::NotifyInApp,
            RuleActionType::SendMessage,
            RuleActionType::SendEmail,
            RuleActionType::AddAuditEntry,
            RuleActionType::RequireApproval,
            RuleActionType::EscalateToHigherRole,
            RuleActionType::BlockAction,
            RuleActionType::MarkAsHighRisk,
            RuleActionType::CreateReviewTask,
            RuleActionType::LockAffectedRecord,
            RuleActionType::RequestConfirmationReason,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleActionEntry {
    pub id: Uuid,
    pub action_type: RuleActionType,
    pub recipient: Option<NotificationRecipient>,
    pub priority: ActionPriority,
}

impl RuleActionEntry {
    pub fn new(
        action_type: RuleActionType,
        recipient: Option<NotificationRecipient>,
        priority: ActionPriority,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            action_type,
            recipient,
            priority,
        }
    }

    pub fn summary(&self) -> String {
        let recipient_text = self
            .recipient
            .as_ref()
            .map(|r| r.label())
            .unwrap_or_default();
        format!(
            "{} {} ({})",
            self.action_type.label(),
            recipient_text,
            self.priority.label()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionPriority {
    Low,
    Medium,
    High,
    Blocked,
    ApprovalRequired,
}

impl ActionPriority {
    pub fn label(&self) -> &'static str {
        match self {
            ActionPriority::Low => "Low risk",
            ActionPriority::Medium => "Medium risk",
            ActionPriority::High => "High risk",
            ActionPriority::Blocked => "Blocked",
            ActionPriority::ApprovalRequired => "Approval required",
        }
    }

    pub fn all() -> Vec<ActionPriority> {
        vec![
            ActionPriority::Low,
            ActionPriority::Medium,
            ActionPriority::High,
            ActionPriority::Blocked,
            ActionPriority::ApprovalRequired,
        ]
    }
}

// ═══════════════════════════════════════════
// Trigger — the event that fires the rule
// ═══════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuleTrigger {
    pub actor_filter: ActorFilter,
    pub action: RuleAction,
    pub target_type: TargetType,
    pub scope: RuleScope,
}

impl RuleTrigger {
    pub fn summary(&self) -> String {
        format!(
            "When {} {} a {} in {}",
            self.actor_filter.label(),
            self.action.label().to_lowercase(),
            self.target_type.label().to_lowercase(),
            self.scope.label()
        )
    }
}

// ═══════════════════════════════════════════
// Rule — the full rule definition
// ═══════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub organization_id: Uuid,
    pub trigger: RuleTrigger,
    pub condition_group: ConditionGroup,
    pub actions: Vec<RuleActionEntry>,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_by: Uuid,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Rule {
    pub fn new(
        name: String,
        organization_id: Uuid,
        created_by: Uuid,
        trigger: RuleTrigger,
        condition_group: ConditionGroup,
        actions: Vec<RuleActionEntry>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            enabled: true,
            organization_id,
            trigger,
            condition_group,
            actions,
            created_by,
            created_at: now,
            updated_by: created_by,
            updated_at: now,
        }
    }

    pub fn plain_english_summary(&self) -> String {
        let trigger_text = self.trigger.summary();
        let condition_text = if self.condition_group.conditions.is_empty() {
            String::new()
        } else {
            format!(" And if: {}", self.condition_group.summary())
        };
        let actions_text = self
            .actions
            .iter()
            .map(|a| a.summary())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{}.{}. Then: {}",
            trigger_text, condition_text, actions_text
        )
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Rule name is required".to_string());
        }
        if self.actions.is_empty() {
            return Err("At least one action is required".to_string());
        }
        for action in &self.actions {
            match action.action_type {
                RuleActionType::NotifyInApp
                | RuleActionType::SendMessage
                | RuleActionType::SendEmail
                | RuleActionType::EscalateToHigherRole
                | RuleActionType::CreateReviewTask => {
                    if action.recipient.is_none() {
                        return Err(format!(
                            "Action '{}' requires a recipient",
                            action.action_type.label()
                        ));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════
// Rule history entry — audit trail for rule changes
// ═══════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleHistoryEntry {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub rule_name: String,
    pub action: String,
    pub performed_by: Uuid,
    pub performed_by_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: String,
}

impl RuleHistoryEntry {
    pub fn new(
        rule_id: Uuid,
        rule_name: String,
        action: String,
        performed_by: Uuid,
        performed_by_name: String,
        details: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            rule_id,
            rule_name,
            action,
            performed_by,
            performed_by_name,
            timestamp: chrono::Utc::now(),
            details,
        }
    }
}
