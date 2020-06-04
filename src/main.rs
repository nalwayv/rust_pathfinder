//!RUST PATH FINDER

#[allow(unused_imports)]
use sfml::{graphics::*, system::*, window::*};
//
use std::fs::File;
use std::io::{prelude::*, BufReader, Result};
//
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

// -----------------------------------
// CONSTS
// -----------------------------------
// 20 * 28 = 560
// 20 * 37 = 740
const SCREEN_WIDTH: u32 = 560;
const SCREEN_HEIGHT: u32 = 740;
const COLS: i32 = 37;
const ROWS: i32 = 28;
const BLOCK_SIZE: f32 = 20.0;

// -----------------------------------
// ENUMS
// -----------------------------------
#[derive(PartialEq)]
enum TileType {
    Open,
    Blocked,
    Light,
    Medium,
    Heavy,
}

// -----------------------------------
// PLAYER
// -----------------------------------
#[allow(dead_code)]
struct Player<'a> {
    width: f32,
    height: f32,
    position: Vector2f,
    direction: (f32, f32),
    rect: RectangleShape<'a>,
}

impl<'a> Player<'a> {
    fn new(position: (f32, f32), scale: (f32, f32)) -> Self {
        let mut rec = RectangleShape::new();
        rec.set_size(scale);
        rec.set_position(position);
        rec.set_origin((0.0, 0.0));
        rec.set_fill_color(Color::YELLOW);

        Self {
            width: scale.0,
            height: scale.1,
            position: Vector2f::new(position.0, position.1),
            direction: (0., 0.),
            rect: rec,
        }
    }

    fn get_x(&self) -> f32 {
        self.rect.position().x
    }

    fn get_y(&self) -> f32 {
        self.rect.position().y
    }

    fn draw(&mut self, window: &mut RenderWindow) {
        window.draw(&self.rect);
    }

    fn set_direction(&mut self, dir_coords: (f32, f32)) {
        // (1,0) | (-1,0) | (0,1) | (0,-1)
        self.direction.0 = dir_coords.0 - (self.get_x() / self.width);
        self.direction.1 = dir_coords.1 - (self.get_y() / self.height);
    }

    /// update player
    fn update(&mut self) {
        // scale
        let new_x = self.direction.0 * self.width;
        let new_y = self.direction.1 * self.height;

        self.rect.move_((new_x, new_y));
    }
}

// -----------------------------------
// NODE
// used with path finder
// for the bineryheap priority quque
// -----------------------------------
#[derive(Debug, Copy, Clone, Eq)]
struct Node {
    priority: i32,
    position: (i32, i32),
}

impl Node {
    fn new(priority: i32, position: (i32, i32)) -> Self {
        Self { priority, position }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

// -----------------------------------
// TILE
// -----------------------------------
#[allow(dead_code)]
struct Tile<'a> {
    width: f32,
    height: f32,
    tile_type: TileType,
    screen_pos: Vector2f,
    rect: RectangleShape<'a>,
}

impl<'a> Tile<'a> {
    fn new_tile(size: (f32, f32), tt: TileType) -> Self {
        Self {
            width: size.0,
            height: size.1,
            tile_type: tt,
            screen_pos: Vector2f::new(0.0, 0.0),
            rect: RectangleShape::new(),
        }
    }

    fn set_screen_position(&mut self, screen_position: (f32, f32)) {
        self.screen_pos.x = screen_position.0;
        self.screen_pos.y = screen_position.1;
    }

    fn set_tile_type(&mut self, t: TileType) {
        self.tile_type = t;
    }

    fn draw(&mut self, win: &mut RenderWindow) {
        self.rect.set_position(self.screen_pos);
        self.rect.set_size((self.width, self.height));
        self.rect.set_origin((0., 0.));

        match self.tile_type {
            TileType::Open => {
                self.rect.set_fill_color(Color::rgb(198, 200, 185));
            }
            TileType::Blocked => {
                self.rect.set_fill_color(Color::rgb(30, 33, 50));
            }

            TileType::Light => {
                self.rect.set_fill_color(Color::rgb(255, 226, 169));
            }
            TileType::Medium => {
                self.rect.set_fill_color(Color::rgb(153, 173, 106));
            }
            TileType::Heavy => {
                self.rect.set_fill_color(Color::rgb(207, 106, 76));
            }
        }

        win.draw(&self.rect);
    }
}

// -----------------------------------
// SELECT
// tile for mouse cursor visual
// -----------------------------------
#[allow(dead_code)]
struct Select<'a> {
    width: f32,
    height: f32,
    screen_pos: Vector2f,
    direction: (f32, f32),
    rect: RectangleShape<'a>,
}

