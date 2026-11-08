/// Centralized Feature and Function Catalog
/// This lightweight list is sent with the initial prompt so the AI knows
/// exactly which features and functions exist, without loading heavy schemas.
pub const FEATURE_CATALOG: &str = r#"
[AVAILABLE SYSTEM CAPABILITIES]
- schedule:
    functions: [create, list, cancel]
- system:
    functions: [capture_screen, launch_app]
"#;

pub fn get_catalog() -> &'static str {
    FEATURE_CATALOG
}
