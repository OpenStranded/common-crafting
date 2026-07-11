use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A single ingredient required by a crafting recipe.
///
/// Each requirement corresponds to one `req=` line inside a `combi=start/end`
/// block.
///
/// # `.inf` format
///
/// ```text
/// req=item_id[,count[,stay]]
/// ```
///
/// * `item_id` — numeric ID of the item required.
/// * `count` (optional, default 1) — how many of this item are needed.
/// * `stay` (optional) — if present, the ingredient is **not** consumed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeRequirement {
    /// Item type ID (matches `ItemDef.id`).
    pub item_id: u32,

    /// Number of items required (default 1).
    #[serde(default = "default_count")]
    pub count: u32,

    /// If `true`, the ingredient is not consumed when crafting.
    #[serde(default)]
    pub stay: bool,
}

const fn default_count() -> u32 {
    1
}

impl RecipeRequirement {
    /// Parse a requirement from a raw `req=` value string.
    ///
    /// Accepts `"itemId"`, `"itemId,count"`, or `"itemId,count,stay"`.
    #[must_use]
    pub fn from_inf_value(s: &str) -> Self {
        let parts: Vec<&str> = s.splitn(3, ',').collect();
        let item_id = parts.first().and_then(|p| p.trim().parse().ok()).unwrap_or(0);
        let count = parts
            .get(1)
            .and_then(|p| p.trim().parse().ok())
            .unwrap_or(1);
        let stay = parts.get(2).map_or(false, |p| p.trim() == "stay");
        Self { item_id, count, stay }
    }
}

/// An item produced by a crafting recipe.
///
/// Each result corresponds to one `gen=` line inside a `combi=start/end`
/// block.
///
/// # `.inf` format
///
/// ```text
/// gen=item_id[,count]
/// ```
///
/// * `item_id` — numeric ID of the item produced.
/// * `count` (optional, default 1) — how many are produced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeResult {
    /// Item type ID (matches `ItemDef.id`).
    pub item_id: u32,

    /// Number of items produced (default 1).
    #[serde(default = "default_count")]
    pub count: u32,
}

impl RecipeResult {
    /// Parse a result from a raw `gen=` value string.
    ///
    /// Accepts `"itemId"` or `"itemId,count"`.
    #[must_use]
    pub fn from_inf_value(s: &str) -> Self {
        let parts: Vec<&str> = s.splitn(2, ',').collect();
        let item_id = parts.first().and_then(|p| p.trim().parse().ok()).unwrap_or(0);
        let count = parts
            .get(1)
            .and_then(|p| p.trim().parse().ok())
            .unwrap_or(1);
        Self { item_id, count }
    }
}

/// A single crafting recipe definition.
///
/// Represents one `combi=start`…`combi=end` block from a `combinations*.inf`
/// file.
///
/// # `.inf` structure
///
/// ```text
/// combi=start
///     id=recipe_name            (optional string identifier)
///     req=item_id[,count[,stay]]
///     req=item_id[,count[,stay]]
///     gen=item_id[,count]
///     genname=override_name     (optional result name override)
///     script=start
///         ... script commands ...
///     script=end
/// combi=end
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeDef {
    /// Numeric serial ID assigned by the loader (1‑based, sequential).
    pub serial_id: u32,

    /// Optional string identifier (from `id=` field, e.g. `"bendablebranch"`).
    #[serde(default)]
    pub id: Option<String>,

    /// Ingredients required by this recipe.
    #[serde(default)]
    pub requirements: Vec<RecipeRequirement>,

    /// Items produced by this recipe.
    #[serde(default)]
    pub results: Vec<RecipeResult>,

    /// Optional override for the generated item's display name.
    #[serde(default)]
    pub gen_name: Option<String>,

    /// Optional script executed when the recipe is crafted.
    #[serde(default)]
    pub script: Option<String>,
}

// ── Helper: parse a Vec<RecipeRequirement> from multivalue "req" ────

fn parse_requirements(fields: &HashMap<String, Vec<String>>) -> Vec<RecipeRequirement> {
    fields
        .get("req")
        .map(|vals| vals.iter().map(|v| RecipeRequirement::from_inf_value(v)).collect())
        .unwrap_or_default()
}

fn parse_results(fields: &HashMap<String, Vec<String>>) -> Vec<RecipeResult> {
    fields
        .get("gen")
        .map(|vals| vals.iter().map(|v| RecipeResult::from_inf_value(v)).collect())
        .unwrap_or_default()
}

// ── Field parsing helpers ──────────────────────────────────────────

fn first_str<'a>(fields: &'a HashMap<String, Vec<String>>, key: &str) -> Option<&'a str> {
    fields.get(key)?.first().map(String::as_str)
}

fn first_string(fields: &HashMap<String, Vec<String>>, key: &str) -> Option<String> {
    first_str(fields, key).map(ToOwned::to_owned)
}

fn first_u32(fields: &HashMap<String, Vec<String>>, key: &str) -> Option<u32> {
    first_str(fields, key)?.parse().ok()
}

impl RecipeDef {
    /// Construct a `RecipeDef` from the raw fields **inside** a `combi`
    /// block (i.e. a `Structured` block content's fields).
    ///
    /// The `serial_id` should be supplied by the caller (sequential
    /// counter across all combi blocks).
    ///
    /// Unknown or unparseable fields are silently ignored; missing
    /// required fields (`gen`) return `None`.
    #[must_use]
    pub fn from_combi_fields(
        serial_id: u32,
        fields: &HashMap<String, Vec<String>>,
    ) -> Option<Self> {
        // A valid recipe must produce at least one item.
        let results = parse_results(fields);
        if results.is_empty() {
            return None;
        }

        let id = first_string(fields, "id");
        let requirements = parse_requirements(fields);
        let gen_name = first_string(fields, "genname");

        // Script is stored as a block in the original data, not as a
        // field in the combi block fields. Callers should pass it in
        // separately; we leave it as None here.
        let script = None;

        Some(Self {
            serial_id,
            id,
            requirements,
            results,
            gen_name,
            script,
        })
    }

