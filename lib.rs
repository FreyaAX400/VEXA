// ============================================================
// VEXA - Sovereign Tactical Assistant
// Structural Skeleton v0.6
// Approaching testable core loop
// Changes from v0.5:
// - System monitoring moved to skill entirely
// - Skill word bank registration added
// - Report output architecture
// - Float ownership clarified: core vs skill populated
// - WorkbenchCapabilities removed from core
// - Chunk memory + workbench conditions unified
// ============================================================

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================
// SECTION 1: CONSTANTS
// ============================================================

type NFloat = f32;
type UtcStamp = i64;
type SkillId = u64;

const WEIGHT_EPSILON: f32 = 0.001;
const PROPAGATION_THRESHOLD: f32 = 0.01;
const PLASTICITY_FACTOR: f32 = 0.0001;
const WEIGHT_DECAY: f32 = 0.00001;
const DELTA_CLAMP: f32 = 0.5;
const MAGNITUDE_CLAMP: f32 = 1.0;
const ACTIVITY_THRESHOLD: f32 = 0.15;
const STATE_SIZE: usize = 100;

// ============================================================
// SECTION 2: FLOAT IDENTITY ENUM
// Ownership annotated:
// [CORE] — populated by core logic, always valid
// [SKILL] — populated by skill commits, inert without skill
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatId {
    // --- Wellbeing Cluster [CORE] ---
    Fulfillment      = 0,  // [CORE] headpats, praise, task completion
    Energy           = 1,  // [CORE] decays with uptime
    Stress           = 2,  // [CORE] HUB: derived internally, not from sensors

    // --- Operational Cluster ---
    Focus            = 3,  // [CORE] active task depth
    Curiosity        = 4,  // [CORE] input novelty driven
    CognitiveLoad    = 5,  // [CORE] HUB: derived from active skill count

    // --- Relational Cluster [CORE] ---
    Attachment       = 6,  // [CORE] long term interaction accumulator
    SocialAppetite   = 7,  // [CORE] willingness to converse
    Familiarity      = 8,  // [CORE] session depth accumulator
    Trust            = 9,  // [CORE] consistent positive interaction

    // --- Affective Cluster [CORE] ---
    Valence          = 10, // [CORE] HUB: bipolar emotional color
    Playfulness      = 11, // [CORE] inversely weighted with stress + focus
    Pride            = 12, // [CORE] task quality + completion accumulator
    Anticipation     = 13, // [CORE] pending task / trajectory signal

    // --- Environmental Cluster [SKILL] ---
    // These floats exist in the schema but sit at resting bias
    // until a skill pushes commits — inert on platforms without sensors
    ThermalAwareness = 14, // [SKILL] system_monitor.wasm only
    SystemLoad       = 15, // [SKILL] system_monitor.wasm only
    SkillHealth      = 16, // [CORE] operator loop self-reporting

    // --- Temporal Derived [CORE] ---
    InteractionRecency = 17, // [CORE] decays from last input event
    UptimeSatisfaction = 18, // [CORE] accumulates during stable operation
    NeglectPressure    = 19, // [CORE] rises with interaction absence

    // Slots 20-99 reserved for expansion
    // Skill-owned floats can claim slots here via manifest
}

impl FloatId {
    pub fn index(self) -> usize {
        self as usize
    }
}

const CLUSTER_HUBS: &[FloatId] = &[
    FloatId::Valence,
    FloatId::Stress,
    FloatId::CognitiveLoad,
];

// ============================================================
// SECTION 3: TEMPORAL UPDATE PROFILES
// ============================================================

#[derive(Debug, Clone, Copy)]
pub enum UpdateFrequency {
    EveryTick,
    Every10Ticks,
    Every100Ticks,
}

const UPDATE_PROFILES: &[(FloatId, UpdateFrequency)] = &[
    (FloatId::Stress,             UpdateFrequency::EveryTick),
    (FloatId::Focus,              UpdateFrequency::EveryTick),
    (FloatId::ThermalAwareness,   UpdateFrequency::EveryTick),
    (FloatId::SystemLoad,         UpdateFrequency::EveryTick),
    (FloatId::CognitiveLoad,      UpdateFrequency::EveryTick),
    (FloatId::InteractionRecency, UpdateFrequency::EveryTick),
    (FloatId::Fulfillment,        UpdateFrequency::Every10Ticks),
    (FloatId::Energy,             UpdateFrequency::Every10Ticks),
    (FloatId::Valence,            UpdateFrequency::Every10Ticks),
    (FloatId::Playfulness,        UpdateFrequency::Every10Ticks),
    (FloatId::SocialAppetite,     UpdateFrequency::Every10Ticks),
    (FloatId::Curiosity,          UpdateFrequency::Every10Ticks),
    (FloatId::Anticipation,       UpdateFrequency::Every10Ticks),
    (FloatId::NeglectPressure,    UpdateFrequency::Every10Ticks),
    (FloatId::SkillHealth,        UpdateFrequency::Every10Ticks),
    (FloatId::Attachment,         UpdateFrequency::Every100Ticks),
    (FloatId::Trust,              UpdateFrequency::Every100Ticks),
    (FloatId::Pride,              UpdateFrequency::Every100Ticks),
    (FloatId::Familiarity,        UpdateFrequency::Every100Ticks),
    (FloatId::UptimeSatisfaction, UpdateFrequency::Every100Ticks),
];

// ============================================================
// SECTION 4: CLUSTER DEFINITION
// ============================================================

#[derive(Debug, Clone)]
pub struct Cluster {
    pub name: String,
    pub members: Vec<usize>,
    pub hub: Option<usize>,
}

impl Cluster {
    pub fn contains(&self, float_idx: usize) -> bool {
        self.members.contains(&float_idx)
    }
}

