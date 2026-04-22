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
