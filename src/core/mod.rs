use std::ops::Range;

pub struct StrChunkIter<'a> {
    v: &'a str,
    chunk_size: usize,
}

pub trait StrChunk {
    fn chunks(&self, chunk_size: usize) -> StrChunkIter<'_>;
}

pub fn clamp<T>(value: T, min: T, max: T) -> T where T: Copy + PartialOrd {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn range_intersect<T: Copy + PartialOrd + Ord>(a: Range<T>, b: Range<T>) -> Range<T> {
    use std::cmp::{max, min};

    let m = max(a.start, b.start);
    let n = min(a.end, b.end);

    if m > n {
        m..m
    } else {
        m..n
    }
}

impl<T> StrChunk for T where T: AsRef<str> {
    fn chunks(&self, chunk_size: usize) -> StrChunkIter<'_> {
        assert_ne!(chunk_size, 0);
        StrChunkIter { v: self.as_ref(), chunk_size }
    }
}

impl<'a> Iterator for StrChunkIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.v.len() > 0 {
            let upper_bound = self.chunk_size.min(self.v.len());
            let slice = &self.v[0..upper_bound];
            self.v = &self.v[upper_bound..];

            Some(slice)
        } else {
            None
        }
    }
}

#[cfg_attr(test, test)]
fn test_chunks() {
    let s = "00 01 02 03";
    let mut it = s.chunks(3);

    assert_eq!(it.next(), Some("00 "));
    assert_eq!(it.next(), Some("01 "));
    assert_eq!(it.next(), Some("02 "));
    assert_eq!(it.next(), Some("03"));
    assert_eq!(it.next(), None);
}
