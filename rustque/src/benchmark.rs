use std::time::Instant;
use crate::Que;
use crate::config::Config;
use std::sync::Arc;
use tokio::sync::Notify;
use futures::future::join_all;

#[derive(Debug,Clone,Default)]
pub struct Benchmark{
    pub no_of_writers:u8,
    pub no_of_writes:u64,
    pub map_files:Vec<String>,
    pub write_size:usize,
    pub min_que_size:u64,
    pub expansion_size:u64,
    pub no_of_disk_workers:u8
}

#[allow(dead_code)]
impl Benchmark{
    pub fn new(
        no_of_writers:u8,
        no_of_writes:u64,
        map_files:Vec<String>,
        write_size:usize,
        min_que_size:u64,
        expansion_size:u64,
        no_of_disk_workers:u8
    )->Benchmark{
        Benchmark{
            no_of_writers:no_of_writers,
            no_of_writes:no_of_writes,
            map_files:map_files,
            write_size:write_size,
            min_que_size:min_que_size,
            expansion_size:expansion_size,
            no_of_disk_workers:no_of_disk_workers
        }
    }
}

#[derive(Debug,Clone,Default)]
pub struct BenchmarkBuilder{
    file_path:String,
    collect:Vec<Benchmark>
}

impl BenchmarkBuilder{
    pub fn new(file_path:String)->BenchmarkBuilder{
        BenchmarkBuilder{
            file_path:file_path,
            collect:Vec::new()
        }
    }
    pub fn add(&mut self,a:Benchmark){
        self.collect.push(a);
    }
    pub async fn run(&mut self){

        println!("\nbenchmarking started");

        crate::io::delete_file(&self.file_path).await;

        let mut collect =format!("{}{}{}{}{}{}{}{}{}{}",
            padding(format!("no_of_files")),
            padding(format!("que_size")),
            padding(format!("disk_workers")),
            padding(format!("no_of_writers")),
            padding(format!("no_of_writes")),
            padding(format!("value_size")),
            padding(format!("qued_in")),
            padding(format!("written_in")),
            padding(format!("total_writes")),
            padding(format!("writes/sec")),
        );

        for i in self.collect.iter(){
            
            match run(i.clone()).await{
                Ok(v)=>{
                    // println!("\nresults : {:?}",collect);
                    // println!("\nresults : {:?}",v);
                    if collect.len() == 0{
                        collect += &format!("{}",v);
                    } else {
                        collect += &format!("\n{}",v);
                    }
                },
                Err(_e)=>{
                    println!("!!! failed-run-benchmark : {:?}",_e);
                    return;
                }
            }
        }

        match crate::io::write_new_file(&self.file_path,collect.clone().as_bytes().to_vec()).await{
            Ok(_)=>{
                println!(">>> bechmark complete");
                // println!("{}",collect);
            },
            Err(_)=>{
                println!("!!! failed-write-results");
            }
        }

    }
}

pub async fn run(benchmark:Benchmark)->Result<String,&'static str>{

    println!("\nbechmarking");

    let start_time = Instant::now();

    for file in benchmark.map_files.iter(){
        crate::io::delete_file(file).await;
    }

    println!("que reset complete : {}",benchmark.map_files.len());

    //---------------------------
    //initiate que
    //---------------------------
    let que:Que;
    match Que::new(Config::new(
        benchmark.map_files.clone(),
        benchmark.min_que_size.clone(),
        benchmark.expansion_size.clone(),
        benchmark.no_of_disk_workers.clone(),
    )).await{
        Ok(v)=>{
            que = v;
        },
        Err(e)=>{
            println!("!!! failed-que::new => {:?}",e);
            return Err("failed-init-que");
        }
    }
    let qued_in_time = start_time.elapsed();
    println!("que initiated : {:?}",qued_in_time);

    let mut big_value = vec![];
    let mut last_put = 0;
    loop {
        if big_value.len() == benchmark.write_size.clone(){break;}
        if last_put == 200{last_put = 1;} else {last_put += 1;}
        big_value.push(last_put);
    }

    // println!("{:?}",big_value);
    //1_000_000

    //---------------------------
    //write items to the que
    //---------------------------
    let write_time_final = Instant::now();
    let sleeper = Arc::new(Notify::new());
    let waker = sleeper.clone();
    let total_files = (benchmark.no_of_writers as u64) * benchmark.no_of_writes;
    // let mut failed = 0;
    // let mut success = 0;
    let no_of_spawns = benchmark.no_of_writers.clone();
    let no_of_writes = benchmark.no_of_writes.clone();
    for _nsp in 0..no_of_spawns{
        let que_to_move = que.clone();
        let waker_to_move = waker.clone();
        let hold_big_value = big_value.clone();
        let no_of_writes = no_of_writes.clone();
        tokio::spawn(async move {
            // let write_spawn_time = Instant::now();
            let mut que = que_to_move;
            let mut collect = Vec::new();
            for _n in 0..no_of_writes{
                match que.add(hold_big_value.clone()).await{
                    Ok(mut que_response)=>{
                        collect.push(async move{
                            que_response.check().await
                        });
                        // let _quer_resp = que_response.check().await;
                        // println!("{:?} add resp : {:?}",_n,_quer_resp);
                        // println!(">>> success-que-add {:?}",_n);
                    },
                    Err(_e)=>{
                        println!("!!! failed-que-add : {:?}",_e);
                    }
                }
            }
            join_all(collect).await;
            // for r in join_all(collect).await.iter(){
            //     if !r{failed += 1;} else {success += 1;}
            // }
            // println!("{:?} write_spawn_time : {:?} {:?} {:?}",_nsp,write_spawn_time.elapsed(),failed,success);
            waker_to_move.notify_one();
        });
    }
    for _ in 0..no_of_spawns{
        sleeper.notified().await
    }
    let write_in_time = write_time_final.elapsed();
    let write_in_time_in_secs = write_in_time.as_secs();
    let writes_per_sec = total_files / write_in_time_in_secs;
    println!("write complete : {:?}",write_in_time);

    // pub no_of_writers:u8,
    // pub no_of_writes:u64,
    // pub map_files:Vec<String>,
    // pub write_size:usize,
    // pub min_que_size:u64,
    // pub expansion_size:u64,
    // pub no_of_disk_workers:u8

    let build =format!("{}{}{}{}{}{}{}{}{}{}",
        padding(format!("{}",benchmark.map_files.len())),
        padding(format!("{}MB",(benchmark.min_que_size.clone()/1000000))),
        padding(format!("{}",benchmark.no_of_disk_workers.clone())),
        padding(format!("{}",benchmark.no_of_writers.clone())),
        padding(format!("{}",benchmark.no_of_writes.clone())),
        padding(format!("{}",benchmark.write_size.clone())),
        padding(format!("{:?}",qued_in_time)),
        padding(format!("{:?}",write_in_time)),
        padding(format!("{:?}",total_files)),
        padding(format!("{:?}",writes_per_sec)),
        // padding(format!("{:?}",success)),
        // padding(format!("{:?}",failed)),
    );

    return Ok(build);

    // return Err("no_error");

}

fn padding(s:String)->String{
    let mut h = s;
    let len = h.len();
    let padd_by = 13 - len;
    if padd_by == 0{return h + "| ";}
    for _ in 0..padd_by{
        h += &" ";
    }
    h += "| ";
    return h;
}