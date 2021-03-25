//! Octree implementation to store voxel maps
use std::collections::HashMap;

use dotrix_math::Vec3i;

const LEFT_TOP_BACK: usize = 0;
const RIGHT_TOP_BACK: usize = 1;
const RIGHT_TOP_FRONT: usize = 2;
const LEFT_TOP_FRONT: usize = 3;
const LEFT_BOTTOM_BACK: usize = 4;
const RIGHT_BOTTOM_BACK: usize = 5;
const RIGHT_BOTTOM_FRONT: usize = 6;
const LEFT_BOTTOM_FRONT: usize = 7;

/// Octree
pub struct Octree {
    /// Root node position
    pub root: Vec3i,
    /// Nodes storage
    pub nodes: HashMap<Vec3i, Node>,
    /// Depth of the tree
    pub depth: usize,
    /// Size of the root node
    pub size: usize,
}

/// Node of the [`Octree`]
#[derive(Debug)]
pub struct Node {
    /// Node level
    pub level: usize,
    /// Node size
    pub size: usize,
    /// Positions of children
    pub children: Option<[Vec3i; 8]>,
}

impl Octree {
    /// Constructs the [`Octree`] from a node
    pub fn new(root: Vec3i, size: usize) -> Self {
        let mut nodes = HashMap::new();

        nodes.insert(root, Node {
            level: 0,
            size,
            children: Some(Node::children(&root, size as i32 / 4)),
        });

        Self {
            size,
            root,
            nodes,
            depth: 1,
        }
    }

    /// Stores a new node at specified position
    pub fn store(&mut self, key: Vec3i) {
        self.store_node(self.root, self.size, 0, key);
    }

    fn store_node(
        &mut self,
        cursor: Vec3i,
        cursor_size: usize,
        cursor_level: usize,
        target: Vec3i
    ) -> bool {
        let mut cursor_node = self.nodes
            .entry(cursor)
            .or_insert(
                Node {
                    level: cursor_level,
                    size: cursor_size,
                    children: None,
                }
            );

        if cursor == target {
            return true;
        }

        // Check if cursor owns the target node
        let offset = cursor_size as i32 / 2;

        if cursor_level != 0 {
            let target_parent = Vec3i::new(
                (target.x as f32 / cursor_size as f32).floor() as i32 * cursor_size as i32 + offset,
                (target.y as f32 / cursor_size as f32).floor() as i32 * cursor_size as i32 + offset,
                (target.z as f32 / cursor_size as f32).floor() as i32 * cursor_size as i32 + offset
            );

            if target_parent != cursor {
                return false;
            }
        }

        if cursor_node.children.is_none() {
            cursor_node.children = Some(Node::children(&cursor, offset / 2));
        }

        let children = cursor_node.children.unwrap();

        let mut is_stored = false;
        for &child in children.iter() {
            if self.store_node(child, offset as usize, cursor_level + 1, target) {
                is_stored = true;
            }
        }
        is_stored
    }

    /// Loads node from the [`Octree`] by the position
    pub fn load(&self, key: &Vec3i) -> Option<&Node> {
        self.nodes.get(&key)
    }

    /// Loads node as mutable from the [`Octree`] by the position
    pub fn load_mut(&mut self, key: &Vec3i) -> Option<&mut Node> {
        self.nodes.get_mut(&key)
    }

    /// Finds node or it's closest parent in the [`Octree`]
    pub fn find(&self, key: &Vec3i) -> Option<(Vec3i, &Node)> {
        if let Some(node) = self.nodes.get(key) {
            return Some((*key, node));
        }
        let half_size = self.size as i32 / 2 + 1;
        if key.x.abs() < half_size && key.y.abs() < half_size && key.z.abs() < half_size {
            self.find_child(key, Vec3i::new(0, 0, 0), self.size / 2)
        } else {
            None
        }
    }

    fn find_child(&self, target: &Vec3i, cursor: Vec3i, size: usize) -> Option<(Vec3i, &Node)> {
        let offset = size as i32 / 2;
        let node = Vec3i::new(
            (target.x as f32 / size as f32).floor() as i32 * size as i32 + offset,
            (target.y as f32 / size as f32).floor() as i32 * size as i32 + offset,
            (target.z as f32 / size as f32).floor() as i32 * size as i32 + offset
        );

        self.nodes.get(&node)
            .map(
                |child| child.children.as_ref().map(
                    |_| self.find_child(target, node, offset as usize)
                ).unwrap_or_else(|| Some((node, child)))
            ).unwrap_or_else(
                || self.nodes.get(&cursor).map(|node| (cursor, node))
            )
        /*
            )
        if let Some(child) = self.nodes.get(&node) {
            return if child.children.is_some() {
                self.find_child(target, node, offset as usize)
            } else {
                (node, child)
            };
        }

        // fallback
        self.nodes.get(&cursor)
            .map(|node| node.payload.as_ref().map(|_| (cursor, node)))
            .unwrap_or(None)
        */
    }

