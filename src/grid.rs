use crate::row::Row;

const sum_of_row: u64 = 9 * (9 + 1) / 2;
const sum_of_rows: u64 = 9 * (0..8).map(|x| sum_of_row << (x << 2)).sum::<u64>();

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
                    if !initial[i].col_successor(initial[j]) {
                        continue 'outer;
                    }
                }
                for j in (i - (i % 3))..i {
                    // Ensure it's a box successor
                    if !initial[i].box_successor(initial[j]) {
                        continue 'outer;
                    }
                }
                // It's a column successor and a box successor
                break;
            }
        }
        initial
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
        sum_of_rows - self.rows.iter().sum()
    }

    pub fn format(&self) -> String {
        // Ideally, just do this
        //return format!(grid_template, self.rows().map(|r| row.iter()).flatten()...);

        // But if we can't do that, either build manually, or replace chars in the template with
        // the appropriate numbers.
        let mut buf =
            String::with_capacity(19 * ("┏━━━┯━━━┯━━━┳━━━┯━━━┯━━━┳━━━┯━━━┯━━━┓".into().len() + 1));
        buf.push_str("┏━━━┯━━━┯━━━┳━━━┯━━━┯━━━┳━━━┯━━━┯━━━┓\n");
        for row in self.rows.iter() {
            self.format_and_push_row(row, buf);
            buf.push_str("┠───┼───┼───╂───┼───┼───╂───┼───┼───┨\n");
        }
        self.format_and_push_row(self.ninth_row(), buf);
        buf.push_str("┗━━━┷━━━┷━━━┻━━━┷━━━┷━━━┻━━━┷━━━┷━━━┛");
        buf
    }

    fn format_and_push_row(row: &Row, buf: &mut String) {
        for chunk_iter in row.box_chunks() {
            buf.push_str(format!(
                "┃ {} │ {} │ {} ",
                chunk_iter.next().unwrap(),
                chunk_iter.next().unwrap(),
                chunk_iter.next().unwrap()
            ));
        }
        buf.push_str("┃\n");
    }
}

const grid_template: &str = "
┏━━━┯━━━┯━━━┳━━━┯━━━┯━━━┳━━━┯━━━┯━━━┓
┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃
┣━━━┿━━━┿━━━╋━━━┿━━━┿━━━╋━━━┿━━━┿━━━┫
┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃
┣━━━┿━━━┿━━━╋━━━┿━━━┿━━━╋━━━┿━━━┿━━━┫
┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃
┠───┼───┼───╂───┼───┼───╂───┼───┼───┨
┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃ {} │ {} │ {} ┃
┗━━━┷━━━┷━━━┻━━━┷━━━┷━━━┻━━━┷━━━┷━━━┛
";

/// Iterates through the rows in the grid.
pub struct Iter {
    grid: &Grid,
    index: u8,
    acc: u64,
}

impl Iter {
    fn from(grid: &Grid) -> Self {
        Self {
            grid: grid,
            index: 0,
            acc: 0,
        }
    }
}

impl Iterator for Iter {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < 8 {
            let current = self.rows[self.index];
            self.acc += current as u64;
            self.index += 1;
            return Some(current);
        }
        if self.index == 8 {
            self.index += 1;
            return Some((sum_of_rows - self.acc) as Row);
        }
        None
    }
}
