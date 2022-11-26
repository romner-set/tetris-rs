
use std::{thread, sync::{Arc, Mutex}, time::Duration};

use bombs::Fuse;

use crate::{renderable_object::*, CollisionResult, Block, PF_HEIGHT, PF_WIDTH};

const DEFAULT_SPEED: f64 = 5E+8; //nanosecs between updates

pub fn thread(
    args: Arc<crate::Args>,
    bomb: bombs::Bomb<()>,
    objects: Arc<Mutex<Vec<RenderableObject>>>,
    current_block: Arc<Mutex<crate::Block>>,
    block_defs: Arc<[Block; 7]>,
    fuse: Arc<Mutex<Option<Fuse<()>>>>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
    // Main loop
        let scale = current_block.lock().unwrap().obj.scale;
        while bomb.exploded().is_none() { //check for close signal
        // Wait between updates
            thread::sleep(Duration::from_nanos((DEFAULT_SPEED/args.speed) as u64));

        // Do stuff
            let cblock = &mut current_block.lock().unwrap();

            if cblock.obj.check_collision(&objects.lock().unwrap()[0]) == CollisionResult::GameOver { //gameover check
                if let Some(_fuse) = std::mem::take(&mut *fuse.lock().unwrap()) {
                    _=_fuse.light(());

                    thread::sleep(Duration::from_nanos(2_000_000_000/args.framerate as u64)); //wait 2 frames for rendering thread to close

                    let mut stdoutl = std::io::stdout().lock();
                    let playfield = &mut objects.lock().unwrap()[0];
                    
                    crossterm::execute!(std::io::stdout().lock(), crossterm::cursor::MoveUp(1)).unwrap();
                    playfield.render(&mut stdoutl);

                    RenderableObject::new([playfield.pos[0]+PF_WIDTH as isize/2*playfield.scale.0-6, PF_HEIGHT/2*playfield.scale.1-2], std::collections::VecDeque::from(vec![
                        Vec::from(*b" GAME  OVER "),
                        Vec::from(*b"------------"),
                        Vec::from(*b"Esc to exit."),
                    ]), (1,1), true).render(&mut stdoutl);
                    break;
                }
            }
            if cblock.mov(0, scale.1 as isize, &objects.lock().unwrap()[0]) != CollisionResult::NoCollision { //move down
                cblock.obj.imprint_to(&mut objects.lock().unwrap()[0]);
                **cblock = Block::new_random(&block_defs);
                objects.lock().unwrap()[0].check_line_fills();
            }
        }
    
    // Exit
        //let mut stdoutl = io::stdout().lock();
        //_=write!(stdoutl, "Exiting...\r\n"); _=stdoutl.flush();
    })
}