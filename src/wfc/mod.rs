use std::rc::Rc;
use std::cell::RefCell;
use std::collections::VecDeque;

/// A cell in the process of being generated.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PendingCell<TCellType: Clone> {
    /// The remaining possible cell types this could be
    pub possible_types: Vec<TCellType>,
    pub location: GridLocation,
    max_cell_types: usize,
}

impl<TCellType: Clone> PendingCell<TCellType> {
    /// The cell has been narrowed down to 1 or 0 options
    pub fn is_settled(&self) -> bool {
        self.possible_types.len() <= 1
    }

    pub fn is_untouched(&self) -> bool {
        self.possible_types.len() == self.max_cell_types
    }
}

/// A location on the grid (not a game world coordinate!)
/// Using i64 to make negative locations possible for grid visiting reasons
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GridLocation {
    pub row: i64,
    pub col: i64,
}

impl GridLocation {
    // Creates a new GridLocation
    pub fn new(row: i64, col: i64) -> GridLocation {
        GridLocation {
            row,
            col,
        }
    }

    /// Gets the coordinates of all Cells that touch this one. Note that these may include coordinates off the map. The order of coordinates is:
    /// Top left
    /// Top right
    /// Right
    /// Borrom right
    /// Bottom left
    /// Left
    pub fn get_neighbors(&self) -> Vec<GridLocation> {
        // The grid convention is that the first row is the upper left, the second row is shifted to the right, and so on
        //
        //           / \ / \ / \ / \
        //  Row 0    |0| |1| |2| |3|
        //           \ / \ / \ / \ / \
        //  Row 1     | 0 | 1 | 2 | 3 |
        //           / \ / \ / \ / \ /
        //  Row 2    |0| |1| |2| |3|
        //           \ / \ / \ / \ /

        // So, for "Odd" (even-index) rows, left is col - 1, and "Even" (odd-index) rows, left is col.
        let left_index = if self.row % 2 == 0 { self.col - 1 } else { self.col };

        vec![
            GridLocation::new(self.row - 1, left_index),
            GridLocation::new(self.row - 1, left_index + 1),
            GridLocation::new(self.row , self.col + 1),
            GridLocation::new(self.row + 1, left_index + 1),
            GridLocation::new(self.row + 1, left_index),
            GridLocation::new(self.row , self.col - 1),
        ]
    }
}

/// A grid of hexagonal cells
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HexGrid<TCellType: Clone> {
    pub grid: Vec<Vec<Rc<RefCell<PendingCell<TCellType>>>>>,
}

/// A cell, or not ¯\_(ツ)_/¯
pub type MaybeCell<TCellType: Clone> = Option<Rc<RefCell<PendingCell<TCellType>>>>;

impl<TCellType: Clone> HexGrid<TCellType> {
    /// Creates a new HexGrid
    pub fn new(rows: usize, cols: usize, init_types: &Vec<TCellType>) -> HexGrid<TCellType> {
        let mut row = 0;
        HexGrid::<TCellType> {
            // Not using vec! iterators because those clone references to the same cell
            grid: (0..rows).map(|_| {
                let mut col = 0;
                let cell_row = (0..cols).map(|_| {
                    let cell = Rc::new(RefCell::new(PendingCell::<TCellType> {
                        possible_types: init_types.to_vec(),
                        max_cell_types: init_types.len(),
                        location: GridLocation {
                            row,
                            col,
                        },
                    }));

                    col += 1;

                    cell
                }).collect();

                row += 1;

                cell_row
            }).collect(),
        }
    }

    /// Gets a reference to the cell at the given location
    pub fn get_cell(&self, loc: &GridLocation) -> MaybeCell<TCellType> {
        if loc.row < 0 || loc.col < 0 || loc.row >= self.grid.len() as i64 || loc.col >= self.grid[0].len() as i64 {
            None
        } else {
            Some(self.grid[loc.row as usize][loc.col as usize].clone())
        }
    }
}

/// Given a cell, its grid location, and the grid, reduce the number of possible cell types.
/// Return true if the cell's possibility's changed, otherwise false.
pub type WafeFunctionReducer<TCellType> = fn(loc: &GridLocation, cell: Rc<RefCell<PendingCell<TCellType>>>, grid: &HexGrid<TCellType>) -> bool;

/// Holds the context of an in-progress Wave Function Collapse resolution.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WaveFunctionCollapseContext<TCellType: Clone> {
    /// The grid of cells in progress
    grid: HexGrid<TCellType>,
    /// The queue of cells to collapse
    queue: VecDeque<GridLocation>,
}

impl<TCellType: Clone> WaveFunctionCollapseContext<TCellType> {
    pub fn new(rows: usize, cols: usize, init_types: &Vec<TCellType>) -> WaveFunctionCollapseContext<TCellType> {
        WaveFunctionCollapseContext::<TCellType> {
            grid: HexGrid::<TCellType>::new(rows, cols, init_types),
            queue: VecDeque::<GridLocation>::new(),
        }
    }

    /// Sets the type of each cell at each location
    pub fn apply_types(&mut self, types: Vec<(GridLocation, Vec<TCellType>)>) {
        for cell_type in types {
            match self.grid.get_cell(&cell_type.0) {
                None => {},
                Some(cell) => {
                    cell.borrow_mut().possible_types = cell_type.1;
                    for neighbor in cell_type.0.get_neighbors() {
                        // Since we're directly applying something here, always re-visit
                        self.queue_cell(&neighbor);
                    }
                },
            }
        }
    }

    pub fn apply_types_and_process_immediately(&mut self, types: Vec<(GridLocation, Vec<TCellType>)>, reduce_types: WafeFunctionReducer<TCellType>) {
        for cell_type in types {
            self.apply_types(vec![cell_type]);
            self.iterate_queue_complete(reduce_types);
        }
    }

    /// Processes a single cell queued to be collapsed.
    /// Passing reduce_types in here to help allow this struct to live in Yew state data
    pub fn iterate_queue(&mut self, reduce_types: WafeFunctionReducer<TCellType>) {
        let coord = self.queue.pop_front();
        match coord {
            None => return,
            Some(coord) => {
                match self.grid.get_cell(&coord) {
                    None => return,
                    Some(cell) => {
                        let cell_changed = reduce_types(&coord, cell.clone(), &self.grid);
                        if cell_changed {
                            for neighbor in coord.get_neighbors().iter() {
                                let neighbor_cell = self.grid.get_cell(neighbor);
                                match neighbor_cell {
                                    None => {},
                                    Some(neighbor_cell) => {
                                        if !neighbor_cell.borrow().is_settled() {
                                            self.queue_cell(neighbor);
                                        }
                                    }
                                }
                            }
                        }
                    },
                };
            },
        };
    }

    /// Collapse all queued cells
    pub fn iterate_queue_complete(&mut self, reduce_types: WafeFunctionReducer<TCellType>) {
        while !self.queue.is_empty() {
            self.iterate_queue(reduce_types);
        }
    }

    /// Get an immutable reference to the grid
    pub fn get_grid(&self) -> &HexGrid<TCellType> {
        &self.grid
    }

    /// Get an immutable reference to the queue.
    /// This is mainly for the UI.
    pub fn get_queue(&self) -> &VecDeque<GridLocation> {
        &self.queue
    }

    /// All cells have settled.
    pub fn is_settled(&self) -> bool {
        self.grid.grid.iter().all(|row| row.iter().all(|cell| (**cell).borrow().is_settled()))
    }

    fn queue_cell(&mut self, loc: &GridLocation) {
        if !self.queue.contains(loc) {
            self.queue.push_back(*loc);
        }
    }
}