///
impl<'a> Select<'a> {
    fn new(position: (f32, f32), scale: (f32, f32)) -> Self {
        let mut rec = RectangleShape::new();
        rec.set_size(scale);
        rec.set_position(position);
        rec.set_origin((0.0, 0.0));
        rec.set_fill_color(Color::TRANSPARENT);
        rec.set_outline_color(Color::RED);
        rec.set_outline_thickness(2.0);

        Self {
            width: scale.0,
            height: scale.1,
            screen_pos: Vector2f::new(scale.0, scale.1),
            direction: (0.0, 0.0),
            rect: rec,
        }
    }

    fn draw(&mut self, window: &mut RenderWindow) {
        window.draw(&self.rect);
    }

    fn set_direction(&mut self, dir_coords: (f32, f32)) {
        self.direction.0 = dir_coords.0;
        self.direction.1 = dir_coords.1;
    }

    fn update(&mut self) {
        let new_x = self.direction.0 * self.width;
        let new_y = self.direction.1 * self.height;

        self.rect.set_position((new_x, new_y));
    }
}

// -----------------------------------
// MAP
// uses tile coords to get position
// so (1, 2) instead of (20., 40.)
// -----------------------------------
#[allow(dead_code)]
struct Map<'a> {
    tiles: Vec<Tile<'a>>,
    width: i32,
    height: i32,
}

impl<'a> Map<'a> {
    fn new(width: i32, height: i32, data: Vec<Tile<'a>>) -> Self {
        // update to store its screen positions
        let mut map_data = data;
        for x in 0..width {
            for y in 0..height {
                let coord = x + width * y;
                if let Some(tile) = map_data.get_mut(coord as usize) {
                    let pos_x = x as f32 * tile.width;
                    let pos_y = y as f32 * tile.height;

                    tile.set_screen_position((pos_x, pos_y));
                }
            }
        }

        Self {
            tiles: map_data,
            width,
            height,
        }
    }

    /// get a vector of current tiles neighbours
    fn get_neighbours(&self, coords: (i32, i32)) -> Vec<(i32, i32)> {
        let mut result: Vec<(i32, i32)> = vec![];
        let dirs = [(0, -1), (0, 1), (-1, 0), (1, 0)];

        for d in dirs.iter() {
            let dx = coords.0 + d.0;
            let dy = coords.1 + d.1;

            let coord = dx + self.width * dy;

            if let Some(t) = self.tiles.get(coord as usize) {
                if t.tile_type != TileType::Blocked {
                    result.push((dx, dy));
                }
            }
        }

        result
    }

    /// set tile type
    fn set_tile(&mut self, coords: (i32, i32), tt: TileType) {
        let tile_id = coords.0 + self.width * coords.1;

        if let Some(t) = self.tiles.get_mut(tile_id as usize) {
            t.set_tile_type(tt);
        }
    }

    /// check if tile is of this type
    fn is_tile_type(&self, coords: (i32, i32), is_type: TileType) -> bool {
        let tile_id = coords.0 + self.width * coords.1;
        if let Some(t) = self.tiles.get(tile_id as usize) {
            return t.tile_type == is_type;
        }

        false
    }

    /// get tile cost
    fn get_tile_cost(&self, coord: (i32, i32)) -> i32 {
        let tile_id = coord.0 + self.width * coord.1;

        let mut cost = 0;
        if let Some(t) = self.tiles.get(tile_id as usize) {
            let type_cost = match t.tile_type {
                TileType::Heavy => 8,
                TileType::Medium => 4,
                TileType::Light => 2,
                _ => 1,
            };

            cost = type_cost;
        }

        cost
    }

    /// a simple heuristic from one coord to another
    fn get_distance_cost(&self, from_coord: (i32, i32), to_coord: (i32, i32)) -> i32 {
        let dx = (from_coord.0 - to_coord.0).abs();
        let dy = (from_coord.1 - to_coord.1).abs();

        dx + dy
    }

    /// draw
    fn draw(&mut self, window: &mut RenderWindow) {
        for t in self.tiles.iter_mut() {
            t.draw(window);
        }
    }
}

