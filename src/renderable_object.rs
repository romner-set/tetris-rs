use std::{collections::VecDeque, io::Write};
use crossterm::{cursor, execute, style::{SetForegroundColor, Color}};

use crate::CollisionResult;

#[derive(Debug, Clone)]
pub struct RenderableObject {
    pub pos: [isize; 2],
    pub shape: VecDeque<Vec<u8>>,
    pub scale: (isize, isize),
    pub is_bordered: bool,
}
impl RenderableObject {
    pub fn new(pos: [isize; 2], shape: VecDeque<Vec<u8>>, scale: (isize, isize), is_bordered: bool) -> Self {Self {pos, shape, scale, is_bordered}}
    pub fn check_collision(&self, other: &Self) -> CollisionResult {
        for (y, row) in self.shape.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                if col == 1 {
                    let actual_x = self.scale.0*x as isize + self.pos[0];
                    let actual_y = self.scale.1*y as isize + self.pos[1];

                // Wall boundary check
                    if actual_y <= other.is_bordered as isize+other.pos[1] || actual_y >= crate::PF_HEIGHT+2*other.is_bordered as isize
                    || actual_x < other.pos[0] || actual_x > other.pos[0] + other.scale.0*crate::PF_WIDTH as isize
                    {return CollisionResult::OutOfBounds}

                // Collision detection
                    if other.shape
                        [((self.scale.1*y as isize + self.pos[1] - other.pos[1])/other.scale.1 as isize - 2*other.is_bordered as isize) as usize]
                        [((self.scale.0*x as isize + self.pos[0] - other.pos[0])/other.scale.0 as isize) as usize] == 1
                    {return if self.pos[1] == 2 {CollisionResult::GameOver} else {CollisionResult::BlockCollision}}
                }
            }
        }
        CollisionResult::NoCollision
    }

// Check for & remove filled lines
    pub fn check_line_fills(&mut self) -> &mut Self{
        let len = self.shape.len(); //cache original length
        let mut idxs = Vec::new();

    // Find indexes of filled lines
        for (i, row) in self.shape.iter().enumerate() {
            let mut filled = true;
            for &col in row.iter() {
                if col != 1 {filled = false; break}
            }
            if filled {idxs.push(i);}
        }

    // Remove filled lines
        idxs.sort_unstable(); //sort and rev() the iterator to avoid removing incorrect lines
        for i in idxs.into_iter().rev() {_=self.shape.remove(i);}
        for _ in 0..len-self.shape.len() {self.shape.push_front(vec![0u8; self.shape[0].len()])} //replace removed lines

        self
    }

    pub fn imprint_to(&self, other: &mut Self) -> &Self {
        for (y, row) in self.shape.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                if col == 1 {
                    other.shape
                        [((self.scale.1*y as isize + self.pos[1] - other.pos[1])/other.scale.1 as isize - 2) as usize]
                        [((self.scale.0*x as isize + self.pos[0] - other.pos[0])/other.scale.0 as isize) as usize]
                    = 1;
                }
            }
        }

        self
    }

// Main rendering function
    pub fn render<W: Write>(self: &Self, buf: &mut W) -> &Self {
        let hborder = "─".repeat(self.shape[0].len()*self.scale.0 as usize);
        let move_to_start = cursor::MoveToColumn((self.pos[0]) as u16);

        execute!(buf, move_to_start, cursor::MoveDown((self.pos[1]) as u16)).unwrap(); //move cursor to pos

        if self.is_bordered {write!(buf, "┌{}┐\n", &hborder).unwrap();} //draw upper border
        execute!(buf, move_to_start).unwrap();

        for row in self.shape.iter() {
            for _ in 0..self.scale.1 {
                if self.is_bordered {buf.write_all(&[0xE2, 0x94, 0x82]).unwrap();} //unicode encoding of │ - left border
                for &col in row.iter() {
                    if col==1 { //white
                        for _ in 0..self.scale.0 {
                            buf.write_all(&[0xE2, 0x96, 0x88]).unwrap(); //unicode of █
                        }
                    } else if col==2 { //gray
                        execute!(buf, SetForegroundColor(Color::Grey));
                        for _ in 0..self.scale.0 {
                            buf.write_all(&[0xE2, 0x96, 0x88]).unwrap(); //unicode of █
                        }
                    } else if col >= 0x20 { //ASCII
                        for _ in 0..self.scale.0 {
                            write!(buf, "{}", col as char).unwrap();
                        }
                    } else {execute!(buf, cursor::MoveRight(self.scale.0 as u16)).unwrap();}
                }
                if self.is_bordered {buf.write_all(&[0xE2, 0x94, 0x82]).unwrap();} // │ - right border
                execute!(buf, move_to_start, /*cursor::MoveDown(1)*/crossterm::style::Print("\n")).unwrap();
            }
        }

        if self.is_bordered {write!(buf, "└{}┘", &hborder).unwrap();} //draw lower border
    
        execute!(buf, cursor::MoveToColumn(0), cursor::MoveUp((self.pos[1]+self.shape.len() as isize) as u16 + (self.is_bordered as u16)*2)).unwrap(); //reset cursor

        self
    }
}