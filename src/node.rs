use serde::{ser::SerializeStruct, Serialize, Serializer};
use tree_sitter::Node as OtherNode;

use crate::traits::Search;

/// An `AST` node.
#[derive(Clone, Copy)]
pub struct Node<'a>(OtherNode<'a>);

impl<'a> Node<'a> {
    /// Checks if a node represents a syntax error or contains any syntax errors
    /// anywhere within it.
    pub fn has_error(&self) -> bool {
        self.0.has_error()
    }

    pub(crate) fn new(node: OtherNode<'a>) -> Self {
        Node(node)
    }

    pub(crate) fn object(&self) -> OtherNode<'a> {
        self.0
    }
}

impl<'a> Search<'a> for Node<'a> {
    fn first_occurence(&self, pred: fn(u16) -> bool) -> Option<Node<'a>> {
        let mut cursor = self.0.walk();
        let mut stack = Vec::new();
        let mut children = Vec::new();

        stack.push(*self);

        while let Some(node) = stack.pop() {
            if pred(node.0.kind_id()) {
                return Some(node);
            }
            cursor.reset(node.0);
            if cursor.goto_first_child() {
                loop {
                    children.push(Node::new(cursor.node()));
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                for child in children.drain(..).rev() {
                    stack.push(child);
                }
            }
        }

        None
    }

    fn act_on_node(&self, action: &mut dyn FnMut(&Node<'a>)) {
        let mut cursor = self.0.walk();
        let mut stack = Vec::new();
        let mut children = Vec::new();

        stack.push(*self);

        while let Some(node) = stack.pop() {
            action(&node);
            cursor.reset(node.0);
            if cursor.goto_first_child() {
                loop {
                    children.push(Node::new(cursor.node()));
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                for child in children.drain(..).rev() {
                    stack.push(child);
                }
            }
        }
    }

    fn first_child(&self, pred: fn(u16) -> bool) -> Option<Node<'a>> {
        let mut cursor = self.0.walk();
        for child in self.0.children(&mut cursor) {
            if pred(child.kind_id()) {
                return Some(Node::new(child));
            }
        }
        None
    }

    fn act_on_child(&self, action: &mut dyn FnMut(&Node<'a>)) {
        let mut cursor = self.0.walk();
        for child in self.0.children(&mut cursor) {
            action(&Node::new(child));
        }
    }
}

impl<'a> Serialize for Node<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let node = &self.object();
        let start_position = node.start_position();
        let end_position = node.end_position();

        let mut s = serializer.serialize_struct("Node", 6)?;
        s.serialize_field("kind", &node.kind_id())?;
        s.serialize_field("name", node.kind())?;
        s.serialize_field("start_line",&(start_position.row + 1))?;
        s.serialize_field("start_column",&(start_position.column + 1))?;
        s.serialize_field("end_line",&(end_position.row + 1))?;
        s.serialize_field("end_column",&(end_position.column + 1))?;
        s.end()
    }
}
