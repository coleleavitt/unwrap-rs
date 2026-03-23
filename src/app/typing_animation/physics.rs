use rand::Rng;
use rand::rngs::ThreadRng;

use super::constants::{DELTA_TIME, HEIGHT, WIDTH, random_symbol};
use super::types::{
    AnimationPhase, Connection, ConvergeSubPhase, DissolveSubPhase, Particle, ParticleType,
    ScatterSubPhase, StableSubPhase,
};

pub fn update_connections(connections: &mut [Connection], phase: &AnimationPhase, progress: f32) {
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
                conn.opacity = conn.opacity.mul_add(0.9, target_opacity * 0.1).min(0.6);
                conn.strength = conn.strength.mul_add(0.95, 0.05).min(1.0);
            }
            AnimationPhase::Stable(_) => {
                conn.active = true;
                conn.opacity = (progress * 3.0).sin().mul_add(0.1, 0.5);
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

pub fn update_scatter_phase(
    particles: &mut [Particle],
    sub_phase: ScatterSubPhase,
    progress: f32,
    gravity_center: (f32, f32),
    rng: &mut ThreadRng,
) {
    for p in particles.iter_mut() {
        p.age += DELTA_TIME;
        p.x = (p.vx * DELTA_TIME).mul_add(60.0, p.x);
        p.y = (p.vy * DELTA_TIME).mul_add(60.0, p.y);
        p.z = (p.vz * DELTA_TIME).mul_add(60.0, p.z);
        p.vx *= 0.96;
        p.vy *= 0.96;
        p.vz *= 0.96;
        p.rotation = p.vx.mul_add(2.0, p.rotation) % 360.0;

        match sub_phase {
            ScatterSubPhase::Initial => {
                p.opacity = DELTA_TIME.mul_add(2.0, p.opacity).min(0.7);
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
                p.opacity = DELTA_TIME.mul_add(-0.5, p.opacity).max(0.1);
                let dx = p.x - gravity_center.0;
                let dy = p.y - gravity_center.1;
                let dist_sq = dy.mul_add(dy, dx * dx).max(1.0);
                let force = 80.0 / dist_sq;
                p.vx = (dx * force * p.energy).mul_add(DELTA_TIME, p.vx);
                p.vy = (dy * force * p.energy).mul_add(DELTA_TIME, p.vy);
                if rng.random_bool(0.1) {
                    p.symbol = random_symbol(rng);
                }
            }
            ScatterSubPhase::Contraction => {
                p.opacity = DELTA_TIME.mul_add(1.0, p.opacity).min(0.8);
                let dx = gravity_center.0 - p.x;
                let dy = gravity_center.1 - p.y;
                let attraction = progress.mul_add(0.2, 0.1);
                p.vx = (dx * attraction * p.energy).mul_add(DELTA_TIME, p.vx);
                p.vy = (dy * attraction * p.energy).mul_add(DELTA_TIME, p.vy);
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
                    target_vx = dx.mul_add(0.5, target_vx);
                    target_vy = dy.mul_add(0.5, target_vy);
                    target_vz = dz.mul_add(0.5, target_vz);
                    p.rotation *= 0.85;
                    p.scale = p.scale.mul_add(0.9, 0.04);
                    p.opacity = p.opacity.mul_add(0.9, 0.06);
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
                    target_vx = (dx_grav.mul_add(0.002, dx_text * 0.005) * 60.0)
                        .mul_add(DELTA_TIME, target_vx);
                    target_vy = (dy_grav.mul_add(0.002, dy_text * 0.005) * 60.0)
                        .mul_add(DELTA_TIME, target_vy);
                    target_vz = ((-p.z * 0.01) * 60.0).mul_add(DELTA_TIME, target_vz);
                    p.opacity = DELTA_TIME.mul_add(-0.8, p.opacity).max(0.05);
                    if rng.random_bool(0.1) {
                        p.symbol = random_symbol(rng);
                    }
                }

                let move_factor = progress.mul_add(0.5, 0.5);
                let blend = move_factor * DELTA_TIME;
                p.vx = p.vx.mul_add(1.0 - blend, target_vx * blend);
                p.vy = p.vy.mul_add(1.0 - blend, target_vy * blend);
                p.vz = p.vz.mul_add(1.0 - blend, target_vz * blend);
            }
        }

        if !p.is_text && p.age > p.life {
            p.opacity = DELTA_TIME.mul_add(-2.0, p.opacity).max(0.0);
        }
        p.vx = p.vx.clamp(-6.0, 6.0);
        p.vy = p.vy.clamp(-6.0, 6.0);
        p.vz = p.vz.clamp(-4.0, 4.0);
        p.scale = p.scale.clamp(0.1, 2.5);
        p.opacity = p.opacity.clamp(0.0, 1.0);
    }
}

