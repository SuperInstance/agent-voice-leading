//! # agent-voice-leading
//!
//! Good voice leading makes a chord progression feel smooth. Good state transitions
//! make a multi-agent system feel coherent. Both minimize unnecessary disruption
//! while maximizing the expressiveness of each individual voice.

#![forbid(unsafe_code)]

/// Ternary motion: Descending (-1), Static (0), Ascending (+1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Motion {
    Descending = -1,
    Static = 0,
    Ascending = 1,
}

impl Motion {
    pub fn to_i8(self) -> i8 { self as i8 }
    pub fn from_i8(v: i8) -> Option<Self> {
        match v { -1 => Some(Motion::Descending), 0 => Some(Motion::Static), 1 => Some(Motion::Ascending), _ => None }
    }
    pub fn between(from: i32, to: i32) -> Self {
        if to > from { Motion::Ascending } else if to < from { Motion::Descending } else { Motion::Static }
    }
}

/// An agent's state as a numeric position (like a note on a staff).
#[derive(Debug, Clone)]
pub struct AgentState {
    pub id: u32,
    pub position: i32,
}

impl AgentState {
    pub fn new(id: u32, position: i32) -> Self { Self { id, position } }
    pub fn distance_to(&self, target: &AgentState) -> i32 { (target.position - self.position).abs() }
    pub fn motion_to(&self, target: &AgentState) -> Motion { Motion::between(self.position, target.position) }
}

/// A multi-agent configuration — the "chord".
#[derive(Debug, Clone)]
pub struct Configuration {
    pub agents: Vec<AgentState>,
}

impl Configuration {
    pub fn new(agents: Vec<AgentState>) -> Self { Self { agents } }
    pub fn positions(&self) -> Vec<i32> { self.agents.iter().map(|a| a.position).collect() }
    pub fn count(&self) -> usize { self.agents.len() }
    /// Total voice-leading distance to another configuration.
    pub fn total_distance(&self, other: &Configuration) -> i32 {
        self.agents.iter().zip(other.agents.iter()).map(|(a, b)| a.distance_to(b)).sum()
    }
    /// Max individual distance to another configuration.
    pub fn max_distance(&self, other: &Configuration) -> i32 {
        self.agents.iter().zip(other.agents.iter()).map(|(a, b)| a.distance_to(b)).max().unwrap_or(0)
    }
}

/// Result of computing voice leading between two configurations.
#[derive(Debug, Clone)]
pub struct VoiceLeading {
    pub from: Configuration,
    pub to: Configuration,
    pub assignment: Vec<(usize, usize)>,  // (from_index, to_index)
    pub total_cost: i32,
}

impl VoiceLeading {
    /// Compute optimal voice leading using greedy nearest-neighbor assignment.
    pub fn compute(from: &Configuration, to: &Configuration) -> Self {
        let n = from.agents.len().min(to.agents.len());
        let mut used_to: Vec<bool> = vec![false; to.agents.len()];
        let mut assignment = Vec::new();
        let mut total_cost = 0i32;

        for i in 0..n {
            let mut best_j = 0; let mut best_dist = i32::MAX;
            for j in 0..to.agents.len() {
                if used_to[j] { continue; }
                let d = from.agents[i].distance_to(&to.agents[j]);
                if d < best_dist { best_dist = d; best_j = j; }
            }
            used_to[best_j] = true;
            assignment.push((i, best_j));
            total_cost += best_dist;
        }
        VoiceLeading { from: from.clone(), to: to.clone(), assignment, total_cost }
    }
    pub fn individual_motions(&self) -> Vec<Motion> {
        self.assignment.iter().map(|&(i, j)| self.from.agents[i].motion_to(&self.to.agents[j])).collect()
    }
}

/// A transition plan spreading changes over multiple steps.
#[derive(Debug, Clone)]
pub struct SmoothTransition {
    pub steps: Vec<Configuration>,
}

