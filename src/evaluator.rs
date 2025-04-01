use std::collections::HashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InteractionNet {
    // ACSets-inspired structure
    agents: HashMap<AgentId, Agent>,
    ports: HashMap<PortId, Port>,
    connections: HashMap<ConnectionId, Connection>,

    // Type system components
    type_context: TypeContext,
}

type AgentId = usize;
type PortId = usize;
type ConnectionId = usize;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Agent {
    id: AgentId,
    name: String,
    principal_port: PortId,
    auxiliary_ports: Vec<PortId>,
    type_annotation: Option<Type>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Port {
    id: PortId,
    name: String,
    agent: AgentId,
    is_principal: bool,
    type_annotation: Option<Type>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Connection {
    id: ConnectionId,
    port1: PortId,
    port2: PortId,
}

// Gradual type system
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Type {
    Dyn,
    Simple(String),
    Union(Box<Type>, Box<Type>),
    Intersection(Box<Type>, Box<Type>),
    Parametric(String, Vec<Type>),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TypeContext {
    port_types: HashMap<PortId, Type>,
    inferred_types: HashMap<PortId, Type>,
}

impl InteractionNet {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            ports: HashMap::new(),
            connections: HashMap::new(),
            type_context: TypeContext {
                port_types: HashMap::new(),
                inferred_types: HashMap::new(),
            },
        }
    }

    // Methods for adding agents, connecting ports, etc.

    pub fn find_redexes(&self) -> Vec<(AgentId, AgentId)> {
        // Find pairs of agents connected by their principal ports
        let mut redexes = Vec::new();

        for conn in self.connections.values() {
            let port1 = &self.ports[&conn.port1];
            let port2 = &self.ports[&conn.port2];

            if port1.is_principal && port2.is_principal {
                redexes.push((port1.agent, port2.agent));
            }
        }

        redexes
    }

    pub fn apply_reduction(
        &mut self,
        _redex: (AgentId, AgentId),
        _rules: &ReductionRules,
    ) -> Result<(), String> {
        // Apply a reduction rule to a redex
        // 1. Find matching rule
        // 2. Remove the agents in the redex
        // 3. Add new agents and connections according to the rule
        // 4. Propagate types through the new connections

        // Implementation would go here

        Ok(())
    }

    pub fn infer_types(&mut self) {
        // Propagate type information through connections
        // Starting from annotated ports, push types through the net

        let mut changed = true;
        while changed {
            changed = false;

            // For each connection
            for conn in self.connections.values() {
                // Get the types of the connected ports
                let type1 = self.type_context.inferred_types.get(&conn.port1).cloned();
                let type2 = self.type_context.inferred_types.get(&conn.port2).cloned();

                // Try to unify the types
                match (type1, type2) {
                    (Some(t1), Some(t2)) => {
                        if let Some(unified) = unify_types(&t1, &t2) {
                            // Update both ports with the unified type
                            if self.type_context.inferred_types.get(&conn.port1) != Some(&unified) {
                                self.type_context
                                    .inferred_types
                                    .insert(conn.port1, unified.clone());
                                changed = true;
                            }
                            if self.type_context.inferred_types.get(&conn.port2) != Some(&unified) {
                                self.type_context.inferred_types.insert(conn.port2, unified);
                                changed = true;
                            }
                        }
                    }
                    (Some(t), None) => {
                        self.type_context.inferred_types.insert(conn.port2, t);
                        changed = true;
                    }
                    (None, Some(t)) => {
                        self.type_context.inferred_types.insert(conn.port1, t);
                        changed = true;
                    }
                    _ => {}
                }
            }
        }
    }
}

fn unify_types(t1: &Type, t2: &Type) -> Option<Type> {
    match (t1, t2) {
        // Dyn is compatible with everything
        (Type::Dyn, _) => Some(t2.clone()),
        (_, Type::Dyn) => Some(t1.clone()),

        // Simple types must match exactly
        (Type::Simple(n1), Type::Simple(n2)) if n1 == n2 => Some(t1.clone()),

        // For union and intersection types, we need set-theoretic operations
        // This would be more complex in a real implementation

        // For parametric types, recursively unify parameters
        (Type::Parametric(n1, params1), Type::Parametric(n2, params2))
            if n1 == n2 && params1.len() == params2.len() =>
        {
            let mut unified_params = Vec::new();
            for (p1, p2) in params1.iter().zip(params2.iter()) {
                if let Some(unified) = unify_types(p1, p2) {
                    unified_params.push(unified);
                } else {
                    return None;
                }
            }
            Some(Type::Parametric(n1.clone(), unified_params))
        }

        // Default: types are incompatible
        _ => None,
    }
}

#[allow(dead_code)]
pub struct ReductionRules {
    rules: Vec<ReductionRule>,
}

impl ReductionRules {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ReductionRule {
    name: String,
    left_agent: String,
    right_agent: String,
    replacement: Vec<ReplacementAction>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ReplacementAction {
    CreateAgent {
        name: String,
        ports: Vec<String>,
    },
    Connect {
        port1: (String, String),
        port2: (String, String),
    },
}

impl Default for InteractionNet {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ReductionRules {
    fn default() -> Self {
        Self::new()
    }
}
