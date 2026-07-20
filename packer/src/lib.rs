#[derive(Debug, Clone, Copy)]
pub struct Shelf {
    y: u32,      // top of this shelf
    height: u32, // set by first (tallest) image placed
    cursor_x: u32,
}

#[derive(Debug, Clone)]
pub struct PagePacker {
    size: u32, // e.g. 4096
    shelves: Vec<Shelf>,
    next_y: u32,  // top of unallocated space
    padding: u32, // e.g. 1 or 2 px
}

pub struct Placement {
    pub x: u32,
    pub y: u32,
}

impl PagePacker {
    pub fn new(size: u32, padding: u32) -> Self {
        Self {
            size,
            shelves: Vec::new(),
            next_y: 0,
            padding,
        }
    }

    pub fn insert(&mut self, w: u32, h: u32) -> Option<Placement> {
        let (pw, ph) = (w + self.padding, h + self.padding);

        // 1. Try existing shelves: pick the one wasting the least height
        let mut best: Option<(usize, u32)> = None;
        for (i, s) in self.shelves.iter().enumerate() {
            if ph <= s.height && s.cursor_x + pw <= self.size {
                let waste = s.height - ph;
                if best.map_or(true, |(_, bw)| waste < bw) {
                    best = Some((i, waste));
                }
            }
        }
        if let Some((i, _)) = best {
            let s = &mut self.shelves[i];
            let p = Placement {
                x: s.cursor_x,
                y: s.y,
            };
            s.cursor_x += pw;
            return Some(p);
        }

        // 2. Open a new shelf
        if self.next_y + ph <= self.size && pw <= self.size {
            let shelf = Shelf {
                y: self.next_y,
                height: ph,
                cursor_x: pw,
            };
            let p = Placement {
                x: 0,
                y: self.next_y,
            };
            self.next_y += ph;
            self.shelves.push(shelf);
            return Some(p);
        }

        None // page full → caller opens a new page
    }
}
