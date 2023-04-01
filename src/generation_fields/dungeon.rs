use std::ops::Index;
use std::{rc::Rc, cell::Cell};
use std::cell::RefCell;
use lazy_static::lazy_static;
use gloo_console::{error, log};
use ran::{set_seeds, Rnum};
use crate::wfc::*;
use std::collections::VecDeque;

/// Whether the cells at each of the six neighbors connect
#[derive(Debug, Eq, Copy, Clone, PartialEq)]
pub struct CellConnections {
    pub top_left: bool,
    pub top_right: bool,
    pub right: bool,
    pub bottom_right: bool,
    pub bottom_left: bool,
    pub left: bool,
}

// IMPROVE: There's a lot of going back and forth between the struct and vector representations.
//          Eliminate one of them.

impl CellConnections {
    pub const TOP_LEFT: usize = 0;
    pub const TOP_RIGHT: usize = 1;
    pub const RIGHT: usize = 2;
    pub const BOTTOM_RIGHT: usize = 3;
    pub const BOTTOM_LEFT: usize = 4;
    pub const LEFT: usize = 5;

    pub fn new(top_left: bool, top_right: bool, right: bool, bottom_right: bool, bottom_left: bool, left: bool) -> CellConnections {
        CellConnections {
            top_left,
            top_right,
            right,
            bottom_right,
            bottom_left,
            left,
        }
    }

    /// The number of connections
    pub fn count(&self) -> usize {
        let mut count = 0;
        if self.top_left { count += 1; }
        if self.top_right { count += 1; }
        if self.right { count += 1; }
        if self.bottom_right { count += 1; }
        if self.bottom_left { count += 1; }
        if self.left { count += 1; }

        count
    }

    /// Switch from the standard vector representation to the struct version
    pub fn from_vec(vec: Vec<bool>) -> CellConnections {
        CellConnections {
            top_left: vec[CellConnections::TOP_LEFT],
            top_right: vec[CellConnections::TOP_RIGHT],
            right: vec[CellConnections::RIGHT],
            bottom_right: vec[CellConnections::BOTTOM_RIGHT],
            bottom_left: vec[CellConnections::BOTTOM_LEFT],
            left: vec[CellConnections::LEFT],
        }
    }

    /// Initialize connected to everything
    pub fn all() -> CellConnections {
        CellConnections {
            top_left: true,
            top_right: true,
            right: true,
            bottom_right: true,
            bottom_left: true,
            left: true,
        }
    }

    /// Initialize connected to nothing
    pub fn none() -> CellConnections {
        CellConnections {
            top_left: false,
            top_right: false,
            right: false,
            bottom_right: false,
            bottom_left: false,
            left: false,
        }
    }

    /// Convert to a vector
    pub fn to_vec(&self) -> Vec<bool> {
        vec![
            self.top_left,
            self.top_right,
            self.right,
            self.bottom_right,
            self.bottom_left,
            self.left,
        ]
    }

    /// Generates all possible combinations of cell connections
    pub fn all_possible() -> Vec<CellConnections> {
        (0u8..64u8).map(|num: u8| {
            CellConnections {
                top_left:     (num & 0b00000001) != 0,
                top_right:    (num & 0b00000010) != 0,
                right:        (num & 0b00000100) != 0,
                bottom_right: (num & 0b00001000) != 0,
                bottom_left:  (num & 0b00010000) != 0,
                left:         (num & 0b00100000) != 0,
            }
        }).collect()
    }

    pub fn all_except(except: &CellConnections) -> Vec<CellConnections> {
        let except_vec = except.to_vec();
        CellConnections::all_possible().into_iter().filter(|connections| {
            // This should be way less complicated...
            let vec = connections.to_vec();
            for i in 0..vec.len() {
                if !except_vec[i] && vec[i] {
                    return false;
                }
            }

            true
        }).collect()
    }

    /// Each element of cons_a or'd with the matching element of cons_b
    pub fn or(cons_a: &Vec<bool>, cons_b: &Vec<bool>) -> Vec<bool> {
        let mut index = 0;
        cons_a.into_iter().map(|val| {
            let result = *val || cons_b[index];
            index += 1;
            result
        }).collect()
    }

    // Gets the index of the opposite side (e.g BOTTOM_RIGHT would return TOP_LEFT)
    pub fn opposite_index(index: usize) -> usize {
        (index + 3) % 6
    }
}

