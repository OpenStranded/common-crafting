# openstranded-common-crafting

Shared data types for the OpenStranded crafting system.

Part of the [OpenStranded](https://github.com/OpenStranded) project.
Published on [crates.io](https://crates.io/crates/openstranded-common-crafting).

## Types

| Type | Purpose |
|------|---------|
| `RecipeDef` | A single crafting recipe (from `combi=start/end` blocks) |
| `RecipeRequirement` | An ingredient: `item_id + count + stay` flag |
| `RecipeResult` | A generated item: `item_id + count` |

## Dependencies

Only `serde` — this is a pure data crate with no engine dependencies.

## License

GPL-3.0-or-later