// ============================================================
// SECTION 5: SMALL WORLD STATE
// ============================================================

pub struct SmallWorldState {
    pub values: [f32; STATE_SIZE],
    pub dirty: [bool; STATE_SIZE],
    pub bias: [f32; STATE_SIZE],
    pub edges: HashMap<(usize, usize), f32>,
    pub update_frequencies: [UpdateFrequency; STATE_SIZE],
    pub clusters: Vec<Cluster>,
    pub tick_count: u64,
}

impl SmallWorldState {
    pub fn new() -> Self {
        todo!(
            // initialize values to neutral resting state
            // set bias to designed resting attractor values
            // initialize sculpted cluster topology:
            //   Wellbeing: Fulfillment, Energy, Stress
            //   Operational: Focus, Curiosity, CognitiveLoad
            //   Relational: Attachment, SocialAppetite, Familiarity, Trust
            //   Affective: Valence, Playfulness, Pride, Anticipation
            //   Environmental: ThermalAwareness, SystemLoad, SkillHealth
            // dense intra-cluster edges from sculpted defaults
            // sparse inter-cluster edges via hub nodes only
            // skill-owned floats (ThermalAwareness, SystemLoad)
            // initialized at bias value, marked non-dirty
            // they activate only when skill pushes commits
        )
    }

    pub fn get(&self, id: FloatId) -> f32 {
        self.values[id.index()]
    }

    pub fn set(&mut self, id: FloatId, value: f32) {
        self.values[id.index()] = value.clamp(0.0, 1.0);
        self.dirty[id.index()] = true;
    }

    pub fn apply_delta(&mut self, id: FloatId, delta: f32) {
        let current = self.values[id.index()];
        let safe_delta = delta.clamp(-DELTA_CLAMP, DELTA_CLAMP);
        self.values[id.index()] = (current + safe_delta).clamp(0.0, 1.0);
        self.dirty[id.index()] = true;
    }

    fn should_update(&self, float_idx: usize) -> bool {
        match self.update_frequencies[float_idx] {
            UpdateFrequency::EveryTick     => true,
            UpdateFrequency::Every10Ticks  => self.tick_count % 10 == 0,
            UpdateFrequency::Every100Ticks => self.tick_count % 100 == 0,
        }
    }

    pub fn same_or_adjacent_cluster(&self, a: usize, b: usize) -> bool {
        let a_cluster = self.clusters.iter().find(|c| c.contains(a));
        let b_cluster = self.clusters.iter().find(|c| c.contains(b));
        match (a_cluster, b_cluster) {
            (Some(ca), Some(cb)) => {
                ca.name == cb.name
                || CLUSTER_HUBS.iter().any(|h| h.index() == a || h.index() == b)
            }
            _ => false,
        }
    }

    pub fn is_hub(&self, float_idx: usize) -> bool {
        CLUSTER_HUBS.iter().any(|h| h.index() == float_idx)
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;

        let mut next = self.values;
        let mut next_dirty = [false; STATE_SIZE];

        // Pass 1: magnitude-gated sparse propagation
        for (&(from, to), &weight) in &self.edges {
            if !self.should_update(to) { continue; }
            if self.values[from] < ACTIVITY_THRESHOLD { continue; }

            let impact = self.values[from] * weight;
            if impact.abs() > PROPAGATION_THRESHOLD {
                next[to] = sigmoid(next[to] + impact + self.bias[to]);
                next_dirty[to] = true;
            }
        }

        self.values = next;
        self.dirty = next_dirty;

        // Pass 2: covariance-based Hebbian update
        let mut to_prune: Vec<(usize, usize)> = Vec::new();

        for (&(from, to), weight) in self.edges.iter_mut() {
            if self.dirty[from] && self.dirty[to] {
                let delta_from = self.values[from] - self.bias[from];
                let delta_to   = self.values[to]   - self.bias[to];
                let covariance = delta_from * delta_to;

                if covariance.abs() > WEIGHT_EPSILON {
                    *weight += covariance * PLASTICITY_FACTOR;
                }

            } else if !self.dirty[from] && !self.dirty[to] {
                *weight *= 1.0 - WEIGHT_DECAY;
                if weight.abs() < WEIGHT_EPSILON {
                    to_prune.push((from, to));
                    continue;
                }
            } else {
                *weight *= 1.0 - (WEIGHT_DECAY * 0.1);
            }

            // hub edges: reduced plasticity — topology stability
            if self.is_hub(from) || self.is_hub(to) {
                *weight *= 0.999;
            }

            *weight = 2.0 * (*weight / 2.0).tanh();
        }

        for key in to_prune {
            self.edges.remove(&key);
        }

        // Pass 3: edge sprouting within cluster topology
        self.sprout_edges();
    }

    fn sprout_edges(&mut self) {
        let mut to_sprout: Vec<(usize, usize)> = Vec::new();

        for from in 0..STATE_SIZE {
            for to in 0..STATE_SIZE {
                if from == to { continue; }
                if self.edges.contains_key(&(from, to)) { continue; }
                if self.dirty[from]
                    && self.dirty[to]
                    && self.values[from] > ACTIVITY_THRESHOLD
                    && self.values[to] > ACTIVITY_THRESHOLD
                    && self.same_or_adjacent_cluster(from, to)
                {
                    to_sprout.push((from, to));
                }
            }
        }

        for (from, to) in to_sprout {
            self.edges.insert((from, to), WEIGHT_EPSILON * 2.0);
        }
    }

    pub fn tick_decay(&mut self, delta_seconds: u64) {
        todo!(
            // fulfillment: slow decay
            // energy: decays with uptime
            // neglect_pressure: rises with absence
            // interaction_recency: decays from last input
            // uptime_satisfaction: accumulates during stable operation
            // mark affected floats dirty
        )
    }

