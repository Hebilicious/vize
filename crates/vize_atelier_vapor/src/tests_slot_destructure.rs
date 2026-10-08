//! Renamed and nested destructuring in `v-slot` reads each name from its own
//! path in the slot props, as Vue's compiler does.

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
fn a_renamed_slot_prop_reads_its_original_key() {
    let source =
        r#"<Child v-slot="{ props: trigger }"><button v-bind="trigger">x</button></Child>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains("_slotProps0.props"), "{code}");
        assert!(!code.contains("_slotProps0.trigger"), "{code}");
    }
}

#[test]
fn a_nested_slot_prop_reads_its_full_path() {
    let source = r#"<Child v-slot="{ item: { label }, index }"><span>{{ label }} {{ index }}</span></Child>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains("_slotProps0.item.label"), "{code}");
        assert!(code.contains("_slotProps0.index"), "{code}");
    }
}
