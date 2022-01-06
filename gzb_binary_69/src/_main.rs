
pub mod parser;
pub mod reader;
pub mod workers;

pub use reader::{Reader,PointerType};
use std::time::Instant;

fn main() {

    let time_start = Instant::now();

    //-------------------------
    //make big value
    //-------------------------

    let mut big_value = vec![];
    let mut last_put = 0;
    loop {
        if big_value.len() == 1024{break;}
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

    // add curropt data test
    if false{
        collect.append(&mut vec![0,0,0,0/*3*/,1,2/*5*/,0/*6*/,3/*7*/,0,0,0/*10*/]);
    }

    //add for fill space test
    if true {
        for _ in 0..3{collect.push(0);}
        collect.append(&mut parser::writer::init(
            (1 as u64).to_be_bytes().to_vec(),
            String::from("value").as_bytes().to_vec(),
        ).unwrap());
        for _ in 0..3{collect.push(0);}
        collect.append(&mut parser::writer::init(
            (2 as u64).to_be_bytes().to_vec(),
            String::from("value").as_bytes().to_vec(),
        ).unwrap());
        // for _ in 0..100{collect.push(0);}
        println!("bytes alloted : {:?}",time_start.elapsed().as_millis());
    }

    let mut last_key_filled:u64 = 1;

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
    if false {
        // let mut index:u64 = 1;
        for _ in 0..500_000{
            let mut build = parser::writer::init(
                last_key_filled.to_be_bytes().to_vec(),
                big_value.clone()
            ).unwrap();
            // collect.append(&mut vec![0,0,0,0]);
            collect.append(&mut build);
            last_key_filled += 1;
        }
        // collect.append(&mut vec![0,0,0,0]);
        println!("lines alloted : {:?} {:?} {:?}ms",last_key_filled,collect.len(),time_start.elapsed());
    }

    //add for empty space test
    if true {
        for _ in 0..0_500_000_010{
            collect.push(0);
        }
        println!("bytes alloted : {:?} {:?}",collect.len(),time_start.elapsed());
    }

    //-------------------------
    //debug input buffer
    //-------------------------

    if false{

        //30034 starts at 9_099_999
        //30033 starts at 9_099_696

        let test_build = parser::writer::init(
            (30034 as u64).to_be_bytes().to_vec(),
            big_value.clone()
        ).unwrap();

        let mut test = vec![];
        for i in 9_099_999-10..9_099_999+303+10{
            test.push(collect[i].clone());
        }

        // println!("{:?}",test);

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
    if true{
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
    let mut buffer = vec![];
    for i in collect{
        if &buffer.len() == &100000{
            pool.push(buffer.clone());
            buffer.clear();
            buffer.push(i);
        } else {
            buffer.push(i);
        }
    }
    if buffer.len() > 0{
        pool.push(buffer);
    }

    let reader_time = Instant::now();
    loop{
        if pool.len()==0{
            // println!("bytes pushed : {:?}",time_start.elapsed().as_millis());
            break;
        }
        let mut part = pool.remove(0);
        // println!("part : {:?}",part);
        r.map(&mut part);
    }
    // r.map(&mut vec![0,0,0,0,0]);
    // r.map(&mut vec![0,0,0]);
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
        println!("{:?}",r.empty_map);
        // println!("{:?}",r.empty_end);
        // println!("{:?}",r.empty_start);
        println!("{:?}",r.pointers.len());
        println!("{:?}",r.corrupt.len());
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

    if false {
        let mut keys:Vec<u64> = vec![];
        for i in r.pointers.keys(){
            keys.push(crate::workers::u64_from_bytes(i.to_vec()).unwrap());
        }

        keys.sort();

        let mut not_found = vec![];
        for b in 1..30_150{
            if !keys.contains(&b){
                not_found.push(b);
            }
        }

        println!("{:?}",not_found);
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


