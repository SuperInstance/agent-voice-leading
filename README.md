# agent-voice-leading

> Good voice leading makes a chord progression feel smooth. Good state transitions make a multi-agent system feel coherent. Both minimize unnecessary disruption while maximizing the expressiveness of each individual voice.

Voice leading for agent migration. When agents change roles, states, or configurations, this crate ensures the transitions are smooth — minimal disruption, maximal continuity, no jarring leaps.

If the `agent-riff` family is about *what happens during the jam*, `agent-voice-leading` is about *what happens between jams* — how you move from one configuration to the next without dropping the groove.

## Why This Crate Exists

Multi-agent systems have a problem that music theory solved 400 years ago: *how do you move from one state to another without it sounding terrible?*

In music, "voice leading" is the art of connecting chords so each individual voice (soprano, alto, tenor, bass) moves as little as possible. You don't jump from a C to an F# if you can walk there. The smoother the motion, the more coherent the harmony.

Multi-agent systems have the same problem. When agents change configurations — new roles, new positions, new responsibilities — you want each agent to move as little as possible. The agent that was "indexing documents" shouldn't suddenly become "monitoring network traffic." It should become "indexing documents better" or "indexing and searching documents."

This crate formalizes that intuition. Agents are voices. Configurations are chords. Transitions are voice leading. And the rules of counterpoint — no parallel motion, require contrary motion, limit leaps — become rules for agent migration.

## The Core Idea

An **agent** has a **position** — a numeric value representing its current state, role, or configuration. A **configuration** is a set of agents at positions — a "chord."

When you transition from one configuration to another, you want to minimize the total distance every agent moves. This is the **voice-leading problem**: find the optimal assignment of agents in the source configuration to positions in the target configuration.

```
From: Agent 0 at position 0, Agent 1 at position 10
To:   Agent 0 at position 1, Agent 1 at position 11

Voice leading: 0→1 (distance 1), 10→11 (distance 1)
Total cost: 2  ✓ Smooth
```

vs.

```
From: Agent 0 at position 0, Agent 1 at position 10
To:   Agent 0 at position 10, Agent 1 at position 0

Voice leading: 0→0, 10→10 (crossed assignment)
Total cost: 20  ✗ Disruptive (agents swap roles)
```

The `VoiceLeading::compute()` function finds the optimal (minimum-cost) assignment using greedy nearest-neighbor matching.

### Beyond Distance: Counterpoint Rules

Smooth transitions aren't just about minimizing total distance. They're about the *quality* of the motion. This crate implements four counterpoint rules:

| Rule | Musical Origin | Agent Interpretation |
|------|---------------|---------------------|
| **No parallel motion** | Two voices moving the same direction sounds like one voice | Two agents shifting the same way is just one change, duplicated |
| **Require contrary motion** | At least one pair moving oppositely creates tension/release | Some agent should move differently from the others |
| **Max leap** | No voice should jump more than a 3rd | No agent should change too drastically in one step |
| **Require resolution** | Leading tones must resolve | Agents pulled toward a target should eventually reach it |