/// The potential Dungeon cells
#[derive(Debug, Eq, Copy, Clone, PartialEq)]
pub enum DungeonCellType {
    /// Nothing. Block it off
    None,
    /// A hallway
    Hall(CellConnections),
    /// A room
    Room(CellConnections),
}

impl DungeonCellType {
    /// All possible DungeonCellType variants (including all possible connection directions)
    pub fn all() -> Vec<DungeonCellType> {
        let mut types = vec![DungeonCellType::None];

        types.append(&mut ALL_DUNGEON_CELL_HALLS.clone());
        types.append(&mut ALL_DUNGEON_CELL_ROOMS.clone());

        types
    }
}

fn all_halls() -> Vec<DungeonCellType> {
    CellConnections::all_possible().iter().map(|connection| DungeonCellType::Hall(*connection)).collect()
}

fn all_rooms() -> Vec<DungeonCellType> {
    CellConnections::all_possible().iter().map(|connection| DungeonCellType::Room(*connection)).collect()
}

type CellProbabilities = Vec<(DungeonCellType, f64)>;

lazy_static! {
    /// The total numhber of possible dunceon cell types
    pub static ref NUM_POSSIBLE_DUNGEON_CELLS: usize = {
        DungeonCellType::all().len()
    };

    pub static ref ALL_DUNGEON_CELL_HALLS: Vec<DungeonCellType> = {
        all_halls()
    };

    pub static ref ALL_DUNGEON_CELL_ROOMS: Vec<DungeonCellType> = {
        all_rooms()
    };

    pub static ref CELL_PROBABILITIES: CellProbabilities = {
        let mut probabilities = vec![(DungeonCellType::None, 0.1f64)];

        probabilities.append(&mut all_halls().into_iter().map(|hall| {
            let num_connections = match hall {
                DungeonCellType::Hall(cons) => cons.count(),
                _ => 0,
            } as f64;

            let probability = ((4.0 - num_connections).abs() / 6.0) * 0.7;

            (hall, if num_connections > 2.0 { 0.0 } else { probability })
        }).collect::<Vec<(DungeonCellType, f64)>>());

        probabilities.append(&mut all_rooms().into_iter().map(|room| {
            let num_connections = match room {
                DungeonCellType::Room(cons) => cons.count(),
                _ => 0,
            } as f64;

            let probability = ((4.0 - num_connections) / 6.0) * 0.1;

            (room, if num_connections > 2.0 { 0.0 } else { probability })
        }).collect::<Vec<(DungeonCellType, f64)>>());

        probabilities
    };
}

/// The different ways rooms can be shaped
#[derive(Debug, Eq, Copy, Clone, PartialEq)]
pub enum DungeonRoomShape {
    /// A single-cell room
    Single(CellConnections),
    /// A cluster of three cells. Order is peak, bottom left, bottom right
    Cluster3(CellConnections, CellConnections, CellConnections),
}

/// A descriptor for generating a room in the dungeon
#[derive(Debug, Eq, Copy, Clone, PartialEq)]
pub struct DungeonRoom {
    pub shape: DungeonRoomShape,
    pub location: GridLocation,
}

#[derive(Debug, Eq, Copy, Clone, PartialEq)]
enum DungeonGeneratorState {
    Init,
    PlacedRooms,
    ForgingPath,
    Wfc,
    Fill,
    Clean,
    Idle,
}

/// Generates dungeons leveraging Wave Function Collapse
pub struct DungeonGenerator {
    /// The wave function collapse context
    pub wfc: WaveFunctionCollapseContext<DungeonCellType>,
    pub seed: u64,
    pub start_location: GridLocation,
    pub goal_location: GridLocation,
    pub goal_locations: Vec<GridLocation>,
    pub goal_entrance_location: GridLocation,
    pub rows: usize,
    pub cols: usize,
    pub meander_factor: f64,
    state: DungeonGeneratorState,
    random: Rnum,
    cursor_location: GridLocation,
    iteration_count: i32,
    valid_path_cells: Vec<Rc<RefCell<PendingCell<DungeonCellType>>>>,
    unfilled_cells: Vec<Rc<RefCell<PendingCell<DungeonCellType>>>>,
}

impl DungeonGenerator {
    pub fn new(rows: usize, cols: usize) -> DungeonGenerator {
        DungeonGenerator {
            wfc: WaveFunctionCollapseContext::<DungeonCellType>::new(rows, cols, &DungeonCellType::all()),
            seed: 1u64,
            start_location: GridLocation::new(-1, -1),
            goal_location: GridLocation::new(-1, -1),
            goal_locations: vec![],
            goal_entrance_location: GridLocation::new(-1, -1),
            rows,
            cols,
            meander_factor: 0.7,
            state: DungeonGeneratorState::Init,
            random: Rnum::newf64(),
            cursor_location: GridLocation::new(-1, -1),
            iteration_count: 0,
            valid_path_cells: vec![],
            unfilled_cells: vec![],
        }
    }

