// src/app/typing_animation.rs
#![allow(unused_imports, dead_code)] // Allow unused imports and dead code for now

// --- Imports ---
use super::YewComponent; // Assumes YewComponent trait is defined in src/app/mod.rs or src/app.rs
use gloo_timers::callback::Timeout;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;

// --- Core Structs ---

/// Transformium-inspired animation mimicking fluid metal transformation.
/// Note: Use of Rc<RefCell> may need formal verification for concurrent access in safety-critical contexts.
pub struct TypingAnimation {
    // Particle system state. Consider fixed-size arenas for heap usage bounds (RLOC-1).
    particles: Rc<RefCell<Vec<Particle>>>,
    connections: Rc<RefCell<Vec<Connection>>>,
    target_text: String,
    // Animation state machine. Could be modeled for SEER verification (RLOC-4).
    phase: Rc<RefCell<AnimationPhase>>,
    // Progress counter. Ensure updates are temporally predictable (RLOC-2).
    progress: Rc<RefCell<f32>>,
    is_complete: Rc<RefCell<bool>>,
    // Shared state. Access patterns need analysis for data races if concurrency increases.
    gravity_center: Rc<RefCell<(f32, f32)>>,
    color_scheme: Rc<RefCell<ColorScheme>>,
    // Add RNG instance to avoid repeated calls to thread_rng()
    rng: Rc<RefCell<ThreadRng>>,
}

/// Connection between particles for the metallic mesh effect.
#[derive(Clone)]
struct Connection {
    particle1_idx: usize,
    particle2_idx: usize,
    strength: f32, // Consider fixed-point arithmetic for deterministic calculations.
    opacity: f32,
    active: bool,
}

/// Color scheme for the animation.
#[derive(Clone)]
struct ColorScheme {
    primary: String,
    secondary: String,
    accent: String,
    background: String, // Use validated color formats.
}

/// A single particle in the transformation effect.
/// State updates should be bounded.
#[derive(Clone)]
struct Particle {
    x: f32,
    y: f32,
    z: f32, // Position. Consider radiation-hardened data types (RLOC-3).
    target_x: f32,
    target_y: f32,
    target_z: f32, // Target position.
    symbol: char, // Visual representation.
    target_symbol: char, // Final character.
    vx: f32,
    vy: f32,
    vz: f32, // Velocity. Bounded updates are critical (RLOC-2).
    opacity: f32,
    scale: f32,
    rotation: f32, // Visual properties.
    is_text: bool, // Flag for core text particles.
    particle_type: ParticleType, // Behavior type.
    energy: f32, // Affects behavior, ensure predictable energy levels.
}

// --- Enums ---

/// Different types of particles for varied behaviors.
#[derive(Clone, Copy, PartialEq, Debug)]
enum ParticleType {
    Core,
    Orbiter,
    Swarm,
    Fragment,
    Connector,
}

/// Animation phases with sub-phases for complex transitions.
/// State machine logic should be verifiable (RLOC-4).
#[derive(Clone, Debug)] // Added Debug for easier inspection if needed
enum AnimationPhase {
    Scatter(ScatterSubPhase),
    Converge(ConvergeSubPhase),
    Stable(StableSubPhase),
    Dissolve(DissolveSubPhase),
}

/// Sub-phases for Scatter. Ensure bounded transition times (RLOC-2).
#[derive(Clone, Copy, PartialEq, Debug)]
enum ScatterSubPhase {
    Initial,
    Expansion,
    Contraction,
    PreConverge,
}

/// Sub-phases for Converge.
#[derive(Clone, Copy, PartialEq, Debug)]
enum ConvergeSubPhase {
    Alignment,
    Formation,
    Refinement,
    Solidification,
}

/// Sub-phases for Stable.
#[derive(Clone, Copy, PartialEq, Debug)]
enum StableSubPhase {
    Pulse,
    Orbit,
    Ripple,
    PreDissolve,
}

/// Sub-phases for Dissolve.
#[derive(Clone, Copy, PartialEq, Debug)]
enum DissolveSubPhase {
    Fracture,
    Explosion,
    Dispersion,
    Fade,
}

/// Messages for component updates. Handlers should have bounded execution time (RLOC-2).
pub enum Msg {
    Tick, // Main animation loop trigger.
    Reset, // Reset animation state.
    AdvancePhase, // Move to the next main animation phase.
    // AdvanceSubPhase message removed, logic integrated into Tick
    ShiftGravityCenter, // Update the swarm behavior center.
    ChangeColorScheme, // Update visual colors.
}

// --- Implementations ---

// Implementation for the YewComponent trait (assuming defined elsewhere in `super`)
impl YewComponent for TypingAnimation {
    /// Renders the component state to HTML. Ensure rendering is deterministic and bounded.
    fn render(&self) -> Html {
        html! {
            // Use a background color from the scheme, applied directly to the field
            <div class="transformium-field" style={format!("background-color: {}; transform-style: preserve-3d;", self.color_scheme.borrow().background)}>
                { self.render_connections() }
                { self.render_particles() }
            </div>
        }
    }

    /// Creates the initial component state. Use certified allocators if heap is needed (RLOC-1).
    fn create_component() -> Self {
        let target_text = ".unwrap()"; // Example text
        let mut rng = rand::rng(); // Create RNG once for initialization
        // RLOC-1: Consider static allocation or certified arenas instead of Vec for particles/connections.
        // RLOC-2: Ensure particle count has a verifiable upper bound.
        let particles = Self::initialize_particles(target_text, 250, &mut rng); // Pass RNG
        let connections = Self::initialize_connections(&particles, &mut rng); // Pass RNG

        let color_scheme = ColorScheme {
            primary: "#66d9ef".to_string(),    // Light Blue/Cyan
            secondary: "#a6e22e".to_string(),  // Lime Green
            accent: "#f92672".to_string(),     // Pink/Magenta
            background: "rgba(39, 40, 34, 0.0)".to_string(), // Fully transparent background for the field itself
        };

        TypingAnimation {
            particles: Rc::new(RefCell::new(particles)),
            connections: Rc::new(RefCell::new(connections)),
            target_text: target_text.to_string(),
            phase: Rc::new(RefCell::new(AnimationPhase::Scatter(
                ScatterSubPhase::Initial,
            ))),
            progress: Rc::new(RefCell::new(0.0)),
            is_complete: Rc::new(RefCell::new(false)),
            gravity_center: Rc::new(RefCell::new((150.0, 75.0))), // Initial center
            color_scheme: Rc::new(RefCell::new(color_scheme)),
            rng: Rc::new(RefCell::new(rng)), // Store RNG
        }
    }
}

