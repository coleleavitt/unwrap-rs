#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorScheme {
    pub primary: &'static str,
    pub secondary: &'static str,
    pub accent: &'static str,
    pub background: &'static str,
}

impl ColorScheme {
    pub const DEFAULT: Self = Self {
        primary: "#00ff88",
        secondary: "#0088ff",
        accent: "#ff00ff",
        background: "transparent",
    };
}

#[derive(Clone, Copy, Debug)]
pub struct Connection {
    pub particle1_idx: usize,
    pub particle2_idx: usize,
    pub strength: f32,
    pub opacity: f32,
    pub active: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParticleType {
    Core,
    Orbiter,
    Swarm,
    Fragment,
    Connector,
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub target_z: f32,
    pub symbol: char,
    pub target_symbol: char,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub opacity: f32,
    pub scale: f32,
    pub rotation: f32,
    pub is_text: bool,
    pub kind: ParticleType,
    pub energy: f32,
    pub age: f32,
    pub life: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScatterSubPhase {
    Initial,
    Expansion,
    Contraction,
    PreConverge,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConvergeSubPhase {
    Alignment,
    Formation,
    Refinement,
    Solidification,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StableSubPhase {
    Pulse,
    Orbit,
    Ripple,
    Shimmer,
    PreDissolve,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DissolveSubPhase {
    Fracture,
    Explosion,
    Dispersion,
    Fade,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationPhase {
    Scatter(ScatterSubPhase),
    Converge(ConvergeSubPhase),
    Stable(StableSubPhase),
    Dissolve(DissolveSubPhase),
}
