mod constants;
mod physics;
mod state_machine;
mod types;

use leptos::prelude::*;
use rand::Rng;
use std::time::Duration;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

use constants::{EXTRA_PARTICLES, HEIGHT, TARGET_TEXT, WIDTH, map_char, random_symbol};
use physics::{
    update_connections, update_converge_phase, update_dissolve_phase, update_scatter_phase,
    update_stable_phase,
};
use types::{AnimationPhase, ColorScheme, Connection, Particle, ParticleType, StableSubPhase};

fn initialize_particles() -> Vec<Particle> {
    let mut rng = rand::rng();
    let mut particles = Vec::with_capacity(TARGET_TEXT.len() + EXTRA_PARTICLES);

    let char_count = TARGET_TEXT.chars().count();
    let text_width = char_count as f32 * 20.0;
    let text_start_x = (WIDTH - text_width) / 2.0 + 10.0;
    let text_y = HEIGHT * 0.6;

    for (i, ch) in TARGET_TEXT.chars().enumerate() {
        particles.push(Particle {
            x: rng.random_range(0.0..WIDTH),
            y: rng.random_range(0.0..HEIGHT),
            z: rng.random_range(-30.0..30.0),
            target_x: (i as f32).mul_add(20.0, text_start_x),
            target_y: text_y,
            target_z: 0.0,
            symbol: random_symbol(&mut rng),
            target_symbol: map_char(ch),
            vx: rng.random_range(-2.5..2.5),
            vy: rng.random_range(-2.5..2.5),
            vz: rng.random_range(-1.5..1.5),
            opacity: 0.0,
            scale: rng.random_range(0.2..0.8),
            rotation: rng.random_range(0.0..360.0),
            is_text: true,
            kind: ParticleType::Core,
            energy: rng.random_range(0.8..1.2),
            age: 0.0,
            life: rng.random_range(5.0..10.0),
        });
    }

    for i in 0..EXTRA_PARTICLES {
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
            x: rng.random_range(-WIDTH * 0.1..WIDTH * 1.1),
            y: rng.random_range(-HEIGHT * 0.1..HEIGHT * 1.1),
            z: rng.random_range(-60.0..60.0),
            target_x: rng.random_range(text_start_x..text_start_x + text_width),
            target_y: text_y + rng.random_range(-40.0..40.0),
            target_z: rng.random_range(-20.0..20.0),
            symbol: random_symbol(&mut rng),
            target_symbol: random_symbol(&mut rng),
            vx: rng.random_range(-3.5..3.5),
            vy: rng.random_range(-3.5..3.5),
            vz: rng.random_range(-2.0..2.0),
            opacity: 0.0,
            scale: rng.random_range(0.3..0.9),
            rotation: rng.random_range(0.0..360.0),
            is_text: false,
            kind: particle_type,
            energy: rng.random_range(0.5..1.5),
            age: 0.0,
            life,
        });
    }
    particles
}

fn initialize_connections(particles: &[Particle]) -> Vec<Connection> {
    let core_indices: Vec<usize> = particles
        .iter()
        .enumerate()
        .filter(|(_, p)| p.is_text)
        .map(|(i, _)| i)
        .collect();

    let mut connections = Vec::new();
    for i in 0..core_indices.len().saturating_sub(1) {
        connections.push(Connection {
            particle1_idx: core_indices[i],
            particle2_idx: core_indices[i + 1],
            strength: 0.8,
            opacity: 0.0,
            active: false,
        });
    }
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

fn set_interval<F: FnMut() + 'static>(callback: F, duration: Duration) -> i32 {
    let window = web_sys::window().unwrap();
    let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
    let id = window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            duration.as_millis() as i32,
        )
        .unwrap();
    closure.forget();
    id
}