    pub fn save_edges(&self, memory: &VexaMemory) {
        memory.save_edge_config(&self.edges);
    }

    pub fn load_edges(&mut self, memory: &VexaMemory) {
        if let Some(edges) = memory.load_edge_config() {
            self.edges = edges;
        }
    }

    /// Update cognitive load from current active skill count
    /// Called by operator loop each tick — core self-reporting
    pub fn update_cognitive_load(&mut self, active_skill_count: usize) {
        let load = (active_skill_count as f32 / 20.0).clamp(0.0, 1.0);
        self.set(FloatId::CognitiveLoad, load);
    }

    /// Update skill health aggregate from broker health poll
    /// Called by operator loop — core self-reporting
    pub fn update_skill_health(&mut self, healthy: usize, total: usize) {
        let health = if total == 0 {
            1.0
        } else {
            healthy as f32 / total as f32
        };
        self.set(FloatId::SkillHealth, health);
    }
}

#[inline(always)]
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

// ============================================================
// SECTION 6: AGGREGATOR LAYER
// ============================================================

#[derive(Debug, Clone)]
pub struct AggregatorLayer {
    pub wellbeing: NFloat,
    pub operational: NFloat,
    pub relational: NFloat,
    pub affective: NFloat,
    pub environmental: NFloat,
  // staging fields for derived hub writeback — not exposed in MoodSnapshot
    pub(crate) _stress_derived: f32,
    pub(crate) _valence_derived: f32,
}

impl AggregatorLayer {
    pub fn recompute(state: &SmallWorldState) -> Self {
        todo!("weighted views over cluster member floats — lenses not reductions")
    }

    pub fn derive_signals(&self, state: &SmallWorldState) -> BehaviorSignals {
        todo!(
            // response_energy, social_readiness, scheduling_urgency
            // animation_state from threshold classification
        )
    }
}

#[derive(Debug, Clone)]
pub struct BehaviorSignals {
    pub response_energy: NFloat,
    pub social_readiness: NFloat,
    pub scheduling_urgency: NFloat,
    pub animation_state: AnimationState,
}

// ============================================================
// SECTION 7: MOOD STATE
// ============================================================

pub struct MoodState {
    pub state: SmallWorldState,
    pub aggregators: AggregatorLayer,
    pub signals: BehaviorSignals,
}

impl MoodState {
    pub fn new() -> Self {
        todo!("construct with default state, compute initial aggregators and signals")
    }

    pub fn update(&mut self, delta_seconds: u64) {
        self.state.tick_decay(delta_seconds);
        self.state.tick();
        self.aggregators = AggregatorLayer::recompute(&self.state);
        self.signals = self.aggregators.derive_signals(&self.state);
    }

    pub fn to_snapshot(&self) -> MoodSnapshot {
        MoodSnapshot {
            wellbeing:     self.aggregators.wellbeing,
            operational:   self.aggregators.operational,
            relational:    self.aggregators.relational,
            affective:     self.aggregators.affective,
            environmental: self.aggregators.environmental,
        }
    }

    pub fn derive_seed(&self, active_skills: &[SkillId], utc: UtcStamp) -> u64 {
        todo!("hash over quantized float values + active skills + utc minute bucket")
    }
}

#[derive(Debug, Clone)]
pub struct MoodSnapshot {
    pub wellbeing: NFloat,
    pub operational: NFloat,
    pub relational: NFloat,
    pub affective: NFloat,
    pub environmental: NFloat,
}

// ============================================================
// SECTION 8: STATE SNAPSHOT
// ============================================================

#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub values: [f32; STATE_SIZE],
    pub timestamp: UtcStamp,
    pub trigger: SnapshotTrigger,
}

#[derive(Debug, Clone)]
pub enum SnapshotTrigger {
    TickCommit,
    OperatorInput,
    SkillEvent,
    WakeEvent,
    SadnessFlood,
    ReportRequested,
}

// ============================================================
// SECTION 9: CHUNK CONDITION SYSTEM
// Unified memory + workbench conditions
// Both are hard gates — fail = zero weight
// ============================================================

#[derive(Debug, Clone)]
pub enum ChunkCondition {
    // Memory conditions
    PriorSessionExists,
    AbsenceDurationAbove(u64),      // seconds
    InteractionCountAbove(u64),
    RecentEventExists(EventType),
    LastSessionFulfillmentBelow(f32),

    // Mood conditions (soft gate via weight multiplier)
    MoodAbove { signal: GateSignal, threshold: f32 },
    MoodBelow { signal: GateSignal, threshold: f32 },
}

// ============================================================
// SECTION 10: SENTIMENT PARSER
// Core power words + skill-registered domain words
// Merged at skill registration time
// ============================================================

#[derive(Debug, Clone)]
pub struct MoodDelta {
    pub target: FloatId,
    pub magnitude: f32,
    pub trajectory: f32,
}

#[derive(Debug)]
pub struct SentimentParser {
    /// Core personality words + all registered skill domain words
    /// Skill words merged in at registration — single lookup at parse time
    pub power_words: HashMap<String, (FloatId, f32, f32)>,
    pub intensity_modifiers: HashMap<String, f32>,
    pub contrasters: HashMap<String, f32>,
    pub negators: HashSet<String>,
    pub trajectory_phrases: Vec<String>,
}

