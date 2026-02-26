use qryvanta_core::AppResult;
use qryvanta_domain::FieldType;
use serde_json::Value;

/// Logical composition mode for runtime query conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRecordLogicalMode {
    /// Every condition must match.
    And,
    /// Any condition may match.
    Or,
}

impl RuntimeRecordLogicalMode {
    /// Parses transport value into a logical mode.
    pub fn parse_transport(value: &str) -> AppResult<Self> {
        match value {
            "and" => Ok(Self::And),
            "or" => Ok(Self::Or),
            _ => Err(qryvanta_core::AppError::Validation(format!(
                "unknown runtime query logical mode '{value}'"
            ))),
        }
    }

    /// Returns the stable transport value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::And => "and",
            Self::Or => "or",
        }
    }
}

/// Runtime query comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRecordOperator {
    /// JSON equality.
    Eq,
    /// JSON inequality.
    Neq,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Gte,
    /// Less than.
    Lt,
    /// Less than or equal.
    Lte,
    /// String contains comparison.
    Contains,
    /// Membership in a set of values.
    In,
}

impl RuntimeRecordOperator {
    /// Parses transport value into an operator.
    pub fn parse_transport(value: &str) -> AppResult<Self> {
        match value {
            "eq" => Ok(Self::Eq),
            "neq" => Ok(Self::Neq),
            "gt" => Ok(Self::Gt),
            "gte" => Ok(Self::Gte),
            "lt" => Ok(Self::Lt),
            "lte" => Ok(Self::Lte),
            "contains" => Ok(Self::Contains),
            "in" => Ok(Self::In),
            _ => Err(qryvanta_core::AppError::Validation(format!(
                "unknown runtime query operator '{value}'"
            ))),
        }
    }

    /// Returns the stable transport value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Neq => "neq",
            Self::Gt => "gt",
            Self::Gte => "gte",
            Self::Lt => "lt",
            Self::Lte => "lte",
            Self::Contains => "contains",
            Self::In => "in",
        }
    }
}

/// Runtime query sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRecordSortDirection {
    /// Ascending sort direction.
    Asc,
    /// Descending sort direction.
    Desc,
}

/// Runtime query join type for link-entity semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRecordJoinType {
    /// Only matching linked records are included.
    Inner,
    /// Parent records are preserved when link target is missing.
    Left,
}

impl RuntimeRecordJoinType {
    /// Parses transport value into join type.
    pub fn parse_transport(value: &str) -> AppResult<Self> {
        match value {
            "inner" => Ok(Self::Inner),
            "left" => Ok(Self::Left),
            _ => Err(qryvanta_core::AppError::Validation(format!(
                "unknown runtime query join type '{value}'"
            ))),
        }
    }

    /// Returns the stable transport value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inner => "inner",
            Self::Left => "left",
        }
    }
}

impl RuntimeRecordSortDirection {
    /// Parses transport value into sort direction.
    pub fn parse_transport(value: &str) -> AppResult<Self> {
        match value {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err(qryvanta_core::AppError::Validation(format!(
                "unknown runtime query sort direction '{value}'"
            ))),
        }
    }

    /// Returns the stable transport value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// Uniqueness index entry persisted alongside runtime records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniqueFieldValue {
    /// Field logical name.
    pub field_logical_name: String,
    /// Stable hash for the field value.
    pub field_value_hash: String,
}

/// Query inputs for runtime record listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordListQuery {
    /// Maximum rows returned.
    pub limit: usize,
    /// Number of rows skipped for offset pagination.
    pub offset: usize,
    /// Optional subject ownership filter.
    pub owner_subject: Option<String>,
}

/// Typed condition for runtime record queries.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeRecordFilter {
    /// Optional linked-entity alias scope.
    pub scope_alias: Option<String>,
    /// Field logical name to compare.
    pub field_logical_name: String,
    /// Comparison operator.
    pub operator: RuntimeRecordOperator,
    /// Field type from the published schema.
    pub field_type: FieldType,
    /// Expected field value.
    pub field_value: Value,
}

/// Sort instruction for runtime record queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRecordSort {
    /// Optional linked-entity alias scope.
    pub scope_alias: Option<String>,
    /// Field logical name to sort by.
    pub field_logical_name: String,
    /// Field type from the published schema.
    pub field_type: FieldType,
    /// Sort direction.
    pub direction: RuntimeRecordSortDirection,
}

/// Link-entity query scope rooted in a relation field on a parent scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRecordLink {
    /// Stable alias used by filter/sort scope resolution.
    pub alias: String,
    /// Optional parent alias; `None` means the root entity.
    pub parent_alias: Option<String>,
    /// Relation field on the parent scope that points to target records.
    pub relation_field_logical_name: String,
    /// Target entity resolved from the relation field metadata.
    pub target_entity_logical_name: String,
    /// Join behavior for missing relation targets.
    pub join_type: RuntimeRecordJoinType,
}

/// Recursive runtime query condition tree.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeRecordConditionNode {
    /// One typed condition.
    Filter(RuntimeRecordFilter),
    /// Nested logical group.
    Group(RuntimeRecordConditionGroup),
}

/// Recursive logical group for runtime query conditions.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeRecordConditionGroup {
    /// Logical mode for evaluating child nodes.
    pub logical_mode: RuntimeRecordLogicalMode,
    /// Child condition nodes.
    pub nodes: Vec<RuntimeRecordConditionNode>,
}

/// Query inputs for runtime record listing with exact-match filters.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeRecordQuery {
    /// Maximum rows returned.
    pub limit: usize,
    /// Number of rows skipped for offset pagination.
    pub offset: usize,
    /// Logical composition mode for conditions.
    pub logical_mode: RuntimeRecordLogicalMode,
    /// Optional recursive where-clause tree.
    pub where_clause: Option<RuntimeRecordConditionGroup>,
    /// Typed query conditions.
    pub filters: Vec<RuntimeRecordFilter>,
    /// Link-entity definitions used by alias-scoped filters/sorts.
    pub links: Vec<RuntimeRecordLink>,
    /// Sort instructions.
    pub sort: Vec<RuntimeRecordSort>,
    /// Optional subject ownership filter.
    pub owner_subject: Option<String>,
}
