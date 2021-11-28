
pub mod parser;
pub mod reader;
pub mod workers;

pub use reader::{Reader,PointerType};
use std::time::Instant;

fn main() {

    let time_start = Instant::now();

    //-------------------------
    //parse test
    //-------------------------

    let mut parsed_1 = parser::writer::init(
        (1 as u64).to_be_bytes().to_vec(),
        String::from("value 12").as_bytes().to_vec(),
    ).unwrap();

    let mut parsed_2 = parser::writer::init(
        (2 as u64).to_be_bytes().to_vec(),
        String::from("value 23334").as_bytes().to_vec(),
    ).unwrap();

    let mut parsed_3 = parser::writer::init(
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

    //add for empty space test
    if false {
        for _ in 0..99_999{
            collect.push(0);
        }
        println!("bytes alloted : {:?}",time_start.elapsed().as_millis());
    }

    //add for fill space test
    if false {
        collect.append(&mut parser::writer::init(
            (1 as u64).to_be_bytes().to_vec(),
            String::from("value").as_bytes().to_vec(),
        ).unwrap());
        for _ in 0..3{collect.push(0);}
        collect.append(&mut parser::writer::init(
            (2 as u64).to_be_bytes().to_vec(),
            String::from("value").as_bytes().to_vec(),
        ).unwrap());
        for _ in 0..100{collect.push(0);}
        println!("bytes alloted : {:?}",time_start.elapsed().as_millis());
    }

    //add for line quantity test
    if false {
        let mut index:u64 = 1;
        for _ in 0..1000{
            let mut build = parser::writer::init(
                index.to_be_bytes().to_vec(),
                String::from("value").as_bytes().to_vec(),
            ).unwrap();
            collect.append(&mut vec![0,0,0,0]);
            collect.append(&mut build);
            index += 1;
        }
        collect.append(&mut vec![0,0,0,0]);
        println!("lines alloted : {:?} {:?}",index,time_start.elapsed().as_millis());
    }

    //add for remove test
    if true{
        collect.append(&mut vec![0,0,0,0]);
        collect.append(&mut parsed_1);
        collect.append(&mut vec![0,0,0,0]);
        collect.append(&mut parsed_2);
        collect.append(&mut vec![0,0,0,0]);
        collect.append(&mut parsed_3);
        // collect.append(&mut vec![0,0,0,0]);
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
        if &buffer.len() == &10000{
            &pool.push(buffer.clone());
            &buffer.clear();
            &buffer.push(i);
        } else {
            &buffer.push(i);
        }
    }
    if buffer.len() > 0{
        pool.push(buffer);
    }

    loop{
        if pool.len()==0{
            // println!("bytes pushed : {:?}",time_start.elapsed().as_millis());
            break;
        }
        let mut part = pool.remove(0);
        match &r.map(&mut part){
            Ok(_)=>{},
            Err(_)=>{}
        }
    }
    match &r.end(){
        Ok(_)=>{
            println!("map ended : {:?}",time_start.elapsed().as_millis());
        },
        Err(_)=>{}
    }

    //-------------------------
    //test edits 
    //-------------------------

    //test fill
    if false{
        println!("\nfill test==");
        let key = (6 as u64).to_be_bytes().to_vec();
        let value = String::from("value").as_bytes().to_vec();
        match &r.fill(key,value.len()){
            Ok(_)=>{
                println!("==fill successfull");
            },
            Err(_)=>{
                println!("\n==fill failed");
            }
        }
    }

    //test get
    if false{
        match &r.find(&(1 as u64).to_be_bytes().to_vec()){
            Ok(v)=>{
                match &r.clear(*v){
                    Ok(_)=>{
                        println!("\n>>> clear successfull");
                    },
                    Err(_)=>{
                        println!("!!! clear failed");
                    }
                }
            },
            Err(_)=>{
                println!("!!! find failed");
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
    //debug reader map
    //-------------------------

    if true{workers::print_pointers(&mut r);}
    if false {workers::print_raw_pointers(&mut r);}
    if false{workers::print_last_100_pointers(&mut r);}

    if false{
        println!("final map len : {:?} {:?}",r.map.len(),time_start.elapsed().as_millis());
    }
    if false{
        println!("final end : {:?}",time_start.elapsed().as_millis());
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