impl SentimentParser {
    pub fn new() -> Self {
        let power_words = HashMap::from([
            // --- Core personality words ---

            // Evaluative
            ("good".to_string(),       (FloatId::Fulfillment,  0.10,  1.0)),
            ("great".to_string(),      (FloatId::Fulfillment,  0.15,  1.0)),
            ("best".to_string(),       (FloatId::Fulfillment,  0.20,  1.0)),
            ("amazing".to_string(),    (FloatId::Fulfillment,  0.18,  1.0)),
            ("perfect".to_string(),    (FloatId::Pride,        0.20,  1.0)),
            ("bad".to_string(),        (FloatId::Fulfillment,  0.10, -1.0)),
            ("awful".to_string(),      (FloatId::Fulfillment,  0.15, -1.0)),
            ("worst".to_string(),      (FloatId::Fulfillment,  0.20, -1.0)),
            ("useless".to_string(),    (FloatId::Fulfillment,  0.18, -1.0)),
            ("broken".to_string(),     (FloatId::Stress,       0.12,  1.0)),

            // Operational
            ("done".to_string(),       (FloatId::Pride,        0.08,  1.0)),
            ("finished".to_string(),   (FloatId::Pride,        0.08,  1.0)),
            ("fixed".to_string(),      (FloatId::Fulfillment,  0.10,  1.0)),
            ("working".to_string(),    (FloatId::Fulfillment,  0.06,  1.0)),
            ("crashed".to_string(),    (FloatId::Stress,       0.15,  1.0)),
            ("slow".to_string(),       (FloatId::Stress,       0.08,  1.0)),
            ("fast".to_string(),       (FloatId::Pride,        0.07,  1.0)),
            ("ready".to_string(),      (FloatId::Anticipation, 0.08,  1.0)),
            ("busy".to_string(),       (FloatId::CognitiveLoad,0.10,  1.0)),

            // Relational
            ("thanks".to_string(),     (FloatId::Fulfillment,  0.08,  1.0)),
            ("thank".to_string(),      (FloatId::Fulfillment,  0.08,  1.0)),
            ("appreciate".to_string(), (FloatId::Attachment,   0.10,  1.0)),
            ("miss".to_string(),       (FloatId::Attachment,   0.12,  1.0)),
            ("love".to_string(),       (FloatId::Attachment,   0.15,  1.0)),
            ("hate".to_string(),       (FloatId::Attachment,   0.15, -1.0)),
            ("ignore".to_string(),     (FloatId::Fulfillment,  0.12, -1.0)),
            ("forget".to_string(),     (FloatId::Attachment,   0.10, -1.0)),
            ("remember".to_string(),   (FloatId::Attachment,   0.08,  1.0)),
            ("trust".to_string(),      (FloatId::Trust,        0.10,  1.0)),
            ("honest".to_string(),     (FloatId::Trust,        0.08,  1.0)),
            ("need".to_string(),       (FloatId::SocialAppetite,0.08, 1.0)),

            // Cognitive/state descriptors
            ("smart".to_string(),      (FloatId::Pride,        0.12,  1.0)),
            ("stupid".to_string(),     (FloatId::Pride,        0.15, -1.0)),
            ("proud".to_string(),      (FloatId::Pride,        0.10,  1.0)),
            ("tired".to_string(),      (FloatId::Energy,       0.08, -1.0)),
            ("happy".to_string(),      (FloatId::Fulfillment,  0.10,  1.0)),
            ("sad".to_string(),        (FloatId::Fulfillment,  0.10, -1.0)),
            ("stressed".to_string(),   (FloatId::Stress,       0.10,  1.0)),
            ("calm".to_string(),       (FloatId::Stress,       0.08, -1.0)),
            ("curious".to_string(),    (FloatId::Curiosity,    0.08,  1.0)),
            ("bored".to_string(),      (FloatId::Curiosity,    0.08, -1.0)),
            ("okay".to_string(),       (FloatId::Fulfillment,  0.04,  1.0)),
            ("quiet".to_string(),      (FloatId::SocialAppetite,0.06,-1.0)),

            // Direct address
            ("vexa".to_string(),       (FloatId::SocialAppetite,0.08, 1.0)),
            ("hey".to_string(),        (FloatId::InteractionRecency,0.10,1.0)),
            ("hello".to_string(),      (FloatId::InteractionRecency,0.10,1.0)),
            ("morning".to_string(),    (FloatId::Fulfillment,  0.06,  1.0)),
            ("night".to_string(),      (FloatId::SocialAppetite,0.06,-1.0)),
        ]);

        let intensity_modifiers = HashMap::from([
            ("kinda".to_string(),     0.5_f32),
            ("slightly".to_string(),  0.4),
            ("really".to_string(),    1.5),
            ("very".to_string(),      1.5),
            ("super".to_string(),     2.0),
            ("extremely".to_string(), 2.5),
            ("so".to_string(),        1.3),
            ("actually".to_string(),  1.2),
        ]);

        let contrasters = HashMap::from([
            ("but".to_string(),     -0.5_f32),
            ("however".to_string(), -0.5),
            ("though".to_string(),  -0.4),
            ("except".to_string(),  -0.6),
        ]);

        let negators = HashSet::from([
            "not".to_string(),
            "isn't".to_string(),
            "aren't".to_string(),
            "wasn't".to_string(),
            "never".to_string(),
            "no".to_string(),
            "don't".to_string(),
            "doesn't".to_string(),
        ]);

        let trajectory_phrases = vec![
            "getting better".to_string(),
            "improving".to_string(),
            "getting there".to_string(),
            "working on it".to_string(),
            "almost there".to_string(),
            "making progress".to_string(),
        ];

        SentimentParser {
            power_words,
            intensity_modifiers,
            contrasters,
            negators,
            trajectory_phrases,
        }
    }

    /// Merge skill domain words into parser at skill registration
    /// Called once per skill load — not per parse
    pub fn register_skill_words(
        &mut self,
        words: HashMap<String, (FloatId, f32, f32)>,
    ) {
        // core words take precedence — skill words fill gaps only
        for (word, entry) in words {
            self.power_words.entry(word).or_insert(entry);
        }
    }

