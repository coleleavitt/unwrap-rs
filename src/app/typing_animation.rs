use super::YewComponent;
use yew::prelude::*;
use gloo_timers::callback::Timeout;
use std::rc::Rc;
use std::cell::RefCell;
use rand::Rng;

/// Transformium-inspired animation that mimics the fluid metal transformation
pub struct TypingAnimation {
    // Particle system for the transformation effect
    particles: Rc<RefCell<Vec<Particle>>>,
    // Target text to display
    target_text: String,
    // Current animation phase
    phase: Rc<RefCell<AnimationPhase>>,
    // Animation progress counter
    progress: Rc<RefCell<f32>>,
    // Whether animation is complete
    is_complete: Rc<RefCell<bool>>,
}

/// A single particle in the transformation effect
struct Particle {
    // Current position
    x: f32,
    y: f32,
    // Target position
    target_x: f32,
    target_y: f32,
    // Visual character
    symbol: char,
    // Target character
    target_symbol: char,
    // Particle velocity
    vx: f32,
    vy: f32,
    // Visual properties
    opacity: f32,
    scale: f32,
    // Whether this particle is part of the final text
    is_text: bool,
}

/// Animation phases
enum AnimationPhase {
    // Initial scatter of particles
    Scatter,
    // Particles converging to form text
    Converge,
    // Text fully formed and stable
    Stable,
    // Text breaking apart
    Dissolve,
}

pub enum Msg {
    Tick,
    Reset,
    AdvancePhase,
}

impl YewComponent for TypingAnimation {
    fn render(&self) -> Html {
        html! {
            <div class="transformium-container">
                <span class="base-text">{"Result"}</span>
                {self.render_particles()}
            </div>
        }
    }

    fn create_component() -> Self {
        let target_text = ".unwrap()";
        let particles = Self::initialize_particles(target_text, 150); // 150 particles

        TypingAnimation {
            particles: Rc::new(RefCell::new(particles)),
            target_text: target_text.to_string(),
            phase: Rc::new(RefCell::new(AnimationPhase::Scatter)),
            progress: Rc::new(RefCell::new(0.0)),
            is_complete: Rc::new(RefCell::new(false)),
        }
    }
}

impl Component for TypingAnimation {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let component = Self::create_component();
        component.schedule_tick(ctx);
        component
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                let mut phase = self.phase.borrow_mut();
                let mut progress = self.progress.borrow_mut();
                let mut particles = self.particles.borrow_mut();

                // Update progress
                *progress += 0.02;

                // Update particles based on current phase
                match *phase {
                    AnimationPhase::Scatter => {
                        // Particles moving randomly
                        self.update_scatter_phase(&mut particles);

                        // Advance to next phase after some time
                        if *progress > 1.0 {
                            self.schedule_phase_advance(ctx);
                        }
                    },
                    AnimationPhase::Converge => {
                        // Particles converging to form text
                        self.update_converge_phase(&mut particles, *progress);

                        // Advance to stable phase when complete
                        if *progress > 2.0 {
                            self.schedule_phase_advance(ctx);
                        }
                    },
                    AnimationPhase::Stable => {
                        // Text fully formed and pulsing slightly
                        self.update_stable_phase(&mut particles, *progress);

                        // After a delay, start dissolving
                        if *progress > 3.5 {
                            self.schedule_phase_advance(ctx);
                        }
                    },
                    AnimationPhase::Dissolve => {
                        // Text breaking apart
                        self.update_dissolve_phase(&mut particles, *progress);

                        // Reset animation when complete
                        if *progress > 5.0 {
                            *self.is_complete.borrow_mut() = true;
                            self.schedule_reset(ctx);
                        }
                    }
                }

