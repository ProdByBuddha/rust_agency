/// A.1 Holonic Foundation
/// 
/// The root ontology for FPF: Entity -> Holon -> {System, Episteme}
/// Separation of Identity (Entity) from Structure (Holon) and Function (Role - A.2).

use serde::{Serialize, Deserialize};

/// U.Entity: Primitive of Distinction
/// Anything that can be individuated and referenced.
pub trait Entity {
    fn id(&self) -> &str;
}

/// U.Boundary: Interface primitive
/// Defines the separation between the Holon and its environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boundary {
    pub kind: BoundaryKind,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundaryKind {
    Open,      // Exchanges matter, energy, information
    Closed,    // Exchanges energy, information (no matter)
    Permeable, // User-filtered subset
}

/// U.Holon: Unit of Composition
/// A U.Entity that is simultaneously a whole composed of parts and a part within a larger whole.
pub trait Holon: Entity {
    fn boundary(&self) -> &Boundary;
    // Parts can be accessed via composition methods (B.1), not necessarily stored on the struct
}

/// U.System: Physical/Operational Holon
/// Can bear behavioural roles (Transformer, Agent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CharacteristicValue {
    Numeric(f64),
    Boolean(bool),
    Text(String),
}

/// U.System: Physical/Operational Holon
/// Can bear behavioural roles (Transformer, Agent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    pub id: String,
    pub boundary: Boundary,
    pub characteristics: std::collections::HashMap<String, CharacteristicValue>,
}

impl Entity for System {
    fn id(&self) -> &str { &self.id }
}

impl Holon for System {
    fn boundary(&self) -> &Boundary { &self.boundary }
}

/// U.Episteme: Knowledge Holon
/// Passive content (axioms, evidence). Can bear status roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episteme {
    pub id: String,
    pub boundary: Boundary,
    pub content: String, // Or structured content
    pub version: String,
    pub characteristics: std::collections::HashMap<String, CharacteristicValue>,
}

impl Entity for Episteme {
    fn id(&self) -> &str { &self.id }
}

impl Holon for Episteme {
    fn boundary(&self) -> &Boundary { &self.boundary }
}

/// U.Interaction: Flow crossing a boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub source_id: String,
    pub target_id: String,
    pub content: String, // Matter, Energy, or Information
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
