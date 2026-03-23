use gloo_timers::callback::{Interval, Timeout};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;

pub struct TypingAnimation {
    particles: Rc<RefCell<Vec<Particle>>>,
    connections: Rc<RefCell<Vec<Connection>>>,
    target_text: String,
    phase: Rc<RefCell<AnimationPhase>>,
    progress: Rc<RefCell<f32>>,
    is_complete: Rc<RefCell<bool>>,
    gravity_center: Rc<RefCell<(f32, f32)>>,
    color_scheme: Rc<RefCell<ColorScheme>>,
    rng: Rc<RefCell<ThreadRng>>,
    width: f32,
    height: f32,
    // Timer handles - stored to prevent memory leaks and enable cancellation
    tick_timer: Rc<RefCell<Option<Timeout>>>,
    phase_timer: Rc<RefCell<Option<Timeout>>>,
    _gravity_interval: Interval,
    _color_interval: Interval,
    reset_timer: Rc<RefCell<Option<Timeout>>>,
}

#[derive(Clone)]
struct Connection {
    particle1_idx: usize,
    particle2_idx: usize,
    strength: f32,
    opacity: f32,
    active: bool,
}

#[derive(Clone)]
struct ColorScheme {
    primary: String,
    secondary: String,
    accent: String,
    background: String,
}

