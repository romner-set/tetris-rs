use std::{thread, sync::{Arc, Mutex}, io::{self, Write}, time::Duration};
use crossterm::{execute, cursor, terminal::{Clear, ClearType}};

use crate::renderable_object::*;


pub fn thread(
    args: Arc<crate::Args>,
    bomb: bombs::Bomb<()>,
    objects: Arc<Mutex<Vec<RenderableObject>>>,
    current_block: Arc<Mutex<crate::Block>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
    // Main loop
        while bomb.exploded().is_none() { //check for close signal
        // The actual rendering
            execute!(io::stdout().lock(), cursor::MoveUp(1)).unwrap();
            {
                let mut stdoutl = io::stdout().lock();
                
            // Debug text
                if args.debug {
                    objects.lock().unwrap()[1].shape[0] = current_block.lock().unwrap().obj.pos[0].to_string().bytes().collect();
                    objects.lock().unwrap()[1].shape[1] = current_block.lock().unwrap().obj.pos[1].to_string().bytes().collect();
                }

            // Call rendering functions
                current_block.lock().unwrap().obj.render(&mut stdoutl);
                for obj in objects.lock().unwrap().iter() {
                    obj.render(&mut stdoutl);
                }
                
                _=stdoutl.flush();
            }

        // Wait between frames
            thread::sleep(Duration::from_nanos(1_000_000_000/args.framerate as u64));

        // Clear screen
            execute!(io::stdout().lock(), cursor::MoveDown(1), Clear(ClearType::FromCursorDown)).unwrap();
        }
    
    // Exit
        //let mut stdoutl = io::stdout().lock();
        //_=write!(stdoutl, "Exiting...\r\n"); _=stdoutl.flush();
    })
}