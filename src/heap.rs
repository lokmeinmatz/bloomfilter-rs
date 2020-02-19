use std::cmp::Ordering;

pub struct MinHeap<T, F> {
    data: Vec<T>,
    n: usize,
    order_fn: F
}


impl <T, F: Fn(&T, &T) -> Ordering> MinHeap<T, F> {
    pub fn new(ordering: F, num_children: usize) -> Self {
        MinHeap {
            data: Vec::new(),
            order_fn: ordering,
            n: num_children
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    fn parent(&self, i: usize) -> usize { (i - 1) / self.n }

    fn nth_child(&self,parent: usize, n: usize) -> usize {
        parent * self.n + n + 1
    }

    // TODO no recursion (see wikipedia)
    fn heapify(&mut self, i: usize) {
        // TODO assert isheap for all children
        let mut min = i;
        for n_child in 0..self.n {
            let idx_child = self.nth_child(i, n_child);
            if idx_child < self.data.len() && (self.order_fn)(&self.data[min], &self.data[idx_child]) == Ordering::Less {
                min = idx_child;
            }
        }

        if min != i {
            self.data.swap(i, min);
            self.heapify(min);
        }
    }

    pub fn insert(&mut self, item: T) {
        let mut i = self.len();
        self.data.push(item);

        while i > 0 {
            let parent = self.parent(i);
            match (self.order_fn)(&self.data[i], &self.data[parent]) {
                Ordering::Less => {
                    self.data.swap(i, parent);
                    i = parent;
                },
                _ => break
            }
        }
    }

    pub fn extract(&mut self) -> Option<T> {
        if self.len() == 0 { return None; }
        let root = self.data.swap_remove(0);
        self.heapify(0);
        Some(root)
    }
}



#[cfg(test)]
mod tests {
    use crate::heap::MinHeap;

    #[test]
    fn heap_test_basic() {
        let mut heap: MinHeap<usize, _> = MinHeap::new(|a: &usize, b: &usize| a.cmp(b), 4);

        assert_eq!(heap.len(), 0);

        heap.insert(11);

        assert_eq!(heap.len(), 1);
        heap.insert(5);
        heap.insert(7);

        assert_eq!(heap.len(), 3);

        assert_eq!(heap.extract(), Some(5));
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.extract(), Some(7));
        assert_eq!(heap.extract(), Some(11));
        assert_eq!(heap.extract(), None);
        assert_eq!(heap.len(), 0);


    }
}
