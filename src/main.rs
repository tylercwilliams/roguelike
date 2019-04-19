extern crate tcod;

use tcod::console::*;
use tcod::colors::{self, Color};

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

    let player = Object::new(SCREEN_WIDTH/2, SCREEN_HEIGHT/2, '@', colors::WHITE);
    let mut map = make_map(); 
    map[30][22] = Tile::wall();
    map[50][22] = Tile::wall();

    let objects: Vec<Object> = vec!{};
    let mut state = GameState::new(player, objects, map);

    while !root.window_closed() {
        state.render_to(&mut con);

        blit(&mut con, (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), &mut root, (0, 0), 1.0, 1.0);
        root.flush();

        state.clear(&mut con);

        let key = read_keys(&mut root);
        match key {
            //player movement inputs
            PlayerCommand::MoveUp   |
            PlayerCommand::MoveDown |
            PlayerCommand::MoveLeft |
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

fn read_keys(root: &mut Root) -> PlayerCommand {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    
    match root.wait_for_keypress(true) {
        Key { code: Up, .. } => PlayerCommand::MoveUp,
        Key { code: Down, .. } => PlayerCommand::MoveDown,
        Key { code: Left, .. } =>  PlayerCommand::MoveLeft,
        Key { code: Right, .. } => PlayerCommand::MoveRight, 
        Key { code: Enter, alt: true, .. } => PlayerCommand::FullScreen ,
        Key { code: Escape, .. } => PlayerCommand::Exit,
        _ => PlayerCommand::Unknown
    }
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

type Map = Vec<Vec<Tile>>;
fn make_map() -> Map {
    vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize]
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
        &self.player.draw(con);
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
        &self.player.clear(con);
        for o in &self.objects {
            o.clear(con);
        }
    }
}