#[component]
pub fn TypingAnimation() -> impl IntoView {
    let particles = RwSignal::new(initialize_particles());
    let connections = RwSignal::new(particles.with_untracked(|p| initialize_connections(p)));
    let phase = RwSignal::new(AnimationPhase::initial());
    let progress = RwSignal::new(0.0f32);
    let gravity_center = RwSignal::new((WIDTH / 2.0, HEIGHT / 2.0));
    let phase_advancing = RwSignal::new(false);

    Effect::new(move |_| {
        set_interval(
            move || {
                let mut rng = rand::rng();
                let current_phase = phase.get();
                let current_progress = progress.get();
                let grav = gravity_center.get();

                progress.set(current_progress + 0.02);

                particles.update(|ps| match &current_phase {
                    AnimationPhase::Scatter(sub) => {
                        update_scatter_phase(ps, *sub, current_progress, grav, &mut rng);
                    }
                    AnimationPhase::Converge(sub) => {
                        update_converge_phase(ps, *sub, current_progress, grav, &mut rng);
                    }
                    AnimationPhase::Stable(sub) => {
                        update_stable_phase(ps, *sub, current_progress, &mut rng);
                    }
                    AnimationPhase::Dissolve(sub) => update_dissolve_phase(ps, *sub, &mut rng),
                });

                connections.update(|cs| update_connections(cs, current_phase, current_progress));

                if current_progress > current_phase.duration() && !phase_advancing.get() {
                    phase_advancing.set(true);
                    if let Some(next) = current_phase.next() {
                        phase.set(next);
                    } else {
                        let new_particles = initialize_particles();
                        let new_connections = initialize_connections(&new_particles);
                        particles.set(new_particles);
                        connections.set(new_connections);
                        phase.set(AnimationPhase::initial());
                    }
                    progress.set(0.0);
                    phase_advancing.set(false);
                }
            },
            Duration::from_millis(16),
        );
    });

    Effect::new(move |_| {
        set_interval(
            move || {
                let mut rng = rand::rng();
                gravity_center.set((
                    WIDTH / 2.0 + rng.random_range(-30.0..30.0),
                    HEIGHT / 2.0 + rng.random_range(-20.0..20.0),
                ));
            },
            Duration::from_millis(2500),
        );
    });

    let scheme = ColorScheme::DEFAULT;

    view! {
        <div class="transformium-field" style=format!("background-color: {}; transform-style: preserve-3d;", scheme.background)>
            <svg class="connections-layer" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: 1; pointer-events: none; opacity: 0.7;">
                {move || connections.with(|cs| particles.with(|ps| {
                    cs.iter()
                        .filter(|c| c.active && c.opacity > 0.01)
                        .filter_map(|conn| ps.get(conn.particle1_idx).zip(ps.get(conn.particle2_idx)).map(|(p1, p2)| (p1, p2, conn)))
                        .map(|(p1, p2, conn)| {
                            let stroke_width = (conn.strength * 0.6).clamp(0.1, 0.6);
                            view! {
                                <line
                                    x1=p1.x.to_string() y1=p1.y.to_string()
                                    x2=p2.x.to_string() y2=p2.y.to_string()
                                    stroke=scheme.primary stroke-width=stroke_width.to_string()
                                    stroke-opacity=(conn.opacity * 0.7).to_string() stroke-linecap="round"
                                />
                            }
                        })
                        .collect_view()
                }))}
            </svg>
            <div class="transformium-particles">
                {move || {
                    let current_phase = phase.get();
                    particles.with(|ps| {
                        ps.iter()
                            .filter(|p| p.opacity > 0.01)
                            .map(|p| {
                                let z_index = ((p.z + 100.0).clamp(0.0, 200.0) / 2.0).round() as i32;
                                let color = match (p.is_text, &p.kind, &current_phase) {
                                    (true, _, AnimationPhase::Stable(StableSubPhase::Ripple | StableSubPhase::Shimmer)) => scheme.accent,
                                    (true, _, _) => scheme.primary,
                                    (false, ParticleType::Orbiter, _) => scheme.accent,
                                    _ => scheme.secondary,
                                };
                                let z_scale = (1.0 + p.z / 500.0).clamp(0.4, 2.5);
                                let font_size = (14.0 * p.scale * z_scale).clamp(4.0, 28.0);
                                let style = format!(
                                    "position: absolute; left: {:.1}px; top: {:.1}px; opacity: {:.2}; z-index: {}; \
                                     transform: translate3d(-50%, -50%, {:.1}px) scale({:.2}) rotate({:.1}deg); \
                                     color: {}; font-size: {:.1}px; will-change: transform, opacity;",
                                    p.x, p.y, p.opacity, z_index, p.z, 1.0, p.rotation, color, font_size
                                );
                                let classes = if p.is_text && matches!(current_phase, AnimationPhase::Stable(s) if !matches!(s, StableSubPhase::PreDissolve | StableSubPhase::Shimmer)) {
                                    "particle metallic-glow"
                                } else {
                                    "particle"
                                };
                                view! { <span class=classes style=style>{p.symbol.to_string()}</span> }
                            })
                            .collect_view()
                    })
                }}
            </div>
        </div>
    }
}