#[derive(Clone)]
struct Particle {
    x: f32,
    y: f32,
    z: f32,
    target_x: f32,
    target_y: f32,
    target_z: f32,
    symbol: char,
    target_symbol: char,
    vx: f32,
    vy: f32,
    vz: f32,
    opacity: f32,
    scale: f32,
    rotation: f32,
    is_text: bool,
    particle_type: ParticleType,
    energy: f32,
    age: f32,
    life: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ParticleType {
    Core,
    Orbiter,
    Swarm,
    Fragment,
    Connector,
}

#[derive(Clone, Debug)]
enum AnimationPhase {
    Scatter(ScatterSubPhase),
    Converge(ConvergeSubPhase),
    Stable(StableSubPhase),
    Dissolve(DissolveSubPhase),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ScatterSubPhase {
    Initial,
    Expansion,
    Contraction,
    PreConverge,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ConvergeSubPhase {
    Alignment,
    Formation,
    Refinement,
    Solidification,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum StableSubPhase {
    Pulse,
    Orbit,
    Ripple,
    Shimmer,
    PreDissolve,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum DissolveSubPhase {
    Fracture,
    Explosion,
    Dispersion,
    Fade,
}

pub enum Msg {
    Tick,
    Reset,
    AdvancePhase,
    ShiftGravityCenter,
    ChangeColorScheme,
}

struct CybertronianMapper;

impl CybertronianMapper {
    fn map_char(c: char) -> char {
        match c {
            // Uppercase letters based on provided Cybertronian alphabet images
            'A' => '⏃',
            'B' => 'ᗷ',
            'C' => 'ᑕ',
            'D' => 'ᗞ',
            'E' => '⟊',
            'F' => '⎎',
            'G' => 'Ꮆ',
            'H' => '⋔',
            'I' => '⟙',
            'J' => '⟗',
            'K' => 'Ꮶ',
            'L' => '⅃',
            'M' => '⏁',
            'N' => 'ᑎ',
            'O' => '〇',
            'P' => '℘',
            'Q' => 'Ϙ',
            'R' => '尺',
            'S' => '⟅',
            'T' => 'ナ',
            'U' => '⋒',
            'V' => '٧',
            'W' => '山',
            'X' => '〤',
            'Y' => 'Ꭹ',
            'Z' => 'ㄗ',

            // Lowercase letters (using variants from different Cybertronian styles)
            'a' => 'ค',
            'b' => '♭',
            'c' => 'ᑢ',
            'd' => 'ↁ',
            'e' => '⋿',
            'f' => 'ℱ',
            'g' => 'Ꮆ',
            'h' => 'Ђ',
            'i' => '|',
            'j' => 'ן',
            'k' => 'к',
            'l' => '↳',
            'm' => '൩',
            'n' => 'ภ',
            'o' => '◯',
            'p' => '℘',
            'q' => 'զ',
            'r' => 'г',
            's' => '≠',
            't' => 'ፐ',
            'u' => '∪',
            'v' => '∨',
            'w' => 'พ',
            'x' => '⊗',
            'y' => 'у',
            'z' => 'չ',

            // Numbers (from provided Cybertronian numerals)
            '0' => '⦿',
            '1' => '|',
            '2' => 'ᒿ',
            '3' => '≡',
            '4' => '⫓',
            '5' => '⫔',
            '6' => '⏀',
            '7' => '⫛',
            '8' => '∞',
            '9' => 'ⴤ',

            // Special characters (approximations for Cybertronian symbols)
            '.' => '•',
            '(' => '⦑',
            ')' => '⦒',
            '_' => '⎼',
            '!' => '⫝',
            '@' => '⊛',
            '#' => '⧇',
            '$' => '⧫',
            '%' => '⧮',
            '^' => '△',
            '&' => '⊼',
            '*' => '⋆',
            '-' => '⎯',
            '+' => '⊕',
            '=' => '⋕',
            '{' => '⦓',
            '}' => '⦔',
            '[' => '【',
            ']' => '】',
            '|' => '┃',
            '\\' => '⧹',
            ':' => '∴',
            ';' => '⁏',
            '\'' => '´',
            '"' => '‶',
            ',' => '⸴',
            '<' => '⫷',
            '>' => '⫸',
            '/' => '⧸',
            '?' => '⸮',
            '`' => '῾',
            '~' => '∿',

            // Default for any unmatched character
            _ => '⧠',
        }
    }

    fn map_random_symbol(rng: &mut impl Rng) -> char {
        // Pool of visually interesting symbols inspired by Cybertronian script
        let cyber_random_chars =
            "⏃ᗷᑕᗞ⟊⎎Ꮆ⋔⟙⟗Ꮶ⅃⏁ᑎ〇℘Ϙ尺⟅ナ⋒٧山〤Ꭹㄗค♭ᑢↁ⋿ℱᎶЂ|ן⊗≠ፐ∪∨พу⊼⋕⦿|ᒿ≡⫓⫔⏀⫛∞ⴤ•⦑⦒⎼⊕⧠";
        let index = rng.random_range(0..cyber_random_chars.len());
        cyber_random_chars.chars().nth(index).unwrap_or('⧠')
    }
}

impl Component for TypingAnimation {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let target_text = ".unwrap()";
        let mut rng = rand::rng();
        const EXTRA_PARTICLES: usize = 80;
        let width = 300.0;
        let height = 150.0;

        let particles =
            Self::initialize_particles(target_text, EXTRA_PARTICLES, width, height, &mut rng);
        let connections = Self::initialize_connections(&particles, &mut rng);

        let color_scheme = ColorScheme {
            primary: "#66d9ef".to_string(),
            secondary: "#a6e22e".to_string(),
            accent: "#f92672".to_string(),
            background: "rgba(39, 40, 34, 0.0)".to_string(),
        };

        let tick_timer = Rc::new(RefCell::new(None));
        let phase_timer = Rc::new(RefCell::new(None));
        let reset_timer = Rc::new(RefCell::new(None));

        let gravity_interval = {
            let link = ctx.link().clone();
            Interval::new(2500, move || link.send_message(Msg::ShiftGravityCenter))
        };

        let color_interval = {
            let link = ctx.link().clone();
            Interval::new(4000, move || link.send_message(Msg::ChangeColorScheme))
        };

        let initial_tick = {
            let link = ctx.link().clone();
            Timeout::new(16, move || link.send_message(Msg::Tick))
        };
        *tick_timer.borrow_mut() = Some(initial_tick);

        TypingAnimation {
            particles: Rc::new(RefCell::new(particles)),
            connections: Rc::new(RefCell::new(connections)),
            target_text: target_text.to_string(),
            phase: Rc::new(RefCell::new(AnimationPhase::Scatter(
                ScatterSubPhase::Initial,
            ))),
            progress: Rc::new(RefCell::new(0.0)),
            is_complete: Rc::new(RefCell::new(false)),
            gravity_center: Rc::new(RefCell::new((width / 2.0, height / 2.0))),
            color_scheme: Rc::new(RefCell::new(color_scheme)),
            rng: Rc::new(RefCell::new(rng)),
            width,
            height,
            tick_timer,
            phase_timer,
            _gravity_interval: gravity_interval,
            _color_interval: color_interval,
            reset_timer,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                if *self.is_complete.borrow() {
                    return false;
                }

                let phase = self.phase.borrow().clone();
                let mut progress = self.progress.borrow_mut();
                let gravity_center = *self.gravity_center.borrow();
                *progress += 0.02;

                let mut particles_ref = self.particles.borrow_mut();
                let mut connections_ref = self.connections.borrow_mut();
                let mut rng = self.rng.borrow_mut();

                self.update_connections(
                    &mut connections_ref,
                    &particles_ref,
                    &phase,
                    *progress,
                    &mut rng,
                );

                match phase {
                    AnimationPhase::Scatter(sub_phase) => {
                        self.update_scatter_phase(
                            &mut particles_ref,
                            sub_phase,
                            *progress,
                            gravity_center,
                            &mut rng,
                        );
                        let next_sub = match sub_phase {
                            ScatterSubPhase::Initial if *progress > 0.6 => {
                                Some(ScatterSubPhase::Expansion)
                            }
                            ScatterSubPhase::Expansion if *progress > 1.2 => {
                                Some(ScatterSubPhase::Contraction)
                            }
                            ScatterSubPhase::Contraction if *progress > 1.8 => {
                                Some(ScatterSubPhase::PreConverge)
                            }
                            ScatterSubPhase::PreConverge if *progress > 2.5 => None,
                            _ => None,
                        };
                        if let Some(next) = next_sub {
                            *self.phase.borrow_mut() = AnimationPhase::Scatter(next);
                            *progress = 0.0;
                        } else if sub_phase == ScatterSubPhase::PreConverge
                            && *progress > 2.5
                            && self.phase_timer.borrow().is_none()
                        {
                            self.schedule_phase_advance(ctx);
                        }
                    }
                    AnimationPhase::Converge(sub_phase) => {
                        self.update_converge_phase(
                            &mut particles_ref,
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
                            ConvergeSubPhase::Solidification if *progress > 2.2 => None,
                            _ => None,
                        };
                        if let Some(next) = next_sub {
                            *self.phase.borrow_mut() = AnimationPhase::Converge(next);
                            *progress = 0.0;
                        } else if sub_phase == ConvergeSubPhase::Solidification
                            && *progress > 2.2
                            && self.phase_timer.borrow().is_none()
                        {
                            self.schedule_phase_advance(ctx);
                        }
                    }
                    AnimationPhase::Stable(sub_phase) => {
                        self.update_stable_phase(
                            &mut particles_ref,
                            sub_phase,
                            *progress,
                            gravity_center,
                            &mut rng,
                        );
                        let next_sub = match sub_phase {
                            StableSubPhase::Pulse if *progress > 1.5 => Some(StableSubPhase::Orbit),
                            StableSubPhase::Orbit if *progress > 2.5 => {
                                Some(StableSubPhase::Ripple)
                            }
                            StableSubPhase::Ripple if *progress > 2.0 => {
                                Some(StableSubPhase::Shimmer)
                            }
                            StableSubPhase::Shimmer if *progress > 2.5 => {
                                Some(StableSubPhase::PreDissolve)
                            }
                            StableSubPhase::PreDissolve if *progress > 1.0 => None,
                            _ => None,
                        };
                        if let Some(next) = next_sub {
                            *self.phase.borrow_mut() = AnimationPhase::Stable(next);
                            *progress = 0.0;
                        } else if sub_phase == StableSubPhase::PreDissolve
                            && *progress > 1.0
                            && self.phase_timer.borrow().is_none()
                        {
                            self.schedule_phase_advance(ctx);
                        }
                    }
                    AnimationPhase::Dissolve(sub_phase) => {
                        self.update_dissolve_phase(
                            &mut particles_ref,
                            &connections_ref,
                            sub_phase,
                            *progress,
                            gravity_center,
                            &mut rng,
                        );
                        let next_sub = match sub_phase {
                            DissolveSubPhase::Fracture if *progress > 0.8 => {
                                Some(DissolveSubPhase::Explosion)
                            }
                            DissolveSubPhase::Explosion if *progress > 1.5 => {
                                Some(DissolveSubPhase::Dispersion)
                            }
                            DissolveSubPhase::Dispersion if *progress > 2.5 => {
                                Some(DissolveSubPhase::Fade)
                            }
                            DissolveSubPhase::Fade if *progress > 3.5 => None,
                            _ => None,
                        };
                        if let Some(next) = next_sub {
                            *self.phase.borrow_mut() = AnimationPhase::Dissolve(next);
                            *progress = 0.0;
                        } else if sub_phase == DissolveSubPhase::Fade && *progress > 3.5 {
                            *self.is_complete.borrow_mut() = true;
                            if self.reset_timer.borrow().is_none() {
                                self.schedule_reset(ctx);
                            }
                        }
                    }
                }

                drop(particles_ref);
                drop(connections_ref);
                drop(rng);

                if !*self.is_complete.borrow() {
                    self.schedule_tick(ctx);
                }
                true
            }
            Msg::AdvancePhase => {
                *self.phase_timer.borrow_mut() = None; // Clear so next phase can schedule
                let mut phase = self.phase.borrow_mut();
                *self.progress.borrow_mut() = 0.0;
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
                    }
                };
                true
            }
            Msg::ShiftGravityCenter => {
                let mut gravity_center = self.gravity_center.borrow_mut();
                let mut rng = self.rng.borrow_mut();
                gravity_center.0 = (gravity_center.0 + rng.random_range(-30.0..30.0))
                    .clamp(self.width * 0.1, self.width * 0.9);
                gravity_center.1 = (gravity_center.1 + rng.random_range(-20.0..20.0))
                    .clamp(self.height * 0.1, self.height * 0.9);
                true
            }
            Msg::ChangeColorScheme => false,
            Msg::Reset => {
                *self.phase.borrow_mut() = AnimationPhase::Scatter(ScatterSubPhase::Initial);
                *self.progress.borrow_mut() = 0.0;
                *self.is_complete.borrow_mut() = false;

                let mut rng = self.rng.borrow_mut();
                let particles = Self::initialize_particles(
                    &self.target_text,
                    80,
                    self.width,
                    self.height,
                    &mut rng,
                );
                let connections = Self::initialize_connections(&particles, &mut rng);
                *self.particles.borrow_mut() = particles;
                *self.connections.borrow_mut() = connections;

                drop(rng);
                self.schedule_tick(ctx);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="transformium-field" style={format!("background-color: {}; transform-style: preserve-3d;", self.color_scheme.borrow().background)}>
                { self.render_connections() }
                { self.render_particles() }
            </div>
        }
    }
}

impl TypingAnimation {
    fn initialize_particles(
        target_text: &str,
        extra_particles: usize,
        width: f32,
        height: f32,
        rng: &mut ThreadRng,
    ) -> Vec<Particle> {
        let mut particles = Vec::new();
        let char_count = target_text.chars().count();
        let text_width = char_count as f32 * 20.0;
        let text_start_x = (width - text_width) / 2.0 + 10.0;
        let text_y = height * 0.6;

        // Create main text particles
        for (i, ch) in target_text.chars().enumerate() {
            let x = text_start_x + (i as f32 * 20.0);
            particles.push(Particle {
                x: rng.random_range(0.0..width),
                y: rng.random_range(0.0..height),
                z: rng.random_range(-30.0..30.0),
                target_x: x,
                target_y: text_y,
                target_z: 0.0,
                // Start with random Cybertronian symbol
                symbol: CybertronianMapper::map_random_symbol(rng),
                // Map target character to Cybertronian equivalent
                target_symbol: CybertronianMapper::map_char(ch),
                vx: rng.random_range(-2.5..2.5),
                vy: rng.random_range(-2.5..2.5),
                vz: rng.random_range(-1.5..1.5),
                opacity: 0.0,
                scale: rng.random_range(0.2..0.8),
                rotation: rng.random_range(0.0..360.0),
                is_text: true,
                particle_type: ParticleType::Core,
                energy: rng.random_range(0.8..1.2),
                age: 0.0,
                life: rng.random_range(5.0..10.0),
            });
        }

        // Create supporting particles
        for i in 0..extra_particles {
            let particle_type = match i % 5 {
                0 => ParticleType::Orbiter,
                1 => ParticleType::Swarm,
                2 => ParticleType::Fragment,
                3 => ParticleType::Connector,
                _ => ParticleType::Swarm,
            };

            let life = match particle_type {
                ParticleType::Fragment | ParticleType::Connector => rng.random_range(2.0..5.0),
                _ => rng.random_range(4.0..8.0),
            };

            particles.push(Particle {
                x: rng.random_range(-width * 0.1..width * 1.1),
                y: rng.random_range(-height * 0.1..height * 1.1),
                z: rng.random_range(-60.0..60.0),
                target_x: rng.random_range(text_start_x..text_start_x + text_width),
                target_y: text_y + rng.random_range(-40.0..40.0),
                target_z: rng.random_range(-20.0..20.0),
                // Random Cybertronian symbols for non-text particles
                symbol: CybertronianMapper::map_random_symbol(rng),
                target_symbol: CybertronianMapper::map_random_symbol(rng),
                vx: rng.random_range(-3.5..3.5),
                vy: rng.random_range(-3.5..3.5),
                vz: rng.random_range(-2.0..2.0),
                opacity: 0.0,
                scale: rng.random_range(0.3..0.9),
                rotation: rng.random_range(0.0..360.0),
                is_text: false,
                particle_type,
                energy: rng.random_range(0.5..1.5),
                age: 0.0,
                life,
            });
        }
        particles
    }

