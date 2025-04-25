//! Module providing utilities for working with collections of text lines.
//!
//! This module defines the `Lines` trait and its implementations to provide
//! consistent interfaces for iterating through lines in different directions
//! and with different options.

/// Trait for collections of text lines that provides bidirectional iteration.
///
/// This trait provides methods to iterate forward and backward through
/// collections of text lines, with optional limits and offset information.
pub trait Lines<'a> {
    /// Iterates forward through the lines.
    ///
    /// # Arguments
    /// * `limit` - Optional maximum number of lines to return
    ///
    /// # Returns
    /// An iterator yielding lines in forward order
    fn iter_forward(&'a self, limit: Option<usize>) -> impl Iterator<Item = &'a str>;

    /// Iterates forward through the lines with indices.
    ///
    /// # Arguments
    /// * `limit` - Optional maximum number of lines to return
    ///
    /// # Returns
    /// An iterator yielding (index, line) pairs in forward order
    fn enumerate_forward(&'a self, limit: Option<usize>) -> impl Iterator<Item = (usize, &'a str)> {
        self.iter_forward(limit).enumerate()
    }

    /// Iterates backward through the lines.
    ///
    /// # Arguments
    /// * `limit` - Optional maximum number of lines to return
    ///
    /// # Returns
    /// An iterator yielding lines in reverse order
    fn iter_backward(&'a self, limit: Option<usize>) -> impl DoubleEndedIterator<Item = &'a str>;

    /// Iterates forward through the last N lines with indices.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of lines to return from the end
    ///
    /// # Returns
    /// An iterator yielding (index, line) pairs for the last `limit` lines
    fn enumerate_tail_forward(&'a self, limit: usize) -> impl Iterator<Item = (usize, &'a str)> {
        let start_offset = self.len().saturating_sub(limit);
        self.iter_forward(None)
            .skip(start_offset)
            .enumerate()
            .map(move |(i, s)| (i + start_offset, s))
    }

    /// Iterates backward through the lines with original indices.
    ///
    /// # Arguments
    /// * `limit` - Optional maximum number of lines to return
    ///
    /// # Returns
    /// An iterator yielding (original_index, line) pairs in reverse order
    fn enumerate_backward(
        &'a self,
        limit: Option<usize>,
    ) -> impl Iterator<Item = (usize, &'a str)> {
        let len = self.len();
        self.iter_backward(limit)
            .enumerate()
            .map(move |(i, s)| (len - i - 1, s))
    }

    /// Returns the number of lines in the collection.
    ///
    /// # Returns
    /// The line count
    fn len(&self) -> usize;

    /// Checks if the collection is empty.
    ///
    /// # Returns
    /// `true` if the collection contains no lines, `false` otherwise
    fn is_empty(&self) -> bool;
}

impl<'a> Lines<'a> for Vec<&'a str> {
    fn iter_forward(&'a self, limit: Option<usize>) -> impl Iterator<Item = &'a str> {
        let limit = limit.unwrap_or(self.len());
        self.iter().take(limit).cloned()
    }

    fn enumerate_tail_forward(&'a self, limit: usize) -> impl Iterator<Item = (usize, &'a str)> {
        let start_offset = self.len().saturating_sub(limit);

        self[start_offset..]
            .iter()
            .cloned()
            .enumerate()
            .map(move |(i, s)| (i + start_offset, s))
    }

    fn iter_backward(&'a self, limit: Option<usize>) -> impl DoubleEndedIterator<Item = &'a str> {
        let limit = limit.unwrap_or(self.len());
        self.iter().rev().take(limit).cloned()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<'a> Lines<'a> for Vec<String> {
    fn iter_forward(&'a self, limit: Option<usize>) -> impl Iterator<Item = &'a str> {
        let limit = limit.unwrap_or(self.len());
        self.iter().take(limit).map(|s| s.as_str())
    }

    fn iter_backward(&'a self, limit: Option<usize>) -> impl DoubleEndedIterator<Item = &'a str> {
        let limit = limit.unwrap_or(self.len());
        self.iter().rev().take(limit).map(|s| s.as_str())
    }

    fn enumerate_tail_forward(&'a self, limit: usize) -> impl Iterator<Item = (usize, &'a str)> {
        let start_offset = self.len().saturating_sub(limit);

        self[start_offset..]
            .iter()
            .map(|s| s.as_str())
            .enumerate()
            .map(move |(i, s)| (i + start_offset, s))
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<'a> Lines<'a> for &'a [&'a str] {
    fn iter_forward(&'a self, limit: Option<usize>) -> impl Iterator<Item = &'a str> {
        let limit = limit.unwrap_or(self.len());
        self.iter().take(limit).cloned()
    }

    fn iter_backward(&'a self, limit: Option<usize>) -> impl DoubleEndedIterator<Item = &'a str> {
        let limit = limit.unwrap_or(self.len());
        self.iter().rev().take(limit).cloned()
    }

    fn enumerate_tail_forward(&'a self, limit: usize) -> impl Iterator<Item = (usize, &'a str)> {
        let start_offset = self.len().saturating_sub(limit);

        self[start_offset..]
            .iter()
            .cloned()
            .enumerate()
            .map(move |(i, s)| (i + start_offset, s))
    }

    fn len(&self) -> usize {
        <[&str]>::len(self)
    }

    fn is_empty(&self) -> bool {
        <[&str]>::is_empty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_forward() {
        let lines = vec!["a", "b", "c", "d", "e"];
        let iter = lines.iter_forward(None);
        assert_eq!(iter.collect::<Vec<_>>(), vec!["a", "b", "c", "d", "e"]);
        let iter = lines.iter_forward(Some(3));
        assert_eq!(iter.collect::<Vec<_>>(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_iter_backward() {
        let lines = vec!["a", "b", "c", "d", "e"];
        let iter = lines.iter_backward(None);
        assert_eq!(iter.collect::<Vec<_>>(), vec!["e", "d", "c", "b", "a"]);
        let iter = lines.iter_backward(Some(3));
        assert_eq!(iter.collect::<Vec<_>>(), vec!["e", "d", "c"]);
    }

    #[test]
    fn test_enumerate_tail_forward() {
        let lines = vec!["a", "b", "c", "d", "e"];
        let iter = lines.enumerate_tail_forward(3);
        assert_eq!(iter.collect::<Vec<_>>(), vec![(2, "c"), (3, "d"), (4, "e")]);
    }

    #[test]
    fn test_enumerate_forward() {
        let lines = vec!["a", "b", "c", "d", "e"];
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
        let lines = vec!["a", "b", "c", "d", "e"];
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