                self.schedule_tick(ctx);
                true
            },
            Msg::AdvancePhase => {
                let mut phase = self.phase.borrow_mut();
                let mut progress = self.progress.borrow_mut();

                // Advance to next phase
                *phase = match *phase {
                    AnimationPhase::Scatter => AnimationPhase::Converge,
                    AnimationPhase::Converge => AnimationPhase::Stable,
                    AnimationPhase::Stable => AnimationPhase::Dissolve,
                    AnimationPhase::Dissolve => AnimationPhase::Scatter,
                };

                // Reset progress for new phase
                *progress = 0.0;

                true
            },
            Msg::Reset => {
                // Reset animation state
                *self.phase.borrow_mut() = AnimationPhase::Scatter;
                *self.progress.borrow_mut() = 0.0;
                *self.is_complete.borrow_mut() = false;

                // Reinitialize particles
                *self.particles.borrow_mut() = Self::initialize_particles(&self.target_text, 150);

                self.schedule_tick(ctx);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.render()
    }
}

impl TypingAnimation {
    fn initialize_particles(target_text: &str, extra_particles: usize) -> Vec<Particle> {
        let mut rng = rand::thread_rng();
        let mut particles = Vec::new();

        // Create particles for each character in the target text
        let char_count = target_text.chars().count();
        for (i, ch) in target_text.chars().enumerate() {
            // Position in the final text
            let x = 10.0 + (i as f32 * 20.0);
            let y = 50.0;

            // Create a particle for this character
            particles.push(Particle {
                x: rng.gen_range(-100.0..200.0), // Random starting position
                y: rng.gen_range(-50.0..150.0),
                target_x: x,
                target_y: y,
                symbol: Self::random_char(&mut rng),
                target_symbol: ch,
                vx: rng.gen_range(-2.0..2.0),
                vy: rng.gen_range(-2.0..2.0),
                opacity: rng.gen_range(0.3..0.8),
                scale: rng.gen_range(0.5..1.5),
                is_text: true,
            });
        }

        // Add extra particles for the fluid effect
        for _ in 0..extra_particles {
            particles.push(Particle {
                x: rng.gen_range(-100.0..300.0),
                y: rng.gen_range(-100.0..200.0),
                target_x: rng.gen_range(0.0..(char_count as f32 * 20.0 + 20.0)),
                target_y: 50.0 + rng.gen_range(-20.0..20.0),
                symbol: Self::random_char(&mut rng),
                target_symbol: Self::random_char(&mut rng),
                vx: rng.gen_range(-3.0..3.0),
                vy: rng.gen_range(-3.0..3.0),
                opacity: rng.gen_range(0.1..0.5),
                scale: rng.gen_range(0.3..1.0),
                is_text: false,
            });
        }

        particles
    }

    fn random_char(rng: &mut impl Rng) -> char {
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()";
        chars.chars().nth(rng.gen_range(0..chars.len())).unwrap()
    }

    fn update_scatter_phase(&self, particles: &mut Vec<Particle>) {
        let mut rng = rand::thread_rng();

        // Update particle positions with random movement
        for particle in particles.iter_mut() {
            // Random movement
            particle.x += particle.vx;
            particle.y += particle.vy;

            // Randomly change velocity
            if rng.gen::<f32>() < 0.05 {
                particle.vx = rng.gen_range(-2.0..2.0);
                particle.vy = rng.gen_range(-2.0..2.0);
            }

            // Randomly change symbol
            if rng.gen::<f32>() < 0.1 {
                particle.symbol = Self::random_char(&mut rng);
            }

            // Adjust opacity and scale
            particle.opacity = (particle.opacity + rng.gen_range(-0.05..0.05)).clamp(0.1, 0.9);
            particle.scale = (particle.scale + rng.gen_range(-0.05..0.05)).clamp(0.3, 1.5);
        }
    }

    fn update_converge_phase(&self, particles: &mut Vec<Particle>, progress: f32) {
        let mut rng = rand::thread_rng();
        let convergence_factor = (progress / 2.0).min(1.0);

        for particle in particles.iter_mut() {
            // Move toward target position
            let dx = particle.target_x - particle.x;
            let dy = particle.target_y - particle.y;

            particle.x += dx * 0.05 * convergence_factor;
            particle.y += dy * 0.05 * convergence_factor;

            // Text particles converge to their final character
            if particle.is_text && rng.gen::<f32>() < convergence_factor * 0.2 {
                particle.symbol = particle.target_symbol;
            }

            // Non-text particles keep changing
            if !particle.is_text && rng.gen::<f32>() < 0.1 {
                particle.symbol = Self::random_char(&mut rng);
            }

            // Adjust properties
            if particle.is_text {
                particle.opacity = (0.5 + 0.5 * convergence_factor).min(1.0);
                particle.scale = (0.8 + 0.2 * convergence_factor).min(1.0);
            } else {
                particle.opacity = (particle.opacity * 0.98).max(0.1);
                particle.scale = (particle.scale * 0.99).max(0.3);
            }
        }
    }

