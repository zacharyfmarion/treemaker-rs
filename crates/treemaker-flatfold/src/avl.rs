const L: usize = 0;
const R: usize = 1;

#[derive(Debug, Clone)]
struct Node {
    child: [Option<usize>; 2],
    value: usize,
    height: usize,
    next_free: Option<usize>,
}

impl Node {
    fn new(value: usize) -> Self {
        Self {
            child: [None, None],
            value,
            height: 1,
            next_free: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct AvlSet {
    root: Option<usize>,
    free: Option<usize>,
    len: usize,
    nodes: Vec<Node>,
}

impl AvlSet {
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn insert<F>(&mut self, value: usize, comp: F) -> Option<usize>
    where
        F: Fn(usize, usize) -> i32,
    {
        let node = self.obtain(value);
        let mut path = self.path(value, &comp);
        if path.is_empty() {
            self.root = Some(node);
        } else {
            let parent = path[path.len() - 1];
            let existing = self.nodes[parent].value;
            let c = comp(value, existing);
            if c == 0 {
                self.release(node);
                return Some(existing);
            }
            self.nodes[parent].child[if c < 0 { L } else { R }] = Some(node);
        }
        path.push(node);
        self.maintain(path);
        None
    }

    pub(crate) fn next<F>(&self, value: usize, comp: F) -> Option<usize>
    where
        F: Fn(usize, usize) -> i32,
    {
        self.next_dir(value, R, comp)
    }

    pub(crate) fn prev<F>(&self, value: usize, comp: F) -> Option<usize>
    where
        F: Fn(usize, usize) -> i32,
    {
        self.next_dir(value, L, comp)
    }

    pub(crate) fn remove_next<F>(&mut self, value: usize, comp: F) -> Option<usize>
    where
        F: Fn(usize, usize) -> i32,
    {
        self.remove_next_dir(value, R, comp)
    }

    pub(crate) fn remove_prev<F>(&mut self, value: usize, comp: F) -> Option<usize>
    where
        F: Fn(usize, usize) -> i32,
    {
        self.remove_next_dir(value, L, comp)
    }

    fn next_dir<F>(&self, value: usize, direction: usize, comp: F) -> Option<usize>
    where
        F: Fn(usize, usize) -> i32,
    {
        self.root?;
        let mut path = self.path(value, &comp);
        let i = *path.last()?;
        let c = comp(value, self.nodes[i].value);
        if c != 0 && ((direction == R) == (c < 0)) {
            return Some(self.nodes[i].value);
        }
        self.adj(&mut path, direction);
        path.pop().map(|i| self.nodes[i].value)
    }

    fn remove_next_dir<F>(&mut self, value: usize, direction: usize, comp: F) -> Option<usize>
    where
        F: Fn(usize, usize) -> i32,
    {
        self.root?;
        let mut path = self.path(value, &comp);
        let mut i = *path.last()?;
        let c = comp(value, self.nodes[i].value);
        if !(c != 0 && ((direction == R) == (c < 0))) {
            self.adj(&mut path, direction);
            i = path.pop()?;
            path.push(i);
        }
        let value = self.nodes[i].value;
        self.remove_path(path);
        Some(value)
    }

    fn obtain(&mut self, value: usize) -> usize {
        self.len += 1;
        if let Some(index) = self.free {
            self.free = self.nodes[index].next_free;
            self.nodes[index] = Node::new(value);
            index
        } else {
            let index = self.nodes.len();
            self.nodes.push(Node::new(value));
            index
        }
    }

    fn release(&mut self, index: usize) {
        self.len -= 1;
        self.nodes[index] = Node {
            child: [None, None],
            value: 0,
            height: 0,
            next_free: self.free,
        };
        self.free = Some(index);
    }

    fn height(&self, index: Option<usize>) -> usize {
        index.map(|i| self.nodes[i].height).unwrap_or(0)
    }

    fn skew(&self, index: usize) -> isize {
        self.height(self.nodes[index].child[R]) as isize
            - self.height(self.nodes[index].child[L]) as isize
    }

    fn update(&mut self, index: usize) {
        let h_left = self.height(self.nodes[index].child[L]);
        let h_right = self.height(self.nodes[index].child[R]);
        self.nodes[index].height = 1 + h_left.max(h_right);
    }

    fn rotate(&mut self, d_index: usize, r: usize, l: usize) {
        let b_index = self.nodes[d_index].child[l].expect("AVL rotation child");
        let e = self.nodes[d_index].child[r];
        let d_value = self.nodes[d_index].value;
        let a = self.nodes[b_index].child[l];
        let c = self.nodes[b_index].child[r];
        let b_value = self.nodes[b_index].value;

        self.nodes[b_index].child[l] = c;
        self.nodes[b_index].child[r] = e;
        self.nodes[b_index].value = d_value;
        self.nodes[d_index].child[l] = a;
        self.nodes[d_index].child[r] = Some(b_index);
        self.nodes[d_index].value = b_value;
        self.update(b_index);
        self.update(d_index);
    }

    fn maintain(&mut self, mut path: Vec<usize>) {
        while let Some(index) = path.pop() {
            self.update(index);
            let skew = self.skew(index);
            for (target, r, l) in [(2, R, L), (-2, L, R)] {
                if skew != target {
                    continue;
                }
                let child = self.nodes[index].child[r].expect("AVL heavy child");
                if self.skew(child) == -target / 2 {
                    self.rotate(child, r, l);
                }
                self.rotate(index, l, r);
            }
        }
    }

    fn path<F>(&self, value: usize, comp: &F) -> Vec<usize>
    where
        F: Fn(usize, usize) -> i32,
    {
        let Some(mut index) = self.root else {
            return Vec::new();
        };
        let mut path = Vec::new();
        loop {
            path.push(index);
            let c = comp(value, self.nodes[index].value);
            if c == 0 {
                break;
            }
            let next = self.nodes[index].child[if c < 0 { L } else { R }];
            let Some(next) = next else {
                break;
            };
            index = next;
        }
        path
    }

    fn adj(&self, path: &mut Vec<usize>, direction: usize) {
        let Some(mut index) = path.last().copied() else {
            return;
        };
        let mut next = self.nodes[index].child[direction];
        if next.is_none() {
            path.pop();
            while let Some(parent) = path.pop() {
                if self.nodes[parent].child[direction] != Some(index) {
                    path.push(parent);
                    break;
                }
                index = parent;
            }
        } else {
            while let Some(j) = next {
                path.push(j);
                next = self.nodes[j].child[direction ^ 1];
            }
        }
    }

    fn remove_path(&mut self, mut path: Vec<usize>) {
        let Some(mut index) = path.last().copied() else {
            return;
        };
        let mut direction = L;
        let mut child = self.nodes[index].child[L];
        if child.is_none() {
            direction = R;
            child = self.nodes[index].child[R];
        }
        while child.is_some() {
            while let Some(next) = child {
                path.push(next);
                child = self.nodes[next].child[direction ^ 1];
            }
            let replacement = path[path.len() - 1];
            self.nodes[index].value = self.nodes[replacement].value;
            index = replacement;
            child = self.nodes[index].child[direction];
        }
        path.pop();
        self.release(index);
        let Some(parent) = path.last().copied() else {
            self.root = None;
            return;
        };
        if self.nodes[parent].child[L] == Some(index) {
            self.nodes[parent].child[L] = None;
        } else {
            self.nodes[parent].child[R] = None;
        }
        self.maintain(path);
    }
}