    fn initialize_connections(particles: &[Particle], _rng: &mut ThreadRng) -> Vec<Connection> {
        let mut connections = Vec::new();

        // Find all text particle indices
        let core_indices: Vec<usize> = particles
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_text)
            .map(|(i, _)| i)
            .collect();

        // Connect sequential text particles
        for i in 0..core_indices.len().saturating_sub(1) {
            connections.push(Connection {
                particle1_idx: core_indices[i],
                particle2_idx: core_indices[i + 1],
                strength: 0.8,
                opacity: 0.0,
                active: false,
            });
        }

        // Add extra connections to create mesh look
        for i in 0..core_indices.len().saturating_sub(2) {
            connections.push(Connection {
                particle1_idx: core_indices[i],
                particle2_idx: core_indices[i + 2],
                strength: 0.4,
                opacity: 0.0,
                active: false,
            });
        }

        connections
    }

    fn random_char(rng: &mut impl Rng) -> char {
        CybertronianMapper::map_random_symbol(rng)
    }

    fn update_connections(
        &self,
        connections: &mut [Connection],
        _particles: &[Particle],
        phase: &AnimationPhase,
        progress: f32,
        _rng: &mut ThreadRng,
    ) {
        for conn in connections.iter_mut() {
            match phase {
                AnimationPhase::Scatter(_) => {
                    conn.active = false;
                    conn.opacity = (conn.opacity * 0.8).max(0.0);
                }
                AnimationPhase::Converge(sub) => {
                    conn.active = true;
                    let target_opacity = match sub {
                        ConvergeSubPhase::Alignment => 0.1,
                        ConvergeSubPhase::Formation => 0.3,
                        ConvergeSubPhase::Refinement => 0.5,
                        ConvergeSubPhase::Solidification => 0.6,
                    };
                    conn.opacity = (conn.opacity * 0.9 + target_opacity * 0.1).min(0.6);
                    conn.strength = (conn.strength * 0.95 + 1.0 * 0.05).min(1.0);
                }
                AnimationPhase::Stable(_) => {
                    conn.active = true;
                    conn.opacity = 0.5 + (progress * 3.0).sin() * 0.1;
                    conn.strength = 1.0;
                }
                AnimationPhase::Dissolve(_) => {
                    conn.active = false;
                    conn.opacity *= 0.9;
                    conn.strength *= 0.95;
                }
            }

            // Ensure bounded values
            conn.opacity = conn.opacity.clamp(0.0, 1.0);
            conn.strength = conn.strength.clamp(0.0, 1.0);
        }
    }

