use crate::runtime::ExecutionPlan;

pub struct WorkflowStep {
    pub id: String,
    pub plan: ExecutionPlan,
    pub depends_on: Vec<String>,
}
