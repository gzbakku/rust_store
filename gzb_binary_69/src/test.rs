use std::time::Instant;

pub fn init(){

    let allotment_time = Instant::now();
    let mut pool = Vec::new();
    let mut last_put = 0;
    loop {
        if pool.len() == 100000000{break;}
        if last_put == 200{last_put = 1;} else {last_put += 1;}
        pool.push(last_put);
    }
    println!("allotment_time : {:?}",allotment_time.elapsed());
    println!("pool len : {:?}",pool.len());


    let split_time = Instant::now();

    loop{
        if pool.len() > 33000000{
            pool = pool.split_off(33000000);
            println!("pool len : {:?}",pool.len());
        } else {
            break;
        }
    }
    println!("split_time : {:?}",split_time.elapsed());

}