// -----------------------------------
// FUNCS
// -----------------------------------
/// helper that takes screen position x and y and gets tile coords.
fn get_tile_coords(screen_x: i32, screen_y: i32, rows: i32, cols: i32) -> (i32, i32) {
    let mut tx = screen_x / BLOCK_SIZE as i32;
    let mut ty = screen_y / BLOCK_SIZE as i32;
    // clamp value
    let clamp = |v: i32, min_v: i32, max_v: i32| -> i32 {
        let mut res = v;
        if v <= min_v {
            res = min_v;
        }
        if v >= max_v {
            res = max_v;
        }

        res
    };

    tx = clamp(tx, 0, rows);
    ty = clamp(ty, 0, cols);

    (tx, ty)
}

/// Dijkstra’s / Astar
///
/// ### Link
/// [redblobgames a-star](https://www.redblobgames.com/pathfinding/a-star/introduction.html)
///
fn find_path(start: (i32, i32), end: (i32, i32), map: &Map) -> Option<Vec<(i32, i32)>> {
    if map.is_tile_type(start, TileType::Blocked) || map.is_tile_type(end, TileType::Blocked) {
        return None;
    }

    let mut frontier = BinaryHeap::new();
    let mut path: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut cost_so_far: HashMap<(i32, i32), i32> = HashMap::new();

    let mut found_path = false;

    frontier.push(Node::new(0, start));
    cost_so_far.insert(start, 0);

    while let Some(current) = frontier.pop() {
        if current.position == end {
            found_path = true;
            break;
        }

        // neighbours
        for neighbour in map.get_neighbours(current.position).iter() {
            let mut new_cost = 0;
            if let Some(current_cost) = cost_so_far.get(&current.position) {
                new_cost = current_cost + map.get_tile_cost(*neighbour);
            }

            let mut is_less = false;
            if let Some(neighbour_cost) = cost_so_far.get(&neighbour) {
                is_less = new_cost < *neighbour_cost;
            }

            if !path.contains_key(&neighbour) || is_less {
                cost_so_far.insert(*neighbour, new_cost);

                let heuristic = map.get_distance_cost(end, *neighbour);

                // update frontier
                frontier.push(Node::new(new_cost + heuristic, *neighbour));
                path.insert(*neighbour, current.position);
            }
        }
    }

    // get path back
    if found_path && !path.is_empty() {
        println!("found a path");

        let mut current = end;
        let mut result = vec![];
        while current != start {
            result.push(current);
            // get next coords path from hashmap
            if let Some(next) = path.get(&current) {
                current = *next;
            } else {
                break;
            }
        }

        return Some(result);
    }

    println!("no path found!");

    None
}

/// load map data from txt file
fn load_from_file<'a>(file_path: &str) -> Result<Vec<Tile<'a>>> {
    let mut tiles = Vec::new();

    let f = File::open(file_path)?;
    let buffer = BufReader::new(f);

    for line in buffer.lines() {
        let ch: Vec<char> = line?.chars().collect();
        for c in ch {
            let tile_type = match c {
                '0' => TileType::Open,
                '1' => TileType::Blocked,
                '2' => TileType::Light,
                '3' => TileType::Medium,
                '4' => TileType::Heavy,
                _ => TileType::Blocked,
            };

            let new_tile = Tile::new_tile((BLOCK_SIZE, BLOCK_SIZE), tile_type);
            tiles.push(new_tile);
        }
    }

    Ok(tiles)
}

