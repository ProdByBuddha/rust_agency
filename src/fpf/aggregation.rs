/// Part B – Trans-disciplinary Reasoning Cluster
/// 
/// B.1 - Universal Algebra of Aggregation (Γ)
/// B.1.1 - Dependency Graph & Proofs
/// B.1.2 - System-specific Aggregation Γ_sys
/// B.1.3 - Γ_epist - Knowledge-Specific Aggregation
/// B.1.4 - Contextual & Temporal Aggregation (Γ_ctx & Γ_time)
/// B.1.5 - Γ_method — Order-Sensitive Method Composition & Instantiation
/// B.1.6 - Γ_work — Work as Spent Resource

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use super::holon::{System, Episteme, Boundary, BoundaryKind, CharacteristicValue};
use super::mereology::MereologicalRelation;
use super::assurance::{AssuranceTuple, CongruenceLevel};
use super::transformer::MethodDescription;

/// B.1.4.1 NC-invariants OrderSpec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSpec {
    pub sequence: Vec<String>, // Sequence of Holon IDs
    pub partial_order: Option<HashMap<String, HashSet<String>>>,
    pub context_id: String,
}

/// B.1.1 DependencyGraph D = (V, E, scope, notes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<String>, // Holon IDs
    pub edges: Vec<(String, String, MereologicalRelation)>, // (from, to, relation)
    pub design_run_tag: DesignRunTag,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesignRunTag {
    Design,
    Run,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeClass {
    Extensive, // Sum
    Intensive, // Min (WLNK)
    BooleanOr, // Or
    BooleanAnd, // And
}

/// B.1:4.2 The Five Grounding Invariants (Invariant Quintet)
pub trait InvariantQuintet {
    fn verify_idempotence(&self) -> bool;
    fn verify_commutativity(&self) -> bool;
    fn verify_locality(&self) -> bool;
    fn verify_weakest_link(&self) -> bool;
    fn verify_monotonicity(&self) -> bool;
}

/// B.1.2 System-specific Aggregation Γ_sys
pub struct SysCAL;

impl SysCAL {
    pub fn aggregate(&self, id: &str, graph: &DependencyGraph, parts: &HashMap<String, System>, attr_classes: &HashMap<String, AttributeClass>) -> System {
        let mut agg_characteristics = HashMap::new();
        
        for node_id in &graph.nodes {
            if let Some(system) = parts.get(node_id) {
                for (key, val) in &system.characteristics {
                    let class = attr_classes.get(key).cloned().unwrap_or(AttributeClass::Extensive);
                    
                    if !agg_characteristics.contains_key(key) {
                        agg_characteristics.insert(key.clone(), val.clone());
                        continue;
                    }

                    let entry = agg_characteristics.get_mut(key).unwrap();
                    
                    match (entry, val, class) {
                        (CharacteristicValue::Numeric(e), CharacteristicValue::Numeric(v), AttributeClass::Extensive) => {
                            *e += *v;
                        }
                        (CharacteristicValue::Numeric(e), CharacteristicValue::Numeric(v), AttributeClass::Intensive) => {
                            if *v < *e { *e = *v; }
                        }
                        (CharacteristicValue::Boolean(e), CharacteristicValue::Boolean(v), AttributeClass::BooleanOr) => {
                            *e = *e || *v;
                        }
                        (CharacteristicValue::Boolean(e), CharacteristicValue::Boolean(v), AttributeClass::BooleanAnd) => {
                            *e = *e && *v;
                        }
                        _ => {} 
                    }
                }
            }
        }

        System {
            id: id.to_string(),
            boundary: Boundary {
                kind: BoundaryKind::Open,
                description: format!("Aggregated system from {} parts", graph.nodes.len()),
            },
            characteristics: agg_characteristics,
        }
    }
}

/// B.1.3 Γ_epist - Knowledge-Specific Aggregation
pub struct EpistemeCAL;

impl EpistemeCAL {
    pub fn aggregate(
        &self, 
        id: &str, 
        graph: &DependencyGraph, 
        parts: &HashMap<String, Episteme>, 
        assurance_map: &HashMap<String, AssuranceTuple>,
        congruence_level: CongruenceLevel
    ) -> (Episteme, AssuranceTuple) {
        
        let mut first_tuple: Option<AssuranceTuple> = None;
        let mut combined_content = String::new();
        
        for node_id in &graph.nodes {
            if let (Some(episteme), Some(tuple)) = (parts.get(node_id), assurance_map.get(node_id)) {
                combined_content.push_str(&episteme.content);
                combined_content.push_str("\n---\n");
                
                if let Some(ref mut current_agg) = first_tuple {
                    *current_agg = AssuranceTuple::aggregate(current_agg, tuple, congruence_level);
                } else {
                    first_tuple = Some(tuple.clone());
                }
            }
        }

        let result_tuple = first_tuple.unwrap_or_else(|| panic!("Empty aggregation"));
        
        let result_episteme = Episteme {
            id: id.to_string(),
            boundary: Boundary {
                kind: BoundaryKind::Permeable,
                description: "Aggregated episteme".to_string(),
            },
            content: combined_content,
            version: "1.0".to_string(),
            characteristics: HashMap::new(),
        };

        (result_episteme, result_tuple)
    }
}

/// B.1.4 Γ_ctx - Contextual Aggregation (Sequential/Non-commutative)
pub struct ContextCAL;

impl ContextCAL {
    pub fn aggregate_sequential(&self, id: &str, order: &OrderSpec, _parts: &HashMap<String, System>) -> System {
        // Implementation of procedural aggregation
        System {
            id: id.to_string(),
            boundary: Boundary {
                kind: BoundaryKind::Open,
                description: format!("Sequentially aggregated system following context {}", order.context_id),
            },
            characteristics: HashMap::new(),
        }
    }
}

/// B.1.5 Γ_method — Order-Sensitive Method Composition
pub struct MethodCAL;

impl MethodCAL {
    pub fn compose(&self, id: &str, sequence: &[MethodDescription]) -> MethodDescription {
        let mut combined_content = String::new();
        let mut required_roles = HashSet::new();
        
        for m in sequence {
            combined_content.push_str(&m.content);
            combined_content.push_str(" » ");
            for role in &m.required_roles {
                required_roles.insert(role.clone());
            }
        }

        MethodDescription {
            id: id.to_string(),
            content: combined_content,
            version: "1.0".to_string(),
            required_roles: required_roles.into_iter().collect(),
        }
    }
}

/// B.1.6 Γ_work — Work as Spent Resource (Additivity)
pub struct WorkCAL;

impl WorkCAL {
    pub fn sum_resources(&self, batch: &[HashMap<String, f64>]) -> HashMap<String, f64> {
        let mut total = HashMap::new();
        for resources in batch {
            for (k, v) in resources {
                *total.entry(k.clone()).or_insert(0.0) += *v;
            }
        }
        total
    }
}
