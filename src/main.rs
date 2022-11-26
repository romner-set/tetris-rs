// Imports
mod physics;
mod rendering;
mod renderable_object;
use renderable_object::*;

use std::{io, time::Duration, thread, sync::{Arc, Mutex}, collections::VecDeque};
use bombs::Bomb;
use crossterm::{execute, terminal::*, event::*, cursor, style::*};
use clap::Parser;
use rand::Rng;

// Constants
const PF_WIDTH: usize = 10; //unscaled playfield dimensions
const PF_HEIGHT: isize = 20;

// Console arguments
#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value_t = 15, help = "Framerate at which the game is rendered.")]
    framerate: u8,

    #[arg(short, long, default_value_t = String::from("ADQEWS "), help = "Controls, in the format <LEFT><RIGHT><ROTATE_LEFT><ROTATE_RIGHT><HOLD><SOFT_DROP><HARD_DROP>.")]
    controls: String,

    #[arg(short, long, default_value_t = 1., help = "Multiplicative gravity strength modifier. Accepts decimals, non-positive values turn gravity off.")]
    speed: f64,

    #[arg(short, long, default_value_t = 2, help = "Multiplicative horizontal scale at which the playfield is rendered. Has to be a natural number.")]
    width_scale: u8,

    #[arg(short, long, default_value_t = 1, help = "Multiplicative vertical scale at which the playfield is rendered. Has to be a natural number.")]
    vertical_scale: u8,

    #[arg(long, help = "Print out some additional information while playing.")]
    debug: bool,

    #[arg(short, long, help = "Disables ghost pieces.")]
    disable_ghost: bool,
}

// Structs
#[derive(Debug, Clone)]
pub struct Block {
    obj: RenderableObject,
    pivot: [usize; 2]
}
impl Block {
    fn new(shape: Vec<Vec<u8>>, pivot: [usize; 2], scale: (isize, isize), offset: isize) -> Self {
        Self {obj: RenderableObject::new([3*scale.0+1+offset, 2], VecDeque::from(shape), scale, false), pivot}
    }
    fn new_random(defs: &Arc<[Self; 7]>) -> Self {defs[rand::thread_rng().gen_range(0..7)].clone()}

    fn mov(&mut self, x: isize, y: isize, playfield: &RenderableObject) -> CollisionResult {
        self.obj.pos[0] = self.obj.pos[0] as isize + x;
        self.obj.pos[1] = self.obj.pos[1] as isize + y;

        let collision = self.obj.check_collision(playfield);
        if collision != CollisionResult::NoCollision {
            self.obj.pos[0] = self.obj.pos[0] as isize - x;
            self.obj.pos[1] = self.obj.pos[1] as isize - y;
        }
        collision
    }

    fn rotate(&mut self, direction: isize, playfield: &RenderableObject) -> CollisionResult {
        let old_shape = self.obj.shape.clone();
        let mut new_shape = VecDeque::from(vec![vec![0u8; 4]; 4]);

        for (y, row) in self.obj.shape.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                if col == 1 {
                    new_shape[(self.pivot[1] as isize + (self.pivot[0] - x*2) as isize * (-direction)) as usize / 2]
                             [(self.pivot[0] as isize + (self.pivot[1] - y*2) as isize * direction) as usize / 2]
                    = 1;
                }
            }
        }

        self.obj.shape = new_shape;

        let collision = self.obj.check_collision(playfield);
        if collision != CollisionResult::NoCollision {self.obj.shape = old_shape;}
        collision
    }
}

struct Controls {
    left: char,
    right: char,
    rotate_left: char,
    rotate_right: char,
    hold: char,
    soft_drop: char,
    hard_drop: char,
}
impl Controls {
    fn parse(arg_str: &str) -> Self {
        let str_lowercase = arg_str.to_lowercase();
        let mut c = str_lowercase.chars();
        Self {
            left:         c.next().unwrap(),
            right:        c.next().unwrap(),
            rotate_left:  c.next().unwrap(),
            rotate_right: c.next().unwrap(),
            hold:         c.next().unwrap(),
            soft_drop:    c.next().unwrap(),
            hard_drop:    c.next().unwrap(),
        }
    }
}