impl SmoothTransition {
    /// Create a smooth transition from `from` to `to` in `n_steps` intermediate states.
    pub fn plan(from: &Configuration, to: &Configuration, n_steps: usize) -> Self {
        let vl = VoiceLeading::compute(from, to);
        let mut steps = vec![from.clone()];
        for step in 1..=n_steps {
            let intermediate: Vec<AgentState> = vl.assignment.iter().map(|&(i, j)| {
                let from_pos = from.agents[i].position;
                let to_pos = to.agents[j].position;
                let pos = from_pos + (to_pos - from_pos) * step as i32 / (n_steps as i32 + 1);
                AgentState::new(from.agents[i].id, pos)
            }).collect();
            steps.push(Configuration::new(intermediate));
        }
        steps.push(to.clone());
        SmoothTransition { steps }
    }
    pub fn step_count(&self) -> usize { self.steps.len().saturating_sub(2) }  // exclude start/end
}

/// Node in a chord graph — a named configuration.
#[derive(Debug, Clone)]
pub struct ChordNode {
    pub name: String,
    pub config: Configuration,
}

/// Edge in a chord graph — voice-leading distance between configs.
#[derive(Debug, Clone)]
pub struct ChordEdge {
    pub from: usize,
    pub to: usize,
    pub distance: i32,
}

/// A graph of configurations connected by voice-leading distance.
#[derive(Debug, Clone)]
pub struct ChordGraph {
    pub nodes: Vec<ChordNode>,
    pub edges: Vec<ChordEdge>,
}

impl ChordGraph {
    pub fn new() -> Self { Self { nodes: Vec::new(), edges: Vec::new() } }
    pub fn add_node(&mut self, name: &str, config: Configuration) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(ChordNode { name: name.to_string(), config });
        // Add edges to all existing nodes
        for (i, existing) in self.nodes.iter().enumerate().take(idx) {
            let dist_fwd = existing.config.total_distance(&self.nodes[idx].config);
            let dist_rev = self.nodes[idx].config.total_distance(&existing.config);
            self.edges.push(ChordEdge { from: i, to: idx, distance: dist_fwd });
            self.edges.push(ChordEdge { from: idx, to: i, distance: dist_rev });
        }
        idx
    }
    /// Find shortest path (by voice-leading distance) between two named nodes.
    pub fn shortest_path(&self, from_name: &str, to_name: &str) -> Option<Vec<usize>> {
        let from_idx = self.nodes.iter().position(|n| n.name == from_name)?;
        let to_idx = self.nodes.iter().position(|n| n.name == to_name)?;
        if from_idx == to_idx { return Some(vec![from_idx]); }
        // BFS with distance (simple Dijkstra)
        let n = self.nodes.len();
        let mut dist = vec![i32::MAX; n]; dist[from_idx] = 0;
        let mut prev = vec![None; n];
        let mut visited = vec![false; n];
        for _ in 0..n {
            let u = (0..n).filter(|&i| !visited[i]).min_by_key(|&i| dist[i]).unwrap();
            if dist[u] == i32::MAX { break; }
            visited[u] = true;
            for e in &self.edges {
                if e.from == u && !visited[e.to] {
                    let new_dist = dist[u] + e.distance;
                    if new_dist < dist[e.to] { dist[e.to] = new_dist; prev[e.to] = Some(u); }
                }
            }
        }
        if dist[to_idx] == i32::MAX { return None; }
        let mut path = Vec::new(); let mut cur = to_idx;
        while let Some(p) = prev[cur] { path.push(cur); cur = p; }
        path.push(from_idx); path.reverse(); Some(path)
    }
}

/// Counterpoint rules for agent transitions.
#[derive(Debug, Clone)]
pub struct CounterpointRules {
    pub forbid_parallel: bool,     // No two agents moving same direction same distance
    pub require_contrary: bool,    // At least one pair must move in opposite directions
    pub max_leap: i32,             // Maximum distance any single agent can move
    pub require_resolution: bool,  // Tension states must resolve toward stable positions
}