    fn update_scatter_phase(
        &self,
        particles: &mut [Particle],
        sub_phase: ScatterSubPhase,
        progress: f32,
        gravity_center: (f32, f32),
        rng: &mut ThreadRng,
    ) {
        let delta_time = 0.02;

        for p in particles.iter_mut() {
            p.age += delta_time;

            // Apply velocities
            p.x += p.vx * delta_time * 60.0;
            p.y += p.vy * delta_time * 60.0;
            p.z += p.vz * delta_time * 60.0;

            // Apply velocity damping
            p.vx *= 0.96;
            p.vy *= 0.96;
            p.vz *= 0.96;

            p.rotation = (p.rotation + p.vx * 2.0) % 360.0;

            match sub_phase {
                ScatterSubPhase::Initial => {
                    // Fade in particles
                    p.opacity = (p.opacity + delta_time * 2.0).min(0.7);

                    // Random movement bursts
                    if rng.random_bool(0.06) {
                        p.vx += rng.random_range(-1.5..1.5);
                        p.vy += rng.random_range(-1.5..1.5);
                        p.vz += rng.random_range(-0.8..0.8);
                    }

                    // Symbol randomization with Cybertronian symbols
                    if rng.random_bool(0.15) {
                        p.symbol = Self::random_char(rng);
                    }
                }
                ScatterSubPhase::Expansion => {
                    // Reduce opacity slightly
                    p.opacity = (p.opacity - delta_time * 0.5).max(0.1);

                    // Apply repulsive force from gravity center
                    let dx = p.x - gravity_center.0;
                    let dy = p.y - gravity_center.1;
                    let dist_sq = (dx * dx + dy * dy).max(1.0);
                    let force = 80.0 / dist_sq;

                    p.vx += dx * force * p.energy * delta_time;
                    p.vy += dy * force * p.energy * delta_time;

                    if rng.random_bool(0.1) {
                        p.symbol = Self::random_char(rng);
                    }
                }
                ScatterSubPhase::Contraction => {
                    // Increase opacity
                    p.opacity = (p.opacity + delta_time * 1.0).min(0.8);

                    // Apply attractive force toward gravity center
                    let dx = gravity_center.0 - p.x;
                    let dy = gravity_center.1 - p.y;
                    let attraction = 0.1 + progress * 0.2;

                    p.vx += dx * attraction * p.energy * delta_time;
                    p.vy += dy * attraction * p.energy * delta_time;

                    // Text particles begin showing their target Cybertronian symbols
                    if p.is_text && rng.random_bool(0.03) {
                        p.symbol = p.target_symbol;
                    } else if rng.random_bool(0.12) {
                        p.symbol = Self::random_char(rng);
                    }
                }
                ScatterSubPhase::PreConverge => {
                    let mut target_vx = p.vx;
                    let mut target_vy = p.vy;
                    let mut target_vz = p.vz;

                    if p.is_text {
                        // Text particles start moving toward final positions
                        let dx = p.target_x - p.x;
                        let dy = p.target_y - p.y;
                        let dz = p.target_z - p.z;

                        target_vx += dx * 0.5;
                        target_vy += dy * 0.5;
                        target_vz += dz * 0.5;

                        // Start aligning rotation and adjusting scale/opacity
                        p.rotation *= 0.85;
                        p.scale = p.scale * 0.9 + 0.4 * 0.1;
                        p.opacity = p.opacity * 0.9 + 0.6 * 0.1;

                        if rng.random_bool(0.15) {
                            p.symbol = p.target_symbol;
                        }
                    } else {
                        // Non-text particles move toward central area
                        let text_center_x = self.width / 2.0;
                        let text_center_y = self.height * 0.6;
                        let dx_text = text_center_x - p.x;
                        let dy_text = text_center_y - p.y;
                        let dx_grav = gravity_center.0 - p.x;
                        let dy_grav = gravity_center.1 - p.y;

                        target_vx += (dx_text * 0.005 + dx_grav * 0.002) * 60.0 * delta_time;
                        target_vy += (dy_text * 0.005 + dy_grav * 0.002) * 60.0 * delta_time;
                        target_vz += (-p.z * 0.01) * 60.0 * delta_time;

                        p.opacity = (p.opacity - delta_time * 0.8).max(0.05);

                        if rng.random_bool(0.1) {
                            p.symbol = Self::random_char(rng);
                        }
                    }

                    // Apply target velocities gradually
                    let move_factor = 0.5 + progress * 0.5;
                    p.vx = p.vx * (1.0 - move_factor * delta_time)
                        + target_vx * move_factor * delta_time;
                    p.vy = p.vy * (1.0 - move_factor * delta_time)
                        + target_vy * move_factor * delta_time;
                    p.vz = p.vz * (1.0 - move_factor * delta_time)
                        + target_vz * move_factor * delta_time;
                }
            }

            // Fade out expired particles
            if !p.is_text && p.age > p.life {
                p.opacity = (p.opacity - delta_time * 2.0).max(0.0);
            }

            // Clamp values to ensure bounds
            p.vx = p.vx.clamp(-6.0, 6.0);
            p.vy = p.vy.clamp(-6.0, 6.0);
            p.vz = p.vz.clamp(-4.0, 4.0);
            p.scale = p.scale.clamp(0.1, 2.5);
            p.opacity = p.opacity.clamp(0.0, 1.0);
        }
    }

