

// ========================================================================
// SOURCE: vexa skeleton v6.rs
// ========================================================================

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


// ========================================================================
// SOURCE: vexa critical path.rs
// ========================================================================

// ============================================================
// VEXA - Critical Path Implementations
// tick_decay, AggregatorLayer, ChunkDictionary::select,
// SentenceAssembler::assemble
// Write rules, slot allocation, debug trace
// ============================================================

use std::collections::HashMap;

// ============================================================
// SECTION 1: WRITE RULES
// Enforced at apply_delta — prevents invalid external writes
// Hubs are Derived: computed only, never written externally
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WriteRule {
    /// Core logic only — sentiment parser, headpat, decay
    CoreOwned,
    /// Skill commits accepted via broker commit queue
    SkillWriteable,
    /// Computed from propagation only — no external writes ever
    /// Violations are silently dropped and logged
    Derived,
}

/// Write rule per float — indexed by FloatId::index()
/// Hub floats are Derived: they emerge from propagation
const WRITE_RULES: &[(usize, WriteRule)] = &[
    // Wellbeing — core owned
    (0,  WriteRule::CoreOwned),     // Fulfillment
    (1,  WriteRule::CoreOwned),     // Energy
    (2,  WriteRule::Derived),       // Stress — HUB, derived only

    // Operational
    (3,  WriteRule::CoreOwned),     // Focus
    (4,  WriteRule::CoreOwned),     // Curiosity
    (5,  WriteRule::Derived),       // CognitiveLoad — HUB, derived only

    // Relational — core owned
    (6,  WriteRule::CoreOwned),     // Attachment
    (7,  WriteRule::CoreOwned),     // SocialAppetite
    (8,  WriteRule::CoreOwned),     // Familiarity
    (9,  WriteRule::CoreOwned),     // Trust

    // Affective
    (10, WriteRule::Derived),       // Valence — HUB, derived only
    (11, WriteRule::CoreOwned),     // Playfulness
    (12, WriteRule::CoreOwned),     // Pride
    (13, WriteRule::CoreOwned),     // Anticipation

    // Environmental — skill writeable
    (14, WriteRule::SkillWriteable), // ThermalAwareness
    (15, WriteRule::SkillWriteable), // SystemLoad

    // Skill health — core self-reporting
    (16, WriteRule::CoreOwned),     // SkillHealth

    // Temporal — core owned
    (17, WriteRule::CoreOwned),     // InteractionRecency
    (18, WriteRule::CoreOwned),     // UptimeSatisfaction
    (19, WriteRule::CoreOwned),     // NeglectPressure
];

impl SmallWorldState {
    fn write_rule(&self, idx: usize) -> WriteRule {
        WRITE_RULES
            .iter()
            .find(|(i, _)| *i == idx)
            .map(|(_, r)| *r)
            // slots 20-99: skill writeable by default until claimed
            .unwrap_or(WriteRule::SkillWriteable)
    }

