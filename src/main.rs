extern crate tcod;

use tcod::console::*;
use tcod::colors::{self, Color};
use std::cmp;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 30;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 150 };

fn main() {
    let mut root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/tcod tutorial")
        .init();

    let mut con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    tcod::system::set_fps(LIMIT_FPS);

    let player = Object::new(21, 16, '@', colors::WHITE);
    let mut map = make_map(); 
    map[30][22] = Tile::wall();
    map[50][22] = Tile::wall();

    let objects: Vec<Object> = vec!{};
    let mut state = GameState::new(player, objects, map);

    while !root.window_closed() {
        state.render_to(&mut con);

        blit(&con, (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), &mut root, (0, 0), 1.0, 1.0);
        root.flush();

        state.clear(&mut con);

        let key = read_keys(&mut root);
        match key {
            //player movement inputs
            PlayerCommand::MoveUp   | PlayerCommand::MoveDown | PlayerCommand::MoveLeft |
            PlayerCommand::MoveRight => { 
                let new_player =  handle_inputs(key,  &state.player, &state.map); 
                state = GameState::new(new_player, state.objects, state.map);
            },

            // misc inputs
            PlayerCommand::FullScreen => root.set_fullscreen(!root.is_fullscreen()),
            PlayerCommand::Exit => break,
            PlayerCommand::Unknown => {},
        }
    }
}

fn read_keys(root: &mut Root) -> PlayerCommand {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    
    match root.wait_for_keypress(true) {
        Key { code: Up, .. } | Key { code: Char, printable: 'k', .. } => PlayerCommand::MoveUp,
        Key { code: Down, .. } | Key { code: Char, printable: 'j', .. } => PlayerCommand::MoveDown,
        Key { code: Left, .. } | Key { code: Char, printable: 'h', .. } => PlayerCommand::MoveLeft,
        Key { code: Right, .. } | Key { code: Char, printable: 'l', .. } => PlayerCommand::MoveRight, 
        Key { code: Enter, alt: true, .. } => PlayerCommand::FullScreen ,
        Key { code: Escape, .. } => PlayerCommand::Exit,
        _ => PlayerCommand::Unknown
    }
}

fn handle_inputs(input: PlayerCommand,  player: &Object, map: &Map) -> Object {
    let (dx, dy) = match input {
            PlayerCommand::MoveUp => (0, -1),
            PlayerCommand::MoveDown => (0, 1),
            PlayerCommand::MoveLeft => (-1, 0),
            PlayerCommand::MoveRight => (1, 0),
            _ => panic!{"unexpected input!"} 
    };
    if can_move(player, map, dx, dy){ player.move_by(dx, dy) } else { *player }
}

fn can_move(object: &Object, map: &Map, dx: i32, dy: i32) -> bool {
    // of the index is in bounds then return if the tile is blocked, else return false because that
    // is off map
    //
    if let Some(y) = map.get((object.x + dx) as usize)
        .and_then(|x| x.get((object.y + dy) as usize)) { !y.blocked } else { false }
}
   
enum PlayerCommand {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    FullScreen,
    Exit,
    Unknown,
}

type Map = Vec<Vec<Tile>>;
fn make_map() -> Map {
    let map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let room1 = Rect::new(20, 15, 10, 15);
    let room2 = Rect::new(50, 15, 10, 15);

    add_room(room2, add_room(room1, map))
}

fn add_room(room: Rect, map: Map) -> Map {
    let mut m = map.clone();
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 +1)..room.y2 {
            m[x as usize][y as usize] = Tile::empty();
        }
    }
    m
}

fn create_h_tunel(x1: i32, x2: i32, y: i32, map: Map) -> Map {
    let mut m = map.clone();
    for x in  cmp::min(x1, x2)..cmp::max(x1, x2) {
        m[x as usize][y as usize] = Tile::empty();
    }
    m
}

fn create_v_tunel(y1: i32, y2: i32, x: i32, map: Map) -> Map {
    let mut m = map.clone();
    for y in  cmp::min(y1, y2)..cmp::max(y1, y2) {
        m[y as usize][x as usize] = Tile::empty();
    }
    m
}

#[derive(Clone, Copy, Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object{ x, y, char, color }
    }

    pub fn move_by(&self, dx: i32, dy: i32)-> Self {
        let new_x = self.x + dx;
        let new_y = self.y + dy;

        Object{ x: new_x, y:  new_y, char: self.char, color: self.color }
    }

    pub fn draw(&self, con: &mut Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    pub fn clear(&self, con: &mut Console) {
        con.put_char(self.x, self.y, ' ', BackgroundFlag::None);
    }
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile{ blocked: false, block_sight: false}
    }
    pub fn wall() -> Self {
        Tile{ blocked: true, block_sight: true }
    }
}

#[derive(Clone, Debug)]
struct GameState {
    player: Object,
    objects: Vec<Object>,
    map: Map,
}

impl GameState {
    pub fn new(player: Object, objects: Vec<Object>, map: Map) -> Self {
        GameState { player, objects, map }
    }

    pub fn render_to(&self, con: &mut Offscreen) {
        self.player.draw(con);
        for o in &self.objects {
            o.draw(con);
        }

        for y in 0.. MAP_HEIGHT {
            for x in 0.. MAP_WIDTH {
                let is_wall = &self.map[x as usize][y as usize].block_sight;
                if *is_wall {
                    con.set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
                } else {
                    con.set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
                }
            }
        }
    }

    pub fn clear(&self, con: &mut Offscreen) {
        self.player.clear(con);
        for o in &self.objects {
            o.clear(con);
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    x2: i32,
    y1: i32,
    y2: i32,
}

impl Rect {
    pub fn new ( x: i32, y: i32, w: i32, h: i32 ) -> Self {
        Rect { x1: x, y1: y, x2: x + w, y2: y + h }
    }
}

