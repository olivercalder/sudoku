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
    const VALUE_MASK: u32 = 0b11;
    const SELECTION_OFFSET: u32 = 20;
    const SELECTION_MASK: u32 = 0b1111;
    const TAKEN_OPTIONS_OFFSET: u32 = 24;

    fn new() -> Cell {
        Cell(0)
    }

    const fn selection(&self) -> Option<NonZero<u32>> {
        NonZero::new((self.0 >> Self::SELECTION_OFFSET) & Self::SELECTION_MASK)
    }

    pub fn available(&self) -> impl Iterator<Item = NonZero<u32>> {
        // safety: 1 is non-zero
        let one = unsafe { NonZero::new_unchecked(1) };
        self.available_starting_with(one)
    }

    fn available_starting_with(&self, value: NonZero<u32>) -> ValueIter {
        ValueIter::new(self.0, value)
    }

    fn select(&mut self, val: NonZero<u32>) {
        debug_assert_eq!((self.0 >> Self::SELECTION_OFFSET) & Self::SELECTION_MASK, 0);
        self.0 |= val.get() << Self::SELECTION_OFFSET;
    }

    fn deselect(&mut self) -> Option<NonZero<u32>> {
        let selection = self.selection()?;
        self.0 &= !(Self::SELECTION_MASK << Self::SELECTION_OFFSET);
        Some(selection)
    }

    /// Handle this cell being affected by a selection in another cell. Increment the value's
    /// conflict count by 1, and if that results in changing the count from 0 to 1, then increment
    /// the count of taken options. Returns true if there is at least one remaining option for
    /// this cell after the selection has had its effect, and thus the selection was valid.
    fn handle_affected(&mut self, value: NonZero<u32>) -> bool {
        let first_time = self.0 & (Self::VALUE_MASK << (value.get() * 2)) == 0;
        if first_time {
            // never allow the final value to be taken, so taken_count should only ever hit 8
            let taken_count = self.0 >> Self::TAKEN_OPTIONS_OFFSET;
            if taken_count >= 8 {
                debug_assert_eq!(taken_count, 8);
                // this cell would have no remaining options, so the selection was invalid
                return false;
            }
            self.0 += 1 << Self::TAKEN_OPTIONS_OFFSET;
        }
        self.0 += 1 << (value.get() * 2);
        true
    }

    fn unhandle_affected(&mut self, value: NonZero<u32>) {
        // ensure we never unhandle_affected when it has not been handled for this value, so
        // the value count must initially be greater than 0
        debug_assert_ne!(self.0 & (Self::VALUE_MASK << (value.get() * 2)), 0);

        self.0 -= 1 << (value.get() * 2);
        let was_first_time = self.0 & (Self::VALUE_MASK << (value.get() * 2)) == 0;
        if was_first_time {
            // Ensure we never subtract a taken option
            debug_assert_ne!(self.0 >> Self::TAKEN_OPTIONS_OFFSET, 0);

            self.0 -= 1 << Self::TAKEN_OPTIONS_OFFSET;
        }
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
                    if self.0 & (Self::VALUE_MASK << (x * 2)) == 0 {
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

struct ValueIter {
    values: u32,
    next_value: NonZero<u32>,
}

impl ValueIter {
    fn new(values: u32, starting_with: NonZero<u32>) -> Self {
        Self {
            values,
            next_value: starting_with,
        }
    }
}

impl Iterator for ValueIter {
    type Item = NonZero<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_value.get() <= WIDTH_U32
            && self.values & (Cell::VALUE_MASK << (self.next_value.get() * 2)) != 0
        {
            // XXX: could use unchecked_add since we know next_value <= WIDTH_U32
            self.next_value = self.next_value.saturating_add(1);
        }
        if self.next_value.get() > WIDTH_U32 {
            return None;
        }
        let result = Some(self.next_value);
        // XXX: could use unchecked_add since we know next_value <= WIDTH_U32
        self.next_value = self.next_value.saturating_add(1);
        result
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
    /// Return the indices of other cells which are affected by a selection of the cell at the
    /// given index. Do not include the given index in the output, so that the selected cell always
    /// appears to have at least one option remaining even when a selection has been made.
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

        // exclude the index itself
        bitvec &= !(1_u128 << index);

        IndexIter(bitvec)
    }

    /// This is a more general-purpose function which allows arbitrary valid selections to be made.
    /// As such, it does not modify `count_selected`, as that is specific to the ordered generation
    /// of sudoku grids. Returns Ok(()) if the selection was valid, else Err(i) with the index of a
    /// cell for which there are no remaining selections available if this selection were made.
    fn make_selection(&mut self, index: usize, value: NonZero<u32>) -> Result<(), usize> {
        self.cells[index].select(value);
        for i in Self::affected_indices(index) {
            if !self.cells[i].handle_affected(value) {
                self.undo_partial_selection(index, i, value);
                self.cells[index].deselect();
                return Err(i);
            }
        }
        Ok(())
    }

    fn undo_partial_selection(&mut self, index: usize, stop_at_index: usize, value: NonZero<u32>) {
        for i in Self::affected_indices(index) {
            if i == stop_at_index {
                break;
            }
            self.cells[i].unhandle_affected(value)
        }
    }

    /// This is a more general-purpose function which allows arbitrary selections to be undone. It
    /// should only be called with indices in the reverse order in which they were called with
    /// `make_selection`. It does not modify `count_selected`, as that is specific to the ordered
    /// generation of sudoku grids.
    fn unmake_selection(&mut self, index: usize) -> Option<NonZero<u32>> {
        let value = self.cells[index].deselect()?;
        for i in Self::affected_indices(index) {
            self.cells[i].unhandle_affected(value)
        }
        Some(value)
    }

    /// Select the value for the next cell which has not yet been selected, and return its index
    /// and value. If no selections are valid, undo the previous selection and make a different
    /// one. Multiple previous selections may be undone if necessary.
    ///
    /// For now, cells are always selected in order, though this is not always optimal in terms of
    /// making selections in the most constrained cells first, since e.g. completing the first
    /// square has fewer options than filling in the second row, at first.
    pub fn make_next_selection(&mut self) -> Option<(usize, NonZero<u32>)> {
        // safety: 1 is non-zero
        let mut starting_value: NonZero<u32> = unsafe { NonZero::new_unchecked(1) };
        loop {
            if self.count_selected < CELL_COUNT {
                let index = self.count_selected;
                if index % WIDTH == 0 {
                    // optimizations:
                    // - top left cell of each left box must be in ascending order
                    // - in each left box, leftmost columns must be in ascending order
                    if index % (WIDTH * BOX_WIDTH) != 0 {
                        // is start of second or third row in box
                        if let Some(prev_row_start) = self.cells[index - WIDTH].selection() {
                            starting_value = starting_value.max(prev_row_start.saturating_add(1));
                        }
                    } else if index != 0 {
                        // is start of first row in non-first box
                        if let Some(prev_box_start) =
                            self.cells[index - (WIDTH * BOX_WIDTH)].selection()
                        {
                            starting_value = starting_value.max(prev_box_start.saturating_add(1));
                        }
                    }
                }
                for selection in self.cells[index].available_starting_with(starting_value) {
                    if self.make_selection(index, selection).is_err() {
                        continue;
                    }
                    self.count_selected += 1;
                    return Some((index, selection));
                }
                // couldn't make a valid move, so try to undo last move
                if self.count_selected <= 9 {
                    // every solved grid is trivially equivalent (via substitution) to a solved
                    // grid with the first row as the numbers 1-9 in order, so don't bother
                    // unmaking moves in the first row.
                    return None;
                }
            }
            starting_value = self
                .unmake_selection(self.count_selected - 1)
                .unwrap() // panic if last move wasn't previously selected
                .saturating_add(1); // should never add past 10, but easier than checked_add.
            self.count_selected -= 1;
        }
    }

    /// Set the first row of the grid to the values 1-9.
    pub fn prepare(&mut self) {
        for index in 0..WIDTH_U32 {
            // could use unsafe new_unchecked() but this is prepare so performance doesn't matter
            let value = NonZero::new(index + 1).unwrap();
            self.make_selection(index as usize, value).unwrap();
            self.count_selected += 1;
        }
    }

    /// Solve the grid. If the grid is already solved, then undo moves to produce the next valid
    /// solved grid. Returns Some(self) if the grid had a valid next solution, else None.
    pub fn solve_next(mut self) -> Option<Self> {
        while let Some((index, _)) = self.make_next_selection() {
            if index == CELL_COUNT - 1 {
                // Grid is complete
                return Some(self);
            }
        }
        None
    }

    pub fn values(&self) -> impl Iterator<Item = Option<NonZero<u32>>> + use<'_> {
        self.cells.iter().map(|x| x.selection())
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
    use crate::grid::Grid;
    use std::num::NonZero;

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

        let expected_selections: Vec<(usize, u32)> = vec![
            // First row should be 1-9
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 8),
            (8, 9),
            // Next row should start with 4 5 6 1 2 but then fail to select 3
            (9, 4),
            (10, 5),
            (11, 6),
            (12, 1),
            (13, 2),
            // Since 3 isn't valid, try 7, then 8, then 9 (each on its own valid, but not next)
            (14, 7),
            (14, 8),
            (14, 9),
            // None worked in index 14, so try 3 in previous cell, then 7 8 9 in the next cell
            // (2 doesn't work in cell 14 for the same reason 3 didn't work previously)
            (13, 3),
            (14, 7),
            (14, 8),
            (14, 9),
            // Neither 3 nor 2 worked in index 13, so try 7
            (13, 7),
            (14, 2),
            (14, 3),
            (14, 8),
            (15, 2),
            (15, 3),
            (14, 9),
            (15, 2),
            (15, 3),
            // Same thing for 8
            (13, 8),
            (14, 2),
            (14, 3),
            (14, 7),
            (15, 2),
            (15, 3),
            (14, 9),
            (15, 2),
            (15, 3),
            // And 9
            (13, 9),
            (14, 2),
            (14, 3),
            (14, 7),
            (15, 2),
            (15, 3),
            (14, 8),
            (15, 2),
            (15, 3),
            // Now the whole thing starting again with 2 in index 12
            (12, 2),
            // Literally copied from above but with 1s and 2s swapped
            (13, 1),
            // Since 3 isn't valid, try 7, then 8, then 9 (each on its own valid, but not next)
            (14, 7),
            (14, 8),
            (14, 9),
            // None worked in index 14, so try 3 in previous cell, then 7 8 9 in the next cell
            // (2 doesn't work in cell 14 for the same reason 3 didn't work previously)
            (13, 3),
            (14, 7),
            (14, 8),
            (14, 9),
            // Neither 3 nor 2 worked in index 13, so try 7
            (13, 7),
            (14, 1),
            (14, 3),
            (14, 8),
            (15, 1),
            (15, 3),
            (14, 9),
            (15, 1),
            (15, 3),
            // Same thing for 8
            (13, 8),
            (14, 1),
            (14, 3),
            (14, 7),
            (15, 1),
            (15, 3),
            (14, 9),
            (15, 1),
            (15, 3),
            // And 9
            (13, 9),
            (14, 1),
            (14, 3),
            (14, 7),
            (15, 1),
            (15, 3),
            (14, 8),
            (15, 1),
            (15, 3),
            // Now the whole thing starting again with 3 in index 12
            (12, 3),
            // Literally copied from above but with 1-3 permuted
            (13, 1),
            // Since 3 isn't valid, try 7, then 8, then 9 (each on its own valid, but not next)
            (14, 7),
            (14, 8),
            (14, 9),
            // None worked in index 14, so try 3 in previous cell, then 7 8 9 in the next cell
            // (2 doesn't work in cell 14 for the same reason 3 didn't work previously)
            (13, 2),
            (14, 7),
            (14, 8),
            (14, 9),
            // Neither 3 nor 2 worked in index 13, so try 7
            (13, 7),
            (14, 1),
            (14, 2),
            (14, 8),
            (15, 1),
            (15, 2),
            (14, 9),
            (15, 1),
            (15, 2),
            // Same thing for 8
            (13, 8),
            (14, 1),
            (14, 2),
            (14, 7),
            (15, 1),
            (15, 2),
            (14, 9),
            (15, 1),
            (15, 2),
            // And 9
            (13, 9),
            (14, 1),
            (14, 2),
            (14, 7),
            (15, 1),
            (15, 2),
            (14, 8),
            (15, 1),
            (15, 2),
            // Now we try 7 in index 12 and actually make some (slow) progress
            (12, 7),
            // First try 1 and fail
            (13, 1),
            (14, 2),
            (14, 3),
            (14, 8),
            (15, 2),
            (15, 3),
            (14, 9),
            (15, 2),
            (15, 3),
            // Then try 2 and fail
            (13, 2),
            (14, 1),
            (14, 3),
            (14, 8),
            (15, 1),
            (15, 3),
            (14, 9),
            (15, 1),
            (15, 3),
            // Then try 3 and fail
            (13, 3),
            (14, 1),
            (14, 2),
            (14, 8),
            (15, 1),
            (15, 2),
            (14, 9),
            (15, 1),
            (15, 2),
            // 8 is correct
            (13, 8),
            // But fail on next
            (14, 1),
            (15, 2),
            (15, 3),
            (14, 2),
            (15, 1),
            (15, 3),
            (14, 3),
            (15, 1),
            (15, 2),
            // 9 is correct, then fill in the rest of the row with 1 2 3
            (14, 9),
            (15, 1),
            (16, 2),
            (17, 3),
            // Then next row is 7 8 9 1 2 3 4 5 6
            (18, 7),
            (19, 8),
            (20, 9),
            (21, 1),
            (22, 2),
            (23, 3),
            (24, 4),
            (25, 5),
            (26, 6),
            // Next things get more complicated
            (27, 2),
            (28, 1),
            (29, 4),
            (30, 3),
            // ... I don't care to continue
        ];

        for (index, value) in expected_selections {
            println!("{}", grid);
            assert_eq!(
                grid.make_next_selection(),
                Some((index, NonZero::new(value).unwrap()))
            );
        }
    }

    #[test]
    fn test_first_solved_grid() {
        let first_solved_expected: Vec<Option<NonZero<u32>>> = vec![
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![4, 5, 6, 7, 8, 9, 1, 2, 3],
            vec![7, 8, 9, 1, 2, 3, 4, 5, 6],
            vec![2, 1, 4, 3, 6, 5, 8, 9, 7],
            vec![3, 6, 5, 8, 9, 7, 2, 1, 4],
            vec![8, 9, 7, 2, 1, 4, 3, 6, 5],
            vec![5, 3, 1, 6, 4, 2, 9, 7, 8],
            vec![6, 4, 2, 9, 7, 8, 5, 3, 1],
            vec![9, 7, 8, 5, 3, 1, 6, 4, 2],
        ]
        .iter()
        .flatten()
        .map(|x| Some(NonZero::new(*x).unwrap()))
        .collect();
        let first_solved: Vec<Option<NonZero<u32>>> =
            Grid::new().solve_next().unwrap().values().collect();
        for (i, val) in first_solved_expected.iter().enumerate() {
            assert_eq!(first_solved[i], *val, "index {i} incorrect")
        }
    }

    #[test]
    fn test_second_solved_grid() {
        let second_solved_expected: Vec<Option<NonZero<u32>>> = vec![
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![4, 5, 6, 7, 8, 9, 1, 2, 3],
            vec![7, 8, 9, 1, 2, 3, 4, 5, 6],
            vec![2, 1, 4, 3, 6, 5, 8, 9, 7],
            vec![3, 6, 5, 8, 9, 7, 2, 1, 4],
            vec![8, 9, 7, 2, 1, 4, 3, 6, 5],
            vec![5, 3, 1, 6, 4, 2, 9, 7, 8],
            vec![6, 4, 8, 9, 7, 1, 5, 3, 2],
            vec![9, 7, 2, 5, 3, 8, 6, 4, 1],
        ]
        .iter()
        .flatten()
        .map(|x| Some(NonZero::new(*x).unwrap()))
        .collect();
        let second_solved: Vec<Option<NonZero<u32>>> = Grid::new()
            .solve_next()
            .unwrap()
            .solve_next()
            .unwrap()
            .values()
            .collect();
        for (i, val) in second_solved_expected.iter().enumerate() {
            assert_eq!(second_solved[i], *val, "index {i} incorrect")
        }
    }
}
