use std::fmt;
use std::num::NonZero;

const WIDTH_U32: u32 = 9;
const WIDTH: usize = 9;
const BOX_WIDTH: usize = 3;
const CELL_COUNT: usize = WIDTH * WIDTH;

/// A Cell holds information about the possible numbers this cell might be, in addition to the
/// actual selected value.
#[derive(Clone)]
pub struct Cell(u32);

impl Cell {
    const SELECTION_OFFSET: u32 = 24;

    fn new() -> Cell {
        Cell(0)
    }

    const fn selection(&self) -> Option<NonZero<u32>> {
        NonZero::new(self.0 >> Self::SELECTION_OFFSET)
    }

    pub fn available(&self) -> impl Iterator<Item = NonZero<u32>> + use<'_> {
        (1..=WIDTH_U32)
            // safety: we're iterating over a range which starts at 1, so can't include 0
            .filter(|x| self.0 & (0b11 << (x * 2)) == 0)
            .map(|x| unsafe { NonZero::new_unchecked(x) })
    }

    fn select(&mut self, val: NonZero<u32>) {
        if cfg!(test) || cfg!(debug_assertions) {
            assert_eq!(self.0 >> Self::SELECTION_OFFSET, 0)
        }
        self.0 |= val.get() << Self::SELECTION_OFFSET;
    }

    fn deselect(&mut self) -> Option<NonZero<u32>> {
        let selection = self.selection()?;
        self.0 &= u32::MAX >> (32 - Self::SELECTION_OFFSET);
        Some(selection)
    }

    /// Returns three rows of seven characters each, representing either the selected value or the
    /// available values for the cell.
    pub fn format_rows(&self) -> Vec<String> {
        if let Some(val) = self.selection() {
            match val.get() {
                1 => vec!["   ╷   ".into(), "   │   ".into(), "   ╵   ".into()],
                2 => vec!["  ╭─╮  ".into(), "  ╭─╯  ".into(), "  ╰─╴  ".into()],
                3 => vec!["  ╭─╮  ".into(), "   ─┤  ".into(), "  ╰─╯  ".into()],
                4 => vec!["  ╷ ╷  ".into(), "  └─┼  ".into(), "    ╵  ".into()],
                5 => vec!["  ┌─╴  ".into(), "  └─╮  ".into(), "  ╶─╯  ".into()],
                6 => vec!["  ╭─╮  ".into(), "  ├─╮  ".into(), "  ╰─╯  ".into()],
                7 => vec!["  ──┐  ".into(), "   ╶┼  ".into(), "    ╵  ".into()],
                8 => vec!["  ╭─╮  ".into(), "  ├─┤  ".into(), "  ╰─╯  ".into()],
                9 => vec!["  ╭─╮  ".into(), "  ╰─┤  ".into(), "    ╵  ".into()],
                _ => {
                    assert!(val.get() > 0 && val.get() < 10);
                    vec![]
                }
            }
        } else {
            // TODO: optimize, this is super naive
            let chars: Vec<String> = (1..=WIDTH_U32)
                .map(|x| {
                    if self.0 & (0b11 << (x * 2)) == 0 {
                        format!("{x}")
                    } else {
                        " ".to_string()
                    }
                })
                .collect();
            vec![
                format!(" {} {} {} ", chars[0], chars[1], chars[2]),
                format!(" {} {} {} ", chars[3], chars[4], chars[5]),
                format!(" {} {} {} ", chars[6], chars[7], chars[8]),
            ]
        }
    }
}

struct IndexIter(u128);

impl Iterator for IndexIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.0.lowest_one()?;
        self.0 &= !self.0.isolate_lowest_one();
        Some(index as usize)
    }
}

