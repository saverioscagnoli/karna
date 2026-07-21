pub struct Placement {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Copy)]
struct Node {
    /// Left edge of this skyline segment
    x: u32,
    /// Height of the contour here (top of whatever is below)
    y: u32,
    /// Width of the segment
    w: u32,
}

#[derive(Debug, Clone)]
pub struct PagePacker {
    size: u32,
    padding: u32,
    nodes: Vec<Node>,
}

impl PagePacker {
    pub fn new(size: u32, padding: u32) -> Self {
        Self {
            size,
            padding,
            nodes: vec![Node {
                x: 0,
                y: 0,
                w: size,
            }],
        }
    }

    pub fn insert(&mut self, w: u32, h: u32) -> Option<Placement> {
        let pw = w + self.padding;
        let ph = h + self.padding;

        if pw > self.size || ph > self.size {
            return None;
        }

        let mut best: Option<(usize, u32, u32)> = None;

        for i in 0..self.nodes.len() {
            if let Some(y) = self.fit(i, pw, ph) {
                let x = self.nodes[i].x;
                let better = match best {
                    None => true,
                    Some((_, bx, by)) => y < by || (y == by && x < bx),
                };

                if better {
                    best = Some((i, x, y));
                }
            }
        }

        let (i, x, y) = best?;
        self.place(i, x, y, pw, ph);

        Some(Placement { x, y })
    }

    fn fit(&self, i: usize, w: u32, h: u32) -> Option<u32> {
        let x = self.nodes[i].x;
        if x + w > self.size {
            return None;
        }

        let mut y = 0u32;
        let mut remaining = w as i64;
        let mut j = i;

        while remaining > 0 {
            let n = self.nodes.get(j)?;
            y = y.max(n.y);
            if y + h > self.size {
                return None;
            }
            remaining -= n.w as i64;
            j += 1;
        }

        Some(y)
    }

    fn place(&mut self, i: usize, x: u32, y: u32, w: u32, h: u32) {
        self.nodes.insert(i, Node { x, y: y + h, w });

        let right = x + w;
        let j = i + 1;

        while j < self.nodes.len() {
            let n = self.nodes[j];

            if n.x >= right {
                break; // past the new rect, untouched
            }

            if n.x + n.w <= right {
                // fully shadowed by the new rect
                self.nodes.remove(j);
            } else {
                // partially shadowed: clip its left edge
                let overlap = right - n.x;
                self.nodes[j].x += overlap;
                self.nodes[j].w -= overlap;
                break;
            }
        }

        let mut k = 0;

        while k + 1 < self.nodes.len() {
            if self.nodes[k].y == self.nodes[k + 1].y {
                self.nodes[k].w += self.nodes[k + 1].w;
                self.nodes.remove(k + 1);
            } else {
                k += 1;
            }
        }
    }
}