The `CounterpointRules::strict()` preset enforces all four. `CounterpointRules::relaxed()` disables all of them (useful for emergency reconfigurations where smooth doesn't matter, speed does).

### Leading Tones

A **leading tone** is an agent that's being pulled toward a specific position. In music, the leading tone (the 7th scale degree) "wants" to resolve to the tonic. Here, an agent with a leading tone has a preferred target and a pull strength.

```rust
let mut lt = LeadingTone::new(0, 2, 5, 0.8); // agent 0, pulled from 2 toward 5
assert!(!lt.is_resolved());
lt.step(); // Move one position closer
lt.step();
lt.step();
assert!(lt.is_resolved()); // Now at 5
```

Leading tones are how you express "agent 0 should eventually be doing X" without forcing it to happen immediately. The transition plan distributes the movement over multiple steps.

### Cadences

Every transition has a **cadence** — the way it ends. Different cadences create different feelings:

| Cadence | Musical Feel | Team Interpretation |
|---------|-------------|-------------------|
| **Perfect** | Strong resolution (V→I) | Team reached full consensus. Done. |
| **Plagal** | Gentle close (IV→I) | Soft landing after a productive sprint. |
| **Deceptive** | Surprise twist (V→vi) | Unexpected pivot — high energy, new direction. |
| **Half** | Pause (ends on V) | Checkpoint — stable but unresolved. More work ahead. |

Cadences are determined by the tension and stability of the transition:

```rust
let cadence = Cadence::from_harmony(tension, stability);
// tension < 0, stability > 0 → Perfect (resolved)
// tension > 0, stability < 0 → Deceptive (surprise!)
// tension ≤ 0, stability ≤ 0 → Plagal (gentle)
// otherwise → Half (pause)
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Configuration (the "chord")                            │
│  [Agent 0: pos 0] [Agent 1: pos 3] [Agent 2: pos 7]    │
└─────────────────────────────────────────────────────────┘
           │
           │ VoiceLeading::compute()
           ▼
┌─────────────────────────────────────────────────────────┐
│  VoiceLeading                                           │
│  assignment: [(0,0), (1,1), (2,2)]  // optimal mapping │
│  total_cost: 6                                          │
│  individual_motions: [Ascending, Static, Ascending]     │
└─────────────────────────────────────────────────────────┘
           │
           │ CounterpointRules::check()
           ▼
┌─────────────────────────────────────────────────────────┐
│  Violations: ["Parallel motion: agents 0 and 2"]        │
└─────────────────────────────────────────────────────────┘
           │
           │ SmoothTransition::plan(from, to, n_steps)
           ▼
┌─────────────────────────────────────────────────────────┐
│  SmoothTransition                                       │
│  steps: [Config0, Config1, Config2, Config3, Config4]   │
│  Each step moves agents 1/n of the way to their target  │
└─────────────────────────────────────────────────────────┘
           │
           │ ChordGraph
           ▼
┌─────────────────────────────────────────────────────────┐
│  ChordGraph                                             │
│  nodes: [Named configurations]                          │
│  edges: [Voice-leading distances between configs]       │
│  .shortest_path("idle", "processing") → [0, 3, 5]      │
└─────────────────────────────────────────────────────────┘
```

## Usage

### Basic Voice Leading

```rust
use agent_voice_leading::{Configuration, AgentState, VoiceLeading};

let from = Configuration::new(vec![
    AgentState::new(0, 0),
    AgentState::new(1, 10),
]);
let to = Configuration::new(vec![
    AgentState::new(0, 1),
    AgentState::new(1, 11),
]);

let vl = VoiceLeading::compute(&from, &to);
assert_eq!(vl.total_cost, 2); // Each agent moved 1 step
assert!(vl.individual_motions().iter().all(|m| *m == Motion::Ascending));
```

### Smooth Transitions

```rust
use agent_voice_leading::SmoothTransition;

let from = Configuration::new(vec![AgentState::new(0, 0)]);
let to = Configuration::new(vec![AgentState::new(0, 10)]);

let transition = SmoothTransition::plan(&from, &to, 4);
assert_eq!(transition.steps.len(), 6); // start + 4 intermediate + end
// Agent 0: 0 → 2 → 4 → 6 → 8 → 10
```

### Counterpoint Checking

```rust
use agent_voice_leading::CounterpointRules;

let rules = CounterpointRules::strict();
let violations = rules.check(&voice_leading);

if !violations.is_empty() {
    for v in &violations {
        println!("Counterpoint violation: {}", v);
    }
}

// Or use relaxed rules for emergency transitions
let relaxed = CounterpointRules::relaxed();
assert!(relaxed.check(&voice_leading).is_empty()); // Always passes
```

### Chord Graph — Finding Paths Between Configurations

```rust
use agent_voice_leading::ChordGraph;

let mut graph = ChordGraph::new();

graph.add_node("idle", Configuration::new(vec![
    AgentState::new(0, 0), AgentState::new(1, 0),
]));
graph.add_node("processing", Configuration::new(vec![
    AgentState::new(0, 5), AgentState::new(1, 5),
]));
graph.add_node("high-load", Configuration::new(vec![
    AgentState::new(0, 10), AgentState::new(1, 10),
]));

// Find the smoothest path from idle to high-load
let path = graph.shortest_path("idle", "high-load").unwrap();
// Might go through "processing" if that's smoother than a direct jump
```

### Leading Tones — Gradual Resolution

```rust
use agent_voice_leading::LeadingTone;

let mut lt = LeadingTone::new(0, 2, 5, 0.8);
assert_eq!(lt.pull(), Motion::Ascending); // Pulled toward 5
assert_eq!(lt.distance(), 3); // 3 steps away

while !lt.is_resolved() {
    lt.step(); // Move one step closer each iteration
}
assert_eq!(lt.current, 5);
```

### Cadence Detection

```rust
use agent_voice_leading::Cadence;

// After a transition, determine the cadence
let cadence = Cadence::from_harmony(-1, 1); // Low tension, high stability
assert_eq!(cadence, Cadence::Perfect);
println!("{}", cadence.description());
// "Strong resolution — team reached full consensus"
```

## API Reference

### `Configuration`

| Method | Description |
|--------|-------------|
| `new(agents: Vec<AgentState>)` | Create a configuration |
| `positions() -> Vec<i32>` | Get all agent positions |
| `count() -> usize` | Number of agents |
| `total_distance(other) -> i32` | Sum of individual distances |
| `max_distance(other) -> i32` | Largest individual distance |

### `AgentState`

| Method | Description |
|--------|-------------|
| `new(id, position)` | Create an agent at a position |
| `distance_to(other) -> i32` | Distance to another agent's position |
| `motion_to(other) -> Motion` | Ascending, Descending, or Static |

### `VoiceLeading`

| Method | Description |
|--------|-------------|
| `compute(from, to) -> VoiceLeading` | Find optimal assignment (greedy nearest-neighbor) |
| `individual_motions() -> Vec<Motion>` | Direction each agent moves |

### `SmoothTransition`

| Method | Description |
|--------|-------------|
| `plan(from, to, n_steps) -> SmoothTransition` | Distribute movement over intermediate states |
| `step_count() -> usize` | Number of intermediate steps (excludes start/end) |

### `ChordGraph`

| Method | Description |
|--------|-------------|
| `new()` | Create an empty graph |
| `add_node(name, config) -> usize` | Add a named configuration, auto-create edges |
| `shortest_path(from_name, to_name) -> Option<Vec<usize>>` | Dijkstra shortest path by voice-leading distance |

### `CounterpointRules`

| Method | Description |
|--------|-------------|
| `strict() -> Self` | All rules enforced |
| `relaxed() -> Self` | No rules enforced |
| `check(voice_leading) -> Vec<String>` | Returns violations (empty = compliant) |

### `LeadingTone`

| Method | Description |
|--------|-------------|
| `new(agent_id, current, target, strength)` | Create a pull toward a target |
| `pull() -> Motion` | Direction of pull |
| `distance() -> i32` | Steps remaining |
| `is_resolved() -> bool` | At target? |
| `step()` | Move one position closer |

### `Cadence`

| Variant | When | Feel |
|---------|------|------|
| `Perfect` | tension < 0, stability > 0 | Full resolution |
| `Plagal` | tension ≤ 0, stability ≤ 0 | Gentle landing |
| `Deceptive` | tension > 0, stability < 0 | Surprise pivot |
| `Half` | otherwise | Pause / checkpoint |

## The Deeper Idea: Configuration Space as a Musical Space

Here's the thing that makes this crate more than a cute analogy: **voice leading is genuinely the right abstraction for agent state transitions.**

Consider the alternative. Most systems model agent transitions as discrete hops: agent goes from state A to state B. No intermediate states. No cost function. No concept of "smooth."

That works fine when agents are independent. But when agents are coordinated — when they're a team — individual hops create collective chaos. Agent 0 jumps to a new role. Agent 1 follows. Agent 2 is confused. The system thrashes.

Voice leading prevents this by optimizing for the *collective* cost of a transition. The total distance metric ensures no configuration is "expensive" to reach. The counterpoint rules ensure the transition has good *shape* — not just minimal distance, but varied motion. The smooth transition planner distributes changes over time so no single step is jarring.

This is exactly what good orchestration does. The cellos don't all jump to the new chord at the same time — they voice-lead there, each taking the shortest path, creating a coherent collective sound.

The chord graph extends this further: by modeling all known configurations as a weighted graph (edges weighted by voice-leading distance), you can find the *smoothest path* between any two configurations. Sometimes the direct path isn't the smoothest — going through an intermediate configuration can reduce total disruption.

## Related Crates

- **agent-riff** — Competitive riffing for agents (12 tests). What happens *during* the jam.
- **agent-riff-v2** — Fleet-aware riffing with cross-session learning (11 tests).
- **agent-riff-v3** — Self-bootstrapping with quality prediction (17 tests).
- **agent-riff-v4** — Fully self-bootstrapping with musician personas (21 tests).

This crate is the complement to the riff chain: riff handles the creative process, voice-leading handles the transitions between creative processes. Together, they form a complete model: compete → learn → transition → compete again, each cycle smoother than the last.

## License

MIT
