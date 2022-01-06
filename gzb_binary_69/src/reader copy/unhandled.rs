

use crate::reader::{Reader};

pub fn init(reader:&mut Reader,end_cursor:usize){

    if reader.buffer.len() == 0{
        return;
    }
    if !reader.build_map{
        reader.buffer = reader.buffer.split_off(end_cursor);
        return;
    }

    // if end_cursor < 100{
    //     crate::workers::print_range(&reader.buffer, 0, end_cursor);
    // } else {
    //     crate::workers::print_range(&reader.buffer, 0, 100);
    // }

    // println!("corrupt len : {:?}",end_cursor);
    
    //make pointers
    let mut counter:usize = 0;
    let mut last:u8 = 0;
    let pending = reader.buffer.split_off(end_cursor);

    for i in &reader.buffer{
        if i != &0 && last == 0{
            if counter>0{

                let empty_index = reader.empty_index;
                let empty_start = reader.map_cursor;
                let empty_end = reader.map_cursor+counter-1;
                match reader.empty_map.insert(empty_index,(counter,(empty_start,empty_end))){
                    Some(_)=>{},
                    None=>{}
                }
                match reader.empty_start.insert(empty_start,empty_index){
                    Some(_)=>{},
                    None=>{}
                }
                match reader.empty_end.insert(empty_end,empty_index){
                    Some(_)=>{},
                    None=>{}
                }
                reader.empty_index += 1;

                reader.map_cursor += counter;
                reader.buffer_cursor += counter;
                counter = 0;
            }
            last = 1;counter += 1;
        } else
        if i == &0 && last == 1{
            if counter>0{
                reader.corrupt.push((counter,(reader.map_cursor,reader.map_cursor+counter-1)));
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
            
            let empty_index = reader.empty_index;
            let empty_start = reader.map_cursor;
            let empty_end = reader.map_cursor+counter-1;
            match reader.empty_map.insert(empty_index,(counter,(empty_start,empty_end))){
                Some(_)=>{},
                None=>{}
            }
            match reader.empty_start.insert(empty_start,empty_index){
                Some(_)=>{},
                None=>{}
            }
            match reader.empty_end.insert(empty_end,empty_index){
                Some(_)=>{},
                None=>{}
            }
            reader.empty_index += 1;

            reader.map_cursor += counter;
            reader.buffer_cursor += counter;
        } else {
            reader.corrupt.push((counter,(reader.map_cursor,reader.map_cursor+counter-1)));
            reader.map_cursor += counter;
            reader.buffer_cursor += counter;
        }
    }

    // println!("\nc : {:?} {:?}",reader.map_cursor,counter);

    reader.buffer_cursor = 0;

}