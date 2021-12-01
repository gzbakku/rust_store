

use crate::reader::{Reader,Pointer,PointerType};

pub fn init(reader:&mut Reader,end_cursor:usize){

    if reader.buffer.len() == 0{
        return;
    }

    //make pointers
    let mut counter:usize = 0;
    let mut last = 0;

    // println!("end_cursor : {:?}",end_cursor);

    // println!("\nm : {:?} b : {:?}\n",reader.map_cursor,reader.buffer_cursor);

    let pending = reader.buffer.split_off(end_cursor);

    for i in &reader.buffer{
        if i != &0 && last == 0{
            if counter>0{
                reader.map.push(Pointer::point(
                    counter,
                    reader.map_cursor,
                    reader.buffer_cursor,
                    PointerType::Empty
                ));
                reader.map_cursor += counter;
                reader.buffer_cursor += counter;
                counter = 0;
            }
            last = 1;counter += 1;
        } else
        if i == &0 && last == 1{
            if counter>0{
                reader.map.push(Pointer::point(
                    counter,
                    reader.map_cursor,
                    reader.buffer_cursor,
                    PointerType::Corrupt
                ));
                reader.map_cursor += counter;
                reader.buffer_cursor += counter;
                counter = 0;
            }
            last = 0;counter += 1;
        }
        else if i == &0 && last == 0{counter += 1;}
        else if i != &0 && last == 1{counter += 1;}
    }

    reader.buffer = pending;

    // println!("unhanled partion looped");
    if counter>0{
        if last == 0{
            reader.map.push(Pointer::point(
                counter,
                reader.map_cursor,
                reader.buffer_cursor,
                PointerType::Empty
            ));
            reader.map_cursor += counter;
            reader.buffer_cursor += counter;
        } else {
            reader.map.push(Pointer::point(
                counter,
                reader.map_cursor,
                reader.buffer_cursor,
                PointerType::Corrupt
            ));
            reader.map_cursor += counter;
            reader.buffer_cursor += counter;
        }
    }

    // println!("\nc : {:?} {:?}",reader.map_cursor,counter);

    reader.buffer_cursor = 0;

}