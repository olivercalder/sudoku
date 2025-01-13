const ROW_WIDTH: u8 = 9;

const ROW_SUM: u8 = 9 * (9 + 1) / 2;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Row {
    row: u32,
}

impl Row {
    /// Create a `Row` from the given slice. Assumes the slice has length 9 and contains the
    /// numbers 1 through 9 exactly once each.
    pub fn from_slice(v: &[u8]) -> Row {
        // 9th element is implied by previous element.
        let mut r: u32 = 0;
        for e in v.iter().take(8) {
            r <<= 4;
            r |= *e as u32 & 0b1111;
        }
        Row { row: r }
    }

    /// Create the first `Row` in lexicographic order: `[1, 2, 3, 4, 5, 6, 7, 8, 9]`.
    pub fn first() -> Row {
        Row { row: 0x12345678 }
    }

    /// Returns the next valid row following `self`, in lexicographic order, if it exists.
    fn next(&self) -> Option<Row> {
        // XXX: Very naive and messy approach.
        // TODO: Do better.

        // Reset entries to the right of the given bit to be the lexicographic minimum.
        fn reset_after_bit(r: u32, bit: usize) -> u32 {
            if bit == 0 {
                return r;
            }
            let preserve_mask = 0xffffffff << bit;
            let filler = 0x12345678 >> (32 - bit);
            (r & preserve_mask) | filler
        }

        let mut current = self.row;
        let mut bit_to_advance = 0; // abstract index is 7 - (bit_to_advance / 4)
        'outer: loop {
            current += 1 << bit_to_advance;
            // Check for overflow
            if (current >> bit_to_advance) & 0xf > 9 {
                if bit_to_advance == 28 {
                    // We're done, since row must have been 0x98765432, then we advanced the 8 and
                    // it became 0x99123456, then we checked for duplicates, saw duplicate 9 at bit
                    // 24, so advanced bit_to_advance to 28 and reset the rest, leaving current as
                    // 0xa1234567
                    debug_assert_eq!(
                        format!("{:x}", current as i64),
                        format!("{:x}", 0xa1234567_i64)
                    );
                    return None;
                }
                // Overflowed, so advance the bit index and reset to initial filler to the right of
                // there.
                bit_to_advance += 4;
                current = reset_after_bit(current, bit_to_advance);
                continue 'outer;
            }
            // Check for duplicates, and find the index of the duplicate, if it exists
            let mut used: u32 = 0;
            let curr_row = Row { row: current };
            for (i, v) in curr_row.iter().enumerate() {
                if used & (1 << v) != 0 {
                    // Already used v, so set i to be index_to_advance, and fill in the rest with
                    // filler.
                    debug_assert!(i < 8); // should be no way to have conflict at index 8, since
                                          // index 8 is computed relative to indices 0..=7.
                    let bit = 28 - (i << 2); // convert from abstract index to u32 bit index.
                    current = reset_after_bit(current, bit);
                    bit_to_advance = bit;
                    continue 'outer;
                }
                used |= 1 << v;
            }
            return Some(curr_row);
        }
    }

    /// Gets the number at the given 1-based index into the row.
    pub fn get(&self, index: u8) -> Option<u8> {
        match index {
            0 | 10.. => None,
            1..9 => Some(self.get_nibble(8 - index)),
            9 => Some((0..8).fold(ROW_SUM, |acc, i| acc - self.get_nibble(i))),
        }
    }

    /// Gets the number at the given 1-based index between 1 and 8 (inclusive) into the row,
    /// without bounds checking.
    pub fn get_unchecked(&self, index: u8) -> u8 {
        self.get_nibble(8 - index)
    }

    /// Gets the number at the given nibble index into the row, without bounds checks.
    fn get_nibble(&self, index: u8) -> u8 {
        ((self.row >> (index << 2)) & 0b1111) as u8
    }

    /// Returns true if `other` is a column successor to `self`. That is, for all identical positions
    /// in `self` and `other`, the numbers in those positions differ.
    pub fn col_successor(&self, other: &Self) -> bool {
        let mut xor = self.row ^ other.row;
        for _ in 0..8 {
            if xor & 0b1111 == 0 {
                return false;
            }
            xor >>= 4;
        }
        true
    }

    /// Returns true if `other` is a box successor to `self`. That is, ensure that no entry would
    /// occur in the same 3x3 box in both `self` and `other`.
    ///
    /// Let A, B, C, X, Y, and Z be 3-element sequences, such that `self` is ABC and `other` is
    /// XYZ. Then treating all sequences as sets, A and X are disjoint, B and Y are disjoint, and C
    /// and Z are disjoint.
    pub fn box_successor(&self, other: &Self) -> bool {
        let boxes = self.box_chunks().zip(other.box_chunks());
        for (s_box, o_box) in boxes {
            for s in &s_box {
                if o_box.contains(s) {
                    return false;
                }
            }
            for o in &o_box {
                if s_box.contains(o) {
                    return false;
                }
            }
        }
        true
    }

    /// Returns a `RowIter` of the elements in the row.
    pub fn iter(&self) -> RowIter {
        RowIter::new(self)
    }

    /// Returns a `RowChunk` iterator, which returns chunks of three elements at a time.
    fn box_chunks(&self) -> RowChunk {
        RowChunk { iter: self.iter() }
    }
}

