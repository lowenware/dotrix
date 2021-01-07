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

pub struct Octree<T> {
    pub root: Vec3i,
    pub nodes: HashMap<Vec3i, Node<T>>,
    pub depth: usize,
    pub size: usize,
}

#[derive(Debug)]
pub struct Node<T> {
    pub level: usize,
    pub size: usize,
    pub children: Option<[Vec3i; 8]>,
    pub payload: Option<T>,
}

impl<T> Octree<T> {
    pub fn new(root: Vec3i, size: usize) -> Self {
        let mut nodes = HashMap::new();

        nodes.insert(root, Node {
            level: 0,
            size,
            payload: None,
            children: Some(Node::<T>::children(&root, size as i32 / 4)),
        });

        Self {
            size,
            root,
            nodes,
            depth: 1,
        }
    }

    pub fn store(&mut self, key: Vec3i, payload: T) {
        if key == self.root {
            if let Some(root) = self.nodes.get_mut(&key) {
                root.payload = Some(payload);
            }
        } else {
            self.store_node(key, payload, self.size / 2, 1);
        }
    }

    fn store_node(
        &mut self,
        target: Vec3i,
        payload: T,
        size: usize,
        level: usize
    ) {

        let offset = size as i32 / 2;

        let node = Vec3i::new(
            (target.x as f32 / size as f32).floor() as i32 * size as i32 + offset,
            (target.y as f32 / size as f32).floor() as i32 * size as i32 + offset,
            (target.z as f32 / size as f32).floor() as i32 * size as i32 + offset
        );
        // let index = Self::child_index((node - parent), offset);

        let payload = {
            let mut child = self.nodes.entry(node).or_insert(Node {
                level,
                size,
                children: None,
                payload: None,
            });

            if node == target {
                child.payload = Some(payload);
                None
            } else {
                if child.children.is_none() {
                    // println!("set children for {:?}", node);
                    child.children = Some(Node::<T>::children(&node, offset / 2));
                }
                if self.depth == level {
                    self.depth += 1;
                }
                Some(payload)
            }
        };

        if let Some(payload) = payload {
            self.store_node(target, payload, offset as usize, level + 1);
        }
    }

    pub fn load(&self, key: &Vec3i) -> Option<&Node<T>> {
        self.nodes.get(&key).map(|n| n.payload.as_ref().map(|_| n)).unwrap_or(None)
    }

    pub fn find(&self, key: &Vec3i) -> Option<(Vec3i, &Node<T>)> {
        if let Some(node) = self.nodes.get(key) {
            if node.payload.is_some() {
                return Some((*key, node));
            }
        }
        let half_size = self.size as i32 / 2 + 1;
        if key.x.abs() < half_size && key.y.abs() < half_size && key.z.abs() < half_size {
            self.find_child(key, Vec3i::new(0, 0, 0), self.size / 2)
        } else {
            None
        }
    }

    fn find_child(&self, target: &Vec3i, cursor: Vec3i, size: usize) -> Option<(Vec3i, &Node<T>)> {
        let offset = size as i32 / 2;
        let node = Vec3i::new(
            (target.x as f32 / size as f32).floor() as i32 * size as i32 + offset,
            (target.y as f32 / size as f32).floor() as i32 * size as i32 + offset,
            (target.z as f32 / size as f32).floor() as i32 * size as i32 + offset
        );

        if let Some(child) = self.nodes.get(&node) {
            return if child.children.is_some() {
                self.find_child(target, node, offset as usize)
            } else {
                child.payload.as_ref().map(|_| (node, child))
            };
        }

        // fallback
        self.nodes.get(&cursor)
            .map(|node| node.payload.as_ref().map(|_| (cursor, node)))
            .unwrap_or(None)
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn children(&self, key: &Vec3i) -> Option<&[Vec3i; 8]> {
        self.nodes.get(&key)
            .map(|n| n.children.as_ref())
            .unwrap_or(None)
    }
}

impl<T> Node<T> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn children_indices_are_correct() {
        let parent = Vec3i::new(0, 0, 0);
        let children = Node::<u32>::children(&parent, 1);
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
        let mut octree = Octree::<u32>::new(Vec3i::new(0, 0, 0), 32);
        octree.store(Vec3i::new(-15, 1, -9), 123);
        assert_eq!(octree.nodes.len(), 5);
        let node = octree.load(&Vec3i::new(-15, 1, -9));
        assert_eq!(node.unwrap().payload.unwrap(), 123);
    }

    #[test]
    fn can_find_highest_available_lod() {
        let mut octree = Octree::<u32>::new(Vec3i::new(0, 0, 0), 32);
        octree.store(Vec3i::new(-15, 1, -9), 1);
        octree.store(Vec3i::new(8, 8, 8), 2);

        let (key, node) = octree.find(&Vec3i::new(-15, 1, -9)).unwrap();
        assert_eq!(key, Vec3i::new(-15, 1, -9));
        assert_eq!(node.payload.unwrap(), 1);

        let (key, node) = octree.find(&Vec3i::new(4, 0, 4)).unwrap();
        assert_eq!(key, Vec3i::new(8, 8, 8));
        assert_eq!(node.payload.unwrap(), 2);

        let (key, node) = octree.find(&Vec3i::new(12, 0, 4)).unwrap();
        assert_eq!(key, Vec3i::new(8, 8, 8));
        assert_eq!(node.payload.unwrap(), 2);
    }
}