/// A Grid holds the full state of the sudoku grid, with each cell either selected or containing
/// the set of valid potential values the cell could be. The grid also holds the stack of previous
/// moves.
pub struct Grid {
    cells: Vec<Cell>,
    count_selected: usize,
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            cells: vec![Cell::new(); CELL_COUNT],
            count_selected: 0,
        }
    }

    // XXX: it's essential that indices are unique in the output
    // TODO: hard-code affected indices for each
    fn affected_indices(index: usize) -> IndexIter {
        let mut bitvec: u128 = 0;

        let row_index = index / WIDTH;
        for i in (row_index * WIDTH)..(row_index * WIDTH + WIDTH) {
            bitvec |= 1_u128 << i
        }

        let col_index = index % WIDTH;
        for i in 0..WIDTH {
            bitvec |= 1_u128 << (col_index + (WIDTH * i))
        }

        const BOX_ROW_INDICES: usize = WIDTH * BOX_WIDTH;

        let box_left_col_index = ((index / BOX_WIDTH) * BOX_WIDTH) % WIDTH;
        let box_top_row_start_index = (index / BOX_ROW_INDICES) * BOX_ROW_INDICES;
        let box_start_index = box_top_row_start_index + box_left_col_index;
        for y in 0..BOX_WIDTH {
            for x in 0..BOX_WIDTH {
                bitvec |= 1_u128 << (box_start_index + x + (y * WIDTH))
            }
        }

        IndexIter(bitvec)
    }

    /// This is a more general-purpose function which allows arbitrary valid selections to be made.
    /// As such, it does not modify `count_selected`, as that is specific to the ordered generation
    /// of sudoku grids.
    ///
    /// For now, affected cells are not inspected, simply incremented blindly. As a result, making
    /// a selection can cause another cell to have no remaining viable moves, so the selection is
    /// invalid. Without checks, this is not known until later when one attempts to make the move
    /// in that cell. It may be more efficient to record how many valid moves remain for each cell,
    /// and whenever a possibility changes from 0 to 1 (becomes non-viable), that counter should be
    /// decremented. The cell could hold this in bits 20-23.
    fn make_selection(&mut self, index: usize, value: NonZero<u32>) {
        self.cells[index].select(value);
        for i in Self::affected_indices(index) {
            self.cells[i].0 += 1 << (value.get() * 2);
        }
    }

    /// This is a more general-purpose function which allows arbitrary selections to be undone. It
    /// should only be called with indices in the reverse order in which they were called with
    /// `make_selection`. It does not modify `count_selected`, as that is specific to the ordered
    /// generation of sudoku grids.
    fn unmake_selection(&mut self, index: usize) -> Option<NonZero<u32>> {
        let value = self.cells[index].deselect()?;
        for i in Self::affected_indices(index) {
            self.cells[i].0 -= 1 << (value.get() * 2);
        }
        Some(value)
    }

    /// Select the value for the next cell which has not yet been selected, and return its index.
    ///
    /// For now, cells are always selected in order, though this is not always optimal in terms of
    /// making selections in the most constrained cells first, since e.g. completing the first
    /// square has fewer options than filling in the second row, at first.
    pub fn make_next_selection(&mut self) -> Option<usize> {
        if self.count_selected == CELL_COUNT {
            // made all selections already
            return None;
        }
        let index = self.count_selected;
        let selection = self.cells[index].available().next()?;
        self.make_selection(index, selection);
        self.count_selected += 1;
        Some(index)
    }

    /// Set the first row of the grid to the values 1-9.
    pub fn prepare(&mut self) {
        for index in 0..WIDTH_U32 {
            // could use unsafe new_unchecked() but this is prepare so performance doesn't matter
            let value = NonZero::new(index + 1).unwrap();
            self.make_selection(index as usize, value);
            self.count_selected += 1;
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..WIDTH {
            if row == 0 {
                writeln!(
                    f,
                    "┏━━━━━━━┯━━━━━━━┯━━━━━━━┳━━━━━━━┯━━━━━━━┯━━━━━━━┳━━━━━━━┯━━━━━━━┯━━━━━━━┓"
                )?;
            } else if row % 3 == 0 {
                writeln!(
                    f,
                    "┣━━━━━━━┿━━━━━━━┿━━━━━━━╋━━━━━━━┿━━━━━━━┿━━━━━━━╋━━━━━━━┿━━━━━━━┿━━━━━━━┫"
                )?;
            } else {
                writeln!(
                    f,
                    "┠───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────┨"
                )?;
            }
            let formatted_cells: Vec<Vec<String>> = self.cells
                [(row * WIDTH)..(row * WIDTH + WIDTH)]
                .iter()
                .map(|c| c.format_rows())
                .collect();
            for printed_row_index in 0..BOX_WIDTH {
                for (i, formatted) in formatted_cells.iter().enumerate() {
                    if i % BOX_WIDTH == 0 {
                        write!(f, "┃")?;
                    } else {
                        write!(f, "│")?;
                    }
                    write!(f, "{}", formatted[printed_row_index])?;
                }
                writeln!(f, "┃")?;
            }
        }
        write!(
            f,
            "┗━━━━━━━┷━━━━━━━┷━━━━━━━┻━━━━━━━┷━━━━━━━┷━━━━━━━┻━━━━━━━┷━━━━━━━┷━━━━━━━┛"
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::{Grid, CELL_COUNT};

    const INITIAL_GRID: &str = "
┏━━━━━━━┯━━━━━━━┯━━━━━━━┳━━━━━━━┯━━━━━━━┯━━━━━━━┳━━━━━━━┯━━━━━━━┯━━━━━━━┓
┃   ╷   │  ╭─╮  │  ╭─╮  ┃  ╷ ╷  │  ┌─╴  │  ╭─╮  ┃  ──┐  │  ╭─╮  │  ╭─╮  ┃
┃   │   │  ╭─╯  │   ─┤  ┃  └─┼  │  └─╮  │  ├─╮  ┃   ╶┼  │  ├─┤  │  ╰─┤  ┃
┃   ╵   │  ╰─╴  │  ╰─╯  ┃    ╵  │  ╶─╯  │  ╰─╯  ┃    ╵  │  ╰─╯  │    ╵  ┃
┠───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────┨
┃       │       │       ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃
┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃       │       │       ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃
┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃       │       │       ┃
┠───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────┨
┃       │       │       ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃
┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃       │       │       ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃
┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃       │       │       ┃
┣━━━━━━━┿━━━━━━━┿━━━━━━━╋━━━━━━━┿━━━━━━━┿━━━━━━━╋━━━━━━━┿━━━━━━━┿━━━━━━━┫
┃   2 3 │ 1   3 │ 1 2   ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃
┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃   5 6 │ 4   6 │ 4 5   ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃
┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃   8 9 │ 7   9 │ 7 8   ┃
┠───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────┨
┃   2 3 │ 1   3 │ 1 2   ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃
┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃   5 6 │ 4   6 │ 4 5   ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃
┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃   8 9 │ 7   9 │ 7 8   ┃
┠───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────┨
┃   2 3 │ 1   3 │ 1 2   ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃
┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃   5 6 │ 4   6 │ 4 5   ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃
┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃   8 9 │ 7   9 │ 7 8   ┃
┣━━━━━━━┿━━━━━━━┿━━━━━━━╋━━━━━━━┿━━━━━━━┿━━━━━━━╋━━━━━━━┿━━━━━━━┿━━━━━━━┫
┃   2 3 │ 1   3 │ 1 2   ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃
┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃   5 6 │ 4   6 │ 4 5   ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃
┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃   8 9 │ 7   9 │ 7 8   ┃
┠───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────┨
┃   2 3 │ 1   3 │ 1 2   ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃
┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃   5 6 │ 4   6 │ 4 5   ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃
┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃   8 9 │ 7   9 │ 7 8   ┃
┠───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────┨
┃   2 3 │ 1   3 │ 1 2   ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃
┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃   5 6 │ 4   6 │ 4 5   ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃
┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃   8 9 │ 7   9 │ 7 8   ┃
┗━━━━━━━┷━━━━━━━┷━━━━━━━┻━━━━━━━┷━━━━━━━┷━━━━━━━┻━━━━━━━┷━━━━━━━┷━━━━━━━┛
";

    #[test]
    fn test_prepare_format_rows() {
        let mut grid = Grid::new();
        grid.prepare();
        assert_eq!(format!("\n{}\n", grid), INITIAL_GRID);
    }

    #[test]
    fn test_make_next_selection() {
        let mut grid = Grid::new();
        for i in 0..CELL_COUNT {
            println!("{}", grid);
            assert_eq!(grid.make_next_selection(), Some(i))
        }
        assert_eq!(grid.make_next_selection(), None)
    }
}
