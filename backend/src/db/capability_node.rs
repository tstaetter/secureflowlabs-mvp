use crate::AppResult;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Sample JSON
/// {
///   "capability": "create_customer",
///   "provider": "stripe",
///   "intent_labels": [
///     "create client",
///     "new customer",
///     "register buyer"
///   ],
///   "input_requirements": [
///     "email"
///   ],
///   "optional_inputs": [
///     "name",
///     "phone"
///   ],
///   "side_effect": true,
///   "risk_level": "medium",
///   "execution": {
///     "endpoint_id": "stripe.customers.create"
///   }
/// }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityNode {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub normalized_entry: ObjectId,
    pub capability: String,
    pub intent_labels: Vec<String>,
    pub required_inputs: Vec<String>,
    pub optional_inputs: Vec<String>,
    pub side_effect: bool,
    pub risk_level: RiskLevel,
}

impl CapabilityNode {
    async fn execution_endpoint() -> AppResult<String> {
        todo!()
    }
}
