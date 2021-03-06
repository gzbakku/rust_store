
use tokio::fs::{File,OpenOptions,remove_file};
use std::path::Path;
use tokio::io::{AsyncWriteExt,AsyncReadExt,AsyncSeekExt};
use std::fs::Metadata;
use std::io::{SeekFrom};
// use crate::workers::{debug_error,debug_message};
// use std::time::Instant;

// const DEBUG:bool = false;
// const ERROR:bool = true;

const MB_1:u64 = 1_000_000;

pub async fn init_map(path:String,min_que_size:u64)->Result<(File,Metadata),&'static str>{

    let build = Path::new(&path);
    if !build.exists() || false{

        //build map with frame size
        let mut file_builder = OpenOptions::new();
        file_builder.write(true)
        .read(true)
        .create(true);
        // .create_new(true);
        let mut file:File;
        match file_builder.open(build).await{
            Ok(f)=>{
                file = f;
            },
            Err(_)=>{
                // debug_error("failed-create_file-init_map-io.rs",ERROR);
                return Err("failed-create_file");
            }
        }

        match expand(&mut file, &min_que_size).await{
            Ok(_)=>{},
            Err(_e)=>{
                println!("!!! failed-expand-new_map : {:?}",_e);
                return Err("failed-expand-new_map");
            }
        }

        match file.metadata().await{
            Ok(v)=>{
                return Ok((file,v));
            },
            Err(_)=>{
                // debug_error("failed-get_metadata-init_map-io.rs",ERROR);
                return Err("failed-get-metadata");
            }
        }

    }

    //map already exists check if map size is less then min que size

    let mut file:File;
    match OpenOptions::new()
    .write(true)
    .read(true)
    // .create(true)
    .open(build)
    .await
    {
        Ok(f)=>{
            file = f;
        },
        Err(_)=>{
            // debug_error("failed-create_file-init_map-io.rs",ERROR);
            return Err("failed-create_file");
        }
    }

    let metadata:Metadata;
    match file.metadata().await{
        Ok(v)=>{
            // return Ok((file,v));
            metadata = v;
        },
        Err(_)=>{
            // debug_error("failed-existing-get_metadata-init_map-io.rs",ERROR);
            return Err("failed-get-metadata");
        }
    }

    // println!("min_que_size : {:?}",min_que_size);

    let file_size = metadata.len();
    if file_size < min_que_size{
        match expand(&mut file, &(min_que_size - file_size)).await{
            Ok(_)=>{
                // return Ok((file,metadata));
            },
            Err(_e)=>{
                // println!("!!! failed-expand-existing_map : {:?}",_e);
                return Err("failed-expand-existing_map");
            }
        }
    }

    match file.metadata().await{
        Ok(metadata)=>{
            // return Ok((file,v));
            return Ok((file,metadata));
        },
        Err(_)=>{
            // debug_error("failed-existing-get_metadata-init_map-io.rs",ERROR);
            return Err("failed-get-metadata");
        }
    }

    // return Ok((file,metadata));

}

pub async fn expand(file:&mut File,size_og:&u64)->Result<(),&'static str>{

    // println!("expanding");

    match file.seek(SeekFrom::End(0)).await{
        Ok(_)=>{},
        Err(_)=>{
            return Err("failed-seek");
        }
    }

    let mut size = size_og.clone();

    // println!("size : {:?}",size);

    let min_expansion_mb:u64;
    if size > (MB_1 * 100){
        min_expansion_mb = 100 * MB_1;//println!("100");
    } else if size > (MB_1 * 50){
        min_expansion_mb = 50 * MB_1;//println!("50");
    } else if size > (MB_1 * 25){
        min_expansion_mb = 25 * MB_1;//println!("25");
    } else if size > (MB_1 * 20){
        min_expansion_mb = 20 * MB_1;//println!("20");
    } else if size > (MB_1 * 15){
        min_expansion_mb = 15 * MB_1;//println!("15");
    } else if size > (MB_1 * 10){
        min_expansion_mb = 10 * MB_1;//println!("10");
    } else if size > (MB_1 * 5){
        min_expansion_mb = 5 * MB_1;//println!("5");
    } else {
        min_expansion_mb = 0;//println!("0");
    }

    // println!("min_expansion_mb : {:?}",min_expansion_mb);
    
    if min_expansion_mb > 0{
        // println!("building min_expansion_mb");
        let mut build = vec![];
        for _ in 0..min_expansion_mb{
            build.push(0);
        }
        // println!("built min_expansion_mb : {:?}",build.len());
        loop{
            if size < min_expansion_mb{break;}
            // println!("expanding min_expansion_mb");
            match file.write_all(&build).await{
                Ok(_v)=>{
                    // println!("expanded min_expansion_mb : {:?}",_v);
                    size -= min_expansion_mb;
                },
                Err(_)=>{
                    return Err("failed-write_to_file");
                }
            }
        }
    }

    // println!("building end");
    let mut build = vec![];
    for _ in 0..size.clone(){
        build.push(0);
    }
    // println!("built end : {:?}",build.len());
    // println!("expanding end");
    match file.write_all(&build).await{
        Ok(_)=>{
            // println!("expanded end");
            return Ok(());
        },
        Err(_)=>{
            return Err("failed-write_to_file");
        }
    }

    // return Err("no_error");

}

