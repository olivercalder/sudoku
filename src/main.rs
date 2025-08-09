use std::io::Write;

mod grid;
mod row;

fn main() {
    let mut stdout = std::io::stdout().lock();

    write!(stdout, "building all rows...\n");

    let all_rows = row::build_rows();
    write!(stdout, "total rows: {}\n", all_rows.len());

    let (col_count, box_count) = row::successors_per_row();
    write!(stdout, "column successors per row: {}\n", col_count);
    write!(stdout, "box successors per row: {}\n", box_count);

    write!(stdout, "computing grids...\n");
    let mut grid = grid::Grid::first();
    stdout.write_all(grid.format().as_bytes()).unwrap();
    let mut count = 1;

    while grid.next() {
        count += 1;
        stdout.write_all(grid.format().as_bytes()).unwrap();
    }

    write!(stdout, "total grids: {}", count);
}

// TODO: use an iter which holds the grid and format string
// TODO: see if making `Grid` `Copy` is faster than dereferencing
// TODO: only update parts of the format string which have changed when printing
// TODO: use https://github.com/jonhoo/inferno to profile and find bottlenecks
