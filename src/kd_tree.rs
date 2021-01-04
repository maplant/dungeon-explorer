//! Simple KD-tree for determining rectangle overlap.
use crate::rect::Rect;

/// Simple KD-tree for determining if rectangles overlap during map generation.
/// Adding rectangles is irreversible.
///
/// This data structure does not automatically balance itself, and thus its
/// efficiency depends on the rectangles to be spatially balanced by being unable to
/// overlap with each other.
///
/// This invariant is not upheld by any of the functions to insert rectangles; it must
/// be upheld by the caller.
#[derive(Debug)]
pub enum KDTree {
    Empty,
    Populated { root: TreeNode },
}

impl KDTree {
    /// Add a rectangle to the KDTree. This cannot be undone.
    pub fn add_rect(&mut self, r: Rect) {
        match self {
            Self::Empty => {
                *self = KDTree::Populated {
                    root: TreeNode {
                        left: None,
                        right: None,
                        rect: r,
                    },
                }
            }
            Self::Populated { ref mut root } => root.add_rect(r, 0_usize),
        }
    }

    /// Determine if a rectangle overlaps with any in the tree.
    pub fn overlaps(&self, r: &Rect) -> bool {
        match self {
            Self::Empty => false,
            Self::Populated { ref root } => root.overlaps(r, 0_usize),
        }
    }
}

impl Default for KDTree {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug)]
pub struct TreeNode {
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
    rect: Rect,
}

impl TreeNode {
    fn new(rect: Rect) -> Self {
        Self {
            left: None,
            right: None,
            rect,
        }
    }

    fn add_rect(&mut self, r: Rect, curr_dim: usize) {
        let next_dim = (curr_dim + 1) % 4;
        if r.is_dim_less(&self.rect, curr_dim) {
            if self.left.is_none() {
                self.left = Some(Box::new(TreeNode::new(r)));
            } else {
                self.left.as_mut().map(|l| l.add_rect(r, next_dim));
            }
        } else {
            if self.right.is_none() {
                self.right = Some(Box::new(TreeNode::new(r)));
            } else {
                self.right.as_mut().map(|n| n.add_rect(r, next_dim));
            }
        }
    }

    fn overlaps(&self, rect: &Rect, curr_dim: usize) -> bool {
        if self.rect.overlaps(rect) {
            return true;
        }
        let next_dim = (curr_dim + 1) % 4;
        match curr_dim {
            i @ 0..=1 => {
                if self
                    .left
                    .as_ref()
                    .map_or(false, |l| l.overlaps(rect, next_dim))
                {
                    true
                } else if rect.max[i] < self.rect.min[i] {
                    false
                } else {
                    self.right
                        .as_ref()
                        .map_or(false, |r| r.overlaps(rect, next_dim))
                }
            }
            i @ 2..=3 => {
                let i = i - 2;
                if self
                    .right
                    .as_ref()
                    .map_or(false, |r| r.overlaps(rect, next_dim))
                {
                    true
                } else if rect.min[i] > self.rect.max[i] {
                    false
                } else {
                    self.left
                        .as_ref()
                        .map_or(false, |l| l.overlaps(rect, next_dim))
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_overlap() {
        let r1 = Rect {
            min: point!(0, 0),
            max: point!(10, 10),
        };
        let r2 = Rect {
            min: point!(10, 10),
            max: point!(20, 20),
        };
        let r3 = Rect {
            min: point!(20, 20),
            max: point!(30, 30),
        };
        let mut tree = KDTree::default();
        tree.add_rect(r1);
        tree.add_rect(r2);
        tree.add_rect(r3);
        let r1 = Rect {
            min: point!(5, 5),
            max: point!(10, 10),
        };
        let r2 = Rect {
            min: point!(30, 30),
            max: point!(40, 40),
        };
        let r3 = Rect {
            min: point!(25, 25),
            max: point!(40, 40),
        };
        assert!(tree.overlaps(&r1));
        assert!(!tree.overlaps(&r2));
        assert!(tree.overlaps(&r3));
    }
}
