//! `:key` outside `v-for` replaces its node when the key changes, as Vue's
//! `createKeyedFragment` does; see `keyed_blocks.rs`.

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
fn a_keyed_element_is_a_block_keyed_by_its_key() {
    let source = r#"<div><button @click="version++">Next</button><p :key="version">Version {{ version }}</p></div>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains("_createFor(() => ([_ctx.version])"), "{code}");
        assert!(code.contains("(__vize_keyed) => (_ctx.version))"), "{code}");
        // The keyed element leaves its parent's template, which keeps the button.
        assert!(
            code.contains(r#"_template("<div><button>Next</button></div>", true)"#),
            "{code}"
        );
    }
}

#[test]
fn a_keyed_component_is_a_block_keyed_by_its_key() {
    let source = r#"<section><Pages :key="page" /></section>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains("_createFor(() => ([_ctx.page])"), "{code}");
        assert!(code.contains("(__vize_keyed) => (_ctx.page))"), "{code}");
    }
}

#[test]
fn keys_that_already_key_something_are_left_alone() {
    for source in [
        r#"<ul><li v-for="item in items" :key="item.id">{{ item.name }}</li></ul>"#,
        r#"<div><p key="fixed">Fixed</p></div>"#,
        r#"<div v-once><p :key="version">Once</p></div>"#,
    ] {
        let code = compile(source, true);
        assert!(!code.contains("__vize_keyed"), "{source}\n{code}");
    }
}

#[test]
fn a_keyed_branch_keys_its_node_and_keeps_its_else() {
    let source = r#"<div><p v-if="busy" :key="message">{{ message }}</p><p v-else>Idle</p></div>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains("_createIf(() => (_ctx.busy)"), "{code}");
        assert!(code.contains("_createFor(() => ([_ctx.message])"), "{code}");
        assert!(code.contains("(__vize_keyed) => (_ctx.message))"), "{code}");
        assert!(code.contains("Idle"), "{code}");
    }
}
