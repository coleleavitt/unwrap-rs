use leptos::prelude::*;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const WIDTH: f32 = 300.0;
const HEIGHT: f32 = 150.0;
const EXTRA_PARTICLES: usize = 80;
const TARGET_TEXT: &str = ".unwrap()";

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

fn map_char(c: char) -> char {
    match c {
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
        _ => '⧠',
    }
}

fn random_symbol(rng: &mut impl Rng) -> char {
    let chars = "⏃ᗷᑕᗞ⟊⎎Ꮆ⋔⟙⟗Ꮶ⅃⏁ᑎ〇℘Ϙ尺⟅ナ⋒٧山〤Ꭹㄗค♭ᑢↁ⋿ℱᎶЂ|ן⊗≠ፐ∪∨พу⊼⋕⦿|ᒿ≡⫓⫔⏀⫛∞ⴤ•⦑⦒⎼⊕⧠";
    let index = rng.random_range(0..chars.len());
    chars.chars().nth(index).unwrap_or('⧠')
}

fn initialize_particles(rng: &mut ThreadRng) -> Vec<Particle> {
    let mut particles = Vec::new();
    let char_count = TARGET_TEXT.chars().count();
    let text_width = char_count as f32 * 20.0;
    let text_start_x = (WIDTH - text_width) / 2.0 + 10.0;
    let text_y = HEIGHT * 0.6;

    for (i, ch) in TARGET_TEXT.chars().enumerate() {
        let x = text_start_x + (i as f32 * 20.0);
        particles.push(Particle {
            x: rng.random_range(0.0..WIDTH),
            y: rng.random_range(0.0..HEIGHT),
            z: rng.random_range(-30.0..30.0),
            target_x: x,
            target_y: text_y,
            target_z: 0.0,
            symbol: random_symbol(rng),
            target_symbol: map_char(ch),
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
            symbol: random_symbol(rng),
            target_symbol: random_symbol(rng),
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

fn initialize_connections(particles: &[Particle]) -> Vec<Connection> {
    let mut connections = Vec::new();
    let core_indices: Vec<usize> = particles
        .iter()
        .enumerate()
        .filter(|(_, p)| p.is_text)
        .map(|(i, _)| i)
        .collect();

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

fn update_connections(connections: &mut [Connection], phase: &AnimationPhase, progress: f32) {
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
        conn.opacity = conn.opacity.clamp(0.0, 1.0);
        conn.strength = conn.strength.clamp(0.0, 1.0);
    }
}

fn update_scatter_phase(
    particles: &mut [Particle],
    sub_phase: ScatterSubPhase,
    progress: f32,
    gravity_center: (f32, f32),
    rng: &mut ThreadRng,
) {
    let delta_time = 0.02;
    for p in particles.iter_mut() {
        p.age += delta_time;
        p.x += p.vx * delta_time * 60.0;
        p.y += p.vy * delta_time * 60.0;
        p.z += p.vz * delta_time * 60.0;
        p.vx *= 0.96;
        p.vy *= 0.96;
        p.vz *= 0.96;
        p.rotation = (p.rotation + p.vx * 2.0) % 360.0;

        match sub_phase {
            ScatterSubPhase::Initial => {
                p.opacity = (p.opacity + delta_time * 2.0).min(0.7);
                if rng.random_bool(0.06) {
                    p.vx += rng.random_range(-1.5..1.5);
                    p.vy += rng.random_range(-1.5..1.5);
                    p.vz += rng.random_range(-0.8..0.8);
                }
                if rng.random_bool(0.15) {
                    p.symbol = random_symbol(rng);
                }
            }
            ScatterSubPhase::Expansion => {
                p.opacity = (p.opacity - delta_time * 0.5).max(0.1);
                let dx = p.x - gravity_center.0;
                let dy = p.y - gravity_center.1;
                let dist_sq = (dx * dx + dy * dy).max(1.0);
                let force = 80.0 / dist_sq;
                p.vx += dx * force * p.energy * delta_time;
                p.vy += dy * force * p.energy * delta_time;
                if rng.random_bool(0.1) {
                    p.symbol = random_symbol(rng);
                }
            }
            ScatterSubPhase::Contraction => {
                p.opacity = (p.opacity + delta_time * 1.0).min(0.8);
                let dx = gravity_center.0 - p.x;
                let dy = gravity_center.1 - p.y;
                let attraction = 0.1 + progress * 0.2;
                p.vx += dx * attraction * p.energy * delta_time;
                p.vy += dy * attraction * p.energy * delta_time;
                if p.is_text && rng.random_bool(0.03) {
                    p.symbol = p.target_symbol;
                } else if rng.random_bool(0.12) {
                    p.symbol = random_symbol(rng);
                }
            }
            ScatterSubPhase::PreConverge => {
                let mut target_vx = p.vx;
                let mut target_vy = p.vy;
                let mut target_vz = p.vz;

                if p.is_text {
                    let dx = p.target_x - p.x;
                    let dy = p.target_y - p.y;
                    let dz = p.target_z - p.z;
                    target_vx += dx * 0.5;
                    target_vy += dy * 0.5;
                    target_vz += dz * 0.5;
                    p.rotation *= 0.85;
                    p.scale = p.scale * 0.9 + 0.4 * 0.1;
                    p.opacity = p.opacity * 0.9 + 0.6 * 0.1;
                    if rng.random_bool(0.15) {
                        p.symbol = p.target_symbol;
                    }
                } else {
                    let text_center_x = WIDTH / 2.0;
                    let text_center_y = HEIGHT * 0.6;
                    let dx_text = text_center_x - p.x;
                    let dy_text = text_center_y - p.y;
                    let dx_grav = gravity_center.0 - p.x;
                    let dy_grav = gravity_center.1 - p.y;
                    target_vx += (dx_text * 0.005 + dx_grav * 0.002) * 60.0 * delta_time;
                    target_vy += (dy_text * 0.005 + dy_grav * 0.002) * 60.0 * delta_time;
                    target_vz += (-p.z * 0.01) * 60.0 * delta_time;
                    p.opacity = (p.opacity - delta_time * 0.8).max(0.05);
                    if rng.random_bool(0.1) {
                        p.symbol = random_symbol(rng);
                    }
                }

                let move_factor = 0.5 + progress * 0.5;
                p.vx =
                    p.vx * (1.0 - move_factor * delta_time) + target_vx * move_factor * delta_time;
                p.vy =
                    p.vy * (1.0 - move_factor * delta_time) + target_vy * move_factor * delta_time;
                p.vz =
                    p.vz * (1.0 - move_factor * delta_time) + target_vz * move_factor * delta_time;
            }
        }

        if !p.is_text && p.age > p.life {
            p.opacity = (p.opacity - delta_time * 2.0).max(0.0);
        }
        p.vx = p.vx.clamp(-6.0, 6.0);
        p.vy = p.vy.clamp(-6.0, 6.0);
        p.vz = p.vz.clamp(-4.0, 4.0);
        p.scale = p.scale.clamp(0.1, 2.5);
        p.opacity = p.opacity.clamp(0.0, 1.0);
    }
}

fn update_converge_phase(
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
        p.x += p.vx * delta_time * 60.0;
        p.y += p.vy * delta_time * 60.0;
        p.z += p.vz * delta_time * 60.0;
        p.vx *= 0.94;
        p.vy *= 0.94;
        p.vz *= 0.94;

        if p.is_text {
            let dx = p.target_x - p.x;
            let dy = p.target_y - p.y;
            let dz = p.target_z - p.z;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);
            let force_factor = (dist / 10.0).clamp(0.5, 2.0) * converge_speed;
            p.vx += dx * force_factor * delta_time;
            p.vy += dy * force_factor * delta_time;
            p.vz += dz * force_factor * delta_time;

            if matches!(
                sub_phase,
                ConvergeSubPhase::Refinement | ConvergeSubPhase::Solidification
            ) || rng.random_bool(0.3)
            {
                p.symbol = p.target_symbol;
            } else if rng.random_bool(0.1) {
                p.symbol = random_symbol(rng);
            }

            let target_scale = match sub_phase {
                ConvergeSubPhase::Alignment => 0.4,
                ConvergeSubPhase::Formation => 0.7,
                ConvergeSubPhase::Refinement => 1.0,
                ConvergeSubPhase::Solidification => 1.1,
            };
            p.scale = p.scale * 0.85 + target_scale * 0.15;
            let target_opacity = if matches!(sub_phase, ConvergeSubPhase::Solidification) {
                1.0
            } else {
                0.8
            };
            p.opacity = (p.opacity * 0.8 + target_opacity * 0.2).min(1.0);
            p.rotation *= 0.7;
        } else {
            match p.particle_type {
                ParticleType::Orbiter => {
                    let text_center_x = WIDTH / 2.0;
                    let text_center_y = HEIGHT * 0.6;
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
                    let dx_grav = gravity_center.0 - p.x;
                    let dy_grav = gravity_center.1 - p.y;
                    p.vx += dx_grav * 0.008 * delta_time * 60.0;
                    p.vy += dy_grav * 0.008 * delta_time * 60.0;
                    let turbulence_angle = p.age * p.energy * 1.5;
                    p.vx += turbulence_angle.sin() * 0.2 * delta_time * 60.0;
                    p.vy += turbulence_angle.cos() * 0.2 * delta_time * 60.0;
                    p.opacity = (p.opacity * 0.96).clamp(0.0, 0.3);
                }
                ParticleType::Fragment | ParticleType::Connector => {
                    if p.age < 0.5 {
                        p.vx += rng.random_range(-1.0..1.0) * p.energy;
                        p.vy += rng.random_range(-1.0..1.0) * p.energy;
                        p.vz += rng.random_range(-0.5..0.5) * p.energy;
                    }
                    p.opacity = (p.opacity - delta_time * 3.0).max(0.0);
                    p.scale *= 0.95;
                    p.rotation += p.vx * 5.0;
                }
                ParticleType::Core => {}
            }
            if p.opacity > 0.0 && rng.random_bool(0.08) {
                p.symbol = random_symbol(rng);
            }
        }

        p.vx = p.vx.clamp(-5.0, 5.0);
        p.vy = p.vy.clamp(-5.0, 5.0);
        p.vz = p.vz.clamp(-3.0, 3.0);
        p.scale = p.scale.clamp(0.0, 2.0);
        p.opacity = p.opacity.clamp(0.0, 1.0);
    }
}

fn update_stable_phase(
    particles: &mut [Particle],
    sub_phase: StableSubPhase,
    _progress: f32,
    rng: &mut ThreadRng,
) {
    let delta_time = 0.02;
    let text_particle_targets: Vec<(f32, f32, char)> = particles
        .iter()
        .filter(|p| p.is_text)
        .map(|p| (p.target_x, p.target_y, p.target_symbol))
        .collect();

    for p in particles.iter_mut() {
        p.age += delta_time;
        p.vx *= 0.85;
        p.vy *= 0.85;
        p.vz *= 0.85;
        p.x += p.vx * delta_time * 60.0;
        p.y += p.vy * delta_time * 60.0;
        p.z += p.vz * delta_time * 60.0;

        if p.is_text {
            p.symbol = p.target_symbol;
            p.opacity = 1.0;
            let base_x = p.target_x;
            let base_y = p.target_y;
            let base_z = p.target_z;

            match sub_phase {
                StableSubPhase::Pulse => {
                    p.scale = 1.0 + (p.age * 8.0).sin() * 0.08;
                    p.z = base_z + (p.age * 10.0).cos() * 1.0;
                    p.x = base_x + (p.age * 2.0).sin() * 0.2;
                    p.y = base_y + (p.age * 2.5).cos() * 0.2;
                }
                StableSubPhase::Orbit => {
                    let angle = p.age * 1.5;
                    let orbit_radius = 0.8;
                    p.x = base_x + angle.sin() * orbit_radius;
                    p.y = base_y + angle.cos() * orbit_radius;
                    p.z = base_z + (angle * 2.0).sin() * 1.5;
                    p.rotation = angle * 10.0;
                    p.scale = 1.0 + (angle * 1.5).cos() * 0.05;
                }
                StableSubPhase::Ripple => {
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
                    p.z = base_z + rng.random_range(-1.5..1.5);
                    p.scale = 1.0 + rng.random_range(-0.03..0.03);
                    if rng.random_bool(0.05) {
                        p.symbol = random_symbol(rng);
                    } else {
                        p.symbol = p.target_symbol;
                    }
                    p.x = base_x + rng.random_range(-0.2..0.2);
                    p.y = base_y + rng.random_range(-0.2..0.2);
                }
                StableSubPhase::PreDissolve => {
                    if rng.random_bool(0.05) {
                        p.vx += rng.random_range(-0.3..0.3);
                        p.vy += rng.random_range(-0.3..0.3);
                        p.vz += rng.random_range(-0.2..0.2);
                    }
                    p.x = base_x + p.vx;
                    p.y = base_y + p.vy;
                    p.z = base_z + p.vz;
                    if rng.random_bool(0.03) {
                        p.symbol = random_symbol(rng);
                    } else {
                        p.symbol = p.target_symbol;
                    }
                    p.scale = 1.0 + rng.random_range(-0.05..0.05);
                    p.opacity = 1.0 - rng.random_range(0.0..0.1);
                }
            }

            p.x = p.x.clamp(base_x - 2.0, base_x + 2.0);
            p.y = p.y.clamp(base_y - 2.0, base_y + 2.0);
            p.z = p.z.clamp(base_z - 5.0, base_z + 5.0);
            p.scale = p.scale.clamp(0.8, 1.2);
        } else {
            match p.particle_type {
                ParticleType::Orbiter => p.opacity *= 0.99,
                ParticleType::Swarm => p.opacity *= 0.98,
                _ => p.opacity *= 0.97,
            }
            if p.opacity > 0.0 && rng.random_bool(0.03) {
                p.symbol = random_symbol(rng);
            }
        }

        p.vx = p.vx.clamp(-3.0, 3.0);
        p.vy = p.vy.clamp(-3.0, 3.0);
        p.vz = p.vz.clamp(-2.0, 2.0);
        p.opacity = p.opacity.clamp(0.0, 1.0);
        p.scale = p.scale.clamp(0.0, 2.0);
    }
}

fn update_dissolve_phase(
    particles: &mut [Particle],
    sub_phase: DissolveSubPhase,
    rng: &mut ThreadRng,
) {
    let delta_time = 0.02;
    let explosion_factor = match sub_phase {
        DissolveSubPhase::Fracture => 0.2,
        DissolveSubPhase::Explosion => 1.5,
        DissolveSubPhase::Dispersion => 0.8,
        DissolveSubPhase::Fade => 0.3,
    };
    let fade_rate: f32 = match sub_phase {
        DissolveSubPhase::Fracture => 0.99,
        DissolveSubPhase::Explosion => 0.97,
        DissolveSubPhase::Dispersion => 0.95,
        DissolveSubPhase::Fade => 0.92,
    };

    for p in particles.iter_mut() {
        p.age += delta_time;
        let origin_x = if p.is_text { p.target_x } else { p.x };
        let origin_y = if p.is_text { p.target_y } else { p.y };
        let dx = p.x - origin_x;
        let dy = p.y - origin_y;
        let dist_sq = (dx * dx + dy * dy).max(1.0);
        let force = explosion_factor * 80.0 / dist_sq;
        p.vx += dx * force * p.energy * rng.random_range(0.7..1.3) * delta_time;
        p.vy += dy * force * p.energy * rng.random_range(0.7..1.3) * delta_time;
        if matches!(sub_phase, DissolveSubPhase::Explosion) {
            p.vz += rng.random_range(-1.0..1.0) * explosion_factor * p.energy * delta_time;
        }
        p.x += p.vx * delta_time * 60.0;
        p.y += p.vy * delta_time * 60.0;
        p.z += p.vz * delta_time * 60.0;
        p.vx *= 0.985;
        p.vy *= 0.985;
        p.vz *= 0.985;
        p.opacity *= fade_rate.powf(delta_time * 60.0);
        if matches!(sub_phase, DissolveSubPhase::Fade) {
            p.scale *= 0.97f32.powf(delta_time * 60.0);
        }
        p.rotation = (p.rotation + p.vx * 4.0) % 360.0;
        if rng.random_bool(0.2) {
            p.symbol = random_symbol(rng);
        }
        p.vx = p.vx.clamp(-8.0, 8.0);
        p.vy = p.vy.clamp(-8.0, 8.0);
        p.vz = p.vz.clamp(-6.0, 6.0);
        p.opacity = p.opacity.clamp(0.0, 1.0);
        p.scale = p.scale.clamp(0.0, 2.5);
    }
}

#[component]
pub fn TypingAnimation() -> impl IntoView {
    let rng = Rc::new(RefCell::new(rand::rng()));

    let particles = RwSignal::new({
        let mut r = rng.borrow_mut();
        initialize_particles(&mut r)
    });
    let connections = RwSignal::new(initialize_connections(&particles.get_untracked()));
    let phase = RwSignal::new(AnimationPhase::Scatter(ScatterSubPhase::Initial));
    let progress = RwSignal::new(0.0f32);
    let gravity_center = RwSignal::new((WIDTH / 2.0, HEIGHT / 2.0));
    let color_scheme = RwSignal::new(ColorScheme {
        primary: "#66d9ef".to_string(),
        secondary: "#a6e22e".to_string(),
        accent: "#f92672".to_string(),
        background: "rgba(39, 40, 34, 0.0)".to_string(),
    });
    let phase_timer_active = RwSignal::new(false);
    let reset_scheduled = RwSignal::new(false);

    let rng_tick = Rc::clone(&rng);
    Effect::new(move |_| {
        let handle = set_interval_with_handle(
            {
                let rng = Rc::clone(&rng_tick);
                move || {
                    let current_phase = phase.get();
                    let current_progress = progress.get();
                    let grav = gravity_center.get();
                    progress.set(current_progress + 0.02);

                    particles.update(|ps| {
                        let mut r = rng.borrow_mut();
                        match &current_phase {
                            AnimationPhase::Scatter(sub) => {
                                update_scatter_phase(ps, *sub, current_progress, grav, &mut r);
                            }
                            AnimationPhase::Converge(sub) => {
                                update_converge_phase(ps, *sub, current_progress, grav, &mut r);
                            }
                            AnimationPhase::Stable(sub) => {
                                update_stable_phase(ps, *sub, current_progress, &mut r);
                            }
                            AnimationPhase::Dissolve(sub) => {
                                update_dissolve_phase(ps, *sub, &mut r);
                            }
                        }
                    });

                    connections.update(|cs| {
                        update_connections(cs, &current_phase, current_progress);
                    });

                    let should_advance = match &current_phase {
                        AnimationPhase::Scatter(ScatterSubPhase::Initial) => current_progress > 0.6,
                        AnimationPhase::Scatter(ScatterSubPhase::Expansion) => {
                            current_progress > 1.2
                        }
                        AnimationPhase::Scatter(ScatterSubPhase::Contraction) => {
                            current_progress > 1.8
                        }
                        AnimationPhase::Scatter(ScatterSubPhase::PreConverge) => {
                            current_progress > 2.5
                        }
                        AnimationPhase::Converge(ConvergeSubPhase::Alignment) => {
                            current_progress > 0.8
                        }
                        AnimationPhase::Converge(ConvergeSubPhase::Formation) => {
                            current_progress > 1.4
                        }
                        AnimationPhase::Converge(ConvergeSubPhase::Refinement) => {
                            current_progress > 1.8
                        }
                        AnimationPhase::Converge(ConvergeSubPhase::Solidification) => {
                            current_progress > 2.2
                        }
                        AnimationPhase::Stable(StableSubPhase::Pulse) => current_progress > 0.6,
                        AnimationPhase::Stable(StableSubPhase::Orbit) => current_progress > 0.6,
                        AnimationPhase::Stable(StableSubPhase::Ripple) => current_progress > 0.6,
                        AnimationPhase::Stable(StableSubPhase::Shimmer) => current_progress > 0.6,
                        AnimationPhase::Stable(StableSubPhase::PreDissolve) => {
                            current_progress > 1.0
                        }
                        AnimationPhase::Dissolve(DissolveSubPhase::Fracture) => {
                            current_progress > 0.5
                        }
                        AnimationPhase::Dissolve(DissolveSubPhase::Explosion) => {
                            current_progress > 1.0
                        }
                        AnimationPhase::Dissolve(DissolveSubPhase::Dispersion) => {
                            current_progress > 1.5
                        }
                        AnimationPhase::Dissolve(DissolveSubPhase::Fade) => current_progress > 2.0,
                    };

                    if should_advance && !phase_timer_active.get() {
                        phase_timer_active.set(true);
                        let next_phase = match &current_phase {
                            AnimationPhase::Scatter(ScatterSubPhase::Initial) => {
                                Some(AnimationPhase::Scatter(ScatterSubPhase::Expansion))
                            }
                            AnimationPhase::Scatter(ScatterSubPhase::Expansion) => {
                                Some(AnimationPhase::Scatter(ScatterSubPhase::Contraction))
                            }
                            AnimationPhase::Scatter(ScatterSubPhase::Contraction) => {
                                Some(AnimationPhase::Scatter(ScatterSubPhase::PreConverge))
                            }
                            AnimationPhase::Scatter(ScatterSubPhase::PreConverge) => {
                                Some(AnimationPhase::Converge(ConvergeSubPhase::Alignment))
                            }
                            AnimationPhase::Converge(ConvergeSubPhase::Alignment) => {
                                Some(AnimationPhase::Converge(ConvergeSubPhase::Formation))
                            }
                            AnimationPhase::Converge(ConvergeSubPhase::Formation) => {
                                Some(AnimationPhase::Converge(ConvergeSubPhase::Refinement))
                            }
                            AnimationPhase::Converge(ConvergeSubPhase::Refinement) => {
                                Some(AnimationPhase::Converge(ConvergeSubPhase::Solidification))
                            }
                            AnimationPhase::Converge(ConvergeSubPhase::Solidification) => {
                                Some(AnimationPhase::Stable(StableSubPhase::Pulse))
                            }
                            AnimationPhase::Stable(StableSubPhase::Pulse) => {
                                Some(AnimationPhase::Stable(StableSubPhase::Orbit))
                            }
                            AnimationPhase::Stable(StableSubPhase::Orbit) => {
                                Some(AnimationPhase::Stable(StableSubPhase::Ripple))
                            }
                            AnimationPhase::Stable(StableSubPhase::Ripple) => {
                                Some(AnimationPhase::Stable(StableSubPhase::Shimmer))
                            }
                            AnimationPhase::Stable(StableSubPhase::Shimmer) => {
                                Some(AnimationPhase::Stable(StableSubPhase::PreDissolve))
                            }
                            AnimationPhase::Stable(StableSubPhase::PreDissolve) => {
                                Some(AnimationPhase::Dissolve(DissolveSubPhase::Fracture))
                            }
                            AnimationPhase::Dissolve(DissolveSubPhase::Fracture) => {
                                Some(AnimationPhase::Dissolve(DissolveSubPhase::Explosion))
                            }
                            AnimationPhase::Dissolve(DissolveSubPhase::Explosion) => {
                                Some(AnimationPhase::Dissolve(DissolveSubPhase::Dispersion))
                            }
                            AnimationPhase::Dissolve(DissolveSubPhase::Dispersion) => {
                                Some(AnimationPhase::Dissolve(DissolveSubPhase::Fade))
                            }
                            AnimationPhase::Dissolve(DissolveSubPhase::Fade) => None,
                        };

                        if let Some(next) = next_phase {
                            set_timeout(
                                move || {
                                    phase.set(next.clone());
                                    progress.set(0.0);
                                    phase_timer_active.set(false);
                                },
                                Duration::from_millis(30),
                            );
                        } else if !reset_scheduled.get() {
                            reset_scheduled.set(true);
                            let rng_reset = Rc::clone(&rng);
                            set_timeout(
                                move || {
                                    let mut r = rng_reset.borrow_mut();
                                    let new_particles = initialize_particles(&mut r);
                                    let new_connections = initialize_connections(&new_particles);
                                    particles.set(new_particles);
                                    connections.set(new_connections);
                                    phase.set(AnimationPhase::Scatter(ScatterSubPhase::Initial));
                                    progress.set(0.0);
                                    phase_timer_active.set(false);
                                    reset_scheduled.set(false);
                                },
                                Duration::from_millis(1500),
                            );
                        }
                    }
                }
            },
            Duration::from_millis(16),
        )
        .expect("interval");

        on_cleanup(move || handle.clear());
    });

    let rng_grav = Rc::clone(&rng);
    Effect::new(move |_| {
        let handle = set_interval_with_handle(
            {
                let rng = Rc::clone(&rng_grav);
                move || {
                    let mut r = rng.borrow_mut();
                    gravity_center.set((
                        WIDTH / 2.0 + r.random_range(-30.0..30.0),
                        HEIGHT / 2.0 + r.random_range(-20.0..20.0),
                    ));
                }
            },
            Duration::from_millis(2500),
        )
        .expect("interval");

        on_cleanup(move || handle.clear());
    });

    view! {
        <div
            class="transformium-field"
            style=move || format!("background-color: {}; transform-style: preserve-3d;", color_scheme.get().background)
        >
            <svg
                class="connections-layer"
                style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: 1; pointer-events: none; opacity: 0.7;"
            >
                {move || {
                    let ps = particles.get();
                    let cs = connections.get();
                    let scheme = color_scheme.get();
                    cs.iter()
                        .filter(|c| c.active && c.opacity > 0.01)
                        .filter_map(|conn| {
                            ps.get(conn.particle1_idx).and_then(|p1| {
                                ps.get(conn.particle2_idx).map(|p2| (p1, p2, conn))
                            })
                        })
                        .map(|(p1, p2, conn)| {
                            let stroke_width = (conn.strength * 0.6).clamp(0.1, 0.6);
                            view! {
                                <line
                                    x1=p1.x.to_string()
                                    y1=p1.y.to_string()
                                    x2=p2.x.to_string()
                                    y2=p2.y.to_string()
                                    stroke=scheme.primary.clone()
                                    stroke-width=stroke_width.to_string()
                                    stroke-opacity=(conn.opacity * 0.7).to_string()
                                    stroke-linecap="round"
                                    class="connection-line"
                                />
                            }
                        })
                        .collect_view()
                }}
            </svg>
            <div class="transformium-particles">
                {move || {
                    let ps = particles.get();
                    let scheme = color_scheme.get();
                    let current_phase = phase.get();
                    ps.iter()
                        .filter(|p| p.opacity > 0.01)
                        .map(|p| {
                            let z_index = ((p.z + 100.0).clamp(0.0, 200.0) / 2.0).round() as i32;
                            let color = match (p.is_text, &p.particle_type, &current_phase) {
                                (true, _, AnimationPhase::Stable(StableSubPhase::Ripple | StableSubPhase::Shimmer)) => {
                                    scheme.accent.clone()
                                }
                                (true, _, _) => scheme.primary.clone(),
                                (false, ParticleType::Orbiter, _) => scheme.accent.clone(),
                                _ => scheme.secondary.clone(),
                            };
                            let z_scale = (1.0 + p.z / 500.0).clamp(0.4, 2.5);
                            let font_size = (14.0 * p.scale * z_scale).clamp(4.0, 28.0);
                            let style = format!(
                                "position: absolute; left: {:.1}px; top: {:.1}px; opacity: {:.2}; z-index: {}; \
                                 transform: translate3d(-50%, -50%, {:.1}px) scale({:.2}) rotate({:.1}deg); \
                                 color: {}; font-size: {:.1}px; will-change: transform, opacity;",
                                p.x, p.y, p.opacity, z_index, p.z, 1.0, p.rotation, color, font_size
                            );
                            let classes = if p.is_text
                                && matches!(current_phase, AnimationPhase::Stable(_))
                                && !matches!(
                                    current_phase,
                                    AnimationPhase::Stable(StableSubPhase::PreDissolve | StableSubPhase::Shimmer)
                                )
                            {
                                "particle metallic-glow"
                            } else {
                                "particle"
                            };
                            view! { <span class=classes style=style>{p.symbol.to_string()}</span> }
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }
}
