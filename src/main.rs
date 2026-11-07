mod grid;
mod row;

use crate::grid::Grid;

fn main() {
    println!("building all rows...");
    let all_rows = row::build_rows();
    println!("total rows: {}", all_rows.len());
    let (col_count, box_count) = row::successors_per_row();
    println!("column successors per row: {}", col_count);
    println!("box successors per row: {}", box_count);

    println!("solved grids:");
    let mut grid = Grid::new();
    while let Some(g) = grid.solve_next() {
        println!("{}", g);
        grid = g
    }
}
