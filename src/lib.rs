//! # openstranded-common-crafting
//!
//! Shared data types for the OpenStranded crafting system.
//!
//! This crate defines the canonical structures that represent crafting
//! recipes, their requirements, and their results. Used by both the
//! engine (Bevy ECS resources) and WASM game plugins (parsing `.ron`
//! registry data, defining service contracts).
//!
//! ## Types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`RecipeDef`] | A single crafting recipe (combi block) |
//! | [`RecipeRequirement`] | An ingredient with optional stay flag |
//! | [`RecipeResult`] | A generated item |
//!
//! ## Relationship to `.inf` files
//!
//! In the original game, each `combi=start`…`combi=end` block inside a
//! `combinations*.inf` file corresponds to one [`RecipeDef`].  Each
//! `req=N[,count[,stay]]` line inside becomes a [`RecipeRequirement`],
//! and each `gen=N[,count]` line becomes a [`RecipeResult`].
//!
//! ## Dependency
//!
//! Only `serde` — this is a pure data crate with no engine or plugin API
//! dependencies.

mod recipe_def;

pub use recipe_def::{RecipeDef, RecipeRequirement, RecipeResult};
