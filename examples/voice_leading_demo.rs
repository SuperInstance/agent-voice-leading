//! Two agents "migrate" between roles using smooth voice leading.
//! Shows the optimal assignment path and motion analysis.

use agent_voice_leading::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║      VOICE LEADING DEMO — Smooth Role Transitions          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Initial configuration: 4 agents in a tight cluster
    let from = Configuration::new(vec![
        AgentState::new(0, 2),   // Lead
        AgentState::new(1, 5),   // Support
        AgentState::new(2, 7),   // Observer
        AgentState::new(3, 10),  // Scout
    ]);

    // Target configuration: agents spread out into new roles
    let to = Configuration::new(vec![
        AgentState::new(0, 8),   // becomes Scout
        AgentState::new(1, 3),   // becomes Lead
        AgentState::new(2, 12),  // becomes Explorer
        AgentState::new(3, 6),   // becomes Support
    ]);

    println!("Source configuration:");
    for a in &from.agents {
        let bar: String = "·".repeat(a.position as usize) + "●";
        println!("  Agent {} at {:>3}  |{}", a.id, a.position, bar);
    }
    println!();

    println!("Target configuration:");
    for a in &to.agents {
        let bar: String = "·".repeat(a.position as usize) + "●";
        println!("  Agent {} at {:>3}  |{}", a.id, a.position, bar);
    }
    println!();

    // Compute voice leading
    let vl = VoiceLeading::compute(&from, &to);
    println!("━━━ Optimal Voice Leading ━━━");
    println!("  Total cost: {} position units", vl.total_cost);
    println!();

    let motions = vl.individual_motions();
    for (idx, &(i, j)) in vl.assignment.iter().enumerate() {
        let from_pos = from.agents[i].position;
        let to_pos = to.agents[j].position;
        let dist = (to_pos - from_pos).abs();
        let motion = match motions[idx] {
            Motion::Ascending  => "↑ ascending",
            Motion::Descending => "↓ descending",
            Motion::Static     => "→ static    ",
        };
        println!("  Agent {} → Agent {} : {} → {} (distance: {}, {})",
            from.agents[i].id, to.agents[j].id, from_pos, to_pos, dist, motion);
    }
    println!();

    // Smooth transition
    println!("━━━ Smooth Transition (4 steps) ━━━");
    let transition = SmoothTransition::plan(&from, &to, 4);
    for (step, config) in transition.steps.iter().enumerate() {
        let label = match step {
            0 => "START",
            n if n == transition.steps.len() - 1 => "END  ",
            _ => "     ",
        };
        let positions: Vec<String> = config.agents.iter()
            .map(|a| format!("{}:{:>2}", a.id, a.position))
            .collect();
        let pos_bar: Vec<String> = config.agents.iter().map(|a| {
            let bar: String = "·".repeat(a.position as usize) + "●";
            bar
        }).collect();
        println!("  {} Step {:>2}: [{}]  |{}", label, step, positions.join(", "), pos_bar.join("|"));
    }
    println!();

    // Counterpoint analysis
    println!("━━━ Counterpoint Analysis ━━━");
    let strict = CounterpointRules::strict();
    let violations = strict.check(&vl);
    println!("  Strict rules: max_leap={}", strict.max_leap);
    if violations.is_empty() {
        println!("  ✅ No counterpoint violations");
    } else {
        for v in &violations {
            println!("  ⚠️  {}", v);
        }
    }

    let relaxed = CounterpointRules::relaxed();
    let relaxed_v = relaxed.check(&vl);
    println!("  Relaxed rules: {} violations", relaxed_v.len());
    println!();

    // Chord graph — find shortest path through multiple configurations
    println!("━━━ Chord Graph Path Finding ━━━");
    let mut graph = ChordGraph::new();
    graph.add_node("Home", Configuration::new(vec![
        AgentState::new(0, 0), AgentState::new(1, 4),
    ]));
    graph.add_node("Bridge", Configuration::new(vec![
        AgentState::new(0, 2), AgentState::new(1, 6),
    ]));
    graph.add_node("Climax", Configuration::new(vec![
        AgentState::new(0, 7), AgentState::new(1, 10),
    ]));
    graph.add_node("Resolve", Configuration::new(vec![
        AgentState::new(0, 1), AgentState::new(1, 3),
    ]));

    println!("  Nodes: {}", graph.nodes.iter().map(|n| n.name.as_str()).collect::<Vec<_>>().join(", "));
    if let Some(path) = graph.shortest_path("Home", "Resolve") {
        let names: Vec<&str> = path.iter().map(|&i| graph.nodes[i].name.as_str()).collect();
        println!("  Shortest path Home → Resolve: {}", names.join(" → "));
    }

    // Leading tone resolution
    println!();
    println!("━━━ Leading Tone Resolution ━━━");
    let mut lt = LeadingTone::new(0, 0, 5, 0.9);
    println!("  Agent {} wants to reach position {} (pull strength: {:.1})",
        lt.agent_id, lt.target, lt.strength);
    println!("  Current: {} | Target: {} | Distance: {}", lt.current, lt.target, lt.distance());

    while !lt.is_resolved() {
        lt.step();
        let bar: String = "·".repeat(lt.current as usize) + "●";
        let resolved = if lt.is_resolved() { " ✅ resolved!" } else { "" };
        println!("    step → {}  |{}{}", lt.current, bar, resolved);
    }

    // Cadence
    println!();
    println!("━━━ Cadence Detection ━━━");
    let tension = vl.total_cost as i32 - 10;
    let stability = 12 - vl.total_cost as i32;
    let cadence = Cadence::from_harmony(tension, stability);
    println!("  Tension: {}, Stability: {}", tension, stability);
    println!("  Cadence: {:?}", cadence);
    println!("  → {}", cadence.description());
}