    fn update_stable_phase(&self, particles: &mut Vec<Particle>, progress: f32) {
        let mut rng = rand::thread_rng();
        let pulse = (progress * 5.0).sin() * 0.1 + 0.9;

        for particle in particles.iter_mut() {
            if particle.is_text {
                // Text particles are stable but pulse slightly
                particle.symbol = particle.target_symbol;
                particle.x = particle.target_x + (progress * 7.0 + particle.target_x).sin() * 1.0;
                particle.y = particle.target_y + (progress * 5.0 + particle.target_y).cos() * 1.0;
                particle.opacity = 1.0;
                particle.scale = pulse;
            } else {
                // Non-text particles orbit around
                let angle = progress * 2.0 + particle.x * 0.1;
                particle.x += angle.sin() * 0.5;
                particle.y += angle.cos() * 0.5;

                // Randomly change symbol
                if rng.gen::<f32>() < 0.05 {
                    particle.symbol = Self::random_char(&mut rng);
                }

                particle.opacity = (particle.opacity * 0.99).max(0.05);
            }
        }
    }

    fn update_dissolve_phase(&self, particles: &mut Vec<Particle>, progress: f32) {
        let mut rng = rand::thread_rng();
        let dissolution_factor = (progress / 5.0).min(1.0);

        for particle in particles.iter_mut() {
            // Add increasing randomness to movement
            particle.vx += rng.gen_range(-0.2..0.2) * dissolution_factor;
            particle.vy += rng.gen_range(-0.2..0.2) * dissolution_factor;

            // Apply velocity
            particle.x += particle.vx;
            particle.y += particle.vy;

            // Randomize symbols more as dissolution progresses
            if rng.gen::<f32>() < 0.1 * dissolution_factor {
                particle.symbol = Self::random_char(&mut rng);
            }

            // Fade out
            particle.opacity = (particle.opacity * (1.0 - 0.02 * dissolution_factor)).max(0.0);

            // Vary scale
            if rng.gen::<f32>() < 0.1 {
                particle.scale += rng.gen_range(-0.1..0.1) * dissolution_factor;
                particle.scale = particle.scale.clamp(0.1, 1.5);
            }
        }
    }

    fn schedule_tick(&self, ctx: &Context<Self>) {
        const TICK_DELAY_MS: u32 = 16; // ~60fps

        let link = ctx.link().clone();
        let timeout = Timeout::new(TICK_DELAY_MS, move || {
            link.send_message(Msg::Tick);
        });
        timeout.forget();
    }

    fn schedule_phase_advance(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let timeout = Timeout::new(50, move || {
            link.send_message(Msg::AdvancePhase);
        });
        timeout.forget();
    }

    fn schedule_reset(&self, ctx: &Context<Self>) {
        const RESET_DELAY_MS: u32 = 1000;

        let link = ctx.link().clone();
        let timeout = Timeout::new(RESET_DELAY_MS, move || {
            link.send_message(Msg::Reset);
        });
        timeout.forget();
    }

    fn render_particles(&self) -> Html {
        let particles = self.particles.borrow();

        html! {
            <div class="transformium-field">
                {particles.iter().map(|particle| {
                    let style = format!(
                        "position: absolute; left: {}px; top: {}px; opacity: {}; transform: scale({}); color: {};",
                        particle.x,
                        particle.y,
                        particle.opacity,
                        particle.scale,
                        if particle.is_text { "#66d9ef" } else { "#4a9c8b" }
                    );

                    html! {
                        <span class="particle" style={style}>
                            {particle.symbol}
                        </span>
                    }
                }).collect::<Html>()}
            </div>
        }
    }
}