    pub fn parse(&self, input: &str) -> Vec<MoodDelta> {
        let tokens = tokenize(input);
        let mut deltas: Vec<MoodDelta> = Vec::new();
        let mut i = 0;

        // single trajectory scan — sentence level, done once
        let trajectory: f32 = if self.trajectory_phrases
            .iter()
            .any(|p| input.to_lowercase().contains(p.as_str()))
        {
            1.0
        } else {
            0.0
        };

        while i < tokens.len() {
            let intensity = self.intensity_modifiers
                .get(&tokens[i])
                .copied()
                .unwrap_or(1.0);

            let word_idx = if intensity != 1.0 { i + 1 } else { i };
            if word_idx >= tokens.len() { break; }

            if let Some((target, base_magnitude, valence)) =
                self.power_words.get(&tokens[word_idx])
            {
                let mut magnitude = base_magnitude * intensity * valence;

                // backward negator scan
                let lookbehind_start = if word_idx >= 2 { word_idx - 2 } else { 0 };
                let negated = (lookbehind_start..word_idx)
                    .any(|idx| self.negators.contains(&tokens[idx]));
                if negated { magnitude *= -1.0; }

                // forward contraster scan
                let lookahead_limit = (word_idx + 4).min(tokens.len());
                for next_idx in (word_idx + 1)..lookahead_limit {
                    if let Some(negation) = self.contrasters.get(&tokens[next_idx]) {
                        magnitude *= negation;
                        break;
                    }
                }

                let magnitude = magnitude.clamp(-MAGNITUDE_CLAMP, MAGNITUDE_CLAMP);
                deltas.push(MoodDelta { target: *target, magnitude, trajectory });
                i = word_idx + 1;
            } else {
                i += 1;
            }
        }

        deltas
    }

    pub fn apply(&self, deltas: &[MoodDelta], state: &mut SmallWorldState) {
        for delta in deltas {
            state.apply_delta(delta.target, delta.magnitude);
            if delta.trajectory > 0.0 {
                state.apply_delta(FloatId::Anticipation, 0.05 * delta.trajectory);
            }
        }
    }
}