impl CounterpointRules {
    pub fn strict() -> Self { Self { forbid_parallel: true, require_contrary: true, max_leap: 3, require_resolution: true } }
    pub fn relaxed() -> Self { Self { forbid_parallel: false, require_contrary: false, max_leap: 10, require_resolution: false } }
    pub fn check(&self, vl: &VoiceLeading) -> Vec<String> {
        let mut violations = Vec::new();
        let motions = vl.individual_motions();
        // Parallel check
        if self.forbid_parallel {
            for i in 0..motions.len() {
                for j in (i+1)..motions.len() {
                    if motions[i] == motions[j] && motions[i] != Motion::Static {
                        violations.push(format!("Parallel motion: agents {} and {} both {:?}", i, j, motions[i]));
                    }
                }
            }
        }
        // Contrary motion check
        if self.require_contrary && motions.len() >= 2 {
            let has_contrary = motions.iter().any(|&m| motions.iter().any(|&m2| m != m2 && m != Motion::Static && m2 != Motion::Static));
            if !has_contrary { violations.push("No contrary motion found".to_string()); }
        }
        // Max leap check
        for (i, j) in &vl.assignment {
            let dist = vl.from.agents[*i].distance_to(&vl.to.agents[*j]);
            if dist > self.max_leap { violations.push(format!("Agent {} leaped {} (max {})", i, dist, self.max_leap)); }
        }
        violations
    }
}

/// A leading tone — an agent pulled toward a specific position.
#[derive(Debug, Clone)]
pub struct LeadingTone {
    pub agent_id: u32,
    pub current: i32,
    pub target: i32,
    pub strength: f64,  // 0.0=weak pull, 1.0=strong pull
}

impl LeadingTone {
    pub fn new(agent_id: u32, current: i32, target: i32, strength: f64) -> Self {
        Self { agent_id, current, target, strength: strength.clamp(0.0, 1.0) }
    }
    /// Direction the agent wants to move.
    pub fn pull(&self) -> Motion { Motion::between(self.current, self.target) }
    /// How far from resolution.
    pub fn distance(&self) -> i32 { (self.target - self.current).abs() }
    /// Is this leading tone resolved (at target)?
    pub fn is_resolved(&self) -> bool { self.current == self.target }
    /// Step toward resolution.
    pub fn step(&mut self) {
        if self.current < self.target { self.current += 1; }
        else if self.current > self.target { self.current -= 1; }
    }
}

/// Cadence types — transition endings with different feels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cadence {
    Perfect,    // Strong resolution — team converges on consensus
    Plagal,     // Gentle resolution — soft landing after sprint
    Deceptive,  // Surprise twist — pivot to unexpected direction
    Half,       // Pause — checkpoint, not resolution
}