// Enums
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionResult {NoCollision, OutOfBounds, BlockCollision, GameOver}


fn main() -> Result<(), Box<dyn std::error::Error>> {
// Argument parsing
    let args = Arc::new(Args::parse());

    // Scale
    if args.vertical_scale == 0 || args.width_scale == 0 {
        execute!(io::stdout(),
            SetForegroundColor(Color::Red), SetAttribute(Attribute::Bold), Print("error: "),
            ResetColor, SetAttribute(Attribute::Reset),                    Print("Scales have to be positive\r\n"),
        )?;
        std::process::exit(2)
    }
    let scale = (args.width_scale as isize, args.vertical_scale as isize);

    // Controls
    if args.controls.chars().count() != 7 {
        execute!(io::stdout(),
            SetForegroundColor(Color::Red), SetAttribute(Attribute::Bold), Print("error: "),
            ResetColor, SetAttribute(Attribute::Reset),                    Print("Controls have to be 6 characters long (default examples: '"),
            SetForegroundColor(Color::DarkYellow),                         Print("ADQEWS "),
            ResetColor,                                                    Print("' for QWERTY, '"),
            SetForegroundColor(Color::DarkYellow),                         Print("ASQFWR "),
            ResetColor,                                                    Print("' for Colemak, '"),
            SetForegroundColor(Color::DarkYellow),                         Print("AE'.,O "),
            ResetColor,                                                    Print("' for Dvorak)\r\n"),
        )?;
        std::process::exit(2)
    }
    let controls = Controls::parse(&args.controls);

// Setup
    enable_raw_mode()?; //handle *all* input manually, including stuff like ctrl+c
    execute!(io::stdout(), EnableMouseCapture, cursor::Hide)?;

    let (fuse, bomb) = {
        let (fuse, bomb) = Bomb::new(); //used for exiting all threads on program termination
        (Arc::new(Mutex::new(Some(fuse))), bomb)
    };

    let offset = 2+8*args.debug as isize; //x-axis offset of playfield & blocks
    let block_defs = Arc::new([ //define blocks
        Block::new(vec![
            vec![0,0,0,0],
            vec![1,1,1,1],
            vec![0,0,0,0],
            vec![0,0,0,0]
        ], [3, 3], scale, offset), //all pivot values are multiplied by two, because floats are stupid and I don't want to deal with them if I don't have to
        Block::new(vec![
            vec![1,0,0,0],
            vec![1,1,1,0],
            vec![0,0,0,0],
            vec![0,0,0,0]
        ], [2, 2], scale, offset),
        Block::new(vec![
            vec![0,0,1,0],
            vec![1,1,1,0],
            vec![0,0,0,0],
            vec![0,0,0,0]
        ], [2, 2], scale, offset),
        Block::new(vec![
            vec![0,1,1,0],
            vec![0,1,1,0],
            vec![0,0,0,0],
            vec![0,0,0,0]
        ], [3, 1], scale, offset),
        Block::new(vec![
            vec![0,1,1,0],
            vec![1,1,0,0],
            vec![0,0,0,0],
            vec![0,0,0,0]
        ], [2, 2], scale, offset),
        Block::new(vec![
            vec![0,1,0,0],
            vec![1,1,1,0],
            vec![0,0,0,0],
            vec![0,0,0,0]
        ], [2, 2], scale, offset),
        Block::new(vec![
            vec![1,1,0,0],
            vec![0,1,1,0],
            vec![0,0,0,0],
            vec![0,0,0,0]
        ], [2, 2], scale, offset),
    ]);

    let current_block = Arc::new(Mutex::new(Block::new_random(&block_defs))); //create a new block; separate objects at first, then permanently drawn onto the playfield once dropped
    let mut held_block = (Arc::new(Mutex::new(Block::new(vec![vec![0u8; 4]; 4], [3,3], scale, offset + PF_WIDTH as isize*scale.0))), false);
    held_block.0.lock().unwrap().obj.pos[1] = 0;
    held_block.0.lock().unwrap().obj.is_bordered = true;

    let objects_to_render = Arc::new(Mutex::new(Vec::with_capacity(4)));
        objects_to_render.lock().unwrap().push(RenderableObject::new([offset,0], VecDeque::from(vec![vec![0u8; PF_WIDTH]; PF_HEIGHT as usize]), scale, true)); //playfield
    if args.debug {
        objects_to_render.lock().unwrap().push(RenderableObject::new([0,2], VecDeque::from(vec![vec![0u8; 1]; 11]), (1,1), false)); //debug text
    }

// Spawn other threads
    if args.speed > 0. {physics::thread(
        Arc::clone(&args),
        bomb.clone(),
        Arc::clone(&objects_to_render),
        Arc::clone(&current_block),
        Arc::clone(&block_defs),
        Arc::clone(&fuse)
    );}
    rendering::thread(
        Arc::clone(&args),
        bomb,
        Arc::clone(&objects_to_render),
        Arc::clone(&current_block),
        Arc::clone(&held_block.0),
    );

// Input handling
    loop {
        match read() { //blocking read
            Ok(Event::Key(k)) => match k.code {
                KeyCode::Esc => break,
                KeyCode::Char(c) => match c.to_ascii_lowercase() {
                    'c' if k.modifiers.contains(KeyModifiers::CONTROL) => break,

                    ch => {
                        let cblock = &mut current_block.lock().unwrap();
                        
                    // Controls
                        if ch == controls.left {
                            cblock.mov(-(scale.0 as isize), 0, &objects_to_render.lock().unwrap()[0]);
                        } else if ch == controls.right {
                            cblock.mov(scale.0 as isize, 0, &objects_to_render.lock().unwrap()[0]);
                        
                        } else if ch == controls.rotate_left {
                            cblock.rotate(-1, &objects_to_render.lock().unwrap()[0]);
                        } else if ch == controls.rotate_right {
                            cblock.rotate(1, &objects_to_render.lock().unwrap()[0]);
                        
                        } else if ch == controls.hold {
                            let hblock = &mut held_block.0.lock().unwrap();
                            
                            if !held_block.1 {
                                hblock.obj.shape = Block::new_random(&block_defs).obj.shape;
                                held_block.1 = true;
                            }

                            std::mem::swap(&mut cblock.obj.shape, &mut hblock.obj.shape);
                            std::mem::swap(&mut cblock.pivot, &mut hblock.pivot);

                            cblock.obj.pos = block_defs[0].obj.pos;
                        } else if ch == controls.soft_drop {
                            cblock.mov(0, scale.1 as isize, &objects_to_render.lock().unwrap()[0]);
                        } else if ch == controls.hard_drop {
                            while cblock.mov(0, 1, &objects_to_render.lock().unwrap()[0]) == CollisionResult::NoCollision {}

                            cblock.obj.imprint_to(&mut objects_to_render.lock().unwrap()[0]);
                            **cblock = Block::new_random(&block_defs);

                            objects_to_render.lock().unwrap()[0].check_line_fills();
                        }

                    // Debug text
                        if args.debug {
                            objects_to_render.lock().unwrap()[1].shape[3] = format!("{:#?} {}", k.modifiers, ch).bytes().collect();
                            
                            let mut i=1;
                            for (y, row) in cblock.obj.shape.iter().enumerate() {
                                for (x, &col) in row.iter().enumerate() {
                                    if col==1 {
                                        objects_to_render.lock().unwrap()[1].shape[6+i] = format!("x:{} y:{}", cblock.obj.pos[0]+cblock.obj.scale.0*x as isize, cblock.obj.pos[1]+cblock.obj.scale.1*y as isize).bytes().collect();
                                        i+=1;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => ()
            }
            Ok(_) => (),
            Err(_) => break
        };
    }

// Cleanup
    if let Some(_fuse) = std::mem::take(&mut *fuse.lock().unwrap()) { //if closed manually instead of CollisionResult::GameOver
        let fire = _fuse.light(());                                       //send close signal
        while !fire.extinguished() {thread::sleep(Duration::from_millis(1))} //wait until all threads are closed
    } else {execute!(io::stdout().lock(), cursor::MoveDown(2), Clear(ClearType::FromCursorDown)).unwrap();}

    disable_raw_mode()?;
    execute!(io::stdout(), DisableMouseCapture, cursor::Show)?;

    Ok(())
}