    fn update_converge_phase(
        &self,
        particles: &mut [Particle],
        sub_phase: ConvergeSubPhase,
        _progress: f32,
        gravity_center: (f32, f32),
        rng: &mut ThreadRng,
    ) {
        let delta_time = 0.02;
        let base_converge_speed = 0.1;
        let speed_factor = match sub_phase {
            ConvergeSubPhase::Alignment => 0.6,
            ConvergeSubPhase::Formation => 1.0,
            ConvergeSubPhase::Refinement => 1.5,
            ConvergeSubPhase::Solidification => 2.0,
        };
        let converge_speed = base_converge_speed * speed_factor;

        for p in particles.iter_mut() {
            p.age += delta_time;

            // Apply velocities
            p.x += p.vx * delta_time * 60.0;
            p.y += p.vy * delta_time * 60.0;
            p.z += p.vz * delta_time * 60.0;

            // Apply velocity damping
            p.vx *= 0.94;
            p.vy *= 0.94;
            p.vz *= 0.94;

            if p.is_text {
                // Text particles move precisely to target
                let dx = p.target_x - p.x;
                let dy = p.target_y - p.y;
                let dz = p.target_z - p.z;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);
                let force_factor = (dist / 10.0).clamp(0.5, 2.0) * converge_speed;

                p.vx += dx * force_factor * delta_time;
                p.vy += dy * force_factor * delta_time;
                p.vz += dz * force_factor * delta_time;

                // Start showing target Cybertronian symbol more consistently
                if matches!(
                    sub_phase,
                    ConvergeSubPhase::Refinement | ConvergeSubPhase::Solidification
                ) || rng.random_bool(0.3)
                {
                    p.symbol = p.target_symbol;
                } else if rng.random_bool(0.1) {
                    p.symbol = Self::random_char(rng);
                }

                // Adjust scale based on phase
                let target_scale = match sub_phase {
                    ConvergeSubPhase::Alignment => 0.4,
                    ConvergeSubPhase::Formation => 0.7,
                    ConvergeSubPhase::Refinement => 1.0,
                    ConvergeSubPhase::Solidification => 1.1,
                };
                p.scale = p.scale * 0.85 + target_scale * 0.15;

                // Adjust opacity based on phase
                let target_opacity = match sub_phase {
                    ConvergeSubPhase::Solidification => 1.0,
                    _ => 0.8,
                };
                p.opacity = (p.opacity * 0.8 + target_opacity * 0.2).min(1.0);

                // Reduce rotation
                p.rotation *= 0.7;
            } else {
                // Non-text particles have type-specific behavior
                match p.particle_type {
                    ParticleType::Orbiter => {
                        // Orbit around text center
                        let text_center_x = self.width / 2.0;
                        let text_center_y = self.height * 0.6;
                        let angle = p.age * (0.5 + p.energy * 0.3);
                        let radius_base = 40.0 + (p.target_x % 30.0);
                        let radius_variation = (p.age * 0.8).sin() * 15.0 * p.energy;
                        let orbit_radius = radius_base + radius_variation;

                        let target_x = text_center_x + angle.cos() * orbit_radius;
                        let target_y = text_center_y + angle.sin() * orbit_radius * 0.7;

                        p.vx += (target_x - p.x) * 0.05 * delta_time * 60.0;
                        p.vy += (target_y - p.y) * 0.05 * delta_time * 60.0;
                        p.vz *= 0.9;

                        p.opacity = (p.opacity * 0.97).clamp(0.0, 0.4);
                    }
                    ParticleType::Swarm => {
                        // Swarm around gravity center with turbulence
                        let dx_grav = gravity_center.0 - p.x;
                        let dy_grav = gravity_center.1 - p.y;

                        p.vx += dx_grav * 0.008 * delta_time * 60.0;
                        p.vy += dy_grav * 0.008 * delta_time * 60.0;

                        // Add turbulence
                        let turbulence_angle = p.age * p.energy * 1.5;
                        p.vx += turbulence_angle.sin() * 0.2 * delta_time * 60.0;
                        p.vy += turbulence_angle.cos() * 0.2 * delta_time * 60.0;

                        p.opacity = (p.opacity * 0.96).clamp(0.0, 0.3);
                    }
                    ParticleType::Fragment | ParticleType::Connector => {
                        // Initial burst then fade
                        if p.age < 0.5 {
                            p.vx += rng.random_range(-1.0..1.0) * p.energy;
                            p.vy += rng.random_range(-1.0..1.0) * p.energy;
                            p.vz += rng.random_range(-0.5..0.5) * p.energy;
                        }

                        p.opacity = (p.opacity - delta_time * 3.0).max(0.0);
                        p.scale *= 0.95;
                        p.rotation += p.vx * 5.0;
                    }
                    ParticleType::Core => {} // Should not occur for non-text
                }

                // Non-text particles continue to randomize Cybertronian symbols
                if p.opacity > 0.0 && rng.random_bool(0.08) {
                    p.symbol = Self::random_char(rng);
                }
            }

            // Clamp values
            p.vx = p.vx.clamp(-5.0, 5.0);
            p.vy = p.vy.clamp(-5.0, 5.0);
            p.vz = p.vz.clamp(-3.0, 3.0);
            p.scale = p.scale.clamp(0.0, 2.0);
            p.opacity = p.opacity.clamp(0.0, 1.0);
        }
    }