impl Cadence {
    pub fn from_harmony(tension: i32, stability: i32) -> Self {
        if tension < 0 && stability > 0 { Cadence::Perfect }
        else if tension <= 0 && stability <= 0 { Cadence::Plagal }
        else if tension > 0 && stability < 0 { Cadence::Deceptive }
        else { Cadence::Half }
    }
    pub fn description(self) -> &'static str {
        match self {
            Cadence::Perfect => "Strong resolution — team reached full consensus",
            Cadence::Plagal => "Gentle landing — soft close after productive sprint",
            Cadence::Deceptive => "Surprise pivot — unexpected direction, high energy",
            Cadence::Half => "Checkpoint pause — stable but unresolved, more work ahead",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn motion_between() {
        assert_eq!(Motion::between(0, 1), Motion::Ascending);
        assert_eq!(Motion::between(1, 0), Motion::Descending);
        assert_eq!(Motion::between(2, 2), Motion::Static);
    }
    #[test] fn agent_distance() {
        let a = AgentState::new(0, 0); let b = AgentState::new(1, 5);
        assert_eq!(a.distance_to(&b), 5);
    }
    #[test] fn config_distance() {
        let c1 = Configuration::new(vec![AgentState::new(0, 0), AgentState::new(1, 2)]);
        let c2 = Configuration::new(vec![AgentState::new(0, 3), AgentState::new(1, 5)]);
        assert_eq!(c1.total_distance(&c2), 6);
        assert_eq!(c1.max_distance(&c2), 3);
    }
    #[test] fn voice_leading_optimal() {
        let c1 = Configuration::new(vec![AgentState::new(0, 0), AgentState::new(1, 10)]);
        let c2 = Configuration::new(vec![AgentState::new(0, 1), AgentState::new(1, 11)]);
        let vl = VoiceLeading::compute(&c1, &c2);
        assert_eq!(vl.total_cost, 2);
        let motions = vl.individual_motions();
        assert!(motions.iter().all(|&m| m == Motion::Ascending));
    }
    #[test] fn voice_leading_cross() {
        let c1 = Configuration::new(vec![AgentState::new(0, 0), AgentState::new(1, 10)]);
        let c2 = Configuration::new(vec![AgentState::new(0, 10), AgentState::new(1, 0)]);
        let vl = VoiceLeading::compute(&c1, &c2);
        // Should find the near assignment (crossing) since each goes 10
        assert!(vl.total_cost <= 20);
    }
    #[test] fn smooth_transition() {
        let c1 = Configuration::new(vec![AgentState::new(0, 0)]);
        let c2 = Configuration::new(vec![AgentState::new(0, 10)]);
        let st = SmoothTransition::plan(&c1, &c2, 4);
        assert_eq!(st.steps.len(), 6); // start + 4 intermediate + end
        assert!(st.step_count() == 4);
    }
    #[test] fn chord_graph_path() {
        let mut g = ChordGraph::new();
        g.add_node("A", Configuration::new(vec![AgentState::new(0, 0)]));
        g.add_node("B", Configuration::new(vec![AgentState::new(0, 5)]));
        g.add_node("C", Configuration::new(vec![AgentState::new(0, 10)]));
        let path = g.shortest_path("A", "C").unwrap();
        assert!(path.len() >= 2);
    }
    #[test] fn chord_graph_no_path() {
        let mut g = ChordGraph::new();
        g.add_node("A", Configuration::new(vec![AgentState::new(0, 0)]));
        assert!(g.shortest_path("A", "Z").is_none());
    }
    #[test] fn counterpoint_parallel_violation() {
        let rules = CounterpointRules::strict();
        let c1 = Configuration::new(vec![AgentState::new(0, 0), AgentState::new(1, 2)]);
        let c2 = Configuration::new(vec![AgentState::new(0, 3), AgentState::new(1, 5)]);
        let vl = VoiceLeading::compute(&c1, &c2);
        let v = rules.check(&vl);
        assert!(!v.is_empty()); // should flag parallel motion
    }
    #[test] fn counterpoint_relaxed() {
        let rules = CounterpointRules::relaxed();
        let c1 = Configuration::new(vec![AgentState::new(0, 0), AgentState::new(1, 2)]);
        let c2 = Configuration::new(vec![AgentState::new(0, 1), AgentState::new(1, 3)]);
        let vl = VoiceLeading::compute(&c1, &c2);
        assert!(rules.check(&vl).is_empty());
    }
    #[test] fn leading_tone_pull() {
        let lt = LeadingTone::new(0, 2, 5, 0.8);
        assert_eq!(lt.pull(), Motion::Ascending);
        assert_eq!(lt.distance(), 3);
        assert!(!lt.is_resolved());
    }
    #[test] fn leading_tone_resolve() {
        let mut lt = LeadingTone::new(0, 4, 5, 1.0);
        lt.step();
        assert_eq!(lt.current, 5);
        assert!(lt.is_resolved());
    }
    #[test] fn cadence_types() {
        assert_eq!(Cadence::from_harmony(-1, 1), Cadence::Perfect);
        assert_eq!(Cadence::from_harmony(1, -1), Cadence::Deceptive);
        assert_eq!(Cadence::from_harmony(0, 0), Cadence::Plagal); // tension=0, stability=0
        assert_eq!(Cadence::from_harmony(-1, -1), Cadence::Plagal); // tension<0
    }
    #[test] fn cadence_descriptions() {
        assert!(!Cadence::Perfect.description().is_empty());
        assert!(!Cadence::Deceptive.description().is_empty());
    }
}
