//! `:key` on an element or component outside `v-for`.
//!
//! Vue's Vapor compiler gives such a node a keyed block (`createKeyedFragment`),
//! so a new key replaces the node instead of patching it: an animation, a
//! transition or a component's state starts again. The legacy lane has no keyed
//! block, so before the core transforms run, each such node gains a `v-for`
//! over the single current key. The loop keys its one item by the node's own
//! `:key`, which replaces the node exactly when the key changes, as a keyed
//! block does. A keyed `v-if` branch first moves its branch directive onto a
//! wrapping `<template>`, since a node cannot carry both.

use vize_atelier_core::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, RootNode,
    SimpleExpressionNode, TemplateChildNode,
};
use vize_carton::{Allocator, Box, Vec, cstr};

/// The alias the synthetic loop binds; the node never reads it.
const KEYED_ALIAS: &str = "__vize_keyed";
const BRANCH_DIRECTIVES: [&str; 3] = ["if", "else-if", "else"];

/// Gives every node with a dynamic `:key` and no `v-for` of its own a loop over
/// that key, so the transforms below build the keyed block Vue's compiler would.
pub(crate) fn key_outside_for<'a>(allocator: &'a Allocator, root: &mut RootNode<'a>) {
    visit_children(allocator, &mut root.children, false);
}

fn visit_children<'a>(
    allocator: &'a Allocator,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    in_once: bool,
) {
    for child in children.iter_mut() {
        let TemplateChildNode::Element(element) = child else {
            continue;
        };
        let in_once = in_once || has_directive(element, "once");
        if !in_once && key_of(element).is_some() && is_branch(element) {
            wrap_branch(allocator, child);
        }
        let TemplateChildNode::Element(element) = child else {
            continue;
        };
        if !in_once && let Some(key) = key_of(element).filter(|_| !is_branch(element)) {
            let keyed_loop = loop_over(allocator, key, element);
            element.props.push(keyed_loop);
        }
        vize_carton::ensure_sufficient_stack(|| {
            visit_children(allocator, &mut element.children, in_once);
        });
    }
}

/// Moves a keyed branch's `v-if` / `v-else-if` / `v-else` onto a `<template>`
/// that takes its place, so the branch renders the keyed node as its content.
fn wrap_branch<'a>(allocator: &'a Allocator, slot: &mut TemplateChildNode<'a>) {
    let TemplateChildNode::Element(element) = slot else {
        return;
    };
    let mut wrapper = ElementNode::new(allocator, "template", element.loc.clone());
    wrapper.tag_type = ElementType::Template;
    wrapper.ns = element.ns;
    let mut kept = Vec::new_in(&allocator);
    for prop in std::mem::replace(&mut element.props, Vec::new_in(&allocator)) {
        let branch = matches!(&prop, PropNode::Directive(directive) if BRANCH_DIRECTIVES.contains(&directive.name));
        if branch {
            wrapper.props.push(prop);
        } else {
            kept.push(prop);
        }
    }
    element.props = kept;
    let placeholder = TemplateChildNode::Element(Box::new_in(
        ElementNode::new(allocator, "template", element.loc.clone()),
        &allocator,
    ));
    let keyed = std::mem::replace(slot, placeholder);
    wrapper.children.push(keyed);
    *slot = TemplateChildNode::Element(Box::new_in(wrapper, &allocator));
}

fn loop_over<'a>(
    allocator: &'a Allocator,
    key: &'a str,
    element: &ElementNode<'a>,
) -> PropNode<'a> {
    let content = cstr!("{KEYED_ALIAS} in [{key}]");
    let loc = element.loc.clone();
    let mut directive = DirectiveNode::new(allocator, "for", loc.clone());
    directive.raw_name = Some("v-for");
    directive.exp = Some(ExpressionNode::Simple(Box::new_in(
        SimpleExpressionNode::new(allocator.alloc_str(&content), false, loc),
        &allocator,
    )));
    PropNode::Directive(Box::new_in(directive, &allocator))
}

/// The `:key` expression of a node that Vue's Vapor compiler would key: a
/// dynamic key on a node without `v-for`, and not on a `<template>`, whose key
/// keys a `v-if` branch or a slot.
fn key_of<'a>(element: &ElementNode<'a>) -> Option<&'a str> {
    if has_directive(element, "for") || element.tag_type == ElementType::Template {
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

fn is_branch(element: &ElementNode<'_>) -> bool {
    BRANCH_DIRECTIVES
        .iter()
        .any(|name| has_directive(element, name))
}

fn has_directive(element: &ElementNode<'_>, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == name))
}
