/// Centralized Feature and Function Catalog
/// This lightweight list is sent with the initial prompt so the AI knows
/// exactly which features and functions exist, without loading heavy schemas.
pub const FEATURE_CATALOG: &str = r#"
[AVAILABLE SYSTEM CAPABILITIES]
- schedule:
    functions: [create, list, cancel]
- system:
    functions: [capture_screen, launch_app]
- app:
    functions: [scan, open, close, create_group, delete_group, open_group, close_group, list_groups]
"#;

pub fn get_catalog() -> &'static str {
    FEATURE_CATALOG
}