// Standard Yew Component implementation
impl Component for TypingAnimation {
    type Message = Msg;
    type Properties = ();

    /// Component creation hook. Schedules initial timers.
    fn create(ctx: &Context<Self>) -> Self {
        let component = Self::create_component();
        // RLOC-2: Ensure timer callbacks have bounded execution.
        component.schedule_tick(ctx);
        component.schedule_gravity_shift(ctx);
        component.schedule_color_shift(ctx);
        component
    }

    /// Handles messages to update component state. Critical section for state updates.
    /// RLOC-1/RLOC-4: RefCell borrowing rules prevent data races at compile time, but runtime panics
    /// are possible if borrowing rules are violated. Formal verification might be needed if complex
    /// concurrent updates occur. Cloning large Vecs can impact performance and predictability.
    /// Borrow checker issues addressed by avoiding simultaneous mutable/immutable borrows in loops.
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                if *self.is_complete.borrow() {
                    return false;
                } // Stop ticking if complete

                // Clone phase and progress to avoid nested borrows within match arms.
                let phase = self.phase.borrow().clone();
                let mut progress = self.progress.borrow_mut();
                let gravity_center = *self.gravity_center.borrow();

                // Update progress - ensure rate is bounded.
                *progress += 0.02; // Consider time-delta based updates for consistency.

                // --- State Update Section ---
                // Get mutable references. Cloning avoided to improve performance and reduce potential heap churn.
                let mut particles_ref = self.particles.borrow_mut();
                let mut connections_ref = self.connections.borrow_mut();
                let mut rng = self.rng.borrow_mut(); // Borrow RNG

                // Update connection properties based on current phase.
                self.update_connections(&mut connections_ref, &particles_ref, &phase, *progress, &mut rng);

                // Update particle properties based on current phase and sub-phase.
                // RLOC-2: Each update function must have bounded execution time.
                match phase {
                    AnimationPhase::Scatter(sub_phase) => {
                        self.update_scatter_phase(
                            &mut particles_ref,
                            &connections_ref, // Pass immutable ref here
                            sub_phase,
                            *progress,
                            gravity_center,
                            &mut rng,
                        );
                        // RLOC-2: Phase transitions must occur within bounded time/iterations.
                        // Sub-phase advancement integrated here
                        let next_sub = match sub_phase {
                            ScatterSubPhase::Initial if *progress > 0.5 => {
                                Some(ScatterSubPhase::Expansion)
                            }
                            ScatterSubPhase::Expansion if *progress > 1.0 => {
                                Some(ScatterSubPhase::Contraction)
                            }
                            ScatterSubPhase::Contraction if *progress > 1.5 => {
                                Some(ScatterSubPhase::PreConverge)
                            }
                            ScatterSubPhase::PreConverge if *progress > 2.0 => None, // Advance main phase
                            _ => None,
                        };
                        if let Some(next) = next_sub {
                            *self.phase.borrow_mut() = AnimationPhase::Scatter(next);
                            *progress = 0.0; // Reset progress for sub-phase
                        } else if sub_phase == ScatterSubPhase::PreConverge && *progress > 2.0 {
                            self.schedule_phase_advance(ctx);
                        }
                    }
                    AnimationPhase::Converge(sub_phase) => {
                        self.update_converge_phase(
                            &mut particles_ref,
                            &connections_ref, // Pass immutable ref here
                            sub_phase,
                            *progress,
                            gravity_center,
                            &mut rng,
                        );
                        let next_sub = match sub_phase {
                            ConvergeSubPhase::Alignment if *progress > 0.5 => {
                                Some(ConvergeSubPhase::Formation)
                            }
                            ConvergeSubPhase::Formation if *progress > 1.0 => {
                                Some(ConvergeSubPhase::Refinement)
                            }
                            ConvergeSubPhase::Refinement if *progress > 1.5 => {
                                Some(ConvergeSubPhase::Solidification)
                            }
                            ConvergeSubPhase::Solidification if *progress > 2.0 => None, // Advance main phase
                            _ => None,
                        };
                        if let Some(next) = next_sub {
                            *self.phase.borrow_mut() = AnimationPhase::Converge(next);
                            *progress = 0.0;
                        } else if sub_phase == ConvergeSubPhase::Solidification && *progress > 2.0
                        {
                            self.schedule_phase_advance(ctx);
                        }
                    }
                    AnimationPhase::Stable(sub_phase) => {
                        self.update_stable_phase(
                            &mut particles_ref,
                            &connections_ref, // Pass immutable ref here
                            sub_phase,
                            *progress,
                            gravity_center,
                            &mut rng,
                        );
                        let next_sub = match sub_phase {
                            StableSubPhase::Pulse if *progress > 1.0 => Some(StableSubPhase::Orbit),
                            StableSubPhase::Orbit if *progress > 2.0 => Some(StableSubPhase::Ripple),
                            StableSubPhase::Ripple if *progress > 3.0 => {
                                Some(StableSubPhase::PreDissolve)
                            }
                            StableSubPhase::PreDissolve if *progress > 3.5 => None, // Advance main phase
                            _ => None,
                        };
                        if let Some(next) = next_sub {
                            *self.phase.borrow_mut() = AnimationPhase::Stable(next);
                            *progress = 0.0;
                        } else if sub_phase == StableSubPhase::PreDissolve && *progress > 3.5 {
                            self.schedule_phase_advance(ctx);
                        }
                    }
                    AnimationPhase::Dissolve(sub_phase) => {
                        self.update_dissolve_phase(
                            &mut particles_ref,
                            &connections_ref, // Pass immutable ref here
                            sub_phase,
                            *progress,
                            gravity_center,
                            &mut rng,
                        );
                        let next_sub = match sub_phase {
                            DissolveSubPhase::Fracture if *progress > 1.0 => {
                                Some(DissolveSubPhase::Explosion)
                            }
                            DissolveSubPhase::Explosion if *progress > 2.0 => {
                                Some(DissolveSubPhase::Dispersion)
                            }
                            DissolveSubPhase::Dispersion if *progress > 3.0 => {
                                Some(DissolveSubPhase::Fade)
                            }
                            DissolveSubPhase::Fade if *progress > 4.0 => None, // Reset animation
                            _ => None,
                        };
                        if let Some(next) = next_sub {
                            *self.phase.borrow_mut() = AnimationPhase::Dissolve(next);
                            *progress = 0.0;
                        } else if sub_phase == DissolveSubPhase::Fade && *progress > 4.0 {
                            *self.is_complete.borrow_mut() = true;
                            self.schedule_reset(ctx);
                        }
                    }
                }