    /// Steps forward one step through generation state
    pub fn step(&mut self) {
        match self.state {
            DungeonGeneratorState::Init => {
                log!("DGEN: INIT");

                set_seeds(self.seed);

                self.goal_location = self.random_interior_location();

                self.goal_entrance_location = self.goal_location.get_neighbors()[0];

                let goal_in_small_room = self.random_bool_default();
                let mut goal_connections = CellConnections::new(true, false, false, false, false, false);

                self.goal_locations = if goal_in_small_room {
                    self.add_room(&DungeonRoom { shape: DungeonRoomShape::Single(goal_connections), location: self.goal_location })
                } else {
                    goal_connections.bottom_left = true;
                    goal_connections.bottom_right = true;
                    self.add_room(&DungeonRoom { shape: DungeonRoomShape::Cluster3(
                        goal_connections,
                        CellConnections::new(false, true, true, false, false, false),
                        CellConnections::new(true, false, false, false, false, true),
                    ), location: self.goal_location })
                };

                self.start_location = self.random_interior_location();

                while self.goal_locations.contains(&self.start_location) {
                    self.start_location = self.random_interior_location();
                }

                let start_in_hall = self.random_bool_default();

                if start_in_hall {
                    self.wfc.apply_types(vec![(
                        self.start_location,
                        vec![DungeonCellType::Hall(self.random_connections())],
                    )]);
                } else {
                    // TODO: Don't try to connect to edge
                    let mut start_connections = CellConnections::none().to_vec();
                    start_connections[self.random_in_range(0f64, 5f64) as usize] = true;
                    self.add_room(&DungeonRoom { shape: DungeonRoomShape::Single(CellConnections::from_vec(start_connections)), location: self.start_location });
                }

                self.state = DungeonGeneratorState::PlacedRooms;
            },
            DungeonGeneratorState::PlacedRooms => {
                log!("DGEN: PLACED_ROOMS");
                self.cursor_location = self.start_location;
                self.state = DungeonGeneratorState::Wfc;
            },
            DungeonGeneratorState::ForgingPath => {
                log!("DGEN: FORGING_PATH");

                let neighbor_locations = self.cursor_location.get_neighbors();

                let next_location_index = if self.cursor_location == self.start_location {
                    let start_cell = self.wfc.get_grid().get_cell(&self.start_location).unwrap();
                    let start_connections = match start_cell.borrow().possible_types[0] {
                        DungeonCellType::Hall(cons) => cons,
                        DungeonCellType::Room(cons) => cons,
                        _ => panic!("Start cell is not a connecting type"),
                    }.to_vec();

                    start_connections.iter().position(|i| *i).unwrap()
                } else {
                    let neighbor_distances: Vec<i64> = neighbor_locations.iter().map(|loc| {
                        (self.goal_entrance_location.row as i64 - loc.row as i64).abs() +
                        (self.goal_entrance_location.col as i64 - loc.col as i64).abs()
                    }).collect();


                    let min_distance = neighbor_distances.iter().min().unwrap();
                    let closest_cell = neighbor_distances.iter().position(|val| val == min_distance).unwrap();

                    // Evilness warning: Random selection is how we avoid not passing the if statement later on when it overlaps the start cell, etc.
                    let choose_closest = self.random_bool(self.meander_factor);
                    if choose_closest {
                        closest_cell
                    } else {
                        self.random_in_range(0f64, 5f64) as usize
                    }
                };

                let next_location = neighbor_locations[next_location_index];
                let next_cell = self.wfc.get_grid().get_cell(&next_location);
                if next_location != self.start_location && !self.goal_locations.contains(&next_location) && next_cell.is_some() {
                    let next_cell = next_cell.unwrap();
                    let mut next_required_connections = CellConnections::none().to_vec();
                    next_required_connections[CellConnections::opposite_index(next_location_index)] = true;
                    if next_location == self.goal_entrance_location {
                        // Force connection to goal. For now, the goal entrance is always top left
                        next_required_connections[CellConnections::BOTTOM_RIGHT] = true;
                    }

                    let next_cell_types = next_cell.borrow().possible_types.clone().into_iter().filter(|cell_type| {
                        match cell_type {
                            DungeonCellType::None => false,
                            DungeonCellType::Room(_) => false,
                            DungeonCellType::Hall(connections) => {
                                let connections_vec = connections.to_vec();
                                for i in 0..connections_vec.len() {
                                    if next_required_connections[i] && !connections_vec[i] {
                                        return false;
                                    }
                                }

                                true
                            },
                        }
                    }).collect();

                    self.wfc.apply_types(vec![(
                        next_location,
                        next_cell_types,
                    )]);

                    if self.valid_path_cells.len() > 0 {
                        // make sure the previous cell connects here
                        let previous_cell = self.valid_path_cells.last().unwrap().clone();
                        let mut previous_required_connections = CellConnections::none().to_vec();
                        previous_required_connections[next_location_index] = true;

                        let previous_cell_types: Vec<DungeonCellType> = previous_cell.borrow().possible_types.clone().into_iter().filter(|cell_type| {
                            match cell_type {
                                DungeonCellType::None => false,
                                DungeonCellType::Room(_) => false,
                                DungeonCellType::Hall(connections) => {
                                    let connections_vec = connections.to_vec();
                                    for i in 0..connections_vec.len() {
                                        if previous_required_connections[i] && !connections_vec[i] {
                                            return false;
                                        }
                                    }

                                    true
                                },
                            }
                        }).collect();

                        let previous_location = previous_cell.borrow().location;

                        self.wfc.apply_types(vec![(
                            previous_location,
                            previous_cell_types,
                        )]);
                    }

                    self.valid_path_cells.push(next_cell.clone());

                    self.cursor_location = next_location;
                    if self.cursor_location == self.goal_entrance_location {
                        self.state = DungeonGeneratorState::Wfc;
                    }
                }
            },
            DungeonGeneratorState::Wfc => {
                log!("DGEN: WFC");
                self.wfc.iterate_queue(DungeonGenerator::collapse_cell);
                if self.wfc.get_queue().is_empty() {
                    if self.valid_path_cells.is_empty() {
                        self.state = DungeonGeneratorState::ForgingPath;
                    } else {
                        self.unfilled_cells =
                            self.wfc.get_grid().grid.iter()
                            .flatten()
                            .filter(|cell| !cell.borrow().is_settled())
                            .map(|cell| cell.clone())
                            .collect();

                        self.state = DungeonGeneratorState::Fill;
                    }
                }
            },
            DungeonGeneratorState::Fill => {
                log!("DGEN: FILL");

                if self.wfc.is_settled() {
                    self.state = DungeonGeneratorState::Clean;
                } else {
                    let cell_index = self.random_in_range(0.0, (self.unfilled_cells.len() as f64) - 1.0) as usize;
                    let cell = self.unfilled_cells[cell_index].clone();
                    self.apply_random_cell_type(&vec![cell]);
                    self.state = DungeonGeneratorState::Wfc;
                }
            },
            DungeonGeneratorState::Clean => {
                let start_cell = self.wfc.get_grid().get_cell(&self.start_location).unwrap();
                let mut conencted_locations = vec![start_cell.borrow().location];

                type DepthQueue = VecDeque::<Rc<RefCell<PendingCell<DungeonCellType>>>>;
                let mut depth_queue = DepthQueue::new();
                depth_queue.push_back(start_cell.clone());

                while !depth_queue.is_empty() {
                    let cell = depth_queue.pop_front().unwrap();
                    let connections = match cell.borrow().possible_types[0] {
                        DungeonCellType::Hall(cons) => cons,
                        DungeonCellType::Room(cons) => cons,
                        _ => CellConnections::none(),
                    }.to_vec();

                    let mut neighbor_index = 0;
                    let mut conencted_neighbors: Vec::<GridLocation> = cell.borrow().location.get_neighbors().into_iter()
                        .filter(|loc| {
                            let result = connections[neighbor_index] && !conencted_locations.contains(loc);
                            neighbor_index += 1;
                            result
                        })
                        .collect();

                    let mut conencted_neighbor_cells: DepthQueue = conencted_neighbors.clone().into_iter()
                        .map(|loc| self.wfc.get_grid().get_cell(&loc).unwrap().clone()).collect();

                    depth_queue.append(&mut conencted_neighbor_cells);
                    conencted_locations.append(&mut conencted_neighbors);
                }

                for row in 0..self.rows {
                    for col in 0..self.cols {
                        let loc = GridLocation::new(row as i64, col as i64);

                        // If it doesn't connect to anything we can get to, remove the cell
                        if !conencted_locations.contains(&loc) {
                            self.wfc.get_grid().get_cell(&loc).unwrap().borrow_mut().possible_types = vec![];
                        }
                    }
                }

                self.state = DungeonGeneratorState::Idle;
            },
            DungeonGeneratorState::Idle => {
                if !self.wfc.get_queue().is_empty() {
                    self.state = DungeonGeneratorState::Wfc;
                }
            },
        }

        self.iteration_count += 1;
    }

