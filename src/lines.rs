//! Module providing utilities for working with collections of text lines.
//!
//! This module defines the `Lines` trait and its implementations to provide
//! consistent interfaces for iterating through lines in different directions
//! and with different options.

/// Trait for collections of text lines that provides bidirectional iteration.
///
/// This trait provides methods to iterate forward and backward through
/// collections of text lines, with optional limits and offset information.
/// It is implemented for slices of anything that can be viewed as a `str`,
/// and so is also usable on vectors.
pub trait Lines {
    /// Iterates forward through the lines.
    ///
    /// # Arguments
    /// * `limit` - Optional maximum number of lines to return
    ///
    /// # Returns
    /// An iterator yielding lines in forward order
    fn iter_forward(&self, limit: Option<usize>) -> impl Iterator<Item = &str>;

    /// Iterates forward through the lines with indices.
    ///
    /// # Arguments
    /// * `limit` - Optional maximum number of lines to return
    ///
    /// # Returns
    /// An iterator yielding (index, line) pairs in forward order
    fn enumerate_forward(&self, limit: Option<usize>) -> impl Iterator<Item = (usize, &str)>;

    /// Iterates backward through the lines.
    ///
    /// # Arguments
    /// * `limit` - Optional maximum number of lines to return
    ///
    /// # Returns
    /// An iterator yielding lines in reverse order
    fn iter_backward(&self, limit: Option<usize>) -> impl DoubleEndedIterator<Item = &str>;

    /// Iterates forward through the last N lines with indices.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of lines to return from the end
    ///
    /// # Returns
    /// An iterator yielding (index, line) pairs for the last `limit` lines
    fn enumerate_tail_forward(&self, limit: usize) -> impl Iterator<Item = (usize, &str)>;

    /// Iterates backward through the lines with original indices.
    ///
    /// # Arguments
    /// * `limit` - Optional maximum number of lines to return
    ///
    /// # Returns
    /// An iterator yielding (original_index, line) pairs in reverse order
    fn enumerate_backward(&self, limit: Option<usize>) -> impl Iterator<Item = (usize, &str)>;
}

impl<S: AsRef<str>> Lines for [S] {
    fn iter_forward(&self, limit: Option<usize>) -> impl Iterator<Item = &str> {
        self.iter()
            .take(limit.unwrap_or(self.len()))
            .map(AsRef::as_ref)
    }

    fn enumerate_forward(&self, limit: Option<usize>) -> impl Iterator<Item = (usize, &str)> {
        self.iter_forward(limit).enumerate()
    }

    fn iter_backward(&self, limit: Option<usize>) -> impl DoubleEndedIterator<Item = &str> {
        self.iter()
            .rev()
            .take(limit.unwrap_or(self.len()))
            .map(AsRef::as_ref)
    }

    fn enumerate_tail_forward(&self, limit: usize) -> impl Iterator<Item = (usize, &str)> {
        let start_offset = self.len().saturating_sub(limit);
        self[start_offset..]
            .iter()
            .map(AsRef::as_ref)
            .enumerate()
            .map(move |(i, s)| (i + start_offset, s))
    }