                // Drop mutable borrows before scheduling next tick
                drop(particles_ref);
                drop(connections_ref);
                drop(rng);

                // Schedule the next tick if not complete.
                if !*self.is_complete.borrow() {
                    self.schedule_tick(ctx);
                }
                true // Request re-render
            }
            Msg::AdvancePhase => {
                let mut phase = self.phase.borrow_mut();
                let mut progress = self.progress.borrow_mut();
                *phase = match *phase {
                    AnimationPhase::Scatter(_) => {
                        AnimationPhase::Converge(ConvergeSubPhase::Alignment)
                    }
                    AnimationPhase::Converge(_) => AnimationPhase::Stable(StableSubPhase::Pulse),
                    AnimationPhase::Stable(_) => {
                        AnimationPhase::Dissolve(DissolveSubPhase::Fracture)
                    }
                    AnimationPhase::Dissolve(_) => {
                        AnimationPhase::Scatter(ScatterSubPhase::Initial)
                    } // Loop back
                };
                *progress = 0.0; // Reset progress for the new phase
                true
            }
            Msg::ShiftGravityCenter => {
                let mut gravity_center = self.gravity_center.borrow_mut();
                let mut rng = self.rng.borrow_mut(); // Borrow RNG
                gravity_center.0 =
                    (gravity_center.0 + rng.random_range(-20.0..20.0)).clamp(50.0, 250.0);
                gravity_center.1 =
                    (gravity_center.1 + rng.random_range(-10.0..10.0)).clamp(25.0, 125.0);
                drop(rng); // Drop borrow
                self.schedule_gravity_shift(ctx); // Schedule the next shift
                true
            }
            Msg::ChangeColorScheme => {
                let mut color_scheme = self.color_scheme.borrow_mut();
                let mut rng = self.rng.borrow_mut(); // Borrow RNG
                let hue_shift = rng.random_range(-15..15) as f32; // Reduced shift range
                // RLOC-4: Color shifting logic should be simple and verifiable.
                color_scheme.primary = Self::shift_color_hue(&color_scheme.primary, hue_shift);
                color_scheme.secondary =
                    Self::shift_color_hue(&color_scheme.secondary, hue_shift + 10.0);
                drop(rng); // Drop borrow
                // Consider shifting accent/background too, or using a more robust color manipulation library.
                self.schedule_color_shift(ctx); // Schedule the next shift
                true
            }
            Msg::Reset => {
                // Reset component to initial state.
                *self.phase.borrow_mut() = AnimationPhase::Scatter(ScatterSubPhase::Initial);
                *self.progress.borrow_mut() = 0.0;
                *self.is_complete.borrow_mut() = false;

                // Reinitialize particles and connections.
                // RLOC-1: Ensure reallocation (if Vec is used) is bounded and predictable.
                let mut rng = self.rng.borrow_mut();
                let particles =
                    Self::initialize_particles(&self.target_text, 250, &mut rng);
                let connections = Self::initialize_connections(&particles, &mut rng);
                *self.particles.borrow_mut() = particles;
                *self.connections.borrow_mut() = connections;
                drop(rng);

                // Restart the animation loop.
                self.schedule_tick(ctx);
                true
            }
        }
    }

    /// View function (required by Yew). Delegates to the trait's render method.
    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.render()
    }
}

// --- Helper Methods for TypingAnimation ---
impl TypingAnimation {
    /// Initializes particles, including text and extra decorative particles.
    /// RLOC-2: Loop bounds are determined by text length and `extra_particles`. Ensure these are bounded.
    /// RLOC-4: Randomness makes verification harder. Use deterministic seeding for tests.
    fn initialize_particles(
        target_text: &str,
        extra_particles: usize,
        rng: &mut ThreadRng, // Accept RNG
    ) -> Vec<Particle> {
        // Consider a seeded RNG for reproducibility.
        let mut particles = Vec::new();
        let char_count = target_text.chars().count();

        // Core text particles
        for (i, ch) in target_text.chars().enumerate() {
            let x = 10.0 + (i as f32 * 20.0); // Spacing based on index
            let y = 50.0; // Baseline Y
            particles.push(Particle {
                x: rng.random_range(0.0..300.0),
                y: rng.random_range(0.0..150.0),
                z: rng.random_range(-20.0..20.0),
                target_x: x,
                target_y: y,
                target_z: 0.0,
                symbol: Self::random_char(rng),
                target_symbol: ch,
                vx: rng.random_range(-2.0..2.0),
                vy: rng.random_range(-2.0..2.0),
                vz: rng.random_range(-1.0..1.0),
                opacity: rng.random_range(0.5..1.0),
                scale: rng.random_range(0.8..1.2),
                rotation: rng.random_range(0.0..360.0),
                is_text: true,
                particle_type: ParticleType::Core,
                energy: 1.0,
            });
        }

        // Extra decorative particles
        for i in 0..extra_particles {
            // RLOC-3: Potentially use ECC-encoded types for particle state if SEUs are a concern.
            let particle_type = match i % 5 {
                // Distribute types
                0 => ParticleType::Orbiter,
                1 => ParticleType::Swarm,
                2 => ParticleType::Fragment,
                3 => ParticleType::Connector,
                _ => ParticleType::Swarm,
            };
            particles.push(Particle {
                x: rng.random_range(-50.0..350.0),
                y: rng.random_range(-50.0..200.0),
                z: rng.random_range(-50.0..50.0),
                target_x: rng.random_range(0.0..(char_count as f32 * 20.0 + 20.0)), // Near text area
                target_y: 50.0 + rng.random_range(-30.0..30.0),
                target_z: rng.random_range(-10.0..10.0),
                symbol: Self::random_char(rng),
                target_symbol: Self::random_char(rng), // Random target
                vx: rng.random_range(-3.0..3.0),
                vy: rng.random_range(-3.0..3.0),
                vz: rng.random_range(-1.5..1.5),
                opacity: rng.random_range(0.1..0.6),
                scale: rng.random_range(0.3..1.0),
                rotation: rng.random_range(0.0..360.0),
                is_text: false,
                particle_type,
                energy: rng.random_range(0.2..0.8),
            });
        }
        particles
    }

