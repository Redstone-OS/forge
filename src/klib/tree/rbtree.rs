/// Arquivo: klib/tree/rbtree.rs
///
/// Propósito: Árvore Rubro-Negra (Red-Black Tree).
/// Estrutura de dados balanceada para busca eficiente.
///
/// Detalhes de Implementação:
/// - Chaves ordenáveis (Ord).
/// - Nós alocados na heap (alloc).
/// - Implementação simplificada (BST por enquanto) com estrutura para cores.

/// Red-Black Tree
use alloc::boxed::Box;
use core::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Red,
    Black,
}

struct Node<K, V> {
    key: K,
    value: V,
    color: Color,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            color: Color::Red,
            left: None,
            right: None,
        }
    }
}

pub struct RBTree<K, V> {
    root: Option<Box<Node<K, V>>>,
    len: usize,
}

impl<K: Ord, V> RBTree<K, V> {
    pub const fn new() -> Self {
        Self { root: None, len: 0 }
    }

    pub fn insert(&mut self, key: K, value: V) {
        // TODO: Implementar balanceamento RB completo.
        // Por enquanto, inserção BST simples.
        self.len += 1;
        let new_node = Box::new(Node::new(key, value));

        match &mut self.root {
            None => {
                self.root = Some(new_node);
                // Raiz é sempre preta
                if let Some(ref mut node) = self.root {
                    node.color = Color::Black;
                }
            }
            Some(root) => {
                Self::insert_recursive(root, new_node);
            }
        }
    }

    fn insert_recursive(node: &mut Node<K, V>, new_node: Box<Node<K, V>>) {
        match new_node.key.cmp(&node.key) {
            Ordering::Less => match &mut node.left {
                None => node.left = Some(new_node),
                Some(left) => Self::insert_recursive(left, new_node),
            },
            Ordering::Greater | Ordering::Equal => match &mut node.right {
                None => node.right = Some(new_node),
                Some(right) => Self::insert_recursive(right, new_node),
            },
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut current = &self.root;
        while let Some(node) = current {
            match key.cmp(&node.key) {
                Ordering::Equal => return Some(&node.value),
                Ordering::Less => current = &node.left,
                Ordering::Greater => current = &node.right,
            }
        }
        None
    }
}
