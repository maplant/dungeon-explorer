//! Rectangle geometric primitive.
use cgmath::*;
use std::cmp::Ordering;

/// A two-dimensional rectangle.
#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub(crate) min: Point2<i32>,
    pub(crate) max: Point2<i32>,
}

impl Rect {
    /// Determines if the two rectangles are overlapping.
    pub fn overlaps(&self, rhs: &Self) -> bool {
        self.min.x < rhs.max.x
            && self.max.x > rhs.min.x
            && self.min.y < rhs.max.y
            && self.max.y > rhs.min.y
    }

    /// Rectangles implement a partial order by comparing their min and max
    /// vectors lexicographically. Because this function is only useful for
    /// balancing KD-trees, we don't provide this as Rect's implementation of
    /// PartialEq.
    fn is_lex_less(&self, rhs: &Self) -> bool {
        // Before I was using my own linear algebra library and these AsRefs
        // were unnecessary, as the point type implemented Deref. However,
        // my library is broken on the latest version of Rust (due to using
        // experimental features), so now we have this monstrosity.
        for (lhs, rhs) in AsRef::<[i32; 2]>::as_ref(&self.min)
            .iter()
            .zip(AsRef::<[i32; 2]>::as_ref(&rhs.min).iter())
            .chain(
                AsRef::<[i32; 2]>::as_ref(&self.max)
                    .iter()
                    .zip(AsRef::<[i32; 2]>::as_ref(&rhs.max).iter()),
            )
        {
            match lhs.partial_cmp(rhs) {
                Some(Ordering::Less) => return true,
                Some(Ordering::Greater) | None => return false,
                _ => (),
            }
        }
        false
    }

    /// Determines if `self` is of a lesser dimensional value than `rhs`.
    /// If the values are equal, resort to lexicographical sort.
    pub(crate) fn is_dim_less(&self, rhs: &Self, dim: usize) -> bool {
        let (lhs_dim, rhs_dim) = match dim {
            i @ 0..=1 => (self.min[i], rhs.min[i]),
            i @ 2..=3 => (self.max[i - 2], rhs.max[i - 2]),
            _ => unreachable!(),
        };
        match lhs_dim.partial_cmp(&rhs_dim) {
            Some(Ordering::Less) => true,
            Some(Ordering::Greater) | None => false,
            _ => self.is_lex_less(rhs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_overlap() {
        let r1 = Rect {
            min: Point2::new(0, 0),
            max: Point2::new(10, 10),
        };
        let r2 = Rect {
            min: Point2::new(10, 10),
            max: Point2::new(20, 20),
        };
        assert!(!r1.overlaps(&r2));
        assert!(!r2.overlaps(&r1));
        let r1 = Rect {
            min: Point2::new(0, 0),
            max: Point2::new(10, 10),
        };
        let r2 = Rect {
            min: Point2::new(5, 5),
            max: Point2::new(10, 10),
        };
        assert!(r1.overlaps(&r2));
        assert!(r2.overlaps(&r1));
    }
}