    /// Run through the complete generation process
    pub fn generate(&mut self) {
        while self.can_do_more_work() {
            self.step();
        }
    }

    /// More generation can be done, whether it's dungeon steps or wfc
    pub fn can_do_more_work(&self) -> bool {
        self.state != DungeonGeneratorState::Idle || !self.wfc.get_queue().is_empty()
    }

    /// Adds a room to the dungeon
    /// Returns all locations of the room.
    pub fn add_room(&mut self, room: &DungeonRoom) -> Vec<GridLocation> {
        let neighbor_locations = room.location.get_neighbors();
        let neighbor_cells: Vec<MaybeCell<DungeonCellType>> = neighbor_locations.iter().map(|loc| self.wfc.get_grid().get_cell(&loc)).collect();

        match room.shape {
            DungeonRoomShape::Single(connections) => {
                self.wfc.apply_types(vec![
                    (room.location, vec![DungeonCellType::Room(connections)]),
                ]);

                vec![room.location]
            },
            DungeonRoomShape::Cluster3(cons1, cons2, cons3) => {
                if neighbor_cells[CellConnections::BOTTOM_LEFT].is_none() || neighbor_cells[CellConnections::BOTTOM_RIGHT].is_none() {
                    panic!("Bad cell location adding room");
                }

                self.wfc.apply_types(vec![
                    (room.location, vec![DungeonCellType::Room(cons1)]),
                    (neighbor_locations[CellConnections::BOTTOM_LEFT], vec![DungeonCellType::Room(cons2)]),
                    (neighbor_locations[CellConnections::BOTTOM_RIGHT], vec![DungeonCellType::Room(cons3)]),
                ]);

                vec![
                    room.location,
                    neighbor_locations[CellConnections::BOTTOM_LEFT],
                    neighbor_locations[CellConnections::BOTTOM_RIGHT],
                ]
            },
        }
    }