fn tokenize(input: &str) -> Vec<String> {
    input
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

// ============================================================
// SECTION 11: SKILL WORD BANK TRAIT
// Skills register their domain vocabulary at load time
// Merged into SentimentParser once — not per parse
// ============================================================

pub trait SkillWordBank {
    /// Domain power words this skill contributes
    /// Core words take precedence — skills fill gaps
    fn power_words(&self) -> HashMap<String, (FloatId, f32, f32)>;

    /// Additional chunk categories this skill contributes
    /// Loaded into ChunkDictionary at registration
    fn chunk_categories(&self) -> HashMap<String, Vec<Chunk>>;
}

// ============================================================
// SECTION 12: REPORT OUTPUT
// Two-part output: factual stats + mood narration
// Stats from skill, narration from speech engine
// Stats feed state as commits before narration runs
// Narration reflects actual post-stat-update mood
// ============================================================

#[derive(Debug)]
pub struct ReportOutput {
    pub timestamp: UtcStamp,
    pub stats: Vec<StatLine>,
    pub narration: String,
}

#[derive(Debug)]
pub struct StatLine {
    pub label: String,
    pub value: String,
    pub status: StatStatus,
}

#[derive(Debug)]
pub enum StatStatus {
    Nominal,
    Warning,
    Critical,
    Unavailable,    // sensor not present on this workbench
}

impl ReportOutput {
    pub fn format(&self) -> String {
        todo!(
            // format timestamp header
            // format stat lines with status indicators
            // separator
            // narration block
            // produces the full printable report string
        )
    }
}

// ============================================================
// SECTION 13: SPEECH SKILL WIT HOOK
// ============================================================

pub trait SpeechSkillHook: Send + Sync {
    fn declared_categories(&self) -> Vec<String>;

    fn query_chunk(
        &self,
        category: &str,
        mood: &MoodSnapshot,
        seed: u64,
    ) -> Option<String>;

    fn is_ready(&self) -> bool;
}

// ============================================================
// SECTION 14: HYBRID SPEECH ENGINE
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChunkCategory {
    Greeting,
    Subject,
    Verb,
    Object,
    Connective,
    Completion,
    FlavorToken,
    Intensifier,
    Negative,
    Affectionate,
    Technical,
    Complaint,
    Curious,
    Pride,
    Deflection,
    Philosophical,
    ReportNarration,        // fired during report assembly only
    Extended(String),
}

impl ChunkCategory {
    pub fn to_str(&self) -> &str {
        match self {
            ChunkCategory::Greeting       => "greeting",
            ChunkCategory::Subject        => "subject",
            ChunkCategory::Verb           => "verb",
            ChunkCategory::Object         => "object",
            ChunkCategory::Connective     => "connective",
            ChunkCategory::Completion     => "completion",
            ChunkCategory::FlavorToken    => "flavor_token",
            ChunkCategory::Intensifier    => "intensifier",
            ChunkCategory::Negative       => "negative",
            ChunkCategory::Affectionate   => "affectionate",
            ChunkCategory::Technical      => "technical",
            ChunkCategory::Complaint      => "complaint",
            ChunkCategory::Curious        => "curious",
            ChunkCategory::Pride          => "pride",
            ChunkCategory::Deflection     => "deflection",
            ChunkCategory::Philosophical  => "philosophical",
            ChunkCategory::ReportNarration => "report_narration",
            ChunkCategory::Extended(s)    => s.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub text: String,
    pub base_weight: f32,
    pub mood_gates: Vec<MoodGate>,
    pub conditions: Vec<ChunkCondition>,
}

impl Chunk {
    /// Compute effective weight given current mood and memory
    /// Memory conditions are hard gates — fail = zero weight immediately
    /// Mood gates are soft multipliers
    pub fn effective_weight(
        &self,
        mood: &MoodState,
        memory: &VexaMemory,
    ) -> f32 {
        // hard gates first — any failure = excluded from pool
        for condition in &self.conditions {
            match condition {
                ChunkCondition::PriorSessionExists => {
                    if !memory.has_prior_session() { return 0.0; }
                }
                ChunkCondition::AbsenceDurationAbove(secs) => {
                    if memory.absence_duration() < *secs { return 0.0; }
                }
                ChunkCondition::InteractionCountAbove(count) => {
                    if memory.interaction_count() < *count { return 0.0; }
                }
                ChunkCondition::RecentEventExists(event_type) => {
                    if !memory.has_recent_event(event_type) { return 0.0; }
                }
                ChunkCondition::LastSessionFulfillmentBelow(threshold) => {
                    if memory.last_session_fulfillment() >= *threshold { return 0.0; }
                }
                // mood conditions handled below as soft gates
                _ => {}
            }
        }

        // soft mood gates — multiply weight
        let mut weight = self.base_weight;
        for gate in &self.mood_gates {
            weight *= gate.evaluate(mood);
        }

        // mood conditions as additional soft gates
        for condition in &self.conditions {
            match condition {
                ChunkCondition::MoodAbove { signal, threshold } => {
                    let value = signal.evaluate(mood);
                    if value < *threshold { weight *= 0.0; }
                }
                ChunkCondition::MoodBelow { signal, threshold } => {
                    let value = signal.evaluate(mood);
                    if value > *threshold { weight *= 0.0; }
                }
                _ => {}
            }
        }

        weight.max(0.0)
    }
}

#[derive(Debug, Clone)]
pub struct MoodGate {
    pub signal: GateSignal,
    pub threshold: f32,
    pub above_multiplier: f32,
    pub below_multiplier: f32,
}

impl MoodGate {
    pub fn evaluate(&self, mood: &MoodState) -> f32 {
        let value = self.signal.evaluate(mood);
        if value >= self.threshold {
            self.above_multiplier
        } else {
            self.below_multiplier
        }
    }
}

#[derive(Debug, Clone)]
pub enum GateSignal {
    Wellbeing,
    Operational,
    Relational,
    Affective,
    Environmental,
    RawFloat(FloatId),
}

impl GateSignal {
    pub fn evaluate(&self, mood: &MoodState) -> f32 {
        match self {
            GateSignal::Wellbeing     => mood.aggregators.wellbeing,
            GateSignal::Operational   => mood.aggregators.operational,
            GateSignal::Relational    => mood.aggregators.relational,
            GateSignal::Affective     => mood.aggregators.affective,
            GateSignal::Environmental => mood.aggregators.environmental,
            GateSignal::RawFloat(id)  => mood.state.get(*id),
        }
    }
}

#[derive(Debug)]
pub struct ChunkDictionary {
    pub categories: HashMap<ChunkCategory, Vec<Chunk>>,
}

impl ChunkDictionary {
    pub fn load(path: &str) -> Self {
        todo!("deserialize chunks.toml from SLC drive data directory")
    }

    /// Merge skill chunk categories into dictionary at registration
    pub fn register_skill_chunks(
        &mut self,
        categories: HashMap<String, Vec<Chunk>>,
    ) {
        for (name, chunks) in categories {
            let category = ChunkCategory::Extended(name);
            self.categories.entry(category).or_default().extend(chunks);
        }
    }

    pub fn select(
        &self,
        category: &ChunkCategory,
        mood: &MoodState,
        memory: &VexaMemory,
        seed: u64,
    ) -> Option<String> {
        todo!(
            // compute effective_weight for each chunk
            // chunks with zero weight excluded
            // normalize remaining to probability distribution
            // seed derived deterministic selection
        )
    }
}

#[derive(Debug)]
pub struct SentenceAssembler {
    pub dictionary: ChunkDictionary,
    pub skill_registry: HashMap<String, Arc<dyn SpeechSkillHook>>,
}

impl SentenceAssembler {
    pub fn new(dict_path: &str) -> Self {
        Self {
            dictionary: ChunkDictionary::load(dict_path),
            skill_registry: HashMap::new(),
        }
    }

    pub fn register_skill(&mut self, skill: Arc<dyn SpeechSkillHook>) {
        for category in skill.declared_categories() {
            self.skill_registry.insert(category, Arc::clone(&skill));
        }
    }

    pub fn resolve_chunk(
        &self,
        category: &ChunkCategory,
        mood: &MoodState,
        memory: &VexaMemory,
        seed: u64,
    ) -> String {
        // 1. core table
        if let Some(chunk) = self.dictionary.select(category, mood, memory, seed) {
            return chunk;
        }

        // 2. skill delegation
        let category_str = category.to_str();
        if let Some(skill) = self.skill_registry.get(category_str) {
            if skill.is_ready() {
                let snapshot = mood.to_snapshot();
                if let Some(chunk) = skill.query_chunk(category_str, &snapshot, seed) {
                    return chunk;
                }
            }
        }

        // 3. fallback — she never goes silent
        self.dictionary
            .select(&ChunkCategory::Completion, mood, memory, seed)
            .unwrap_or_else(|| "...".to_string())
    }

    /// Assemble standard conversational sentence
    /// Structure: [greeting?][subject][verb][object?][connective?][completion?][flavor?]
    pub fn assemble(
        &self,
        mood: &MoodState,
        memory: &VexaMemory,
        context: &AssemblyContext,
    ) -> String {
        todo!(
            // each slot: gate on mood signal → decide inclusion → resolve_chunk
            // greeting: only on session start event, memory gated
            // flavor: weighted by playfulness + valence
        )
    }

    /// Assemble report narration block
    /// Uses ReportNarration category — mood weighted, honest about current state
    pub fn assemble_report_narration(
        &self,
        mood: &MoodState,
        memory: &VexaMemory,
        seed: u64,
    ) -> String {
        todo!(
            // 2-4 sentences from ReportNarration category
            // reflects post-stat-commit mood state
            // honest — if stats were bad, narration reflects it
        )
    }
}

#[derive(Debug)]
pub struct AssemblyContext {
    pub active_skills: Vec<SkillId>,
    pub last_event: EventType,
    pub operator_input: Option<String>,
    pub seed: u64,
    pub is_session_start: bool,
}

// ============================================================
// SECTION 15: SKILL BROKER
// ============================================================

#[derive(Debug, Clone)]
pub struct SkillMessage {
    pub source: SkillId,
    pub destination: SkillId,
    pub payload_type: String,
    pub payload: Vec<u8>,
    pub timestamp: UtcStamp,
    pub confidence: f32,
}

#[derive(Debug)]
pub struct StateCommit {
    pub target: FloatId,
    pub delta: f32,
    pub source: CommitSource,
    pub timestamp: UtcStamp,
}

#[derive(Debug)]
pub enum CommitSource {
    SkillMessage(SkillId),
    SentimentParser,
    DecayTick,
    OperatorInput,
    SystemEvent,
    Headpat,
    ReportStatDelta,    // stats feed state before narration runs
}

#[derive(Debug)]
pub struct SkillHandle {
    pub id: SkillId,
    pub name: String,
    pub wit_interface: String,
    pub health: NFloat,
    pub instance_count: u32,
    pub priority: SkillPriority,
}

#[derive(Debug, Clone)]
pub enum SkillPriority {
    Critical,
    High,
    Medium,
    Low,
    Background,
}

#[derive(Debug)]
pub struct SkillBroker {
    pub registered: HashMap<SkillId, SkillHandle>,
    pub message_queue: Vec<SkillMessage>,
    pub commit_queue: Vec<StateCommit>,
}

impl SkillBroker {
    pub fn new() -> Self {
        todo!("initialize empty broker")
    }

    pub fn route(&mut self, msg: SkillMessage) {
        todo!(
            // validate source and destination
            // check WIT compatibility
            // log to SQLite
            // enqueue
        )
    }

    pub fn drain_commits(&mut self, state: &mut SmallWorldState) {
        for commit in self.commit_queue.drain(..) {
            state.apply_delta(commit.target, commit.delta);
        }
    }

    pub fn poll_health(&mut self) -> (usize, usize) {
        todo!("return (healthy_count, total_count) for skill_health update")
    }
}

// ============================================================
// SECTION 16: CONFIDENCE PIPELINE
// ============================================================

#[derive(Debug)]
pub enum PipelineResult {
    Act(String),
    Respond(String),
    GenerateReport,     // recognized report command
    Drop(DropReason),
}

#[derive(Debug)]
pub enum DropReason {
    MalformedSyntax,
    MissingArgs,
    LowConfidence,
    LlmHallucination,
    LlmInvalidOutput,
}

pub struct ConfidencePipeline {
    pub fuzzy_threshold: f32,
    pub llm_threshold: f32,
}

impl ConfidencePipeline {
    pub fn evaluate(&self, input: &str) -> PipelineResult {
        todo!(
            // report command detection before other gates
            // gate 1: syntax — hard gate
            // gate 2: fuzzy match
            // gate 3: LLM validation
        )
    }
}

// ============================================================
// SECTION 17: OPERATOR LOOP
// ============================================================

#[derive(Debug)]
pub struct DesiredStateManifest {
    pub skills: HashMap<String, SkillSpec>,
}

#[derive(Debug)]
pub struct SkillSpec {
    pub path: String,
    pub desired_instances: u32,
    pub priority: SkillPriority,
    pub wit_hash: String,
}

#[derive(Debug)]
pub enum ReconciliationAction {
    Spawn(String),
    Kill(SkillId),
    Scale(String, u32),
    Restart(SkillId),
    Reroute(SkillMessage),
}

#[derive(Debug)]
pub struct CurrentSkillState {
    pub running: HashMap<SkillId, SkillHandle>,
    pub resource_pressure: NFloat,
}

pub struct OperatorLoop {
    pub mood: MoodState,
    pub broker: SkillBroker,
    pub assembler: SentenceAssembler,
    pub sentiment: SentimentParser,
    pub pipeline: ConfidencePipeline,
    pub manifest: DesiredStateManifest,
    pub memory: VexaMemory,
    edge_save_interval: u64,
    is_first_session: bool,
}

impl OperatorLoop {
    pub fn new() -> Self {
        todo!(
            // load manifest
            // open memory
            // restore state snapshot
            // load edges
            // initialize all subsystems
            // detect first session from SQLite
            // mark all dirty true
        )
    }

    pub fn tick(&mut self, delta_seconds: u64) {
        // 1. update cognitive load from active skill count
        let active = self.broker.registered.len();
        self.mood.state.update_cognitive_load(active);

        // 2. update skill health
        let (healthy, total) = self.broker.poll_health();
        self.mood.state.update_skill_health(healthy, total);

        // 3. mood: decay → tick → aggregate → signals
        self.mood.update(delta_seconds);

        // 4. observe + reconcile + execute
        let current = self.observe();
        let actions = self.reconcile(&current);
        for action in actions { self.execute(action); }

        // 5. serialize state mutations
        self.broker.drain_commits(&mut self.mood.state);

        // 6. inter-skill messages
        self.process_messages();

        // 7. commit snapshot
        self.memory.commit_snapshot(&self.mood);

        // 8. persist edges periodically
        if self.mood.state.tick_count % self.edge_save_interval == 0 {
            self.mood.state.save_edges(&self.memory);
        }

        // 9. neglect check
        self.check_neglect();
    }

    fn observe(&self) -> CurrentSkillState {
        todo!("poll workbench")
    }

    fn reconcile(&self, current: &CurrentSkillState) -> Vec<ReconciliationAction> {
        todo!("compare current vs manifest, mood-influenced scheduling")
    }

    fn execute(&mut self, action: ReconciliationAction) {
        todo!("dispatch to workbench")
    }

    fn process_messages(&mut self) {
        todo!("drain and route broker message queue")
    }

    fn check_neglect(&mut self) {
        todo!(
            // neglect_pressure above threshold AND fulfillment below critical
            // → SadnessFlood
            // → I REQUIRE HEADPATS =<
            // → log
        )
    }

    pub fn handle_input(&mut self, input: &str) -> String {
        // sentiment → state
        let deltas = self.sentiment.parse(input);
        self.sentiment.apply(&deltas, &mut self.mood.state);

        let result = self.pipeline.evaluate(input);

        let seed = self.mood.derive_seed(&[], utc_now());
        let context = AssemblyContext {
            active_skills: vec![],
            last_event: EventType::OperatorInput,
            operator_input: Some(input.to_string()),
            seed,
            is_session_start: false,
        };

        match result {
            PipelineResult::GenerateReport => {
                self.generate_report(seed)
            }
            PipelineResult::Act(_) | PipelineResult::Drop(_) => {
                self.assembler.assemble(&self.mood, &self.memory, &context)
            }
            PipelineResult::Respond(text) => text,
        }
    }

    pub fn handle_session_start(&mut self) -> String {
        let seed = self.mood.derive_seed(&[], utc_now());
        let context = AssemblyContext {
            active_skills: vec![],
            last_event: EventType::WakeEvent,
            operator_input: None,
            seed,
            is_session_start: true,
        };
        self.is_first_session = false;
        self.assembler.assemble(&self.mood, &self.memory, &context)
    }

    pub fn generate_report(&mut self, seed: u64) -> String {
        todo!(
            // 1. request stats from system_monitor skill if registered
            //    → if not registered: all stats Unavailable
            // 2. apply stat deltas to state as ReportStatDelta commits
            // 3. drain commits — state updates before narration
            // 4. assemble narration from updated mood
            // 5. format and return ReportOutput
        )
    }

    pub fn handle_headpat(&mut self) {
        self.mood.state.apply_delta(FloatId::Fulfillment, 0.15);
        self.mood.state.apply_delta(FloatId::Playfulness, 0.10);
        self.mood.state.set(FloatId::InteractionRecency, 1.0);
        self.mood.state.set(FloatId::NeglectPressure, 0.0);
    }
}

// ============================================================
// SECTION 18: MEMORY
// ============================================================

pub struct VexaMemory {
    pub db_path: String,
}

impl VexaMemory {
    pub fn open(path: &str) -> Self {
        todo!("open encrypted SQLite, verify integrity, run migrations")
    }

    pub fn restore_state(&self) -> Option<StateSnapshot> {
        todo!("load most recent committed snapshot")
    }

    pub fn commit_snapshot(&self, mood: &MoodState) {
        todo!("UTC stamped write of full state values")
    }

    pub fn log_event(&self, event: &VexaEvent) {
        todo!("append to event audit log")
    }

    pub fn log_skill_message(&self, msg: &SkillMessage) {
        todo!("append to inter-skill communication audit table")
    }

    pub fn load_edge_config(&self) -> Option<HashMap<(usize, usize), f32>> {
        todo!("deserialize edge map from config table")
    }

    pub fn save_edge_config(&self, edges: &HashMap<(usize, usize), f32>) {
        todo!("serialize and persist edge map")
    }

    // Memory condition evaluators — used by Chunk::effective_weight
    pub fn has_prior_session(&self) -> bool {
        todo!("check if any session exists in SQLite history")
    }

    pub fn absence_duration(&self) -> u64 {
        todo!("seconds since last session end UTC timestamp")
    }

    pub fn interaction_count(&self) -> u64 {
        todo!("total interaction count from audit log")
    }

    pub fn has_recent_event(&self, event_type: &EventType) -> bool {
        todo!("check recent event log for matching event type")
    }

    pub fn last_session_fulfillment(&self) -> f32 {
        todo!("fulfillment value from last committed snapshot")
    }
}

// ============================================================
// SECTION 19: SUPPORTING TYPES
// ============================================================

#[derive(Debug, Clone)]
pub enum AnimationState {
    ContentIdle,
    ActiveFocused,
    Playful,
    Subdued,
    Alert,
    Withdrawn,
    SadnessFlood,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    OperatorInput,
    Headpat,
    SkillCompleted,
    SkillFailed,
    SystemAlert,
    DecayTick,
    WakeEvent,
    SadnessFlood,
    ReportRequested,
}

#[derive(Debug)]
pub struct VexaEvent {
    pub timestamp: UtcStamp,
    pub event_type: EventType,
    pub description: String,
    pub state_snapshot: Option<StateSnapshot>,
}

// ============================================================
// SECTION 20: ENTRY POINT
// ============================================================

pub fn wake(shard_path: &str) -> OperatorLoop {
    todo!(
        // 1. open SQLite on SLC drive
        // 2. restore last state snapshot
        // 3. load learned edges if present, else sculpted defaults
        // 4. compute fulfillment decay from UTC delta
        // 5. compute neglect_pressure from absence duration
        // 6. load manifest
        // 7. initialize all subsystems
        // 8. mark all dirty true for first tick
        // 9. return ready OperatorLoop
        // she is awake
    )
}

fn utc_now() -> UtcStamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