    /// Construct a `RecipeDef` from the entry-level fields of a parsed
    /// `.inf` entry that contains `combi` blocks.
    ///
    /// This is useful when the caller has the outer `InfEntry` fields
    /// (which are usually empty for combinations files) and wants to
    /// pass in the script separately.
    ///
    /// Prefer [`from_combi_fields`](Self::from_combi_fields) when you
    /// already have the structured combi block fields.
    #[must_use]
    pub fn from_inf_fields(serial_id: u32, fields: &HashMap<String, Vec<String>>) -> Option<Self> {
        Self::from_combi_fields(serial_id, fields)
    }

    /// Attach a script to this recipe (builder-style).
    #[must_use]
    pub fn with_script(mut self, script: Option<String>) -> Self {
        self.script = script;
        self
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── RecipeRequirement ────────────────────────────────────────────

    #[test]
    fn test_req_from_inf_minimal() {
        let req = RecipeRequirement::from_inf_value("24");
        assert_eq!(req.item_id, 24);
        assert_eq!(req.count, 1);
        assert!(!req.stay);
    }

    #[test]
    fn test_req_from_inf_with_count() {
        let req = RecipeRequirement::from_inf_value("24,5");
        assert_eq!(req.item_id, 24);
        assert_eq!(req.count, 5);
        assert!(!req.stay);
    }

    #[test]
    fn test_req_from_inf_with_stay() {
        let req = RecipeRequirement::from_inf_value("24,1,stay");
        assert_eq!(req.item_id, 24);
        assert_eq!(req.count, 1);
        assert!(req.stay);
    }

    #[test]
    fn test_req_from_inf_with_count_and_stay() {
        let req = RecipeRequirement::from_inf_value("42,3,stay");
        assert_eq!(req.item_id, 42);
        assert_eq!(req.count, 3);
        assert!(req.stay);
    }

    #[test]
    fn test_req_from_inf_empty() {
        let req = RecipeRequirement::from_inf_value("");
        assert_eq!(req.item_id, 0);
        assert_eq!(req.count, 1);
        assert!(!req.stay);
    }

    // ── RecipeResult ─────────────────────────────────────────────────

    #[test]
    fn test_gen_from_inf_minimal() {
        let result = RecipeResult::from_inf_value("25");
        assert_eq!(result.item_id, 25);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_gen_from_inf_with_count() {
        let result = RecipeResult::from_inf_value("25,10");
        assert_eq!(result.item_id, 25);
        assert_eq!(result.count, 10);
    }

    #[test]
    fn test_gen_from_inf_empty() {
        let result = RecipeResult::from_inf_value("");
        assert_eq!(result.item_id, 0);
        assert_eq!(result.count, 1);
    }

    // ── RecipeDef ────────────────────────────────────────────────────

    #[test]
    fn test_recipe_from_combi_fields_basic() {
        let mut fields = HashMap::new();
        fields.insert("id".into(), vec!["bendablebranch".into()]);
        fields.insert("req".into(), vec!["24".into(), "38".into()]);
        fields.insert("gen".into(), vec!["25".into()]);

        let recipe = RecipeDef::from_combi_fields(1, &fields).unwrap();
        assert_eq!(recipe.serial_id, 1);
        assert_eq!(recipe.id.as_deref(), Some("bendablebranch"));
        assert_eq!(recipe.requirements.len(), 2);
        assert_eq!(recipe.requirements[0].item_id, 24);
        assert_eq!(recipe.requirements[1].item_id, 38);
        assert_eq!(recipe.results.len(), 1);
        assert_eq!(recipe.results[0].item_id, 25);
    }

    #[test]
    fn test_recipe_missing_gen_returns_none() {
        let fields = HashMap::new();
        assert!(RecipeDef::from_combi_fields(1, &fields).is_none());
    }

    #[test]
    fn test_recipe_with_genname() {
        let mut fields = HashMap::new();
        fields.insert("gen".into(), vec!["82".into()]);
        fields.insert("genname".into(), vec!["Dough".into()]);

        let recipe = RecipeDef::from_combi_fields(2, &fields).unwrap();
        assert_eq!(recipe.gen_name.as_deref(), Some("Dough"));
    }

    #[test]
    fn test_recipe_with_script() {
        let mut fields = HashMap::new();
        fields.insert("gen".into(), vec!["43".into()]);
        fields.insert("req".into(), vec!["42,9".into(), "23,1,stay".into()]);

        let recipe = RecipeDef::from_combi_fields(3, &fields)
            .unwrap()
            .with_script(Some("play \"grind.wav\";".into()));
        assert_eq!(
            recipe.script.as_deref(),
            Some("play \"grind.wav\";")
        );
    }

    #[test]
    fn test_recipe_serde_derives_compile() {
        // The derives are checked at compile time.
        // This test ensures the struct is constructible.
        let recipe = RecipeDef {
            serial_id: 1,
            id: Some("test_recipe".into()),
            requirements: vec![RecipeRequirement {
                item_id: 10,
                count: 2,
                stay: false,
            }],
            results: vec![RecipeResult {
                item_id: 20,
                count: 1,
            }],
            gen_name: None,
            script: None,
        };
        assert_eq!(recipe.serial_id, 1);
        assert_eq!(recipe.results.len(), 1);
    }
}
