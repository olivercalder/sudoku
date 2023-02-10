# sudoku
Generating, counting, and solving sudoku puzzles.

## Tasks

### Solving sudoku given a partial solution

In order to solve a sudoku puzzle, we need to be able to check whether filling in a given number at a given location is a valid move -- that is, whether doing so continues to make the puzzle solvable.
Naively, this could be done by recursively making arbitrary valid choices until either the grid is complete or there are no more valid choices to make.
Another way of thinking about this is to consider whether it is possible to generate a valid sudoku grid given a set of current assignments.
In this way, a program can solve a sudoku puzzle by making arbitrary choices while verifying whether the puzzle remains solvable after each choice.

### Generating valid sudoku grids

More generally, it would be convenient to be able to generate solved sudoku grids.
Why not generate all of them.
Each row is a permutation of the numbers 1-9, and there are thus 9! = 362880 possible rows.
That's a lot, especially if we expected to allow any permutation of 9 of those rows.
However, the set of remaining valid rows dwindles dramatically as soon as we start choosing more rows.

Once we have chosen a starting row, any row which shares a digit in the same position as the starting row can never be chosen, as that would violate the column rules for sudoku.
Thus, we can pre-compute the set of valid column successors for each given row.
Once a second row is chosen, then the set of valid rows remaining is the intersection of the set of successor rows of the first row and those of the second row.
In general, the set of valid rows remaining is the intersection of the sets of column successors for all previous rows, and the size of the resulting intersection gets smaller with each row chosen.

With uniqueness of numbers in columns and rows taken care of, the remaining constraint is that of the boxes (squares? quadrants?) of the sudoku grid: each number may only occur in each 3x3 box of the board once.
If two rows are in the same 3-row group of rows (that is, the floor of the row index divided by 3 matches), then the first three numbers of the first row can share no numbers in common with the first three numbers of the second row, and so on.
Thus, for each row, we can generate the set of valid box successors; that is, rows which can be in the same row group while satisfying the constraint of uniqueness of numbers within a box.

With these two constraints in mind, one can select a valid row to add to the grid by computing the intersection of the column successor sets of each previous row on the grid, and intersecting the resulting set with the intersection of the box successors of any previous rows which fall in the same row group as the row to be added.
Assuming that taking set intersections are fast, and that the number of valid column and box successors for a given row is sufficiently small, it should be relatively efficient to generate solved sudoku grids.

### Counting the number of valid sudoku grids up to isomorphism

Once we are able to generate solved sudoku grids, we might ask ourselves how many unique sudoku grids exist.
Furthermore, we could consider how many sudoku grids are unique up to rotation and reflection, which cuts down the total number substantially.
Finally, we could consider sudoku grids to be unique up to digit, where switching every occurence of one number with every occurrence of another number (or rotating number assignments, etc.) results in an identical grid.

We could have some fun with combinatorics and abstract algebra to compute this directly, and/or we could find a way of generating only unique sudoku grids under these constraints.

### Generating sudoku puzzles to be solved by hand

Once we have solved sudoku grids, we can prepare them for a human solver by removing numbers while ensuring that there still remains only one valid solution to the puzzle.

### Working with groups and equivalence classes of sudoku puzzles given by partial solutions

There is more than one way to generate a solvable partial grid for a given solved grid.
This is certainly true if we allow partial grids with redundant information (such as the latter of a grid with N entries missing and the same grid with only N-1 entries missing).
Even if we restrict that we only allow partial grids where removing one more entry makes the grid no longer uniquely solvable, there ought to be more than one way to generate a partial grid for a given solved grid.
TODO: Prove this.

If there are multiple valid partial grids which when solved yield a given solved grids, then we can say that those partial grids are equivalent.
There are probably some interesting properties to be found by thinking of sets of equivalent partial grids as equivalence classes or groups.

## Implementation ideas

If we are to be generating millions of rows and row possibilities, we need an efficient way to store lists of rows.
Using one byte per digit would result in 9 bytes per row, which if packed into a struct would result in 16-byte alignment per row... not great.
We could do the same using a nibble (4 bits) per entry, thus resulting in 36 bits total.
This would involve some math to extract a single digit out of a row, but would get us to 8-byte alignment.

But! If we know the first 8 entries in a row, we can easily deduce the 9th.
Thus, we can use nibbles for each entry and thus use a total of 4 bits per row, or we can use 8 bytes with one for each of the first 8 digits and save ourselves a bit of conversion.

If we use nibbles, then printing the 32-bit integer in hex will produce the digits in the entries of the row (aside from the final entry, which we must deduce).
If we use bytes, then we can print the value of each byte directly and add on the extra.

Storing rows as integers or byte arrays also has the added benefit of keeping things sorted.
That is, if we start by generating every valid row in sorted order, and each time we work with a list of rows we traverse it in order, then every list of rows will remain sorted.
This can dramatically speed the time it takes to compute the intersection of sets (assuming those sets are similarly ordered), as we need only traverse each of the sets once.