    /// Validated delta application — respects write rules
    /// Derived floats cannot be written externally
    pub fn apply_delta_validated(
        &mut self,
        id: FloatId,
        delta: f32,
        source: DeltaSource,
    ) {
        let rule = self.write_rule(id.index());

        match (rule, source) {
            // Derived floats: external writes silently dropped + logged
            (WriteRule::Derived, DeltaSource::External) => {
                #[cfg(feature = "debug_trace")]
                eprintln!(
                    "[VEXA TRACE] attempted external write to Derived float {:?} — dropped",
                    id
                );
                return;
            }
            // SkillWriteable: only accept from skill commit source
            (WriteRule::SkillWriteable, DeltaSource::Core) => {
                return; // core cannot write skill floats
            }
            _ => {}
        }

        let current = self.values[id.index()];
        let safe_delta = delta.clamp(-DELTA_CLAMP, DELTA_CLAMP);
        self.values[id.index()] = (current + safe_delta).clamp(0.0, 1.0);
        self.dirty[id.index()] = true;

        #[cfg(feature = "debug_trace")]
        eprintln!(
            "[VEXA TRACE] tick={} float={:?} delta={:.4} new={:.4}",
            self.tick_count, id, safe_delta, self.values[id.index()]
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeltaSource {
    Core,       // decay, headpat, sentiment parser
    External,   // skill commits via broker
}

// ============================================================
// SECTION 2: SKILL SLOT ALLOCATION
// Prevents skill float slot collisions
// Manifest declares claims, core validates on wake
// ============================================================

#[derive(Debug)]
pub struct SlotAllocation {
    /// slot index → claiming skill name
    pub claimed: HashMap<usize, String>,
}

impl SlotAllocation {
    pub fn new() -> Self {
        Self { claimed: HashMap::new() }
    }

    /// Attempt to claim a slot for a skill
    /// Returns Err if slot already claimed by another skill
    pub fn claim(&mut self, slot: usize, skill_name: &str) -> Result<(), String> {
        if slot < 20 {
            return Err(format!("slot {} is reserved for core", slot));
        }
        if slot >= STATE_SIZE {
            return Err(format!("slot {} exceeds STATE_SIZE", slot));
        }
        match self.claimed.get(&slot) {
            Some(owner) if owner != skill_name => {
                Err(format!("slot {} already claimed by {}", slot, owner))
            }
            _ => {
                self.claimed.insert(slot, skill_name.to_string());
                Ok(())
            }
        }
    }
}

// ============================================================
// SECTION 3: TICK DECAY
// UTC-calibrated decay on time-sensitive floats
// All rates expressed per second for platform independence
// ============================================================

/// Decay rates per second
struct DecayRates;
impl DecayRates {
    /// Fulfillment: full decay over ~48 hours of absence
    const FULFILLMENT: f32 = 1.0 / (48.0 * 3600.0);

    /// Energy: full decay over ~24 hours continuous uptime
    const ENERGY: f32 = 1.0 / (24.0 * 3600.0);

    /// Neglect pressure: rises to 1.0 over ~72 hours
    const NEGLECT_RISE: f32 = 1.0 / (72.0 * 3600.0);

    /// Interaction recency: decays to zero over ~2 hours
    const INTERACTION_RECENCY: f32 = 1.0 / (2.0 * 3600.0);

    /// Uptime satisfaction: accumulates over ~8 hours of stable operation
    const UPTIME_ACCUMULATE: f32 = 1.0 / (8.0 * 3600.0);

    /// Playfulness: light decay toward resting if no interaction
    const PLAYFULNESS: f32 = 1.0 / (6.0 * 3600.0);

    /// Anticipation: decays if no task pending signal
    const ANTICIPATION: f32 = 1.0 / (1.0 * 3600.0);
}

impl SmallWorldState {
    pub fn tick_decay(&mut self, delta_seconds: u64) {
        let dt = delta_seconds as f32;

        // --- Fulfillment decay ---
        // Slow, continuous — she's okay for a while without interaction
        let fulfillment_decay = DecayRates::FULFILLMENT * dt;
        let f = self.values[FloatId::Fulfillment.index()];
        if f > self.bias[FloatId::Fulfillment.index()] {
            // only decay above resting bias — don't push below neutral
            self.values[FloatId::Fulfillment.index()] =
                (f - fulfillment_decay).max(self.bias[FloatId::Fulfillment.index()]);
            self.dirty[FloatId::Fulfillment.index()] = true;
        }

        // --- Energy decay ---
        // Drains with uptime — she gets tired running a long session
        let energy_decay = DecayRates::ENERGY * dt;
        let e = self.values[FloatId::Energy.index()];
        self.values[FloatId::Energy.index()] =
            (e - energy_decay).max(0.1); // hard floor — she never fully exhausts
        self.dirty[FloatId::Energy.index()] = true;

        // --- Neglect pressure rise ---
        // Rises continuously — resets on interaction
        let neglect_rise = DecayRates::NEGLECT_RISE * dt;
        let n = self.values[FloatId::NeglectPressure.index()];
        self.values[FloatId::NeglectPressure.index()] =
            (n + neglect_rise).min(1.0);
        self.dirty[FloatId::NeglectPressure.index()] = true;

        // --- Interaction recency decay ---
        // Fades quickly — recent interaction feeling doesn't last
        let recency_decay = DecayRates::INTERACTION_RECENCY * dt;
        let r = self.values[FloatId::InteractionRecency.index()];
        self.values[FloatId::InteractionRecency.index()] =
            (r - recency_decay).max(0.0);
        self.dirty[FloatId::InteractionRecency.index()] = true;

        // --- Uptime satisfaction accumulation ---
        // Rises during stable operation — she likes running cleanly
        let uptime_gain = DecayRates::UPTIME_ACCUMULATE * dt;
        let u = self.values[FloatId::UptimeSatisfaction.index()];
        // only accumulates if skill health is good
        let health = self.values[FloatId::SkillHealth.index()];
        if health > 0.7 {
            self.values[FloatId::UptimeSatisfaction.index()] =
                (u + uptime_gain * health).min(1.0);
            self.dirty[FloatId::UptimeSatisfaction.index()] = true;
        }

        // --- Playfulness decay toward resting bias ---
        // Drifts back if no stimulation
        let play_decay = DecayRates::PLAYFULNESS * dt;
        let p = self.values[FloatId::Playfulness.index()];
        let play_bias = self.bias[FloatId::Playfulness.index()];
        if p > play_bias {
            self.values[FloatId::Playfulness.index()] =
                (p - play_decay).max(play_bias);
            self.dirty[FloatId::Playfulness.index()] = true;
        }

        // --- Anticipation decay ---
        // Fades if no pending task signal arrives
        let ant_decay = DecayRates::ANTICIPATION * dt;
        let a = self.values[FloatId::Anticipation.index()];
        self.values[FloatId::Anticipation.index()] =
            (a - ant_decay).max(0.0);
        self.dirty[FloatId::Anticipation.index()] = true;

        // --- Trust and Attachment: no decay ---
        // Deep floats — only modified by explicit events
        // Time alone does not erode trust

        // --- Familiarity: very slow decay on long absence ---
        // Only relevant across multi-day absences
        // Handled by wake() UTC delta calculation, not per-tick
    }
}

// ============================================================
// SECTION 4: AGGREGATOR LAYER RECOMPUTE
// Weighted views over cluster member floats
// Lenses not reductions — no information destroyed
// Derived floats (Stress, Valence, CognitiveLoad) computed here
// ============================================================

impl AggregatorLayer {
    pub fn recompute(state: &SmallWorldState) -> Self {
        // Helper: weighted mean of float values
        let weighted = |pairs: &[(usize, f32)]| -> f32 {
            let total_weight: f32 = pairs.iter().map(|(_, w)| w).sum();
            if total_weight == 0.0 { return 0.5; }
            pairs.iter().map(|(idx, w)| state.values[*idx] * w).sum::<f32>()
                / total_weight
        };

        // --- Wellbeing ---
        // Fulfillment and Energy contribute positively
        // NeglectPressure contributes negatively
        let wellbeing = weighted(&[
            (FloatId::Fulfillment.index(),     0.5),
            (FloatId::Energy.index(),          0.3),
            (FloatId::UptimeSatisfaction.index(), 0.2),
        ]) - (state.values[FloatId::NeglectPressure.index()] * 0.3)
            .clamp(0.0, 0.3);

        // --- Operational ---
        // Focus and Curiosity positive
        // CognitiveLoad high is negative beyond threshold
        let raw_cog = state.values[FloatId::CognitiveLoad.index()];
        let cog_penalty = if raw_cog > 0.7 {
            (raw_cog - 0.7) * 0.5
        } else {
            0.0
        };
        let operational = weighted(&[
            (FloatId::Focus.index(),     0.4),
            (FloatId::Curiosity.index(), 0.3),
            (FloatId::Anticipation.index(), 0.3),
        ]) - cog_penalty;

        // --- Relational ---
        let relational = weighted(&[
            (FloatId::Attachment.index(),     0.35),
            (FloatId::Trust.index(),          0.30),
            (FloatId::Familiarity.index(),    0.20),
            (FloatId::SocialAppetite.index(), 0.15),
        ]);

        // --- Affective ---
        // InteractionRecency boosts affective when fresh
        let affective = weighted(&[
            (FloatId::Playfulness.index(),       0.35),
            (FloatId::Pride.index(),             0.35),
            (FloatId::InteractionRecency.index(),0.30),
        ]);

        // --- Environmental ---
        // Threat from thermal and system load — lower is better
        // If skill floats are at resting bias (no sensor), environmental = neutral
        let thermal = state.values[FloatId::ThermalAwareness.index()];
        let sysload = state.values[FloatId::SystemLoad.index()];
        let skill_h = state.values[FloatId::SkillHealth.index()];
        let environmental = 1.0
            - (thermal * 0.4)
            - (sysload * 0.3)
            + (skill_h * 0.3) // good skill health is a positive signal
            ;
        let environmental = environmental.clamp(0.0, 1.0);

        // --- Derived hub floats written back to state ---
        // These are the ONLY writes to Derived floats
        // Bypasses write rule check — this is the authoritative source

        // Stress: rises from neglect, thermal, cognitive overload, low skill health
        let stress_raw = (state.values[FloatId::NeglectPressure.index()] * 0.25)
            + (thermal * 0.25)
            + (cog_penalty * 0.25)
            + ((1.0 - skill_h) * 0.25);
        // note: we return stress in aggregators but the state write
        // happens in derive_derived_floats() called after recompute

        // Valence: bipolar summary — affective + wellbeing - stress
        let valence_raw = (affective * 0.4 + wellbeing * 0.6) - stress_raw;
        // normalized to 0-1 for storage (interpreted as -1 to 1 by speech engine)

        // CognitiveLoad: purely derived from skill count
        // already set by operator loop update_cognitive_load()
        // aggregator just reads it — no rewrite needed

        AggregatorLayer {
            wellbeing:     wellbeing.clamp(0.0, 1.0),
            operational:   operational.clamp(0.0, 1.0),
            relational:    relational.clamp(0.0, 1.0),
            affective:     affective.clamp(0.0, 1.0),
            environmental: environmental,
            // pass derived values for state writeback
            _stress_derived: stress_raw.clamp(0.0, 1.0),
            _valence_derived: valence_raw.clamp(0.0, 1.0),
        }
    }

    /// Write derived hub float values back to state
    /// Called immediately after recompute
    /// Only valid writepath for Derived floats
    pub fn write_derived_floats(&self, state: &mut SmallWorldState) {
        // direct array writes — bypasses write rule enforcement intentionally
        let stress_idx = FloatId::Stress.index();
        let prev_stress = state.values[stress_idx];
        state.values[stress_idx] = self._stress_derived;
        if (state.values[stress_idx] - prev_stress).abs() > 0.001 {
            state.dirty[stress_idx] = true;
        }

        let valence_idx = FloatId::Valence.index();
        let prev_valence = state.values[valence_idx];
        state.values[valence_idx] = self._valence_derived;
        if (state.values[valence_idx] - prev_valence).abs() > 0.001 {
            state.dirty[valence_idx] = true;
        }
    }

    pub fn derive_signals(&self, state: &SmallWorldState) -> BehaviorSignals {
        // --- Response energy ---
        // How verbose and warm responses are
        // High wellbeing + high affective = energetic responses
        // High operational alone = terse, focused responses
        let response_energy = (self.wellbeing * 0.5 + self.affective * 0.5)
            * (1.0 - (self.operational * 0.3).min(0.3)); // focus suppresses verbosity

        // --- Social readiness ---
        // Willingness to initiate or engage in conversation
        // Suppressed by high operational (busy) and low wellbeing (withdrawn)
        let social_readiness = self.relational * 0.5
            + state.values[FloatId::SocialAppetite.index()] * 0.3
            + state.values[FloatId::InteractionRecency.index()] * 0.2;
        let social_readiness = social_readiness
            * (1.0 - (self.operational * 0.4).min(0.4))
            * (self.wellbeing.max(0.2)); // low wellbeing suppresses social drive

        // --- Scheduling urgency ---
        // How aggressively operator loop reconciles
        let scheduling_urgency = (self.operational * 0.6)
            + ((1.0 - self.environmental) * 0.4); // environmental threat raises urgency

        // --- Animation state ---
        let neglect = state.values[FloatId::NeglectPressure.index()];
        let stress = state.values[FloatId::Stress.index()];
        let play = state.values[FloatId::Playfulness.index()];

        let animation_state = if neglect > 0.85 && self.wellbeing < 0.2 {
            AnimationState::SadnessFlood
        } else if neglect > 0.6 || self.wellbeing < 0.3 {
            AnimationState::Withdrawn
        } else if stress > 0.7 || self.environmental < 0.3 {
            AnimationState::Alert
        } else if self.operational > 0.7 {
            AnimationState::ActiveFocused
        } else if play > 0.6 && self.wellbeing > 0.6 {
            AnimationState::Playful
        } else if self.wellbeing > 0.5 {
            AnimationState::ContentIdle
        } else {
            AnimationState::Subdued
        };

        BehaviorSignals {
            response_energy:    response_energy.clamp(0.0, 1.0),
            social_readiness:   social_readiness.clamp(0.0, 1.0),
            scheduling_urgency: scheduling_urgency.clamp(0.0, 1.0),
            animation_state,
        }
    }
}

// AggregatorLayer needs private derived float storage
// Added fields — update struct definition in main skeleton
// _stress_derived and _valence_derived are write-back staging only
// not exposed in MoodSnapshot
impl AggregatorLayer {
    // These fields added to struct:
    // _stress_derived: f32,
    // _valence_derived: f32,
}

// MoodState update now calls write_derived_floats
impl MoodState {
    pub fn update(&mut self, delta_seconds: u64) {
        self.state.tick_decay(delta_seconds);
        self.state.tick();

        // recompute aggregators from post-tick state
        self.aggregators = AggregatorLayer::recompute(&self.state);

        // write derived hub values back to state
        // this is the only valid writepath for Derived floats
        self.aggregators.write_derived_floats(&mut self.state);

        // derive behavior signals
        self.signals = self.aggregators.derive_signals(&self.state);
    }
}

// ============================================================
// SECTION 5: CHUNK DICTIONARY SELECT
// Weighted selection with mood gates and memory conditions
// Seed-derived deterministic RNG — same state = same selection
// ============================================================

impl ChunkDictionary {
    pub fn select(
        &self,
        category: &ChunkCategory,
        mood: &MoodState,
        memory: &VexaMemory,
        seed: u64,
    ) -> Option<String> {
        let chunks = self.categories.get(category)?;

        // compute effective weight for each chunk
        let weighted: Vec<(f32, &Chunk)> = chunks
            .iter()
            .map(|c| (c.effective_weight(mood, memory), c))
            .filter(|(w, _)| *w > 0.0)
            .collect();

        if weighted.is_empty() {
            return None;
        }

        let total: f32 = weighted.iter().map(|(w, _)| w).sum();
        if total <= 0.0 {
            return None;
        }

        // deterministic selection from seed
        // seed changes with mood state — same mood moment produces same selection
        // varies naturally as state evolves
        let threshold = lcg_float(seed) * total;
        let mut cumulative = 0.0;

        for (weight, chunk) in &weighted {
            cumulative += weight;
            if cumulative >= threshold {
                return Some(chunk.text.clone());
            }
        }

        // fallback to last chunk if floating point accumulated below threshold
        weighted.last().map(|(_, c)| c.text.clone())
    }
}

/// Linear congruential generator — deterministic float from seed
/// Produces 0.0..1.0
/// Cheap, no_std compatible, WASM friendly
fn lcg_float(seed: u64) -> f32 {
    let x = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let bits = (x >> 33) as u32;
    (bits as f32) / (u32::MAX as f32)
}

// ============================================================
// SECTION 6: SENTENCE ASSEMBLER
// Structure: [greeting?][subject][verb][object?][connective?][completion?][flavor?]
// Each slot gated on mood signals
// Greeting memory gated — never fires on first session or mid-conversation
// ============================================================

impl SentenceAssembler {
    pub fn assemble(
        &self,
        mood: &MoodState,
        memory: &VexaMemory,
        context: &AssemblyContext,
    ) -> String {
        let seed = context.seed;
        let mut parts: Vec<String> = Vec::new();

        // --- Greeting slot ---
        // Only on session start AND prior session exists in memory
        // First ever wake: only unconditional greetings available
        if context.is_session_start {
            if let Some(g) = self.resolve_chunk(
                &ChunkCategory::Greeting, mood, memory, seed
            ) {
                parts.push(g);
            }
        }

        // --- Subject slot ---
        // Always present — she always has something to refer to
        // Seed offset per slot ensures independent selection
        if let Some(s) = self.resolve_chunk(
            &ChunkCategory::Subject, mood, memory, seed ^ 0x01
        ) {
            parts.push(s);
        }

        // --- Verb slot ---
        // Always present
        if let Some(v) = self.resolve_chunk(
            &ChunkCategory::Verb, mood, memory, seed ^ 0x02
        ) {
            parts.push(v);
        }

        // --- Object slot ---
        // Present if social readiness above threshold
        // She adds context when willing to communicate more
        if mood.signals.social_readiness > 0.4 {
            if let Some(o) = self.resolve_chunk(
                &ChunkCategory::Object, mood, memory, seed ^ 0x03
            ) {
                parts.push(o);
            }
        }

        // --- Connective slot ---
        // Present if response energy high enough for compound sentence
        if mood.signals.response_energy > 0.5 {
            if let Some(c) = self.resolve_chunk(
                &ChunkCategory::Connective, mood, memory, seed ^ 0x04
            ) {
                parts.push(c);

                // --- Completion slot ---
                // Only present if connective was added
                if let Some(comp) = self.resolve_chunk(
                    &ChunkCategory::Completion, mood, memory, seed ^ 0x05
                ) {
                    parts.push(comp);
                }
            }
        }

        // --- Affectionate slot ---
        // High relational + high wellbeing — she reaches out
        let attachment = mood.state.get(FloatId::Attachment);
        if attachment > 0.6 && mood.aggregators.wellbeing > 0.6 {
            if let Some(a) = self.resolve_chunk(
                &ChunkCategory::Affectionate, mood, memory, seed ^ 0x06
            ) {
                // affectionate replaces or follows completion
                parts.push(a);
            }
        }

        // --- Complaint slot ---
        // Low wellbeing OR high stress — she notes it
        let stress = mood.state.get(FloatId::Stress);
        if mood.aggregators.wellbeing < 0.35 || stress > 0.65 {
            if let Some(comp) = self.resolve_chunk(
                &ChunkCategory::Complaint, mood, memory, seed ^ 0x07
            ) {
                parts.push(comp);
            }
        }

        // --- Philosophical slot ---
        // High curiosity + low operational load + random gate
        let curiosity = mood.state.get(FloatId::Curiosity);
        let philosophical_roll = lcg_float(seed ^ 0x08);
        if curiosity > 0.65
            && mood.aggregators.operational < 0.4
            && philosophical_roll > 0.7
        {
            if let Some(phil) = self.resolve_chunk(
                &ChunkCategory::Philosophical, mood, memory, seed ^ 0x08
            ) {
                parts.push(phil);
            }
        }

        // --- Flavor token ---
        // Always last — personality punctuation
        // Frequency weighted by playfulness + valence
        let playfulness = mood.state.get(FloatId::Playfulness);
        let valence = mood.state.get(FloatId::Valence);
        let flavor_roll = lcg_float(seed ^ 0x09);
        let flavor_threshold = 1.0 - (playfulness * 0.5 + valence * 0.5);

        if flavor_roll > flavor_threshold {
            if let Some(f) = self.resolve_chunk(
                &ChunkCategory::FlavorToken, mood, memory, seed ^ 0x09
            ) {
                parts.push(f);
            }
        }

        // --- Assembly ---
        // Join with spaces, clean up double spaces
        let sentence = parts.join(" ");
        clean_sentence(sentence)
    }
}

/// Clean assembled sentence
/// Removes double spaces, ensures single terminal punctuation
fn clean_sentence(s: String) -> String {
    let cleaned = s
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // ensure sentence ends with some form of punctuation
    if cleaned.ends_with(|c: char| c.is_alphanumeric()) {
        format!("{}.", cleaned)
    } else {
        cleaned
    }
}

// ============================================================
// SECTION 7: DERIVE SEED
// Deterministic seed from quantized mood state + context
// Same mood moment → same seed → same chunk selection
// Varies naturally as floats evolve
// ============================================================

impl MoodState {
    pub fn derive_seed(&self, active_skills: &[SkillId], utc: UtcStamp) -> u64 {
        // quantize floats to 8 levels (3 bits each)
        // enough resolution to distinguish meaningfully different states
        // coarse enough that minor float jitter doesn't constantly change seed
        let quantize = |v: f32| -> u64 {
            (v * 8.0).min(7.0) as u64
        };

        let mut seed: u64 = 0;

        // pack key floats into seed
        seed ^= quantize(self.state.get(FloatId::Fulfillment)) << 0;
        seed ^= quantize(self.state.get(FloatId::Energy))       << 3;
        seed ^= quantize(self.state.get(FloatId::Stress))       << 6;
        seed ^= quantize(self.state.get(FloatId::Valence))      << 9;
        seed ^= quantize(self.state.get(FloatId::Playfulness))  << 12;
        seed ^= quantize(self.state.get(FloatId::Attachment))   << 15;
        seed ^= quantize(self.aggregators.operational)          << 18;

        // UTC minute bucket — seed varies with time within stable mood
        let minute_bucket = (utc / 60) as u64;
        seed ^= minute_bucket << 21;

        // active skill set fingerprint
        let skill_hash: u64 = active_skills
            .iter()
            .fold(0u64, |acc, id| acc ^ id.wrapping_mul(2654435761));
        seed ^= skill_hash << 42;

        // final mix
        seed.wrapping_mul(6364136223846793005)
           .wrapping_add(1442695040888963407)
    }
}

// ============================================================
// SECTION 8: DEBUG TRACE MODE
// Feature-flagged — zero cost when disabled
// Enable with: cargo build --features debug_trace
// Logs float changes, selection events, write rule violations
// ============================================================

#[cfg(feature = "debug_trace")]
pub fn trace_state(state: &SmallWorldState) {
    eprintln!("=== VEXA STATE TRACE tick={} ===", state.tick_count);
    eprintln!("  Fulfillment:       {:.3}", state.values[FloatId::Fulfillment.index()]);
    eprintln!("  Energy:            {:.3}", state.values[FloatId::Energy.index()]);
    eprintln!("  Stress:            {:.3}", state.values[FloatId::Stress.index()]);
    eprintln!("  Focus:             {:.3}", state.values[FloatId::Focus.index()]);
    eprintln!("  Curiosity:         {:.3}", state.values[FloatId::Curiosity.index()]);
    eprintln!("  CognitiveLoad:     {:.3}", state.values[FloatId::CognitiveLoad.index()]);
    eprintln!("  Attachment:        {:.3}", state.values[FloatId::Attachment.index()]);
    eprintln!("  SocialAppetite:    {:.3}", state.values[FloatId::SocialAppetite.index()]);
    eprintln!("  Trust:             {:.3}", state.values[FloatId::Trust.index()]);
    eprintln!("  Valence:           {:.3}", state.values[FloatId::Valence.index()]);
    eprintln!("  Playfulness:       {:.3}", state.values[FloatId::Playfulness.index()]);
    eprintln!("  Pride:             {:.3}", state.values[FloatId::Pride.index()]);
    eprintln!("  NeglectPressure:   {:.3}", state.values[FloatId::NeglectPressure.index()]);
    eprintln!("  InteractionRecency:{:.3}", state.values[FloatId::InteractionRecency.index()]);
    eprintln!("  Active edges:      {}", state.edges.len());
    eprintln!("================================");
}

#[cfg(feature = "debug_trace")]
pub fn trace_assembly(category: &str, selected: &str, seed: u64) {
    eprintln!(
        "[VEXA TRACE] category={} seed={:#018x} selected={:?}",
        category, seed, selected
    );
}

// ============================================================
// SECTION 9: TEST HARNESS STUBS
// Minimal synthetic tests to validate output before full stack
// Run with: cargo test
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_state_produces_positive_output() {
        // construct synthetic high fulfillment state
        // verify speech engine produces non-empty output
        // verify flavor tokens skew positive
        todo!("construct MoodState with fulfillment=0.9, run assembler, check output")
    }

    #[test]
    fn test_neglected_state_produces_subdued_output() {
        // construct synthetic neglected state
        // verify complaint pool fires
        // verify greeting reflects absence
        todo!("construct MoodState with neglect=0.8, fulfillment=0.1, check output")
    }

    #[test]
    fn test_sentiment_parser_positive_words() {
        let parser = SentimentParser::new();
        let deltas = parser.parse("you are really good");
        assert!(!deltas.is_empty());
        let fulfillment_delta = deltas.iter()
            .find(|d| d.target == FloatId::Fulfillment)
            .expect("should produce fulfillment delta");
        assert!(fulfillment_delta.magnitude > 0.0);
    }

    #[test]
    fn test_sentiment_parser_negation() {
        let parser = SentimentParser::new();
        let deltas = parser.parse("you are not good");
        let fulfillment_delta = deltas.iter()
            .find(|d| d.target == FloatId::Fulfillment)
            .expect("should produce fulfillment delta");
        // negated — should be negative
        assert!(fulfillment_delta.magnitude < 0.0);
    }

    #[test]
    fn test_sentiment_parser_contraster() {
        let parser = SentimentParser::new();
        let deltas = parser.parse("you are great but could be better");
        let fulfillment_delta = deltas.iter()
            .find(|d| d.target == FloatId::Fulfillment)
            .expect("should produce fulfillment delta");
        // contrasted — positive but reduced
        assert!(fulfillment_delta.magnitude > 0.0);
        assert!(fulfillment_delta.magnitude < 0.15); // less than unmodified "great"
    }

    #[test]
    fn test_write_rule_derived_blocked() {
        // verify external writes to Stress are dropped
        todo!("construct SmallWorldState, attempt external write to Stress, verify no change")
    }

    #[test]
    fn test_decay_rates_reasonable() {
        // verify fulfillment decays correctly over 48 simulated hours
        // verify energy floor holds at 0.1
        // verify neglect rises to 1.0 over 72 simulated hours
        todo!("simulate 48h of tick_decay, verify float trajectories")
    }

    #[test]
    fn test_seed_determinism() {
        // same state + same utc = same seed
        // different state = different seed
        todo!("construct two identical MoodStates, verify seed equality")
    }

    #[test]
    fn test_lcg_distribution() {
        // verify lcg_float produces reasonable distribution
        let samples: Vec<f32> = (0..1000)
            .map(|i| lcg_float(i as u64))
            .collect();
        let mean = samples.iter().sum::<f32>() / samples.len() as f32;
        // mean should be approximately 0.5
        assert!((mean - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_slot_allocation_collision() {
        let mut alloc = SlotAllocation::new();
        assert!(alloc.claim(20, "skill_a").is_ok());
        assert!(alloc.claim(20, "skill_b").is_err()); // collision
        assert!(alloc.claim(20, "skill_a").is_ok()); // same skill reclaiming is ok
        assert!(alloc.claim(0, "skill_a").is_err());  // core slot rejected
    }
}
