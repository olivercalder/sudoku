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
        for i in (0..8).rev() {
            r <<= 4;
            r |= v[i] as u32 & 0b1111;
        }
        Row { row: r }
    }

    /// Create the first `Row` in lexicographic order: `[1, 2, 3, 4, 5, 6, 7, 8, 9]`.
    pub fn first() -> Row {
        Row { row: 0x87654321 }
    }

    /// Returns the next valid row following `self`, in lexicographic order, if it exists.
    fn next(&self) -> Option<Row> {
        // XXX: Very naive and messy approach.
        // TODO: Do better.

        fn reset_from_index(r: u32, index: usize) -> u32 {
            if index == 8 {
                return r;
            }
            let preserve_mask = 0xffffffff >> ((8 - index) << 2); // this is ass
            let filler = 0x87654321 << (index << 2);
            (r & preserve_mask) | filler
        }

        let mut current = self.row;
        let mut index_to_advance = 7;
        'outer: loop {
            current = current + (1 << (index_to_advance << 2));
            // Check for overflow
            if (current >> (index_to_advance << 2)) & 0xf > 9 {
                if index_to_advance == 0 {
                    // We're done, since row must be 0x23456789 and we just tried to advance the 9.
                    return None;
                }
                // Overflowed, so reset to initial filler from current index, then move to the
                // previous index and try to advance from there.
                current = reset_from_index(current, index_to_advance);
                index_to_advance -= 1;
                continue 'outer;
            }
            // Check for duplicates, and find the index of the duplicate, if it exists
            let mut used: u32 = 0;
            let curr_row = Row { row: current };
            for (i, v) in curr_row.iter().enumerate() {
                if used & (1 << v) != 0 {
                    // Already used v, so set i to be index_to_advance, and fill in the rest with
                    // filler.
                    assert!(i < 8); // should be no way to have conflict at index 8, since index 8
                                    // is computed relative to indices 0..=7, and we
                    index_to_advance = i as usize;
                    current = reset_from_index(current, i + 1);
                    continue 'outer;
                }
                used |= 1 << v;
            }
            return Some(curr_row);
        }
    }

    /// Gets the number at the given 1-based index in the row.
    pub fn get(&self, index: u8) -> Option<u8> {
        match index {
            0 | 10.. => None,
            1..9 => Some(self.get_unchecked(index - 1)),
            9 => Some((0..8).fold(ROW_SUM, |acc, i| acc - self.get_unchecked(i))),
        }
    }

    /// Gets the number at the given 0-based index in the row, without bounds checks.
    fn get_unchecked(&self, index: u8) -> u8 {
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
        RowIter {
            row: *self,
            index: 0,
            acc: 0,
        }
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

impl Iterator for RowIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > 8 {
            return None;
        }
        if self.index == 8 {
            self.index += 1;
            return Some(ROW_SUM - self.acc);
        }
        let current: u8 = self.row.get_unchecked(self.index);
        self.acc += current;
        self.index += 1;
        Some(current)
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
    let mut rows: Vec<Row> = Vec::with_capacity((1..=9).product());
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
