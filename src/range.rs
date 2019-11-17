use alloc::vec::Vec;
use core::cmp;
use core::cmp::Ordering;
use crate::constants::PAGE_SIZE;

fn is_aligned_by(n: usize, alignment: usize) -> bool {
    (n & (alignment - 1)) == 0
}

#[derive(Eq)]
#[derive(Clone, Debug, Default)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

impl Range {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    fn can_merge(&self, other: &Self) -> bool {
        self.start == other.end || self.end == other.start
    }

    fn merge(&mut self, other: Self) {
        assert!(self.can_merge(&other));
        self.start = cmp::min(self.start, other.start);
        self.end = cmp::max(self.end, other.end);
    }

    fn contains(&self, other: &Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    fn starts_before(&self, other: &Self) -> bool {
        self.start <= other.start
    }

    fn overlaps(&self, other: &Self) -> bool {
        (other.start <= self.start && self.start < other.end) ||
        (self.start <= other.start && other.start < self.end)
    }

    fn can_split(&self, len: usize) -> bool {
         self.len() > len
    }

    fn split(&mut self, len: usize) -> Self {
        assert!(is_aligned_by(len, PAGE_SIZE));
        let r = Range { start: self.end-len, end: self.end };
        self.end -= len;
        r
    }
}

impl Ord for Range {
    fn cmp(&self, other: &Self) -> Ordering {
        self.len().cmp(&other.len())
    }
}

impl PartialOrd for Range {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}



#[derive(Debug, Default)]
pub struct RangeSet {
    set: Vec<Range>,
}

impl RangeSet {
    pub const fn empty() -> Self {
        RangeSet { set: Vec::new() }
    }
    pub fn insert(&mut self, value: Range) {
        let mut insert_index = 0;
        for rg in self.set.iter_mut() {
            assert!(!rg.contains(&value));
            assert!(!rg.overlaps(&value));
            if rg.can_merge(&value) {
                rg.merge(value);
                return;
            }
            if !rg.starts_before(&value) {
                break;
            }
            insert_index += 1;
        }
        self.set.insert(insert_index, value);
    }

    pub fn find(&mut self, sz: usize) -> Option<Range> {
        let mut remove_at = None;
        for (i, rg) in self.set.iter_mut().enumerate() {
            if rg.len() == sz {
                remove_at = Some(i);
                break;
            }
            if rg.can_split(sz) {
                let r = Some(rg.split(sz));
                return r;
            }
        }
        if let Some(i) = remove_at {
            let r =  Some(self.set.remove(i));
            return r;
        }
        None
    }
}

#[cfg(test)]
mod tests {

    extern crate std;
    use super::*;
    #[test]
    fn basic_find_insert() {
        let mut rs = RangeSet::empty();
        rs.insert(Range::new(0, 0x1000));
        rs.insert(Range::new(0x2000, 0x4000));
        rs.insert(Range::new(0x5000, 0x6000));
        rs.insert(Range::new(0x6000, 0x7000));
        assert_eq!(rs.set.len(), 3);
        assert_eq!(rs.find(0x1000), Some(Range::new(0, 0x1000)));
        assert_eq!(rs.set.len(), 2);
        assert_eq!(rs.find(0x1000000), None);
        assert_eq!(rs.set.len(), 2);
        assert_eq!(rs.find(0x2000), Some(Range::new(0x2000, 0x4000)));
        assert_eq!(rs.set.len(), 1);
        assert_eq!(rs.find(0x1000), Some(Range::new(0x6000, 0x7000)));
        assert_eq!(rs.set.len(), 1);
        assert_eq!(rs.find(0x1000), Some(Range::new(0x5000, 0x6000)));
        assert_eq!(rs.set.len(), 0);
        assert_eq!(rs.find(0x1000), None);
        assert_eq!(rs.set.len(), 0);
    }
}
