//! Component listener keys: a static `@event` binds Vue's
//! `toHandlerKey(camelize(event))`, which is the key `emit` looks up.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "regression assertions use std strings and format"
)]

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn compile(source: &str, davinci_retained_lane: bool) -> String {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            davinci_retained_lane,
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    result.code.to_string()
}

#[test]
fn kebab_case_component_listeners_bind_camelized_handler_keys() {
    let source =
        r#"<Child @close-preset="close" @update:selected-preset-id="pick" @save="save" />"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains("onClosePreset:"), "{code}");
        assert!(code.contains(r#""onUpdate:selectedPresetId":"#), "{code}");
        assert!(code.contains("onSave:"), "{code}");
        assert!(!code.contains("onClose-preset"), "{code}");
        assert!(!code.contains("onUpdate:selected-preset-id"), "{code}");
    }
}
