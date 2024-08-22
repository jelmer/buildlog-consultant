pub trait Lines<'a> {
    fn iter_forward(&'a self, limit: Option<usize>) -> impl Iterator<Item = &'a str>;

    fn enumerate_forward(&'a self, limit: Option<usize>) -> impl Iterator<Item = (usize, &'a str)> {
        self.iter_forward(limit).enumerate()
    }

    fn iter_backward(&'a self, limit: Option<usize>) -> impl DoubleEndedIterator<Item = &'a str>;

    fn enumerate_tail_forward(&'a self, limit: usize) -> impl Iterator<Item = (usize, &'a str)> {
        let start_offset = self.len().saturating_sub(limit);
        self.iter_forward(None)
            .skip(start_offset)
            .enumerate()
            .map(move |(i, s)| (i + start_offset, s))
    }

    fn enumerate_backward(
        &'a self,
        limit: Option<usize>,
    ) -> impl Iterator<Item = (usize, &'a str)> {
        let len = self.len();
        self.iter_backward(limit)
            .enumerate()
            .map(move |(i, s)| (len - i - 1, s))
    }

    fn len(&self) -> usize;
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
}