    pub fn debug_state(&self) -> &str {
        match self.state {
            DungeonGeneratorState::Init => "Init",
            DungeonGeneratorState::PlacedRooms => "Placed Rooms",
            DungeonGeneratorState::ForgingPath => "Forging Path",
            DungeonGeneratorState::Wfc => "Wfc",
            DungeonGeneratorState::Fill => "Fill",
            DungeonGeneratorState::Clean => "Clean",
            DungeonGeneratorState::Idle => "Idle",
        }
    }

    fn random_interior_location(&self) -> GridLocation {
        GridLocation::new(
            self.random.rannum_in(1f64, (self.rows as f64) - 2f64).getf64().unwrap() as i64,
            self.random.rannum_in(1f64, (self.cols as f64) - 2f64).getf64().unwrap() as i64,
        )
    }

    fn random_location(&self) -> GridLocation {
        GridLocation::new(
            self.random.rannum_in(0f64, (self.rows as f64) - 1f64).getf64().unwrap() as i64,
            self.random.rannum_in(0f64, (self.cols as f64) - 1f64).getf64().unwrap() as i64,
        )
    }

    fn random_bool(&self, likelihood: f64) -> bool {
        self.random.rannum_in(0f64, 100f64).getf64().unwrap() <= (likelihood * 100f64)
    }

    fn random_bool_default(&self) -> bool {
        self.random_bool(0.5f64)
    }

    fn random_in_range(&self, min: f64, max: f64) -> f64 {
        self.random.rannum_in(min, max).getf64().unwrap()
    }

    fn random_connections(&self) -> CellConnections {
        CellConnections::new(
            self.random_bool_default(),
            self.random_bool_default(),
            self.random_bool_default(),
            self.random_bool_default(),
            self.random_bool_default(),
            self.random_bool_default(),
        )
    }

    fn apply_random_cell_type(&mut self, cells: &Vec<Rc<RefCell<PendingCell<DungeonCellType>>>>) {
        self.wfc.apply_types(cells.into_iter().map(|cell| {
            let location = cell.borrow().location;
            let types = vec![self.get_random_cell_type(&cell.borrow().possible_types)];
            (location, types)
        }).collect());
    }