pub struct RowIter {
    row: Row,
    index: u8,
    acc: u8,
}

impl RowIter {
    fn new(r: &Row) -> Self {
        Self {
            row: *r,
            index: 0,
            acc: 0,
        }
    }
}

impl Iterator for RowIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        match self.index {
            1..=8 => {
                let current: u8 = self.row.get_unchecked(self.index);
                self.acc += current;
                Some(current)
            }
            9 => Some(ROW_SUM - self.acc),
            _ => None,
        }
    }
}

pub struct RowChunk {
    iter: RowIter,
}

impl Iterator for RowChunk {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let items: Vec<u8> = self.iter.by_ref().take(3).collect();
        match items.len() {
            0 => None,
            _ => Some(items),
        }
    }
}

/// Returns the list of all sudoku rows in lexicographic order.
pub fn build_rows() -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::with_capacity((2..=9).product());
    let mut current = Some(Row::first());
    while let Some(r) = current {
        rows.push(r);
        current = r.next();
    }
    rows
}

/// Returns the number of column successors and the number of box successors per row.
pub fn successors_per_row() -> (usize, usize) {
    let first = Row::first();
    let mut current = Some(Row::first());
    let mut col_count: usize = 0;
    let mut box_count: usize = 0;
    while let Some(r) = current {
        if first.col_successor(&r) {
            col_count += 1;
        }
        if first.box_successor(&r) {
            box_count += 1;
        }
        current = r.next();
    }
    (col_count, box_count)
}

#[cfg(test)]
mod tests {
    use super::{build_rows, Row};

    #[test]
    fn test_from_slice_get() {
        let r = Row::from_slice(&[3, 6, 7, 2, 9, 4, 8, 1, 5]);
        assert_eq!(r.get(0), None);
        assert_eq!(r.get(1), Some(3));
        assert_eq!(r.get(2), Some(6));
        assert_eq!(r.get(5), Some(9));
        assert_eq!(r.get(8), Some(1));
        assert_eq!(r.get(9), Some(5));
        assert_eq!(r.get(10), None);
    }

    #[test]
    fn test_from_slice_iter() {
        let r = Row::from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(
            r.iter().collect::<Vec<u8>>(),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
        );

        let r = Row::from_slice(&[9, 8, 7, 6, 5, 4, 3, 2, 1]);
        assert_eq!(
            r.iter().collect::<Vec<u8>>(),
            vec![9, 8, 7, 6, 5, 4, 3, 2, 1]
        );

        let r = Row::from_slice(&[3, 6, 7, 2, 9, 4, 8, 1, 5]);
        assert_eq!(
            r.iter().collect::<Vec<u8>>(),
            vec![3, 6, 7, 2, 9, 4, 8, 1, 5]
        );
    }