    /// Returns size of the [`Octree`]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns children positions of the specified [`Octree`] node by its position
    pub fn children(&self, key: &Vec3i) -> Option<&[Vec3i; 8]> {
        self.nodes.get(&key)
            .map(|n| n.children.as_ref())
            .unwrap_or(None)
    }
}

impl Node {
    /// Calculates children positions of a [`Node`] by its position
    pub fn children(parent: &Vec3i, offset: i32) -> [Vec3i; 8] {
        let mut res = [Vec3i::new(0, 0, 0); 8];
        for (i, child) in res.iter_mut().enumerate() {
            let (x, y, z) = match i {
                LEFT_TOP_BACK => (parent.x - offset, parent.y + offset, parent.z - offset),
                RIGHT_TOP_BACK => (parent.x + offset, parent.y + offset, parent.z - offset),
                RIGHT_TOP_FRONT => (parent.x + offset, parent.y + offset, parent.z + offset),
                LEFT_TOP_FRONT => (parent.x - offset, parent.y + offset, parent.z + offset),
                LEFT_BOTTOM_BACK => (parent.x - offset, parent.y - offset, parent.z - offset),
                RIGHT_BOTTOM_BACK => (parent.x + offset, parent.y - offset, parent.z - offset),
                RIGHT_BOTTOM_FRONT => (parent.x + offset, parent.y - offset, parent.z + offset),
                LEFT_BOTTOM_FRONT => (parent.x - offset, parent.y - offset, parent.z + offset),
                _ => panic!("cube has only 8 corners"),
            };
            child.x = x;
            child.y = y;
            child.z = z;
        }
        res
    }

    // Get node base coordinate
    // pub fn base(&self) -> Vec3i {
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn children_indices_are_correct() {
        let parent = Vec3i::new(0, 0, 0);
        let children = Node::children(&parent, 1);
        assert_eq!(children[LEFT_TOP_BACK], Vec3i::new(-1, 1, -1));
        assert_eq!(children[RIGHT_TOP_BACK], Vec3i::new(1, 1, -1));
        assert_eq!(children[RIGHT_TOP_FRONT], Vec3i::new(1, 1, 1));
        assert_eq!(children[LEFT_TOP_FRONT], Vec3i::new(-1, 1, 1));
        assert_eq!(children[LEFT_BOTTOM_BACK], Vec3i::new(-1, -1, -1));
        assert_eq!(children[RIGHT_BOTTOM_BACK], Vec3i::new(1, -1, -1));
        assert_eq!(children[RIGHT_BOTTOM_FRONT], Vec3i::new(1, -1, 1));
        assert_eq!(children[LEFT_BOTTOM_FRONT], Vec3i::new(-1, -1, 1));
    }

    #[test]
    fn can_store_and_load_a_node() {
        let mut octree = Octree::new(Vec3i::new(0, 0, 0), 32);
        octree.store(Vec3i::new(-8, -8, -8));
        assert_eq!(octree.nodes.len(), 9);
        octree.store(Vec3i::new(-15, 1, -9));
        assert_eq!(octree.nodes.len(), 33);
        let node = octree.load(&Vec3i::new(-15, 1, -9));
        assert_eq!(node.is_some(), true);
    }

    #[test]
    fn can_find_highest_available_lod() {
        let mut octree = Octree::new(Vec3i::new(0, 0, 0), 32);
        octree.store(Vec3i::new(-15, 1, -9));
        octree.store(Vec3i::new(8, 8, 8));

        let (key, node) = octree.find(&Vec3i::new(-15, 1, -9)).unwrap();
        assert_eq!(key, Vec3i::new(-15, 1, -9));

        let (key, node) = octree.find(&Vec3i::new(4, 0, 4)).unwrap();
        assert_eq!(key, Vec3i::new(8, 8, 8));

        let (key, node) = octree.find(&Vec3i::new(12, 0, 4)).unwrap();
        assert_eq!(key, Vec3i::new(8, 8, 8));
    }
}
