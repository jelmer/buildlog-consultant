pub trait Lines {
    fn iter_forward(&self, limit: Option<usize>) -> impl Iterator<Item = &str>;

    fn enumerate_forward(&self, limit: Option<usize>) -> impl Iterator<Item = (usize, &str)> {
        self.iter_forward(limit).enumerate()
    }

    fn iter_backward(&self, limit: Option<usize>) -> impl DoubleEndedIterator<Item = &str>;

    fn iter_tail_forward(&self, limit: usize) -> impl Iterator<Item = (usize, &str)> {
        self.iter_forward(Some(limit)).skip(self.len().saturating_sub(limit)).enumerate()
    }

    fn enumerate_backward(&self, limit: Option<usize>) -> impl Iterator<Item = (usize, &str)> {
        let len = self.len();
        self.iter_backward(limit).enumerate().map(move |(i, s)| (len - i - 1, s))
    }

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl Lines for Vec<&str> {
    fn iter_forward(&self, limit: Option<usize>) -> impl Iterator<Item = &str> {
        let limit = limit.unwrap_or(self.len());
        self.iter().take(limit).cloned()
    }

    fn iter_tail_forward(&self, limit: usize) -> impl Iterator<Item = (usize, &str)> {
        let start_offset = std::cmp::max(0, self.len() as isize - limit as isize) as usize;

        self[start_offset..].iter().cloned().enumerate()
    }

    fn iter_backward(&self, limit: Option<usize>) -> impl DoubleEndedIterator<Item = &str> {
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

impl Lines for Vec<String> {
    fn iter_forward(&self, limit: Option<usize>) -> impl Iterator<Item = &str> {
        let limit = limit.unwrap_or(self.len());
        self.iter().take(limit).map(|s| s.as_str())
    }

    fn iter_backward(&self, limit: Option<usize>) -> impl DoubleEndedIterator<Item = &str> {
        let limit = limit.unwrap_or(self.len());
        self.iter().rev().take(limit).map(|s| s.as_str())
    }

    fn iter_tail_forward(&self, limit: usize) -> impl Iterator<Item = (usize, &str)> {
        let start_offset = std::cmp::max(0, self.len() as isize - limit as isize) as usize;

        self[start_offset..].iter().map(|s| s.as_str()).enumerate()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl Lines for &[&str] {
    fn iter_forward(&self, limit: Option<usize>) -> impl Iterator<Item = &str> {
        let limit = limit.unwrap_or(self.len());
        self.iter().take(limit).cloned()
    }

    fn iter_backward(&self, limit: Option<usize>) -> impl DoubleEndedIterator<Item = &str> {
        let limit = limit.unwrap_or(self.len());
        self.iter().rev().take(limit).cloned()
    }

    fn iter_tail_forward(&self, limit: usize) -> impl Iterator<Item = (usize, &str)> {
        let start_offset = std::cmp::max(0, self.len() as isize - limit as isize) as usize;

        self[start_offset..].iter().cloned().enumerate()
    }

    fn len(&self) -> usize {
        <[&str]>::len(self)
    }

    fn is_empty(&self) -> bool {
        <[&str]>::is_empty(self)
    }
}