    fn get_random_cell_type(&self, possible_types: &Vec<DungeonCellType>) -> DungeonCellType {
        if possible_types.len() == 0 {
            return DungeonCellType::None
        } else if possible_types.len() == 1 {
            return possible_types[0];
        }

        type ProbabilityTriple = (DungeonCellType, f64, f64);
        let mut total = 0f64;

        // Track the running total across the array, so that the larger the type's relative value, the more "space" it takes up in the rolling area. When we
        // pick a number in the range between 0 and total, the highest index where the number is above the running total is the selection.
        let probabilities: Vec<ProbabilityTriple> = CELL_PROBABILITIES.clone().into_iter()
            .filter(|pair| possible_types.contains(&pair.0))
            .map(|pair| {
                let start_value = total;
                let triple = (pair.0, pair.1, start_value);
                total += pair.1;
                triple
            })
            .collect();

        let value = self.random_in_range(0f64, total);

        for i in 0..probabilities.len() {
            if value < probabilities[i].2 {
                if i == 0 {
                    panic!("Shouldn't be possible. probabilities[0].0 should be 0 and 0 is not less than 0.");
                }
                return probabilities[i - 1].0;
            }
        }

        probabilities.last().unwrap().0
    }

    /// The rules passed to the WFC solver
    fn collapse_cell(loc: &GridLocation, cell: Rc<RefCell<PendingCell<DungeonCellType>>>, grid: &HexGrid<DungeonCellType>) -> bool {
        let mut cell = cell.borrow_mut();
        let initial_option_count = cell.possible_types.len();

        if initial_option_count <= 1 {
            return false;
        }

        // Rules should be quite simple:
        // If a cell is none, nothing may connect.
        // For rooms and halls, if they may connect to us, narrow down to what may connect to them

        let cell_connects = |neighbor_index: usize, connections: &CellConnections| {
            let neighbor_connection_index = CellConnections::opposite_index(neighbor_index);
            connections.to_vec()[neighbor_connection_index]
        };

        let neighbor_locations = loc.get_neighbors();
        let neighbor_cells: Vec<(GridLocation, MaybeCell<DungeonCellType>)> =
            neighbor_locations.into_iter().map(|neighbor_loc| (neighbor_loc, grid.get_cell(&neighbor_loc))).collect();

        let mut neighbor_check_index = 0;

        #[derive(PartialEq, Clone, Copy)]
        enum RequirementStatus {
            Required,
            Banned,
            Neutral
        }

        let required_connections: Vec<RequirementStatus> = neighbor_cells.iter().map(|(_, maybe_cell)| {
            let result = match maybe_cell {
                None => RequirementStatus::Banned,
                Some(cell) => {
                    let other_types = &(*cell.borrow()).possible_types;
                    if other_types.len() > 1 {
                        RequirementStatus::Neutral
                    } else if other_types.len() == 0 {
                        RequirementStatus::Banned
                    } else {
                        match other_types[0] {
                            DungeonCellType::None => RequirementStatus::Banned,
                            DungeonCellType::Hall(connections) => if cell_connects(neighbor_check_index, &connections) { RequirementStatus::Required } else { RequirementStatus::Banned },
                            DungeonCellType::Room(connections) => if cell_connects(neighbor_check_index, &connections) { RequirementStatus::Required } else { RequirementStatus::Banned },
                        }
                    }
                },
            };

            neighbor_check_index += 1;

            result
        }).collect();

        let require_connections = |cons: Vec<bool>| {
            let mut index = 0;

            cons.iter().all(|val| {
                let requirement = required_connections[index];
                index += 1;

                if requirement == RequirementStatus::Required {
                    *val
                } else if requirement == RequirementStatus::Banned {
                    !*val
                } else {
                    true
                }
            })
        };

        let must_connect_to_something = required_connections.iter().any(|value| *value == RequirementStatus::Required);

        if required_connections.iter().all(|value| *value == RequirementStatus::Banned) {
            cell.possible_types = vec![DungeonCellType::None];
        } else {
            cell.possible_types = cell.possible_types.clone().into_iter().filter(|cell_type| {
                match cell_type {
                    DungeonCellType::None => !must_connect_to_something,
                    DungeonCellType::Hall(connections) => require_connections(connections.to_vec()),
                    DungeonCellType::Room(connections) => require_connections(connections.to_vec()),
                }
            }).collect();
        }

        cell.possible_types.len() < initial_option_count
    }
}