/// main game function
fn run(width: u32, height: u32) {
    // vars
    let mut select_tile_coords = (0, 0);
    let mut is_running = true;
    let mut is_paused = false;

    // main window
    let mut window = RenderWindow::new((width, height), "A* map", Style::CLOSE, &Default::default());
    window.set_mouse_cursor_visible(true);
    window.set_framerate_limit(30);

    // map
    let map_tiles = load_from_file("assets/maps/28x37.txt").expect("failed to load from file");
    let mut map = Map::new(ROWS, COLS, map_tiles);

    // select
    let mut select_tile = Select::new((1. * BLOCK_SIZE, 1. * BLOCK_SIZE), (BLOCK_SIZE, BLOCK_SIZE));

    // player
    let mut player = Player::new((1. * BLOCK_SIZE, 4. * BLOCK_SIZE), (BLOCK_SIZE, BLOCK_SIZE));
    let mut is_updating_player = false;
    let mut player_clock = Clock::start();
    let mut path_to_take: Vec<(i32, i32)> = Vec::new();

    // main loop
    while is_running && window.is_open() {
        // --------------------------
        // inputs
        // --------------------------
        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => {
                    is_running = false;
                }

                Event::KeyPressed { code, .. } => match code {
                    Key::Escape => is_running = false,
                    Key::P => is_paused = !is_paused,
                    // change map tile type
                    Key::Num1 => {
                        let (tx, ty) = get_tile_coords(
                            select_tile_coords.0,
                            select_tile_coords.1,
                            ROWS - 1,
                            COLS - 1,
                        );
                        // println!("{} {}", tx, ty);
                        if map.is_tile_type((tx, ty), TileType::Light) {
                            map.set_tile((tx, ty), TileType::Open);
                        } else {
                            map.set_tile((tx, ty), TileType::Light);
                        }
                    }
                    Key::Num2 => {
                        let (tx, ty) = get_tile_coords(
                            select_tile_coords.0,
                            select_tile_coords.1,
                            ROWS - 1,
                            COLS - 1,
                        );
                        // println!("{} {}", tx, ty);
                        if map.is_tile_type((tx, ty), TileType::Medium) {
                            map.set_tile((tx, ty), TileType::Open);
                        } else {
                            map.set_tile((tx, ty), TileType::Medium);
                        }
                    }
                    Key::Num3 => {
                        let (tx, ty) = get_tile_coords(
                            select_tile_coords.0,
                            select_tile_coords.1,
                            ROWS - 1,
                            COLS - 1,
                        );
                        // println!("{} {}", tx, ty);
                        if map.is_tile_type((tx, ty), TileType::Heavy) {
                            map.set_tile((tx, ty), TileType::Open);
                        } else {
                            map.set_tile((tx, ty), TileType::Heavy);
                        }
                    }
                    Key::Num4 => {
                        let (tx, ty) = get_tile_coords(
                            select_tile_coords.0,
                            select_tile_coords.1,
                            ROWS - 1,
                            COLS - 1,
                        );
                        // println!("{} {}", tx, ty);
                        if map.is_tile_type((tx, ty), TileType::Blocked) {
                            map.set_tile((tx, ty), TileType::Open);
                        } else {
                            map.set_tile((tx, ty), TileType::Blocked);
                        }
                    }
                    _ => {}
                },

                Event::MouseMoved { x, y } => {
                    // update select
                    select_tile_coords.0 = x;
                    select_tile_coords.1 = y;
                }

                Event::MouseButtonPressed { button, .. } => {
                    match button {
                        mouse::Button::Left => {
                            // end tile coords
                            let (tx, ty) = get_tile_coords(
                                select_tile_coords.0,
                                select_tile_coords.1,
                                ROWS - 1,
                                COLS - 1,
                            );
                            // current player tile coords
                            let (px, py) = get_tile_coords(
                                player.get_x() as i32,
                                player.get_y() as i32,
                                ROWS - 1,
                                COLS - 1,
                            );
                            // dont update if end tile is blocked | currently updating | is paused
                            if !map.is_tile_type((tx, ty), TileType::Blocked)
                                && is_updating_player == false
                                && !is_paused
                            {
                                if let Some(new_path) = find_path((px, py), (tx, ty), &map) {
                                    path_to_take = new_path;
                                    is_updating_player = true;
                                    player_clock.restart();
                                }
                            }
                        }
                        _ => {}
                    }
                }

                _ => {}
            }
        }

        if !is_paused {

            // --------------------------
            // update
            // --------------------------
            let (tx, ty) = get_tile_coords(
                select_tile_coords.0,
                select_tile_coords.1,
                ROWS - 1,
                COLS - 1,
            );
            select_tile.set_direction((tx as f32, ty as f32));
            select_tile.update();
     
            // -- update player positon on C key down
            if is_updating_player {
                if player_clock.elapsed_time().as_milliseconds() >= 100 {
                    if !path_to_take.is_empty() {
                        if let Some(val) = path_to_take.pop() {
                            player.set_direction((val.0 as f32, val.1 as f32));
                            player.update();
                        }
                    } else {
                        is_updating_player = false;
                    }
                    player_clock.restart();
                }
            } else {
                player.set_direction((0.0, 0.0));
            }

            // --------------------------
            // draw
            // --------------------------
            window.clear(Color::WHITE);

            map.draw(&mut window);
            player.draw(&mut window);
            select_tile.draw(&mut window);

            window.display();
        }
    }
}

fn main() {
    run(SCREEN_WIDTH, SCREEN_HEIGHT);
}
