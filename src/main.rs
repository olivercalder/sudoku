mod grid;
mod row;

fn main() {
    println!("building all rows...");
    let all_rows = row::build_rows();
    println!("total rows: {}", all_rows.len());
    let (col_count, box_count) = row::successors_per_row();
    println!("column successors per row: {}", col_count);
    println!("box successors per row: {}", box_count);

    println!("computing first grid...");

    println!("{}", grid::Grid::first().format());
}
