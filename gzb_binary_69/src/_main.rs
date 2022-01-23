
use std::collections::HashSet;

pub mod parser;
pub mod reader;
pub mod workers;

pub use reader::{Reader,PointerType};
use std::time::Instant;

mod test;

fn main() {

    if false{
        test::init();
        return;
    }

    let time_start = Instant::now();

    //-------------------------
    //make big value
    //-------------------------

    let mut big_value = vec![];
    let mut last_put = 0;
    loop {
        if big_value.len() == 512{break;}
        if last_put == 200{last_put = 1;} else {last_put += 1;}
        big_value.push(last_put);
    }

    //-------------------------
    //parse test
    //-------------------------

    let mut _parsed_1 = parser::writer::init(
        (1 as u64).to_be_bytes().to_vec(),
        String::from("value 12").as_bytes().to_vec(),
    ).unwrap();

    let mut _parsed_2 = parser::writer::init(
        (2 as u64).to_be_bytes().to_vec(),
        String::from("value 23334").as_bytes().to_vec(),
    ).unwrap();

    let mut _parsed_3 = parser::writer::init(
        (3 as u64).to_be_bytes().to_vec(),
        String::from("value 999999991").as_bytes().to_vec(),
    ).unwrap();

    //-------------------------
    // build primary buffer
    //-------------------------

    let mut collect = vec![];
    let mut last_key_filled:u64 = 1;

    if false{
        for _ in 0..3000{collect.push(0);}
    }

    // add curropt data test
    if false{
        collect.append(&mut vec![0,0,0,0/*3*/,1,2/*5*/,0/*6*/,3/*7*/,0,0,0/*10*/]);
    }

    //add for fill space test
    if false {
        for _ in 0..5{collect.push(0);}
        collect.append(&mut vec![1,2,3,0,0,0,0,1,0]);
        collect.append(&mut parser::writer::init(
            (1 as u64).to_be_bytes().to_vec(),
            big_value.clone()
        ).unwrap());
        last_key_filled += 1;
        for _ in 0..5{collect.push(0);}
        collect.append(&mut parser::writer::init(
            (2 as u64).to_be_bytes().to_vec(),
            big_value.clone()
        ).unwrap());
        for _ in 0..5{collect.push(0);}
        // last_key_filled += 1;
        // for _ in 0..100{collect.push(0);}
        println!("lines alloted : {:?} {:?} {:?}ms",last_key_filled,collect.len(),time_start.elapsed());
    }

    if false{
        // println!("\ncollect : {:?}\n",collect);
        workers::buff_print(&collect);
    }

    if false{

        // collect.append(&mut parser::writer::init(
        //     (30032 as u64).to_be_bytes().to_vec(),
        //     big_value.clone()
        // ).unwrap());

        // collect.append(&mut parser::writer::init(
        //     (30033 as u64).to_be_bytes().to_vec(),
        //     big_value.clone()
        // ).unwrap());

        // collect.append(&mut parser::writer::init(
        //     (30034 as u64).to_be_bytes().to_vec(),
        //     big_value.clone()
        // ).unwrap());

        println!("custom len : {:?}",parser::writer::init(
            (30034 as u64).to_be_bytes().to_vec(),
            big_value.clone()
        ).unwrap().len());

    }

    //add for remove test
    if false{
        collect.append(&mut vec![0,0,0,0]);
        collect.append(&mut _parsed_1);
        // for _ in 0..53{collect.push(0);}
        // collect.append(&mut vec![1,2,3]);
        collect.append(&mut vec![0,0,0,0]);
        collect.append(&mut _parsed_2);
        // collect.append(&mut vec![0,0,0,0]);
        // collect.append(&mut _parsed_3);
        // collect.append(&mut vec![0,0,0,0]);
        last_key_filled = 4;
    }

    if false {
        for _ in 0..500{collect.push(0);}
    }

    //add for line quantity test
    if true {
        for _ in 0..0_100_000{
            let mut build = parser::writer::init(
                last_key_filled.to_be_bytes().to_vec(),
                big_value.clone()
            ).unwrap();
            for _ in 0..300{collect.push(0);}
            collect.append(&mut build);
            last_key_filled += 1;
        }
        last_key_filled -= 1;
        println!("lines alloted : {:?} {:?} {:?}",last_key_filled,collect.len(),time_start.elapsed());
    }

    //add for empty space test
    if false {
        for _ in 0..0_250_000_000{
            collect.push(0);
        }
        println!("bytes alloted : {:?} {:?}",collect.len(),time_start.elapsed());
    }

    let mut test_buffer = Vec::new();
    if false{
        test_buffer = collect.clone();
    }

    //-------------------------
    //debug input buffer
    //-------------------------

    if false{

        //30034 starts at 9_099_999
        //30033 starts at 9_099_696

        //8170

        //8169 11061528 11062598
        //8168 11197257 11198327
        //8167 11195886 11196956

        //8170 11062598+300+1..=11062598+300+1071

        //     11200000

        let test_build = parser::writer::init(
            (8169 as u64).to_be_bytes().to_vec(),
            big_value.clone()
        ).unwrap();

        let mut test = vec![];
        // let start_at = 11061528;
        for i in 11200000..=11200000+5{
            test.push(collect[i].clone());
        }

        println!("{:?}",test);

        if test_build == test{
            println!("======= are equal");
        }

        // return;

    }

    //-------------------------
    //build reader commands
    //-------------------------

    let mut r = Reader::with_capacity(1000000, 1000000);
    // r.get_values(true);

    //-------------------------
    //test commands
    //-------------------------

    //test find keys
    if false{
        r.enable_find();
        r.find_key((1 as u64).to_be_bytes().to_vec());
        r.find_key((2 as u64).to_be_bytes().to_vec());
        r.find_key((3 as u64).to_be_bytes().to_vec());
    }

    //-------------------------
    //print raw buffer
    //-------------------------

    if false{workers::buff_print(&collect);}
    if false{
        println!("input bytes size : {:?} {:?}",collect.len(),time_start.elapsed().as_millis());
    }

    //-------------------------
    //make and push blocks from primary buffer
    //-------------------------
    let mut pool = vec![];
    let parts_time = Instant::now();
    let max_buffer_size = 100_000_000;
    loop{
        if collect.len() > max_buffer_size{
            let new_buffer = collect.split_off(max_buffer_size);
            pool.push(collect);
            collect = new_buffer;
        } else {
            pool.push(collect);
            break;
        }
    }

    if true {
        println!("no_of_parts : {:?} {:?}",pool.len(),parts_time.elapsed());
    }

    let reader_time = Instant::now();
    loop{
        if pool.len()==0{
            break;
        }
        let mut part = pool.remove(0);
        r.map(&mut part);
    }
    if true{
        match &r.end(){
            Ok(_)=>{
                println!("map ended : {:?} {:?}",reader_time.elapsed(),time_start.elapsed());
            },
            Err(_)=>{}
        }
    }

    //-------------------------
    //test edits 
    //-------------------------

    //test fill
    if false{
        let fill_time_final = Instant::now();
        for _ in 0..1{
            // let fill_time = Instant::now();
            for _ in 0..1{
                let key = last_key_filled.to_be_bytes().to_vec();
                let value = String::from("value").as_bytes().to_vec();
                // let fill_time = Instant::now();
                match &r.fill(key,value.len()){
                    Ok(_)=>{
                        // println!("fill in : {:?}",fill_time.elapsed());
                    },
                    Err(_e)=>{
                        println!("==fill failed : {:?}",_e);
                    }
                }
                last_key_filled += 1;
            }
            // println!("fill_time : {:?}",fill_time.elapsed());
        }
        println!("fill_time_final : {:?} {:?}",fill_time_final.elapsed(),time_start.elapsed());
    }

    if false{
        println!("pointers len : {:?}",r.pointers.len());
        let clear_time_final = Instant::now();
        let hold_pointers = r.pointers.clone();
        let keys = hold_pointers.keys();
        let mut _index = 0;
        for key in keys{
            // if index == 10 {break;}
            // let clear_time = Instant::now();
            match &r.clear(key){
                Ok(_)=>{
                    // println!("clear_time : {:?}",clear_time.elapsed());
                },
                Err(_e)=>{
                    println!("==clear failed : {:?}",_e);
                }
            }
            _index += 1;
        }
        println!("clear_time_final : {:?} {:?}",clear_time_final.elapsed(),time_start.elapsed());
    }

    if true{
        println!("empty : {:?}",r.empty_map.len());
        println!("pointers : {:?}",r.pointers.len());
        println!("corrupt : {:?}",r.corrupt.len());
        // r.calc();
        // println!("pointers : {:?}",r.pointers);
        // println!("empty_map : {:?}",r.empty_map);
        // println!("corrupt : {:?}",r.corrupt);
        // println!("buffer_cursor : {:?}",r.buffer_cursor);
        // println!("empty : {:?}",r.empty_map);
    }

    //test get
    if false{
        match &r.find(&(30033 as u64).to_be_bytes().to_vec()){
            Ok(_v)=>{
                println!("== found : {:?}",_v);
            },
            Err(_)=>{
                println!("!!! find failed");
            }
        }
    }

    if false{
        match &r.clear(&(1 as u64).to_be_bytes().to_vec()){
            Ok(_)=>{
                println!("\n>>> clear successfull");
            },
            Err(_)=>{
                println!("!!! clear failed");
            }
        }
    }

    //test expand
    if false{
        // if true{workers::print_pointers(&mut r);}
        match &r.expand(10){
            Ok(_)=>{
                println!("==expand successfull");
            },
            Err(_)=>{
                println!("==expand failed");
            }
        }
    }

    //-------------------------
    //complex debug reader map
    //-------------------------

    if false{

        let mut failed:Vec<Vec<u8>> = vec![];
        let mut success = 0;

        for (key,value) in r.pointers.iter(){

            let mut collect = vec![];
            for i in value.1.0..=value.1.1{
                collect.push(test_buffer[i]);
            }

            if collect == big_value{
                success += 1;
            } else {
                failed.push(key.clone());
            }

        }

        println!("value check => success : {:?} | failed : {:?}",success,failed.len());

    }

    if false {

        println!(">>> finding missing keys");

        // println!(">>> parsing keys");

        let mut keys:Vec<u64> = vec![];
        for i in r.pointers.keys(){
            keys.push(crate::workers::u64_from_bytes(i.to_vec()).unwrap());
        }

        // println!(">>> sorting keys : {:?}",keys.len());

        let mut hold_sorted = HashSet::new();
        for i in keys{
            hold_sorted.insert(i);
        }

        // println!(">>> checking keys");

        let mut not_found = vec![];
        for b in 1..=last_key_filled{
            if !hold_sorted.contains(&b){
                // println!("not_found : {:?}",b);
                not_found.push(b);
            }
        }

        println!("not_found all : {:?}",not_found);

        for i in not_found{
            match r.pointers.get(&(i-1).to_be_bytes().to_vec()){
                Some(v)=>{
                    println!("before_missing {:?} : {:?}",i-1,v);
                },
                None=>{
                    println!("!!! before_missing not_found");
                }
            }
        }

    }

    if false {

       match r.pointers.get(&(8167 as u64).to_be_bytes().to_vec()){
           Some(v)=>{
               println!("before_missing : {:?}",v);
           },
           None=>{
               println!("!!! before_missing not_found");
           }
       }

    }

    if false {

        match r.empty_end.get(&(11200000-1)){
            Some(v)=>{
                println!(">>> empty found : {:?}",v);
            },
            None=>{
                println!("!!! empty not_found");
            }
        }

    }

    //-------------------------
    //debug reader map
    //-------------------------

    if false{
        println!("\n{:?}\n",r.empty_start);
        println!("\n{:?}\n",r.empty_map);
        println!("\n{:?}\n",r.empty_end);
    }

    if false{
        println!("\n{:?}\n",r.empty_map);
    }

    if false{
        println!("\n{:?}\n",r.pointers);
    }

    if false{
        println!("\n{:?}\n",r.corrupt);
    }

    if false{
        println!("final map len : {:?} {:?}",r.pointers.len(),time_start.elapsed().as_millis());
    }
    if true{
        println!("final end : {:?}",time_start.elapsed());
    }
    if false{
        println!("{:?}",r.values);
    }
    if false{
        for i in r.values{
            println!("{:?}",String::from_utf8(i.1));
        }
    }

}


