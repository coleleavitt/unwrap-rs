use super::types::{
    AnimationPhase, ConvergeSubPhase, DissolveSubPhase, ScatterSubPhase, StableSubPhase,
};

impl AnimationPhase {
    pub const fn next(self) -> Option<Self> {
        use AnimationPhase::*;
        use ConvergeSubPhase::*;
        use DissolveSubPhase::*;
        use ScatterSubPhase::*;
        use StableSubPhase::*;

        match self {
            Scatter(Initial) => Some(Scatter(Expansion)),
            Scatter(Expansion) => Some(Scatter(Contraction)),
            Scatter(Contraction) => Some(Scatter(PreConverge)),
            Scatter(PreConverge) => Some(Converge(Alignment)),

            Converge(Alignment) => Some(Converge(Formation)),
            Converge(Formation) => Some(Converge(Refinement)),
            Converge(Refinement) => Some(Converge(Solidification)),
            Converge(Solidification) => Some(Stable(Pulse)),

            Stable(Pulse) => Some(Stable(Orbit)),
            Stable(Orbit) => Some(Stable(Ripple)),
            Stable(Ripple) => Some(Stable(Shimmer)),
            Stable(Shimmer) => Some(Stable(PreDissolve)),
            Stable(PreDissolve) => Some(Dissolve(Fracture)),

            Dissolve(Fracture) => Some(Dissolve(Explosion)),
            Dissolve(Explosion) => Some(Dissolve(Dispersion)),
            Dissolve(Dispersion) => Some(Dissolve(Fade)),
            Dissolve(Fade) => None,
        }
    }

    pub const fn duration(&self) -> f32 {
        use AnimationPhase::*;
        use ConvergeSubPhase::*;
        use DissolveSubPhase::*;
        use ScatterSubPhase::*;
        use StableSubPhase::*;

        match self {
            Scatter(Initial) => 0.6,
            Scatter(Expansion) => 1.2,
            Scatter(Contraction) => 1.8,
            Scatter(PreConverge) => 2.5,

            Converge(Alignment) => 0.8,
            Converge(Formation) => 1.4,
            Converge(Refinement) => 1.8,
            Converge(Solidification) => 2.2,

            Stable(Pulse) => 0.6,
            Stable(Orbit) => 0.6,
            Stable(Ripple) => 0.6,
            Stable(Shimmer) => 0.6,
            Stable(PreDissolve) => 1.0,

            Dissolve(Fracture) => 0.5,
            Dissolve(Explosion) => 1.0,
            Dissolve(Dispersion) => 1.5,
            Dissolve(Fade) => 2.0,
        }
    }

    pub const fn initial() -> Self {
        Self::Scatter(ScatterSubPhase::Initial)
    }
}
