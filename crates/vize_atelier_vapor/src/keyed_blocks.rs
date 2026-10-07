//! `:key` on an element or component outside `v-for`.
//!
//! Vue's Vapor compiler gives such a node a keyed block (`createKeyedFragment`),
//! so a new key replaces the node instead of patching it: an animation, a
//! transition or a component's state starts again. The legacy lane has no keyed
//! block, so before the core transforms run, each such node gains a `v-for`
//! over the single current key. The loop keys its one item by the node's own
//! `:key`, which replaces the node exactly when the key changes, as a keyed
//! block does.

use vize_atelier_core::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, RootNode,
    SimpleExpressionNode, TemplateChildNode,
};
use vize_carton::{Allocator, Box, cstr};

/// The alias the synthetic loop binds; the node never reads it.
const KEYED_ALIAS: &str = "__vize_keyed";

/// Gives every node with a dynamic `:key` and no `v-for` of its own a loop over
/// that key, so the transforms below build the keyed block Vue's compiler would.
pub(crate) fn key_outside_for<'a>(allocator: &'a Allocator, root: &mut RootNode<'a>) {
    for child in root.children.iter_mut() {
        visit(allocator, child, false);
    }
}

fn visit<'a>(allocator: &'a Allocator, node: &mut TemplateChildNode<'a>, in_once: bool) {
    let TemplateChildNode::Element(element) = node else {
        return;
    };
    let in_once = in_once || has_directive(element, "once");
    if !in_once && let Some(key) = keyed_without_loop(element) {
        let content = cstr!("{KEYED_ALIAS} in [{key}]");
        let loc = element.loc.clone();
        let mut directive = DirectiveNode::new(allocator, "for", loc.clone());
        directive.raw_name = Some("v-for");
        directive.exp = Some(ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(allocator.alloc_str(&content), false, loc),
            &allocator,
        )));
        element
            .props
            .push(PropNode::Directive(Box::new_in(directive, &allocator)));
    }
    for child in element.children.iter_mut() {
        vize_carton::ensure_sufficient_stack(|| visit(allocator, child, in_once));
    }
}

/// The `:key` expression of a node that Vue's Vapor compiler would key: a
/// dynamic key on a node without `v-for`, and not on a `<template>` that keys a
/// `v-if` branch or a slot.
fn keyed_without_loop<'a>(element: &ElementNode<'a>) -> Option<&'a str> {
    if has_directive(element, "for") {
        return None;
    }
    let structural_template = element.tag_type == ElementType::Template
        && ["if", "else-if", "else", "slot"]
            .iter()
            .any(|name| has_directive(element, name));
    // A branch's key keys the branch itself (`branch_key.rs` in the core).
    let branch = ["if", "else-if", "else"]
        .iter()
        .any(|name| has_directive(element, name));
    if structural_template || branch {
        return None;
    }
    element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(directive)
            if directive.name == "bind"
                && matches!(&directive.arg, Some(ExpressionNode::Simple(arg)) if arg.content == "key") =>
        {
            match &directive.exp {
                Some(ExpressionNode::Simple(exp)) if !exp.is_static && !exp.content.trim().is_empty() => {
                    Some(exp.content)
                }
                _ => None,
            }
        }
        _ => None,
    })
}

fn has_directive(element: &ElementNode<'_>, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == name))
}
