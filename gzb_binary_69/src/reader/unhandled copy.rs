
use std::time::Instant;
use crate::reader::{Reader};

pub fn init(reader:&mut Reader,end_cursor:usize){

    println!("unhandled : {:?}",reader.buffer_cursor);

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

pub fn empty_counter(reader:&mut Reader){

    // println!("empty_counter initiated");

    //count zeros
    let count_time = Instant::now();
    let mut counter:usize = 0;

    for i in reader.buffer_cursor..reader.buffer.len(){
        if reader.buffer[i] > 0{
            // println!("break at : {:?}",i);
            break;
        } else {
            counter += 1;
        }
    }

    // for i in &reader.buffer{
    //     if i > &0{
    //         // println!("break at : {:?}",i);
    //         break;
    //     } else {
    //         counter += 1;
    //     }
    // }
    // println!("count_time : {:?}",count_time.elapsed());
    reader.calc.count_time += count_time.elapsed().as_nanos();

    // println!("counter : {:?}",counter,);

    if counter > 0 {counter -= 1};

    // if reader.map_cursor >= 11199698 && reader.map_cursor < 11200000+50{
    //     println!("king kong : {:?}",counter);
    // }

    if counter <= 1{
        return;
    }

    // println!("buffer - : {:?}",reader.buffer);
    // println!("counter : {:?}",counter);

    //make empty_end_pointers
    let empty_end_pointers_time = Instant::now();
    let empty_start_at = reader.map_cursor + reader.buffer_cursor;
    let empty_end_at = empty_start_at + counter - 1;
    let empty_len = empty_end_at - empty_start_at + 1;
    // println!("empty_end_pointers_time : {:?}",empty_end_pointers_time.elapsed());
    reader.calc.empty_end_pointers_time += empty_end_pointers_time.elapsed().as_nanos();

    //update_reader
    let update_reader_time = Instant::now();
    // reader.buffer = reader.buffer.split_off(counter);

    // let new_buffer = reader.buffer.split_off(counter);
    // reader.buffer = new_buffer;
    // for _ in 0..counter{
    //     reader.buffer.remove(0);
    // }

    // reader.map_cursor = empty_end_at + 1;
    // reader.buffer_cursor = 0;
    reader.buffer_cursor += counter;
    // println!("update_reader_time : {:?}",update_reader_time.elapsed());
    reader.calc.update_reader_time += update_reader_time.elapsed().as_nanos();

    // println!("buffer + : {:?}",reader.buffer);

    //find if a empty_countinues
    let empty_countinues_time = Instant::now();
    let mut previous_empty_found:bool = false;
    let mut previous_empty_index:usize = 0;
    if empty_start_at>0{
        match reader.empty_end.remove(&(empty_start_at-1)){
            Some(v)=>{
                previous_empty_found = true;
                previous_empty_index = v.clone();
            },
            None=>{}
        }
    }
    // println!("empty_countinues_time : {:?}",empty_countinues_time.elapsed());
    reader.calc.empty_countinues_time += empty_countinues_time.elapsed().as_nanos();

    if previous_empty_found{
        let previous_empty_found_time = Instant::now();
        match reader.empty_map.get_mut(&previous_empty_index){
            Some(previous_index)=>{
                // println!("previous_index old : {:?}",previous_index);
                previous_index.0 = empty_end_at - previous_index.1.0 + 1;
                previous_index.1.1 = empty_end_at;
                // println!("previous_index new : {:?}",previous_index);
                // println!("new_len : {:?}",new_len);
                match reader.empty_end.insert(previous_index.1.1,previous_empty_index){
                    Some(_)=>{},
                    None=>{}
                }
            },
            None=>{}
        }
        // println!("previous_empty_found_time : {:?}",previous_empty_found_time.elapsed());
        // println!("total_empty_time : {:?}",count_time.elapsed());
        reader.calc.previous_empty_found_time += previous_empty_found_time.elapsed().as_nanos();
        reader.calc.total_empty_time += count_time.elapsed().as_nanos();
        return;
    } else {
        //no previous empty found
        let previous_empty_not_found_time = Instant::now();
        match reader.empty_map.insert(reader.empty_index,(empty_len,(empty_start_at,empty_end_at))){
            Some(_)=>{},
            None=>{}
        }
        match reader.empty_start.insert(empty_start_at,reader.empty_index){
            Some(_)=>{},
            None=>{}
        }
        match reader.empty_end.insert(empty_end_at,reader.empty_index){
            Some(_)=>{},
            None=>{}
        }
        reader.empty_index += 1;
        // println!("previous_empty_not_found_time : {:?}",previous_empty_not_found_time.elapsed());
        // println!("total_empty_time : {:?}",count_time.elapsed());
        reader.calc.previous_empty_not_found_time += previous_empty_not_found_time.elapsed().as_nanos();
        reader.calc.total_empty_time += count_time.elapsed().as_nanos();
        return;
    }
    
    //make mepty map entires
    

    // println!("buffer : {:?}",reader.buffer);
    

    // println!("empty_start_at : {}",empty_start_at);
    // println!("empty_end_at : {}",empty_end_at);
    // println!("counter : {}",counter);
    // println!("reader.buffer_cursor : {}",reader.buffer_cursor);
    // println!("buffer : {:?}",reader.buffer);

}