    #[test]
    fn test_first() {
        let first = Row::first();
        assert_eq!(
            first.iter().collect::<Vec<u8>>(),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
    }

    #[test]
    fn test_next() {
        let r = Row::first();
        assert_eq!(
            r.next().unwrap().iter().collect::<Vec<u8>>(),
            vec![1, 2, 3, 4, 5, 6, 7, 9, 8]
        );

        let r = Row::from_slice(&[9, 8, 7, 6, 5, 4, 3, 2, 1]);
        assert_eq!(r.next(), None);

        let r = Row::from_slice(&[3, 6, 7, 2, 9, 4, 8, 1, 5]);
        assert_eq!(
            r.next().unwrap().iter().collect::<Vec<u8>>(),
            vec![3, 6, 7, 2, 9, 4, 8, 5, 1]
        );
    }

    #[test]
    fn test_col_successor() {
        let r1 = Row::from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let r2 = Row::from_slice(&[2, 3, 4, 5, 6, 7, 8, 9, 1]);
        let r3 = Row::from_slice(&[2, 3, 5, 4, 6, 7, 8, 9, 1]);
        let r4 = Row::from_slice(&[4, 5, 6, 7, 8, 9, 1, 2, 3]);

        assert_eq!(r1.col_successor(&r2), true);
        assert_eq!(r2.col_successor(&r1), true);

        assert_eq!(r1.col_successor(&r3), false);
        assert_eq!(r3.col_successor(&r1), false);

        assert_eq!(r1.col_successor(&r4), true);
        assert_eq!(r4.col_successor(&r1), true);

        assert_eq!(r2.col_successor(&r3), false);
        assert_eq!(r3.col_successor(&r2), false);

        assert_eq!(r2.col_successor(&r4), true);
        assert_eq!(r4.col_successor(&r2), true);

        assert_eq!(r3.col_successor(&r4), true);
        assert_eq!(r4.col_successor(&r3), true);
    }

    #[test]
    fn test_box_successor() {
        let r1 = Row::from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let r2 = Row::from_slice(&[2, 3, 4, 5, 6, 7, 8, 9, 1]);
        let r3 = Row::from_slice(&[2, 3, 5, 4, 6, 7, 8, 9, 1]);
        let r4 = Row::from_slice(&[4, 5, 6, 7, 8, 9, 1, 2, 3]);

        assert_eq!(r1.box_successor(&r2), false);
        assert_eq!(r2.box_successor(&r1), false);

        assert_eq!(r1.box_successor(&r3), false);
        assert_eq!(r3.box_successor(&r1), false);

        assert_eq!(r1.box_successor(&r4), true);
        assert_eq!(r4.box_successor(&r1), true);

        assert_eq!(r2.box_successor(&r3), false);
        assert_eq!(r3.box_successor(&r2), false);

        assert_eq!(r2.box_successor(&r4), false);
        assert_eq!(r4.box_successor(&r2), false);

        assert_eq!(r3.box_successor(&r4), false);
        assert_eq!(r4.box_successor(&r3), false);
    }

    #[test]
    fn test_build_rows() {
        let all_rows = build_rows();
        let l = (1..=9).product();
        assert_eq!(all_rows.len(), l);
        let vecs: Vec<Vec<u8>> = all_rows
            .iter()
            .map(|r| r.iter().collect::<Vec<u8>>())
            .collect();
        assert_eq!(vecs[0], vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(vecs[1], vec![1, 2, 3, 4, 5, 6, 7, 9, 8]);
        assert_eq!(vecs[2], vec![1, 2, 3, 4, 5, 6, 8, 7, 9]);
        assert_eq!(vecs[3], vec![1, 2, 3, 4, 5, 6, 8, 9, 7]);
        assert_eq!(vecs[4], vec![1, 2, 3, 4, 5, 6, 9, 7, 8]);
        assert_eq!(vecs[5], vec![1, 2, 3, 4, 5, 6, 9, 8, 7]);
        assert_eq!(vecs[6], vec![1, 2, 3, 4, 5, 7, 6, 8, 9]);
        assert_eq!(vecs[7], vec![1, 2, 3, 4, 5, 7, 6, 9, 8]);
        assert_eq!(vecs[8], vec![1, 2, 3, 4, 5, 7, 8, 6, 9]);
        assert_eq!(vecs[9], vec![1, 2, 3, 4, 5, 7, 8, 9, 6]);
        assert_eq!(vecs[10], vec![1, 2, 3, 4, 5, 7, 9, 6, 8]);
        assert_eq!(vecs[11], vec![1, 2, 3, 4, 5, 7, 9, 8, 6]);

        assert_eq!(vecs[l - 1], vec![9, 8, 7, 6, 5, 4, 3, 2, 1]);
        assert_eq!(vecs[l - 2], vec![9, 8, 7, 6, 5, 4, 3, 1, 2]);
        assert_eq!(vecs[l - 3], vec![9, 8, 7, 6, 5, 4, 2, 3, 1]);
        assert_eq!(vecs[l - 4], vec![9, 8, 7, 6, 5, 4, 2, 1, 3]);
    }
}