    fn update_stable_phase(
        &self,
        particles: &mut [Particle],
        sub_phase: StableSubPhase,
        _progress: f32,
        _gravity_center: (f32, f32),
        rng: &mut ThreadRng,
    ) {
        let delta_time = 0.02;

        // Create temporary mapping for text particles to avoid borrow checker issues
        let text_particle_targets: Vec<(f32, f32, char)> = particles
            .iter()
            .filter(|p| p.is_text)
            .map(|p| (p.target_x, p.target_y, p.target_symbol))
            .collect();

        for p in particles.iter_mut() {
            p.age += delta_time;

            // Apply velocity damping
            p.vx *= 0.85;
            p.vy *= 0.85;
            p.vz *= 0.85;

            // Apply velocities
            p.x += p.vx * delta_time * 60.0;
            p.y += p.vy * delta_time * 60.0;
            p.z += p.vz * delta_time * 60.0;

            if p.is_text {
                // Text particles fully formed with subtle effects
                p.symbol = p.target_symbol; // Always show the Cybertronian target symbol
                p.opacity = 1.0;

                let base_x = p.target_x;
                let base_y = p.target_y;
                let base_z = p.target_z;

                match sub_phase {
                    StableSubPhase::Pulse => {
                        // Pulsing effect
                        p.scale = 1.0 + (p.age * 8.0).sin() * 0.08;
                        p.z = base_z + (p.age * 10.0).cos() * 1.0;
                        p.x = base_x + (p.age * 2.0).sin() * 0.2;
                        p.y = base_y + (p.age * 2.5).cos() * 0.2;
                    }
                    StableSubPhase::Orbit => {
                        // Orbital motion
                        let angle = p.age * 1.5;
                        let orbit_radius = 0.8;
                        p.x = base_x + angle.sin() * orbit_radius;
                        p.y = base_y + angle.cos() * orbit_radius;
                        p.z = base_z + (angle * 2.0).sin() * 1.5;
                        p.rotation = angle * 10.0;
                        p.scale = 1.0 + (angle * 1.5).cos() * 0.05;
                    }
                    StableSubPhase::Ripple => {
                        // Safe ripple effect using the temporary mapping
                        let particle_index = text_particle_targets
                            .iter()
                            .position(|(tx, _ty, ts)| *ts == p.target_symbol && *tx == p.target_x)
                            .unwrap_or(0);

                        let ripple_offset = particle_index as f32 * 0.5;
                        let ripple_val = ((p.age * 6.0) - ripple_offset).sin();

                        p.y = base_y + ripple_val * 2.0;
                        p.z = base_z + ripple_val.abs() * 3.0;
                        p.scale = 1.0 + ripple_val.abs() * 0.15;
                    }
                    StableSubPhase::Shimmer => {
                        // Random shimmer effect
                        p.z = base_z + rng.random_range(-1.5..1.5);
                        p.scale = 1.0 + rng.random_range(-0.03..0.03);

                        if rng.random_bool(0.05) {
                            p.symbol = Self::random_char(rng);
                        } else {
                            p.symbol = p.target_symbol;
                        }

                        p.x = base_x + rng.random_range(-0.2..0.2);
                        p.y = base_y + rng.random_range(-0.2..0.2);
                    }
                    StableSubPhase::PreDissolve => {
                        // Preparation for dissolve phase
                        if rng.random_bool(0.05) {
                            p.vx += rng.random_range(-0.3..0.3);
                            p.vy += rng.random_range(-0.3..0.3);
                            p.vz += rng.random_range(-0.2..0.2);
                        }

                        p.x = base_x + p.vx;
                        p.y = base_y + p.vy;
                        p.z = base_z + p.vz;

                        if rng.random_bool(0.03) {
                            p.symbol = Self::random_char(rng);
                        } else {
                            p.symbol = p.target_symbol;
                        }

                        p.scale = 1.0 + rng.random_range(-0.05..0.05);
                        p.opacity = 1.0 - rng.random_range(0.0..0.1);
                    }
                }

                // Ensure positions stay within bounds of target
                p.x = p.x.clamp(base_x - 2.0, base_x + 2.0);
                p.y = p.y.clamp(base_y - 2.0, base_y + 2.0);
                p.z = p.z.clamp(base_z - 5.0, base_z + 5.0);
                p.scale = p.scale.clamp(0.8, 1.2);
            } else {
                // Non-text particles gradually fade
                match p.particle_type {
                    ParticleType::Orbiter => {
                        p.opacity *= 0.99;
                    }
                    ParticleType::Swarm => {
                        p.opacity *= 0.98;
                    }
                    _ => {
                        p.opacity *= 0.97;
                    }
                }

                // Occasional symbol randomization with Cybertronian symbols
                if p.opacity > 0.0 && rng.random_bool(0.03) {
                    p.symbol = Self::random_char(rng);
                }
            }

            // Clamp values
            p.vx = p.vx.clamp(-3.0, 3.0);
            p.vy = p.vy.clamp(-3.0, 3.0);
            p.vz = p.vz.clamp(-2.0, 2.0);
            p.opacity = p.opacity.clamp(0.0, 1.0);
            p.scale = p.scale.clamp(0.0, 2.0);
        }
    }