pub fn update_converge_phase(
    particles: &mut [Particle],
    sub_phase: ConvergeSubPhase,
    _progress: f32,
    gravity_center: (f32, f32),
    rng: &mut ThreadRng,
) {
    let base_converge_speed = 0.1;
    let speed_factor = match sub_phase {
        ConvergeSubPhase::Alignment => 0.6,
        ConvergeSubPhase::Formation => 1.0,
        ConvergeSubPhase::Refinement => 1.5,
        ConvergeSubPhase::Solidification => 2.0,
    };
    let converge_speed = base_converge_speed * speed_factor;

    for p in particles.iter_mut() {
        p.age += DELTA_TIME;
        p.x = (p.vx * DELTA_TIME).mul_add(60.0, p.x);
        p.y = (p.vy * DELTA_TIME).mul_add(60.0, p.y);
        p.z = (p.vz * DELTA_TIME).mul_add(60.0, p.z);
        p.vx *= 0.94;
        p.vy *= 0.94;
        p.vz *= 0.94;

        if p.is_text {
            let dx = p.target_x - p.x;
            let dy = p.target_y - p.y;
            let dz = p.target_z - p.z;
            let dist = dz.mul_add(dz, dy.mul_add(dy, dx * dx)).sqrt().max(0.1);
            let force_factor = (dist / 10.0).clamp(0.5, 2.0) * converge_speed;
            p.vx = (dx * force_factor).mul_add(DELTA_TIME, p.vx);
            p.vy = (dy * force_factor).mul_add(DELTA_TIME, p.vy);
            p.vz = (dz * force_factor).mul_add(DELTA_TIME, p.vz);

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
            p.scale = p.scale.mul_add(0.85, target_scale * 0.15);
            let target_opacity = if matches!(sub_phase, ConvergeSubPhase::Solidification) {
                1.0
            } else {
                0.8
            };
            p.opacity = p.opacity.mul_add(0.8, target_opacity * 0.2).min(1.0);
            p.rotation *= 0.7;
        } else {
            match p.particle_type {
                ParticleType::Orbiter => {
                    let text_center_x = WIDTH / 2.0;
                    let text_center_y = HEIGHT * 0.6;
                    let angle = p.energy.mul_add(0.3, 0.5) * p.age;
                    let radius_base = (p.target_x % 30.0) + 40.0;
                    let radius_variation = (p.age * 0.8).sin() * 15.0 * p.energy;
                    let orbit_radius = radius_base + radius_variation;
                    let target_x = angle.cos().mul_add(orbit_radius, text_center_x);
                    let target_y = angle.sin().mul_add(orbit_radius * 0.7, text_center_y);
                    p.vx = ((target_x - p.x) * 0.05 * DELTA_TIME).mul_add(60.0, p.vx);
                    p.vy = ((target_y - p.y) * 0.05 * DELTA_TIME).mul_add(60.0, p.vy);
                    p.vz *= 0.9;
                    p.opacity = (p.opacity * 0.97).clamp(0.0, 0.4);
                }
                ParticleType::Swarm => {
                    let dx_grav = gravity_center.0 - p.x;
                    let dy_grav = gravity_center.1 - p.y;
                    p.vx = (dx_grav * 0.008 * DELTA_TIME).mul_add(60.0, p.vx);
                    p.vy = (dy_grav * 0.008 * DELTA_TIME).mul_add(60.0, p.vy);
                    let turbulence_angle = p.age * p.energy * 1.5;
                    p.vx = (turbulence_angle.sin() * 0.2 * DELTA_TIME).mul_add(60.0, p.vx);
                    p.vy = (turbulence_angle.cos() * 0.2 * DELTA_TIME).mul_add(60.0, p.vy);
                    p.opacity = (p.opacity * 0.96).clamp(0.0, 0.3);
                }
                ParticleType::Fragment | ParticleType::Connector => {
                    if p.age < 0.5 {
                        p.vx = rng.random_range(-1.0_f32..1.0).mul_add(p.energy, p.vx);
                        p.vy = rng.random_range(-1.0_f32..1.0).mul_add(p.energy, p.vy);
                        p.vz = rng.random_range(-0.5_f32..0.5).mul_add(p.energy, p.vz);
                    }
                    p.opacity = DELTA_TIME.mul_add(-3.0, p.opacity).max(0.0);
                    p.scale *= 0.95;
                    p.rotation = p.vx.mul_add(5.0, p.rotation);
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

pub fn update_stable_phase(
    particles: &mut [Particle],
    sub_phase: StableSubPhase,
    _progress: f32,
    rng: &mut ThreadRng,
) {
    let text_particle_targets: Vec<(f32, f32, char)> = particles
        .iter()
        .filter(|p| p.is_text)
        .map(|p| (p.target_x, p.target_y, p.target_symbol))
        .collect();

    for p in particles.iter_mut() {
        p.age += DELTA_TIME;
        p.vx *= 0.85;
        p.vy *= 0.85;
        p.vz *= 0.85;
        p.x = (p.vx * DELTA_TIME).mul_add(60.0, p.x);
        p.y = (p.vy * DELTA_TIME).mul_add(60.0, p.y);
        p.z = (p.vz * DELTA_TIME).mul_add(60.0, p.z);

        if p.is_text {
            p.symbol = p.target_symbol;
            p.opacity = 1.0;
            let base_x = p.target_x;
            let base_y = p.target_y;
            let base_z = p.target_z;

            match sub_phase {
                StableSubPhase::Pulse => {
                    p.scale = (p.age * 8.0).sin().mul_add(0.08, 1.0);
                    p.z = (p.age * 10.0).cos().mul_add(1.0, base_z);
                    p.x = (p.age * 2.0).sin().mul_add(0.2, base_x);
                    p.y = (p.age * 2.5).cos().mul_add(0.2, base_y);
                }
                StableSubPhase::Orbit => {
                    let angle = p.age * 1.5;
                    let orbit_radius = 0.8;
                    p.x = angle.sin().mul_add(orbit_radius, base_x);
                    p.y = angle.cos().mul_add(orbit_radius, base_y);
                    p.z = (angle * 2.0).sin().mul_add(1.5, base_z);
                    p.rotation = angle * 10.0;
                    p.scale = (angle * 1.5).cos().mul_add(0.05, 1.0);
                }
                StableSubPhase::Ripple => {
                    let particle_index = text_particle_targets
                        .iter()
                        .position(|(tx, _ty, ts)| *ts == p.target_symbol && *tx == p.target_x)
                        .unwrap_or(0);
                    let ripple_offset = particle_index as f32 * 0.5;
                    let ripple_val = p.age.mul_add(6.0, -ripple_offset).sin();
                    p.y = ripple_val.mul_add(2.0, base_y);
                    p.z = ripple_val.abs().mul_add(3.0, base_z);
                    p.scale = ripple_val.abs().mul_add(0.15, 1.0);
                }
                StableSubPhase::Shimmer => {
                    p.z = rng.random_range(-1.5..1.5) + base_z;
                    p.scale = rng.random_range(-0.03..0.03) + 1.0;
                    if rng.random_bool(0.05) {
                        p.symbol = random_symbol(rng);
                    } else {
                        p.symbol = p.target_symbol;
                    }
                    p.x = rng.random_range(-0.2..0.2) + base_x;
                    p.y = rng.random_range(-0.2..0.2) + base_y;
                }
                StableSubPhase::PreDissolve => {
                    if rng.random_bool(0.05) {
                        p.vx += rng.random_range(-0.3..0.3);
                        p.vy += rng.random_range(-0.3..0.3);
                        p.vz += rng.random_range(-0.2..0.2);
                    }
                    p.x = p.vx + base_x;
                    p.y = p.vy + base_y;
                    p.z = p.vz + base_z;
                    if rng.random_bool(0.03) {
                        p.symbol = random_symbol(rng);
                    } else {
                        p.symbol = p.target_symbol;
                    }
                    p.scale = rng.random_range(-0.05..0.05) + 1.0;
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

pub fn update_dissolve_phase(
    particles: &mut [Particle],
    sub_phase: DissolveSubPhase,
    rng: &mut ThreadRng,
) {
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
        p.age += DELTA_TIME;
        let origin_x = if p.is_text { p.target_x } else { p.x };
        let origin_y = if p.is_text { p.target_y } else { p.y };
        let dx = p.x - origin_x;
        let dy = p.y - origin_y;
        let dist_sq = dy.mul_add(dy, dx * dx).max(1.0);
        let force = explosion_factor * 80.0 / dist_sq;
        p.vx = (dx * force * p.energy * rng.random_range(0.7..1.3)).mul_add(DELTA_TIME, p.vx);
        p.vy = (dy * force * p.energy * rng.random_range(0.7..1.3)).mul_add(DELTA_TIME, p.vy);
        if matches!(sub_phase, DissolveSubPhase::Explosion) {
            p.vz = (rng.random_range(-1.0..1.0) * explosion_factor * p.energy)
                .mul_add(DELTA_TIME, p.vz);
        }
        p.x = (p.vx * DELTA_TIME).mul_add(60.0, p.x);
        p.y = (p.vy * DELTA_TIME).mul_add(60.0, p.y);
        p.z = (p.vz * DELTA_TIME).mul_add(60.0, p.z);
        p.vx *= 0.985;
        p.vy *= 0.985;
        p.vz *= 0.985;
        p.opacity *= fade_rate.powf(DELTA_TIME * 60.0);
        if matches!(sub_phase, DissolveSubPhase::Fade) {
            p.scale *= 0.97f32.powf(DELTA_TIME * 60.0);
        }
        p.rotation = p.vx.mul_add(4.0, p.rotation) % 360.0;
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