    fn enumerate_backward(&self, limit: Option<usize>) -> impl Iterator<Item = (usize, &str)> {
        let len = self.len();
        self.iter_backward(limit)
            .enumerate()
            .map(move |(i, s)| (len - i - 1, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_forward() {
        let lines = ["a", "b", "c", "d", "e"];
        let iter = lines.iter_forward(None);
        assert_eq!(iter.collect::<Vec<_>>(), vec!["a", "b", "c", "d", "e"]);
        let iter = lines.iter_forward(Some(3));
        assert_eq!(iter.collect::<Vec<_>>(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_iter_backward() {
        let lines = ["a", "b", "c", "d", "e"];
        let iter = lines.iter_backward(None);
        assert_eq!(iter.collect::<Vec<_>>(), vec!["e", "d", "c", "b", "a"]);
        let iter = lines.iter_backward(Some(3));
        assert_eq!(iter.collect::<Vec<_>>(), vec!["e", "d", "c"]);
    }

    #[test]
    fn test_enumerate_tail_forward() {
        let lines = ["a", "b", "c", "d", "e"];
        let iter = lines.enumerate_tail_forward(3);
        assert_eq!(iter.collect::<Vec<_>>(), vec![(2, "c"), (3, "d"), (4, "e")]);
    }

    #[test]
    fn test_enumerate_forward() {
        let lines = ["a", "b", "c", "d", "e"];
        let iter = lines.enumerate_forward(None);
        assert_eq!(
            iter.collect::<Vec<_>>(),
            vec![(0, "a"), (1, "b"), (2, "c"), (3, "d"), (4, "e")]
        );
        let iter = lines.enumerate_forward(Some(3));
        assert_eq!(iter.collect::<Vec<_>>(), vec![(0, "a"), (1, "b"), (2, "c")]);
    }

    #[test]
    fn test_enumerate_backward() {
        let lines = ["a", "b", "c", "d", "e"];
        let iter = lines.enumerate_backward(None);
        assert_eq!(
            iter.collect::<Vec<_>>(),
            vec![(4, "e"), (3, "d"), (2, "c"), (1, "b"), (0, "a")]
        );
        let iter = lines.enumerate_backward(Some(3));
        assert_eq!(iter.collect::<Vec<_>>(), vec![(4, "e"), (3, "d"), (2, "c")]);
    }

    #[test]
    fn test_string_vec_impl() {
        let lines: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(lines.len(), 3);
        assert!(!lines.is_empty());

        // Test iter_forward
        let forward = lines.iter_forward(None).collect::<Vec<_>>();
        assert_eq!(forward, vec!["a", "b", "c"]);

        let limited = lines.iter_forward(Some(2)).collect::<Vec<_>>();
        assert_eq!(limited, vec!["a", "b"]);

        // Test iter_backward
        let backward = lines.iter_backward(None).collect::<Vec<_>>();
        assert_eq!(backward, vec!["c", "b", "a"]);

        let limited_backward = lines.iter_backward(Some(2)).collect::<Vec<_>>();
        assert_eq!(limited_backward, vec!["c", "b"]);

        // Test enumerate_tail_forward
        let tail = lines.enumerate_tail_forward(2).collect::<Vec<_>>();
        assert_eq!(tail, vec![(1, "b"), (2, "c")]);
    }

    #[test]
    fn test_slice_impl() {
        let slice = &["a", "b", "c"][..];
        assert_eq!(slice.len(), 3);
        assert!(!slice.is_empty());

        // Test iter_forward
        let forward = slice.iter_forward(None).collect::<Vec<_>>();
        assert_eq!(forward, vec!["a", "b", "c"]);

        let limited = slice.iter_forward(Some(2)).collect::<Vec<_>>();
        assert_eq!(limited, vec!["a", "b"]);

        // Test iter_backward
        let backward = slice.iter_backward(None).collect::<Vec<_>>();
        assert_eq!(backward, vec!["c", "b", "a"]);

        let limited_backward = slice.iter_backward(Some(2)).collect::<Vec<_>>();
        assert_eq!(limited_backward, vec!["c", "b"]);

        // Test enumerate_tail_forward
        let tail = slice.enumerate_tail_forward(2).collect::<Vec<_>>();
        assert_eq!(tail, vec![(1, "b"), (2, "c")]);
    }

    #[test]
    fn test_empty_collections() {
        let empty_vec: Vec<&str> = Vec::new();
        assert_eq!(empty_vec.len(), 0);
        assert!(empty_vec.is_empty());
        assert_eq!(empty_vec.iter_forward(None).count(), 0);
        assert_eq!(empty_vec.iter_backward(None).count(), 0);
        assert_eq!(empty_vec.enumerate_forward(None).count(), 0);
        assert_eq!(empty_vec.enumerate_backward(None).count(), 0);
        assert_eq!(empty_vec.enumerate_tail_forward(5).count(), 0);

        let empty_string_vec: Vec<String> = Vec::new();
        assert_eq!(empty_string_vec.len(), 0);
        assert!(empty_string_vec.is_empty());
        assert_eq!(empty_string_vec.iter_forward(None).count(), 0);
        assert_eq!(empty_string_vec.iter_backward(None).count(), 0);

        let empty_slice: &[&str] = &[];
        assert_eq!(empty_slice.len(), 0);
        assert!(empty_slice.is_empty());
        assert_eq!(empty_slice.iter_forward(None).count(), 0);
        assert_eq!(empty_slice.iter_backward(None).count(), 0);
    }
}