    fn update_dissolve_phase(
        &self,
        particles: &mut [Particle],
        _connections: &[Connection],
        sub_phase: DissolveSubPhase,
        _progress: f32,
        _gravity_center: (f32, f32),
        rng: &mut ThreadRng,
    ) {
        let delta_time = 0.02;

        // Set explosion force based on subphase
        let explosion_factor = match sub_phase {
            DissolveSubPhase::Fracture => 0.2,
            DissolveSubPhase::Explosion => 1.5,
            DissolveSubPhase::Dispersion => 0.8,
            DissolveSubPhase::Fade => 0.3,
        };

        // Set fade rate based on subphase
        let fade_rate: f32 = match sub_phase {
            DissolveSubPhase::Fracture => 0.99,
            DissolveSubPhase::Explosion => 0.97,
            DissolveSubPhase::Dispersion => 0.95,
            DissolveSubPhase::Fade => 0.92,
        };

        for p in particles.iter_mut() {
            p.age += delta_time;

            // Calculate explosion center (original position for text particles)
            let origin_x = if p.is_text { p.target_x } else { p.x };
            let origin_y = if p.is_text { p.target_y } else { p.y };

            // Calculate explosion forces
            let dx = p.x - origin_x;
            let dy = p.y - origin_y;
            let dist_sq = (dx * dx + dy * dy).max(1.0);
            let force = explosion_factor * 80.0 / dist_sq;

            // Apply explosion forces with randomization
            p.vx += dx * force * p.energy * rng.random_range(0.7..1.3) * delta_time;
            p.vy += dy * force * p.energy * rng.random_range(0.7..1.3) * delta_time;

            // Add Z-axis movement during explosion
            if matches!(sub_phase, DissolveSubPhase::Explosion) {
                p.vz += rng.random_range(-1.0..1.0) * explosion_factor * p.energy * delta_time;
            }

            // Apply velocities
            p.x += p.vx * delta_time * 60.0;
            p.y += p.vy * delta_time * 60.0;
            p.z += p.vz * delta_time * 60.0;

            // Apply velocity damping
            p.vx *= 0.985;
            p.vy *= 0.985;
            p.vz *= 0.985;

            // Apply fade with power function
            p.opacity *= fade_rate.powf(delta_time * 60.0);

            // Shrink during fade phase
            if matches!(sub_phase, DissolveSubPhase::Fade) {
                p.scale *= 0.97f32.powf(delta_time * 60.0);
            }

            p.rotation = (p.rotation + p.vx * 4.0) % 360.0;

            // Frequent symbol randomization with Cybertronian symbols
            if rng.random_bool(0.2) {
                p.symbol = Self::random_char(rng);
            }

            // Clamp values
            p.vx = p.vx.clamp(-8.0, 8.0);
            p.vy = p.vy.clamp(-8.0, 8.0);
            p.vz = p.vz.clamp(-6.0, 6.0);
            p.opacity = p.opacity.clamp(0.0, 1.0);
            p.scale = p.scale.clamp(0.0, 2.5);
        }
    }

