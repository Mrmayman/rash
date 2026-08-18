pub struct GapVec<T> {
    inner: Vec<Option<T>>,
}

impl<T> Default for GapVec<T> {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}

impl<T> GapVec<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, index: usize, item: T) {
        if index < self.inner.len() {
            self.inner[index] = Some(item);
            return;
        }

        self.inner.resize_with(index, || None);
        self.inner.push(Some(item));
    }

    pub fn push_to(self, out: &mut Vec<T>) {
        if self.inner.len() <= out.len() {
            if cfg!(debug_assertions) {
                for i in &self.inner {
                    assert!(i.is_none());
                }
            }
            return;
        }
        out.reserve(self.inner.len() - out.len());
        for item in self.inner.into_iter().skip(out.len()) {
            out.push(item.expect("GapVec shouldn't have gaps after resolving"));
        }
    }
}
