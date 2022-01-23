
// use std::time::Instant;
use crate::reader::{Reader};

pub fn init(reader:&mut Reader,start:usize,end:usize){

    if !reader.build_map{
        reader.buffer_cursor = end + 1;
        return;
    }

    let mut empty_found = false;
    let mut corrupt_found = false;
    let mut empty_start_at = 0;
    let mut corrupt_start_at = 0;

    for i in start..=end{
        let v = reader.buffer[i];
        if v == 0{
            if corrupt_found{
                process_corrupt(reader,corrupt_start_at,i-1);
                corrupt_found = false;
                corrupt_start_at = 0;
            }
            if !empty_found{
                empty_found = true;
                empty_start_at = i;
            }
        } else {
            if empty_found{
                process_empty(reader,empty_start_at,i-1);
                empty_found = false;
                empty_start_at = 0;
            }
            if !corrupt_found{
                corrupt_found = true;
                corrupt_start_at = i;
            }
        }
    }

    if empty_found{
        process_empty(reader,empty_start_at,end);
    }
    if corrupt_found{
        process_corrupt(reader,corrupt_start_at,end);
    }

    fn process_corrupt(reader:&mut Reader,start:usize,end:usize){
        // println!("c start : {:?} end : {:?}",start,end);
        reader.corrupt.push((end - start + 1,(start,end)));
    }

    fn process_empty(reader:&mut Reader,start:usize,end:usize){
        // println!("e start : {:?} end : {:?}",start,end);

        //find previous empty
        let mut previous_empty_found = false;
        let mut previous_empty_index = 0;
        if start > 0{
            match reader.empty_end.remove(&(start-1)){
                Some(v)=>{
                    previous_empty_found = true;
                    previous_empty_index = v;
                    // println!("previous empty : {:?}",v);
                },
                None=>{}
            }
        }

        //expand previous empty
        if previous_empty_found{
            // println!("empty--");
            match reader.empty_map.get_mut(&previous_empty_index){
                Some(mut v)=>{
                    v.1.1 = end;
                    v.0 = v.1.1 - v.1.0 + 1;
                    reader.empty_end.insert(previous_empty_index,v.1.1);
                },
                None=>{}
            }
        }

        //make new empty
        if !previous_empty_found{
            // println!("empty++");
            let empty_len = end - start + 1;
            let empty_index = reader.empty_index;
            reader.empty_map.insert(empty_index,(empty_len,(start,end)));
            reader.empty_end.insert(end,empty_index);
            reader.empty_start.insert(start,empty_index);
            reader.empty_index += 1;
        }

    }

    reader.buffer_cursor = end + 1;

}

pub fn empty_counter(reader:&mut Reader){

    //count zeros
    // let count_time = Instant::now();
    let mut counter:usize = 0;
    for i in reader.buffer_cursor..reader.buffer.len(){
        if reader.buffer[i] > 0{
            break;
        } else {
            counter += 1;
        }
    }
    // reader.calc.count_time += count_time.elapsed().as_nanos();

    if counter > 0 {counter -= 1};
    if counter <= 1{return;}

    //make empty_end_pointers
    // let empty_end_pointers_time = Instant::now();
    let empty_start_at = reader.map_cursor + reader.buffer_cursor;
    let empty_end_at = empty_start_at + counter - 1;
    let empty_len = empty_end_at - empty_start_at + 1;
    // reader.calc.empty_end_pointers_time += empty_end_pointers_time.elapsed().as_nanos();

    //update_reader
    // let update_reader_time = Instant::now();
    reader.buffer_cursor += counter;
    // reader.calc.update_reader_time += update_reader_time.elapsed().as_nanos();

    //find if a empty_countinues
    // let empty_countinues_time = Instant::now();
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
    // reader.calc.empty_countinues_time += empty_countinues_time.elapsed().as_nanos();

    if previous_empty_found{
        // let previous_empty_found_time = Instant::now();
        match reader.empty_map.get_mut(&previous_empty_index){
            Some(previous_index)=>{
                previous_index.0 = empty_end_at - previous_index.1.0 + 1;
                previous_index.1.1 = empty_end_at;
                match reader.empty_end.insert(previous_index.1.1,previous_empty_index){
                    Some(_)=>{},
                    None=>{}
                }
            },
            None=>{}
        }
        // reader.calc.previous_empty_found_time += previous_empty_found_time.elapsed().as_nanos();
        // reader.calc.total_empty_time += count_time.elapsed().as_nanos();
        return;
    } else {
        // let previous_empty_not_found_time = Instant::now();
        //no previous empty found
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
        // reader.calc.previous_empty_not_found_time += previous_empty_not_found_time.elapsed().as_nanos();
        // reader.calc.total_empty_time += count_time.elapsed().as_nanos();
        return;
    }

}