pub async fn remove_chunk(file:&mut File,start_at:usize,len:usize)->Result<(),&'static str>{

    match file.seek(SeekFrom::Start(start_at as u64)).await{
        Ok(_)=>{},
        Err(_e)=>{
            return Err("failed-seek");
        }
    }

    let mut collect:Vec<u8> = Vec::with_capacity(len as usize);
    for _ in 0..len{
        collect.push(0);
    }

    match file.write(&collect).await{
        Ok(_)=>{
            return Ok(());
        },
        Err(_)=>{
            return Err("failed-build_frame-create_file");
        }
    }

}

pub async fn write_chunk(file:&mut File,start_at:u64,buffer:Vec<u8>)->Result<(),&'static str>{

    // let hold = Instant::now();
    match file.seek(SeekFrom::Start(start_at)).await{
        Ok(_)=>{},
        Err(_e)=>{
            // debug_error("failed-file_seek-write_chunk-io.rs",ERROR);
            return Err("failed-seek");
        }
    }
    
    match file.write(&buffer).await{
        Ok(_)=>{
            // println!("write in : {:?}",hold.elapsed());
            // debug_message("chunk writen",DEBUG);
            return Ok(());
        },
        Err(_)=>{
            // debug_error("failed-write_all-write_chunk-io.rs",ERROR);
            return Err("failed-read_chunk");
        }
    }

}

pub async fn read_chunk(file:&mut File,buffer:&mut Vec<u8>,start_at:u64,read_len:u64)->Result<usize,&'static str>{

    match file.seek(SeekFrom::Start(start_at)).await{
        Ok(_)=>{},
        Err(_e)=>{
            // debug_error("failed-seek-read_chunk-io.rs",ERROR);
            return Err("failed-seek");
        }
    }

    match file.take(read_len).read_to_end(buffer).await{
        Ok(v)=>{
            return Ok(v);
        },
        Err(_)=>{
            // debug_error("failed-take-read_chunk-io.rs",ERROR);
            return Err("failed-read_chunk");
        }
    }

}

#[allow(dead_code)]
pub async fn read_full(path:String){

    let mut file_builder = OpenOptions::new();
    file_builder
    .write(true)
    .read(true)
    .create(true);
    let mut file:File;
    match file_builder.open(path).await{
        Ok(f)=>{
            file = f;
        },
        Err(_)=>{
            // debug_error("failed-open_file",ERROR);
            return;
        }
    }

    let mut buffer = Vec::new();
    match file.seek(SeekFrom::Start(0)).await{
        Ok(_)=>{},
        Err(_)=>{
            // debug_error("failed-seek_file",ERROR);
            return;
        }
    }

    match file.read_to_end(&mut buffer).await{
        Ok(_)=>{},
        Err(_)=>{
            // debug_error("failed-read_file",ERROR);
            return;
        }
    }

}

#[allow(dead_code)]
pub async fn delete_file(path:&String){
    match remove_file(path).await{
        Ok(_)=>{},
        Err(_e)=>{
            // println!("!!! failed-delete-file : {:?} => {:?}",&path,_e);
        }
    }
}

#[allow(dead_code)]
pub async fn write_new_file(path:&String,buffer:Vec<u8>)->Result<(),&'static str>{

    let mut file:File;
    match File::create(path).await{
        Ok(f)=>{file = f;},
        Err(_)=>{return Err("failed-open_file");}
    }
    
    match file.write(&buffer).await{
        Ok(_)=>{
            return Ok(());
        },
        Err(_)=>{
            return Err("failed-read_chunk");
        }
    }

}

// pub async fn expand_old(file:&mut File,size:&u64)->Result<(),&'static str>{

//     // let hold = Instant::now();
//     match file.seek(SeekFrom::End(0)).await{
//         Ok(_)=>{},
//         Err(_)=>{
//             // debug_error("failed-file_seek-expand-io.rs",ERROR);
//             return Err("failed-seek");
//         }
//     }

//     let mut collect:Vec<u8> = Vec::with_capacity(1000000);
//     for _ in 0..*size{
//         if collect.len() == 1000000{
//             match file.write(&collect).await{
//                 Ok(_)=>{},
//                 Err(_)=>{
//                     // debug_error("failed-write_block-expand-io.rs",ERROR);
//                     return Err("failed-build_frame-create_file");
//                 }
//             }
//             collect.clear();
//         } else {
//             collect.push(0);
//         }
//     }
//     if collect.len() > 0{
//         match file.write(&collect).await{
//             Ok(_)=>{},
//             Err(_)=>{
//                 // debug_error("failed-write_block-last-expand-io.rs",ERROR);
//             }
//         }
//     }

//     // println!("expand in : {:?}",hold.elapsed());

//     return Ok(());

// }