    fn schedule_tick(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let handle = Timeout::new(16, move || link.send_message(Msg::Tick));
        *self.tick_timer.borrow_mut() = Some(handle);
    }

    fn schedule_phase_advance(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let handle = Timeout::new(30, move || link.send_message(Msg::AdvancePhase));
        *self.phase_timer.borrow_mut() = Some(handle);
    }

    fn schedule_reset(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let handle = Timeout::new(1500, move || link.send_message(Msg::Reset));
        *self.reset_timer.borrow_mut() = Some(handle);
    }

    fn render_particles(&self) -> Html {
        let particles = self.particles.borrow();
        let color_scheme = self.color_scheme.borrow();
        let phase = self.phase.borrow();

        html! {
            <div class="transformium-particles">
                { particles.iter().filter(|p| p.opacity > 0.01).map(|p| {
                    // Calculate visual properties
                    let z_index = ((p.z + 100.0).clamp(0.0, 200.0) / 2.0).round() as i32;

                    // Determine color based on particle type and phase
                    let color = match (p.is_text, &p.particle_type, &*phase) {
                        (true, _, AnimationPhase::Stable(StableSubPhase::Ripple | StableSubPhase::Shimmer)) => color_scheme.accent.clone(),
                        (true, _, _) => color_scheme.primary.clone(),
                        (false, ParticleType::Orbiter, _) => color_scheme.accent.clone(),
                        _ => color_scheme.secondary.clone(),
                    };

                    // Apply 3D perspective scaling
                    let z_scale = (1.0 + p.z / 500.0).clamp(0.4, 2.5);
                    let font_size = (14.0 * p.scale * z_scale).clamp(4.0, 28.0);

                    // Create style string
                    let style = format!(
                        "position: absolute; left: {:.1}px; top: {:.1}px; opacity: {:.2}; z-index: {}; \
                         transform: translate3d(-50%, -50%, {:.1}px) scale({:.2}) rotate({:.1}deg); \
                         color: {}; font-size: {:.1}px; \
                         will-change: transform, opacity;",
                         p.x, p.y, p.opacity, z_index,
                         p.z, 1.0, p.rotation, color, font_size
                    );

                    // Apply CSS classes
                    let mut classes = vec!["particle".to_string()];
                    if p.is_text && matches!(*phase, AnimationPhase::Stable(_)) &&
                       !matches!(*phase, AnimationPhase::Stable(StableSubPhase::PreDissolve | StableSubPhase::Shimmer)) {
                        classes.push("metallic-glow".to_string());
                    }

                    html! { <span class={classes.join(" ")} style={style}>{ p.symbol }</span> }
                }).collect::<Html>() }
            </div>
        }
    }

    fn render_connections(&self) -> Html {
        let particles_borrow = self.particles.borrow();
        let particles = &*particles_borrow;
        let connections = self.connections.borrow();
        let color_scheme = self.color_scheme.borrow();

        html! {
            <svg class="connections-layer" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: 1; pointer-events: none; opacity: 0.7;">
                { connections.iter()
                    .filter(|c| c.active && c.opacity > 0.01)
                    .filter_map(|conn| {
                        particles.get(conn.particle1_idx).and_then(|p1| {
                            particles.get(conn.particle2_idx).map(|p2| (p1, p2))
                        }).map(|(p1, p2)| {
                            let stroke_color = color_scheme.primary.clone();
                            let stroke_width = (conn.strength * 0.6).clamp(0.1, 0.6);

                            html! {
                                <line x1={p1.x.to_string()} y1={p1.y.to_string()}
                                      x2={p2.x.to_string()} y2={p2.y.to_string()}
                                      stroke={stroke_color} stroke-width={stroke_width.to_string()}
                                      stroke-opacity={(conn.opacity * 0.7).to_string()}
                                      stroke-linecap="round"
                                      class="connection-line"
                                />
                            }
                        })
                    }).collect::<Html>()
                }
            </svg>
        }
    }
}
