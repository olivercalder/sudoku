mod grid;
//mod row;

use crate::grid::Grid;
use std::time::SystemTime;

fn main() {
    //println!("building all rows...");
    //let all_rows = row::build_rows();
    //println!("total rows: {}", all_rows.len());
    //let (col_count, box_count) = row::successors_per_row();
    //println!("column successors per row: {}", col_count);
    //println!("box successors per row: {}", box_count);

    println!("solved grids:");
    let mut grid = Grid::new();
    let mut count: usize = 0;
    let start_time = SystemTime::now();
    while let Some(g) = grid.solve_next() {
        count += 1;
        if count == 1 || count % 100000 == 0 {
            println!(
                "{}th solved grid ({} seconds):\n{}",
                count,
                start_time.elapsed().unwrap().as_secs_f64(),
                g
            );
        }
        grid = g
    }
    println!("total solved grids count: {}", count);
}