    /// Initializes connections between particles.
    /// RLOC-2: Loop bounds depend on particle count. Ensure predictable complexity.
    fn initialize_connections(particles: &[Particle], rng: &mut ThreadRng) -> Vec<Connection> {
        // Accept RNG
        let mut connections = Vec::new();
        let core_indices: Vec<usize> = particles
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_text)
            .map(|(i, _)| i)
            .collect();
        let non_text_indices: Vec<usize> = particles
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.is_text)
            .map(|(i, _)| i)
            .collect();

        // Connect core text particles sequentially
        for i in 0..core_indices.len().saturating_sub(1) {
            connections.push(Connection {
                particle1_idx: core_indices[i],
                particle2_idx: core_indices[i + 1],
                strength: 1.0,
                opacity: 0.8,
                active: true,
            });
        }

        // Random connections between non-text particles (potential for high complexity)
        let max_extra_connections = non_text_indices.len() / 2; // Limit connections
        for _ in 0..max_extra_connections {
            if non_text_indices.len() < 2 {
                break;
            }
            let idx1 = rng.random_range(0..non_text_indices.len());
            let mut idx2 = rng.random_range(0..non_text_indices.len());
            while idx1 == idx2 {
                idx2 = rng.random_range(0..non_text_indices.len());
            } // Avoid self-connection

            connections.push(Connection {
                particle1_idx: non_text_indices[idx1],
                particle2_idx: non_text_indices[idx2],
                strength: rng.random_range(0.1..0.5),
                opacity: rng.random_range(0.1..0.4),
                active: rng.random_bool(0.6), // Initially, not all are active
            });
        }
        connections
    }

    /// Selects a random character for particle symbols.
    fn random_char(rng: &mut impl Rng) -> char {
        // Limited, predictable character set
        let chars = ".:*+!=%#$@&?";
        chars
            .chars()
            .nth(rng.random_range(0..chars.len()))
            .unwrap_or('.')
    }

    /// Simple hue shift logic (placeholder).
    /// RLOC-4: Replace with a verified color library or formally prove this implementation if critical.
    fn shift_color_hue(hex_color: &str, _hue_shift: f32) -> String {
        // Placeholder: In a real scenario, use a proper color library (e.g., `palette`)
        // and convert HSL/HSV for accurate hue shifts. This is a naive approximation.
        // For now, just return the original color to avoid incorrect logic.
        hex_color.to_string() // Return original for safety
    }

    // --- Update Logic per Phase ---
    // RLOC-2: All update functions MUST have bounded execution time. Analyze loops and complexities.
    // RLOC-4: Physics/movement logic should be simple or formally verified if precise behavior is needed.
    // Borrow checker fixes applied below.

    fn update_connections(
        &self,
        connections: &mut [Connection],
        _particles: &[Particle], // Keep immutable borrow for potential future use
        phase: &AnimationPhase,
        progress: f32,
        rng: &mut ThreadRng, // Accept RNG
    ) {
        for conn in connections.iter_mut() {
            // Adjust connection properties based on the current animation phase
            match phase {
                AnimationPhase::Scatter(_) => {
                    conn.active = rng.random_bool((0.3 + progress * 0.1).clamp(0.0, 1.0) as f64); // Become more active, ensure valid probability
                    conn.opacity = conn.opacity * 0.9 + rng.random_range(0.0..0.1);
                    conn.strength = conn.strength * 0.95 + 0.01;
                }
                AnimationPhase::Converge(sub) => {
                    conn.active = true; // Most connections active
                    let target_opacity = if matches!(sub, ConvergeSubPhase::Solidification) {
                        0.9
                    } else {
                        0.7
                    };
                    conn.opacity = (conn.opacity * 0.95 + target_opacity * 0.05).min(0.9);
                    conn.strength = (conn.strength * 0.95 + 0.8 * 0.05).min(1.0);
                }
                AnimationPhase::Stable(sub) => {
                    conn.active = true;
                    conn.opacity = 0.7 + (progress * 4.0).sin() * 0.15; // Gentle pulse
                    conn.strength = 0.8 + (progress * 2.5).cos() * 0.1;
                    // Deactivate some connections briefly during PreDissolve
                    if matches!(sub, StableSubPhase::PreDissolve) && rng.random_bool(0.01) {
                        conn.active = false;
                    }
                }
                AnimationPhase::Dissolve(sub) => {
                    let break_chance = match sub {
                        // Probability of connection breaking
                        DissolveSubPhase::Fracture => 0.005,
                        DissolveSubPhase::Explosion => 0.02,
                        DissolveSubPhase::Dispersion => 0.05,
                        DissolveSubPhase::Fade => 0.1,
                    };
                    conn.active = conn.active && !rng.random_bool(break_chance);
                    conn.opacity *= 0.95; // Fade out
                    conn.strength *= 0.97; // Weaken
                }
            }
            conn.opacity = conn.opacity.clamp(0.0, 1.0);
            conn.strength = conn.strength.clamp(0.0, 1.0);
        }
    }

    // Borrow checker fix: Pass immutable `connections` and `particles` when only reading.
    // Use indices carefully to avoid simultaneous mutable/immutable borrows of `particles`.
    fn update_scatter_phase(
        &self,
        particles: &mut [Particle],
        connections: &[Connection], // Now immutable
        sub_phase: ScatterSubPhase,
        progress: f32,
        gravity_center: (f32, f32),
        rng: &mut ThreadRng, // Accept RNG
    ) {
        // Pre-calculate connection influences if needed, to avoid borrowing issues inside the loop
        let mut connection_influences = vec![(0.0f32, 0.0f32); particles.len()];
        if sub_phase == ScatterSubPhase::PreConverge {
            for conn in connections.iter().filter(|c| c.active) {
                // Check indices before accessing particles to prevent panic
                if conn.particle1_idx < particles.len() && conn.particle2_idx < particles.len() {
                    let p1 = &particles[conn.particle1_idx];
                    let p2 = &particles[conn.particle2_idx];
                    let dx = p2.x - p1.x;
                    let dy = p2.y - p1.y;
                    let force_x = dx * conn.strength * 0.001; // Scale force here
                    let force_y = dy * conn.strength * 0.001;
                    connection_influences[conn.particle1_idx].0 += force_x;
                    connection_influences[conn.particle1_idx].1 += force_y;
                    connection_influences[conn.particle2_idx].0 -= force_x; // Action-reaction
                    connection_influences[conn.particle2_idx].1 -= force_y;
                }
            }
        }

        for i in 0..particles.len() {
            // Cannot borrow `particles[i]` mutably and read `particles[other]` immutably in the same scope easily.
            // We use the pre-calculated influences for PreConverge.
            let p = &mut particles[i];

            // Shared logic: Apply velocity, damping, energy update
            p.x += p.vx;
            p.y += p.vy;
            p.z += p.vz;
            p.vx *= 0.97;
            p.vy *= 0.97;
            p.vz *= 0.97; // Damping
            p.energy = (p.energy * 0.98 + rng.random_range(0.0..0.02)).clamp(0.1, 1.0);
            p.rotation = (p.rotation + p.vx * 0.5) % 360.0; // Spin based on velocity

            match sub_phase {
                ScatterSubPhase::Initial => {
                    if rng.random_bool(0.05) {
                        // Random bursts
                        p.vx += rng.random_range(-1.0..1.0);
                        p.vy += rng.random_range(-1.0..1.0);
                        p.vz += rng.random_range(-0.5..0.5);
                    }
                    if rng.random_bool(0.1) {
                        p.symbol = Self::random_char(rng);
                    }
                }
                ScatterSubPhase::Expansion => {
                    let dx = p.x - gravity_center.0;
                    let dy = p.y - gravity_center.1;
                    let dist_sq = (dx * dx + dy * dy).max(1.0);
                    let force = 5.0 / dist_sq; // Repulsive force from center
                    p.vx += dx * force * p.energy;
                    p.vy += dy * force * p.energy;
                }
                ScatterSubPhase::Contraction => {
                    let dx = gravity_center.0 - p.x;
                    let dy = gravity_center.1 - p.y;
                    let attraction = 0.005 * (1.0 + progress); // Increasing attraction
                    p.vx += dx * attraction * p.energy;
                    p.vy += dy * attraction * p.energy;
                    // Start hinting towards target symbol for text particles
                    if p.is_text && rng.random_bool(0.02) {
                        p.symbol = p.target_symbol;
                    }
                }
                ScatterSubPhase::PreConverge => {
                    // Move towards target, influenced by connections (using pre-calculated influences)
                    let mut target_dx = 0.0;
                    let mut target_dy = 0.0;
                    let mut target_dz = 0.0;

                    if p.is_text {
                        target_dx = p.target_x - p.x;
                        target_dy = p.target_y - p.y;
                        target_dz = p.target_z - p.z;
                        // Start aligning rotation and scale
                        p.rotation *= 0.9;
                        p.scale = p.scale * 0.95 + 1.0 * 0.05;
                    } else {
                        // Apply pre-calculated connection forces (check index just in case)
                        if i < connection_influences.len() {
                            target_dx = connection_influences[i].0;
                            target_dy = connection_influences[i].1;
                        }
                        // Apply Z influence if needed (currently 2D influence)
                        target_dz = 0.0; // Placeholder Z influence
                    }

                    let move_factor = 0.02 + progress * 0.01; // Gradually increase movement speed
                    p.vx += target_dx * move_factor; // Apply scaled influence
                    p.vy += target_dy * move_factor; // Apply scaled influence
                    p.vz += target_dz * move_factor;

                    // Converge symbol more strongly
                    if p.is_text && rng.random_bool(0.1) {
                        p.symbol = p.target_symbol;
                    }
                }
            }
            // Clamp velocity to prevent runaway speeds (RLOC-2)
            p.vx = p.vx.clamp(-5.0, 5.0);
            p.vy = p.vy.clamp(-5.0, 5.0);
            p.vz = p.vz.clamp(-3.0, 3.0);
            p.opacity = (p.opacity * 0.98 + 0.5 * 0.02).clamp(0.1, 1.0); // General opacity adjustment
        }
    }

    // Borrow checker fix: Similar approach as scatter phase
    fn update_converge_phase(
        &self,
        particles: &mut [Particle],
        connections: &[Connection], // Immutable borrow
        sub_phase: ConvergeSubPhase,
        progress: f32,
        gravity_center: (f32, f32),
        rng: &mut ThreadRng, // Accept RNG
    ) {
        let base_converge_speed = 0.05;
        let speed_factor = match sub_phase {
            // Speed increases as phase progresses
            ConvergeSubPhase::Alignment => 0.5,
            ConvergeSubPhase::Formation => 0.8,
            ConvergeSubPhase::Refinement => 1.2,
            ConvergeSubPhase::Solidification => 1.5,
        };
        let converge_speed = base_converge_speed * speed_factor;

        // Pre-calculate influences for non-text particles if needed
        let mut connection_forces = vec![(0.0f32, 0.0f32, 0.0f32); particles.len()]; // Store (dx, dy, dz) forces
        let mut avg_positions = vec![(0.0f32, 0.0f32, 0.0f32); particles.len()]; // Store (sum_x, sum_y, count) for connectors

        for conn in connections.iter().filter(|c| c.active) {
            // Check indices before accessing particles
            if conn.particle1_idx < particles.len() && conn.particle2_idx < particles.len() {
                let p1 = &particles[conn.particle1_idx];
                let p2 = &particles[conn.particle2_idx];

                // Swarm/Connector influences
                if !p1.is_text {
                    match p1.particle_type {
                        ParticleType::Swarm => {
                            let force = 0.005 * conn.strength;
                            connection_forces[conn.particle1_idx].0 += (p2.x - p1.x) * force;
                            connection_forces[conn.particle1_idx].1 += (p2.y - p1.y) * force;
                        }
                        ParticleType::Connector => {
                            avg_positions[conn.particle1_idx].0 += p2.x;
                            avg_positions[conn.particle1_idx].1 += p2.y;
                            avg_positions[conn.particle1_idx].2 += 1.0; // Increment count
                        }
                        _ => {}
                    }
                }
                if !p2.is_text {
                    match p2.particle_type {
                        ParticleType::Swarm => {
                            let force = 0.005 * conn.strength;
                            // Action-reaction
                            connection_forces[conn.particle2_idx].0 += (p1.x - p2.x) * force;
                            connection_forces[conn.particle2_idx].1 += (p1.y - p2.y) * force;
                        }
                        ParticleType::Connector => {
                            avg_positions[conn.particle2_idx].0 += p1.x;
                            avg_positions[conn.particle2_idx].1 += p1.y;
                            avg_positions[conn.particle2_idx].2 += 1.0; // Increment count
                        }
                        _ => {}
                    }
                }
            }
        }


        for i in 0..particles.len() {
            let p = &mut particles[i];
            // Apply velocity, damping
            p.x += p.vx;
            p.y += p.vy;
            p.z += p.vz;
            p.vx *= 0.95;
            p.vy *= 0.95;
            p.vz *= 0.95;

            if p.is_text {
                // Core text particles move definitively to target
                let dx = p.target_x - p.x;
                let dy = p.target_y - p.y;
                let dz = p.target_z - p.z;
                p.vx += dx * converge_speed;
                p.vy += dy * converge_speed;
                p.vz += dz * converge_speed;
                p.symbol = p.target_symbol; // Lock symbol
                p.opacity = (p.opacity * 0.9 + 1.0 * 0.1).min(1.0); // Become fully opaque
                p.scale = p.scale * 0.95 + 1.0 * 0.05; // Normalize scale
                p.rotation *= 0.8; // Stop rotation
            } else {
                // Non-text particles have varied behavior based on type
                match p.particle_type {
                    ParticleType::Orbiter => {
                        // Orbit around the forming text center
                        let text_center_x = self.target_text.len() as f32 * 10.0; // Approx center
                        let angle = progress * 1.5 + (i as f32 * 0.3);
                        let radius = 30.0 + (i % 10) as f32 * 2.0;
                        let target_x = text_center_x + angle.cos() * radius;
                        let target_y = 50.0 + angle.sin() * radius * 0.5; // Elliptical
                        p.vx += (target_x - p.x) * 0.03;
                        p.vy += (target_y - p.y) * 0.03;
                        p.opacity = (p.opacity * 0.9 + 0.3 * 0.1).clamp(0.1, 0.5); // Dimmer
                    }
                    ParticleType::Swarm => {
                        // Follow the shifting gravity center
                        let dx_grav = gravity_center.0 - p.x;
                        let dy_grav = gravity_center.1 - p.y;
                        p.vx += dx_grav * 0.002;
                        p.vy += dy_grav * 0.002;
                        // Add pre-calculated connection forces (check index)
                        if i < connection_forces.len() {
                            p.vx += connection_forces[i].0;
                            p.vy += connection_forces[i].1;
                        }

                        p.opacity = (p.opacity * 0.95 + 0.2 * 0.05).clamp(0.1, 0.4);
                    }
                    ParticleType::Fragment => {
                        // Move somewhat erratically, slow down
                        if rng.random_bool(0.05) {
                            p.vx += rng.random_range(-0.5..0.5);
                            p.vy += rng.random_range(-0.5..0.5);
                        }
                        p.opacity *= 0.98; // Fade slightly
                    }
                    ParticleType::Connector => {
                        // Use pre-calculated average position (check index)
                        if i < avg_positions.len() {
                            let avg_pos = avg_positions[i];
                            if avg_pos.2 > 0.0 { // Check count > 0
                                let avg_x = avg_pos.0 / avg_pos.2;
                                let avg_y = avg_pos.1 / avg_pos.2;
                                let influence = 0.01;
                                p.vx += (avg_x - p.x) * influence;
                                p.vy += (avg_y - p.y) * influence;
                            } else {
                                // If no connections, maybe drift slightly
                                p.vx += rng.random_range(-0.05..0.05);
                                p.vy += rng.random_range(-0.05..0.05);
                            }
                        }
                        p.opacity = (p.opacity * 0.9 + 0.5 * 0.1).clamp(0.1, 0.6);
                    }
                    ParticleType::Core => { /* Should not happen if is_text is false */ }
                }
                if rng.random_bool(0.05) {
                    p.symbol = Self::random_char(rng);
                } // Still flicker symbols
            }
            // Clamp velocity
            p.vx = p.vx.clamp(-4.0, 4.0);
            p.vy = p.vy.clamp(-4.0, 4.0);
            p.vz = p.vz.clamp(-2.0, 2.0);
        }
    }

    // Borrow checker fix: Similar approach
    fn update_stable_phase(
        &self,
        particles: &mut [Particle],
        connections: &[Connection], // Immutable borrow
        sub_phase: StableSubPhase,
        progress: f32,
        gravity_center: (f32, f32),
        rng: &mut ThreadRng, // Accept RNG
    ) {
        // Pre-calculate swarm connection forces if needed
        let mut swarm_forces = vec![(0.0f32, 0.0f32); particles.len()];
        for conn in connections.iter().filter(|c| c.active) {
            // Check indices before accessing particles
            if conn.particle1_idx < particles.len() && conn.particle2_idx < particles.len() {
                let p1 = &particles[conn.particle1_idx];
                let p2 = &particles[conn.particle2_idx];

                if !p1.is_text && p1.particle_type == ParticleType::Swarm {
                    let force = 0.002 * conn.strength;
                    swarm_forces[conn.particle1_idx].0 += (p2.x - p1.x) * force;
                    swarm_forces[conn.particle1_idx].1 += (p2.y - p1.y) * force;
                }
                if !p2.is_text && p2.particle_type == ParticleType::Swarm {
                    let force = 0.002 * conn.strength;
                    swarm_forces[conn.particle2_idx].0 += (p1.x - p2.x) * force; // Action-reaction
                    swarm_forces[conn.particle2_idx].1 += (p1.y - p2.y) * force;
                }
            }
        }


        for i in 0..particles.len() {
            let p = &mut particles[i];
            p.vx *= 0.9;
            p.vy *= 0.9;
            p.vz *= 0.9; // Dampen velocity strongly

            if p.is_text {
                // Subtle effects on stable text
                p.symbol = p.target_symbol;
                p.opacity = 1.0;
                p.scale = 1.0;
                p.rotation = 0.0;
                let base_x = p.target_x;
                let base_y = p.target_y;

                match sub_phase {
                    StableSubPhase::Pulse => {
                        let pulse_amount = (progress * 6.0).sin() * 0.05 + 1.0; // Scale pulse
                        p.scale = pulse_amount;
                        // Slight position jitter
                        p.x = base_x + (progress * 10.0 + i as f32 * 0.5).sin() * 0.3;
                        p.y = base_y + (progress * 8.0 + i as f32 * 0.5).cos() * 0.3;
                    }
                    StableSubPhase::Orbit => {
                        let angle = progress * 2.0 + i as f32 * 0.3;
                        p.x = base_x + angle.sin() * 0.5; // Small circular motion
                        p.y = base_y + angle.cos() * 0.5;
                    }
                    StableSubPhase::Ripple => {
                        let ripple_offset = i as f32 * 0.4;
                        let ripple_val = ((progress * 5.0) - ripple_offset).sin();
                        p.y = base_y + ripple_val * 1.5; // Vertical ripple
                        p.scale = 1.0 + ripple_val.abs() * 0.1; // Scale with ripple
                    }
                    StableSubPhase::PreDissolve => {
                        // Gentle random drifts before dissolving
                        if rng.random_bool(0.02) {
                            p.vx += rng.random_range(-0.1..0.1);
                            p.vy += rng.random_range(-0.1..0.1);
                        }
                        // Apply the small drift velocity
                        p.x = base_x + p.vx; // Apply vx directly to base
                        p.y = base_y + p.vy; // Apply vy directly to base

                        // Occasional flicker
                        if rng.random_bool(0.005) {
                            p.symbol = Self::random_char(rng);
                        }
                    }
                }
                // Ensure position stays very close to target (re-clamp after applying drift/effects)
                p.x = p.x.clamp(base_x - 1.5, base_x + 1.5); // Allow slightly more deviation
                p.y = p.y.clamp(base_y - 1.5, base_y + 1.5);
                p.z = p.target_z; // Lock Z
            } else {
                // Non-text particles continue background motion
                p.x += p.vx;
                p.y += p.vy;
                p.z += p.vz;
                match p.particle_type {
                    ParticleType::Orbiter => {
                        let text_center_x = self.target_text.len() as f32 * 10.0;
                        let angle = progress * 1.0 + (i as f32 * 0.3);
                        let radius = 35.0
                            + (i % 10) as f32 * 1.5
                            + (progress * 0.5).sin() * 5.0; // Varying radius
                        let target_x = text_center_x + angle.cos() * radius;
                        let target_y = 50.0 + angle.sin() * radius * 0.6;
                        p.vx += (target_x - p.x) * 0.01;
                        p.vy += (target_y - p.y) * 0.01;
                        p.opacity = 0.3 + (angle * 2.0).sin().abs() * 0.15;
                    }
                    ParticleType::Swarm => {
                        let swarm_target_x = gravity_center.0 + (progress * 1.5).cos() * 30.0;
                        let swarm_target_y = gravity_center.1 + (progress * 1.2).sin() * 20.0;
                        p.vx += (swarm_target_x - p.x) * 0.005;
                        p.vy += (swarm_target_y - p.y) * 0.005;
                        // Apply pre-calculated connection influence (check index)
                        if i < swarm_forces.len() {
                            p.vx += swarm_forces[i].0;
                            p.vy += swarm_forces[i].1;
                        }

                        p.opacity = 0.2 + (progress * 3.0 + i as f32 * 0.1).sin().abs() * 0.1;
                    }
                    _ => {
                        // Fragments and Connectors mostly drift or fade
                        p.opacity *= 0.99;
                    }
                }
                if rng.random_bool(0.02) {
                    p.symbol = Self::random_char(rng);
                }
            }
            // Clamp velocity
            p.vx = p.vx.clamp(-2.0, 2.0);
            p.vy = p.vy.clamp(-2.0, 2.0);
            p.vz = p.vz.clamp(-1.0, 1.0);
        }
    }

    // Removed unused `progress` parameter by prefixing with `_`
    fn update_dissolve_phase(
        &self,
        particles: &mut [Particle],
        _connections: &[Connection], // Mark as unused if truly unused
        sub_phase: DissolveSubPhase,
        _progress: f32, // Prefixed with underscore
        _gravity_center: (f32, f32), // Mark as unused
        rng: &mut ThreadRng,     // Accept RNG
    ) {
        let explosion_factor = match sub_phase {
            // How strong the outward push is
            DissolveSubPhase::Fracture => 0.1,
            DissolveSubPhase::Explosion => 1.0,
            DissolveSubPhase::Dispersion => 0.5,
            DissolveSubPhase::Fade => 0.2,
        };
        let fade_rate = match sub_phase {
            // How quickly particles fade
            DissolveSubPhase::Fracture => 0.995,
            DissolveSubPhase::Explosion => 0.99,
            DissolveSubPhase::Dispersion => 0.98,
            DissolveSubPhase::Fade => 0.95,
        };

        for i in 0..particles.len() {
            let p = &mut particles[i];
            // Apply outward force from original target position (for text) or current position (others)
            let origin_x = if p.is_text { p.target_x } else { p.x };
            let origin_y = if p.is_text { p.target_y } else { p.y };
            let dx = p.x - origin_x;
            let dy = p.y - origin_y;
            let dist_sq = (dx * dx + dy * dy).max(1.0);
            let force = explosion_factor * 5.0 / dist_sq;

            p.vx += dx * force * p.energy * rng.random_range(0.5..1.5); // Add randomness to explosion
            p.vy += dy * force * p.energy * rng.random_range(0.5..1.5);
            if matches!(sub_phase, DissolveSubPhase::Explosion) {
                // Add z-component during explosion
                p.vz += rng.random_range(-0.5..0.5) * explosion_factor;
            }

            // Apply velocity and damping
            p.x += p.vx;
            p.y += p.vy;
            p.z += p.vz;
            p.vx *= 0.98;
            p.vy *= 0.98;
            p.vz *= 0.98; // Consistent damping

            // Fade out
            p.opacity *= fade_rate;
            if matches!(sub_phase, DissolveSubPhase::Fade) {
                p.scale *= 0.98; // Shrink during final fade
            }

            // Randomize symbols frequently during dissolution
            if rng.random_bool(0.15) {
                p.symbol = Self::random_char(rng);
            }

            // Clamp velocity
            p.vx = p.vx.clamp(-6.0, 6.0);
            p.vy = p.vy.clamp(-6.0, 6.0);
            p.vz = p.vz.clamp(-4.0, 4.0);
            p.opacity = p.opacity.clamp(0.0, 1.0);
            p.scale = p.scale.clamp(0.0, 2.0);
        }
    }

    // --- Scheduling Methods ---
    // RLOC-2: Ensure timeout durations are constants or bounded variables.

    fn schedule_tick(&self, ctx: &Context<Self>) {
        const TICK_DELAY_MS: u32 = 16; // ~60 FPS target
        let link = ctx.link().clone();
        // Use Timeout::new for scheduling future messages. `forget()` leaks the timeout handle,
        // which is acceptable here as it's part of the component's lifecycle, but manage handles if needed.
        Timeout::new(TICK_DELAY_MS, move || link.send_message(Msg::Tick)).forget();
    }

    fn schedule_phase_advance(&self, ctx: &Context<Self>) {
        const PHASE_DELAY_MS: u32 = 30; // Short delay before processing next phase
        let link = ctx.link().clone();
        // Use a small delay to prevent potential immediate re-triggering within the same update cycle.
        Timeout::new(PHASE_DELAY_MS, move || link.send_message(Msg::AdvancePhase)).forget();
    }

    // `schedule_sub_phase_advance` removed as logic is integrated into `Msg::Tick`.

    fn schedule_gravity_shift(&self, ctx: &Context<Self>) {
        const GRAVITY_SHIFT_DELAY_MS: u32 = 2500; // How often gravity center shifts
        let link = ctx.link().clone();
        Timeout::new(GRAVITY_SHIFT_DELAY_MS, move || {
            link.send_message(Msg::ShiftGravityCenter)
        })
            .forget();
    }

    fn schedule_color_shift(&self, ctx: &Context<Self>) {
        const COLOR_SHIFT_DELAY_MS: u32 = 4000; // How often colors shift
        let link = ctx.link().clone();
        Timeout::new(COLOR_SHIFT_DELAY_MS, move || {
            link.send_message(Msg::ChangeColorScheme)
        })
            .forget();
    }

    fn schedule_reset(&self, ctx: &Context<Self>) {
        const RESET_DELAY_MS: u32 = 1500; // Delay after fade-out before resetting
        let link = ctx.link().clone();
        Timeout::new(RESET_DELAY_MS, move || link.send_message(Msg::Reset)).forget();
    }

    // --- Rendering Methods ---
    // RLOC-4: Rendering logic should be simple and directly reflect state.

    /// Renders individual particles as styled spans.
    fn render_particles(&self) -> Html {
        let particles = self.particles.borrow();
        let color_scheme = self.color_scheme.borrow();
        let phase = self.phase.borrow(); // Read phase for context-specific rendering

        html! {
            // Container for particles, inherits perspective from parent (.transformium-field)
            <div class="transformium-particles">
                { particles.iter().filter(|p| p.opacity > 0.01).map(|p| { // Filter invisible particles
                    let z_index = ((p.z + 100.0).clamp(0.0, 200.0) / 2.0) as i32; // Map z to 0-100 range for z-index

                    // Determine color based on type, state, phase
                    let color = match (p.is_text, &p.particle_type, &*phase) {
                        (true, _, AnimationPhase::Stable(StableSubPhase::Ripple)) => color_scheme.accent.clone(), // Highlight during ripple
                        (true, _, _) => color_scheme.primary.clone(),
                        (false, ParticleType::Orbiter, _) => color_scheme.accent.clone(),
                        (false, ParticleType::Connector, _) => color_scheme.primary.clone(), // Connectors match text
                        _ => color_scheme.secondary.clone(), // Swarm, Fragment use secondary
                    };

                     // Simple perspective scaling based on Z is handled by the `perspective` style on parent
                     // We adjust scale based on Z relative to the viewer (0 is at screen depth)
                     let z_scale = (1.0 + p.z / 400.0).clamp(0.5, 2.0); // Scale slightly based on depth

                    let style = format!(
                        "position: absolute; left: {:.1}px; top: {:.1}px; opacity: {:.2}; z-index: {}; \
                         transform: translate3d(-50%, -50%, {:.1}px) scale({:.2}) rotate({:.1}deg); \
                         color: {}; font-size: 14px; \
                         will-change: transform, opacity;", // Removed transition for potentially smoother raw updates
                        p.x, p.y, p.opacity, z_index,
                        p.z, // Apply Z translation for 3D
                        p.scale * z_scale, // Combine inherent scale and z-scale
                        p.rotation, color
                    );

                    // Add CSS classes for potential styling hooks
                    let mut classes = vec!["particle".to_string()];
                    if p.is_text {
                         classes.push("text-particle".to_string());
                         // Add glow class only during stable phase for text
                         if matches!(*phase, AnimationPhase::Stable(_)) {
                             classes.push("metallic-glow".to_string());
                         }
                    }

                    // Add flow effect class for specific types if desired
                    // if matches!(p.particle_type, ParticleType::Orbiter | ParticleType::Swarm) {
                    //     classes.push("flow-effect".to_string());
                    //     // Consider adding flow-delay calculation here if using CSS for flow
                    // }


                    html! { <span class={classes.join(" ")} style={style}>{ p.symbol }</span> }
                }).collect::<Html>() }
            </div>
        }
    }

    /// Renders connections between particles as SVG lines.
    fn render_connections(&self) -> Html {
        let particles_borrow = self.particles.borrow(); // Borrow particles once
        let particles = &*particles_borrow; // Deref to slice
        let connections = self.connections.borrow();
        let color_scheme = self.color_scheme.borrow();

        html! {
            // SVG layer positioned absolutely behind particles
            <svg class="connections-layer" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: 1; pointer-events: none;"> // z-index 1 places above background, below particles
                { connections.iter()
                    .filter(|c| c.active && c.opacity > 0.01 && c.strength > 0.01) // Filter inactive/invisible
                    .filter_map(|conn| { // Use filter_map to handle potential index errors gracefully
                        // RLOC-1: Bounds checking is crucial. Ensure indices are always valid.
                        // Get both particles safely.
                        particles.get(conn.particle1_idx).and_then(|p1| {
                            particles.get(conn.particle2_idx).map(|p2| (p1, p2))
                        }).map(|(p1, p2)| {
                            // Determine color based on whether connection is between text particles
                            let stroke_color = if p1.is_text && p2.is_text {
                                color_scheme.primary.clone()
                            } else {
                                color_scheme.secondary.clone()
                            };
                            let stroke_width = (conn.strength * 1.2).clamp(0.1, 1.5); // Thinner lines max 1.5px

                            html! {
                                <line x1={p1.x.to_string()} y1={p1.y.to_string()}
                                      x2={p2.x.to_string()} y2={p2.y.to_string()}
                                      stroke={stroke_color} stroke-width={stroke_width.to_string()}
                                      stroke-opacity={conn.opacity.to_string()}
                                      stroke-linecap="round" // Nicer line endings
                                      class="connection-line" // Add class for potential CSS styling/transitions
                                />
                            }
                        })
                    }).collect::<Html>()
                }
            </svg>
        }
    }
}