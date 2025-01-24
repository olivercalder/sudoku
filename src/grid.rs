use crate::row::Row;

const sum_of_row: u64 = 9 * (9 + 1) / 2;
const sum_of_rows: u64 = 12884901885; // (0..8).map(|x| sum_of_row << (x << 2)).sum::<u64>();

pub struct Grid {
    rows: [Row; 8], // ninth row is implied
}

impl Grid {
    pub fn first() -> Self {
        let mut initial = [Row::first(); 8];
        for i in 1..8 {
            // Set row i to be the first successor to previous rows
            'outer: loop {
                initial[i] = initial[i].next().unwrap();
                for j in 0..i {
                    // Ensure it's a column successor
                    if !initial[i].col_successor(&initial[j]) {
                        continue 'outer;
                    }
                }
                for j in (i - (i % 3))..i {
                    // Ensure it's a box successor
                    if !initial[i].box_successor(&initial[j]) {
                        continue 'outer;
                    }
                }
                // It's a column successor and a box successor
                break;
            }
        }
        Self { rows: initial }
    }

    pub fn rows(&self) -> Iter {
        Iter::from(self)
    }

    /// Returns the `i`th 0-indexed row in the grid.
    fn get(&self, i: usize) -> Option<Row> {
        match i {
            0..8 => Some(self.rows[i]),
            8 => Some(self.ninth_row()),
            _ => None,
        }
    }

    /// Returns the ninth row of the grid.
    fn ninth_row(&self) -> Row {
        Row::from_u32(
            (sum_of_rows - self.rows.iter().map(|r| r.as_u32() as u64).sum::<u64>()) as u32,
        )
    }

    /// Returns a pretty-printed multiline string displaying the grid.
    pub fn format(&self) -> String {
        let mut buf = String::from(grid_template.clone());
        unsafe {
            let bytes = buf.as_bytes_mut();
            x_indices
                .iter()
                .zip(self.rows().map(|r| r.iter()).flatten())
                .for_each(|(i, x)| bytes[*i] = 0x30 + x);
        }
        buf.to_string()
    }
}

const grid_template: &str = "
┏━━━┯━━━┯━━━┳━━━┯━━━┯━━━┳━━━┯━━━┯━━━┓
┃ x │ x │ x ┃ x │ x │ x ┃ x │ x │ x ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ x │ x │ x ┃ x │ x │ x ┃ x │ x │ x ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ x │ x │ x ┃ x │ x │ x ┃ x │ x │ x ┃
┣━━━┿━━━┿━━━╋━━━┿━━━┿━━━╋━━━┿━━━┿━━━┫
┃ x │ x │ x ┃ x │ x │ x ┃ x │ x │ x ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ x │ x │ x ┃ x │ x │ x ┃ x │ x │ x ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ x │ x │ x ┃ x │ x │ x ┃ x │ x │ x ┃
┣━━━┿━━━┿━━━╋━━━┿━━━┿━━━╋━━━┿━━━┿━━━┫
┃ x │ x │ x ┃ x │ x │ x ┃ x │ x │ x ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ x │ x │ x ┃ x │ x │ x ┃ x │ x │ x ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ x │ x │ x ┃ x │ x │ x ┃ x │ x │ x ┃
┗━━━┷━━━┷━━━┻━━━┷━━━┷━━━┻━━━┷━━━┷━━━┛
";

static x_indices: [usize; 81] = [
    117, 123, 129, 135, 141, 147, 153, 159, 165, 287, 293, 299, 305, 311, 317, 323, 329, 335, 457,
    463, 469, 475, 481, 487, 493, 499, 505, 627, 633, 639, 645, 651, 657, 663, 669, 675, 797, 803,
    809, 815, 821, 827, 833, 839, 845, 967, 973, 979, 985, 991, 997, 1003, 1009, 1015, 1137, 1143,
    1149, 1155, 1161, 1167, 1173, 1179, 1185, 1307, 1313, 1319, 1325, 1331, 1337, 1343, 1349, 1355,
    1477, 1483, 1489, 1495, 1501, 1507, 1513, 1519, 1525,
];

/// Iterates through the rows in the grid.
pub struct Iter<'a> {
    grid: &'a Grid,
    index: usize,
    acc: u64,
}

impl<'a> Iter<'a> {
    fn from(grid: &'a Grid) -> Self {
        Self {
            grid: grid,
            index: 0,
            acc: 0,
        }
    }
}

impl Iterator for Iter<'_> {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < 8 {
            let current = self.grid.rows[self.index];
            self.acc += current.as_u32() as u64;
            self.index += 1;
            return Some(current);
        }
        if self.index == 8 {
            self.index += 1;
            return Some(Row::from_u32((sum_of_rows - self.acc) as u32));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::Grid;
    use crate::grid::{grid_template, x_indices};
    use crate::row::Row;

    #[test]
    fn test_first_get() {
        let first = Grid::first();
        assert_eq!(first.get(0), Some(Row::first()));
        assert_eq!(
            first.get(1),
            Some(Row::from_slice(&[4, 5, 6, 7, 8, 9, 1, 2, 3]))
        );
    }

    #[test]
    fn test_format() {
        let first = Grid::first();
        assert_eq!(
            first.format(),
            "
┏━━━┯━━━┯━━━┳━━━┯━━━┯━━━┳━━━┯━━━┯━━━┓
┃ 1 │ 2 │ 3 ┃ 4 │ 5 │ 6 ┃ 7 │ 8 │ 9 ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ 4 │ 5 │ 6 ┃ 7 │ 8 │ 9 ┃ 1 │ 2 │ 3 ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ 7 │ 8 │ 9 ┃ 1 │ 2 │ 3 ┃ 4 │ 5 │ 6 ┃
┣━━━┿━━━┿━━━╋━━━┿━━━┿━━━╋━━━┿━━━┿━━━┫
┃ 2 │ 1 │ 4 ┃ 3 │ 6 │ 5 ┃ 8 │ 9 │ 7 ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ 3 │ 6 │ 5 ┃ 8 │ 9 │ 7 ┃ 2 │ 1 │ 4 ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ 8 │ 9 │ 7 ┃ 2 │ 1 │ 4 ┃ 3 │ 6 │ 5 ┃
┣━━━┿━━━┿━━━╋━━━┿━━━┿━━━╋━━━┿━━━┿━━━┫
┃ 5 │ 3 │ 1 ┃ 6 │ 4 │ 2 ┃ 9 │ 7 │ 8 ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ 6 │ 4 │ 2 ┃ 9 │ 7 │ 8 ┃ 5 │ 3 │ 1 ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ 9 │ 7 │ 8 ┃ 5 │ 3 │ 1 ┃ 6 │ 4 │ 2 ┃
┗━━━┷━━━┷━━━┻━━━┷━━━┷━━━┻━━━┷━━━┷━━━┛
"
        );
    }

    #[test]
    fn test_x_indices() {
        let real_indices: Vec<usize> = grid_template
            .match_indices('x')
            .map(|(i, _x)| i)
            .collect::<Vec<usize>>();
        assert_eq!(&x_indices[..], &real_indices);
    }
}
