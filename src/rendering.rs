use std::{thread, sync::{Arc, Mutex}, io::{self, Write}, time::Duration, collections::VecDeque};
use crossterm::{execute, cursor, terminal::{Clear, ClearType}};

use crate::renderable_object::*;


pub fn thread(
    args: Arc<crate::Args>,
    bomb: bombs::Bomb<()>,
    objects: Arc<Mutex<Vec<RenderableObject>>>,
    current_block: Arc<Mutex<crate::Block>>,
    held_block: Arc<Mutex<crate::Block>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
    // HELD text object
        let held = {
            let hblock = held_block.lock().unwrap();
            RenderableObject::new([hblock.obj.pos[0]+1, hblock.obj.pos[1]], VecDeque::from(vec![match args.width_scale {
                1 => Vec::from(*b"HELD"), 2 => Vec::from(*b"HELD\x00BLK"), _ => Vec::from(*b"HELD\x00BLOCK")
            }]), (1,1), false)
        };

    // Main loop
        while bomb.exploded().is_none() { //check for close signal
        // The actual rendering
            execute!(io::stdout().lock(), cursor::MoveUp(1)).unwrap();
            {
                let mut stdoutl = io::stdout().lock();
                let cblock = &mut current_block.lock().unwrap();
                
            // Debug text
                if args.debug {
                    objects.lock().unwrap()[1].shape[0] = cblock.obj.pos[0].to_string().bytes().collect();
                    objects.lock().unwrap()[1].shape[1] = cblock.obj.pos[1].to_string().bytes().collect();
                }

            // Call rendering functions
                for obj in objects.lock().unwrap().iter() { //playfield && text
                    obj.render(&mut stdoutl);
                }
                
                // Ghost piece
                if !args.disable_ghost {
                    let old_pos = cblock.obj.pos;
                    let old_shape = cblock.obj.shape.clone();
                    
                    while cblock.mov(0, 1, &objects.lock().unwrap()[0]) == crate::CollisionResult::NoCollision {}

                    for row in cblock.obj.shape.iter_mut() {
                        for col in row {
                            if *col != 0 {*col = 2;}
                        }
                    }

                    cblock.obj.render(&mut stdoutl);
                    cblock.obj.pos = old_pos;
                    cblock.obj.shape = old_shape;
                }

                cblock.obj.render(&mut stdoutl);
                held_block.lock().unwrap().obj.render(&mut stdoutl);
                held.render(&mut stdoutl);

                _=stdoutl.flush();
            }

        // Wait between frames
            thread::sleep(Duration::from_nanos(1_000_000_000/args.framerate as u64));

        // Clear screen
            execute!(io::stdout().lock(), Clear(ClearType::FromCursorDown)).unwrap();
        }
    
    // Exit
        //let mut stdoutl = io::stdout().lock();
        //_=write!(stdoutl, "Exiting...\r\n"); _=stdoutl.flush();
    })
}