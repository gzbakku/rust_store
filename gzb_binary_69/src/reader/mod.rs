

mod unhandled;
mod key;
mod value;

pub struct Write{
    pub start:usize,
    pub end:usize
}

#[derive(Debug,Clone)]
pub enum PointerType{
    Data,Corrupt,Empty
}

#[derive(Debug,Clone)]
pub struct Pointer{
    pub pointer_type:PointerType,
    pub start_on_map_cursor:usize,
    pub boundry:((usize,usize),(usize,usize)),//(map(start,end),buffer(start,end))
    pub key:((usize,usize),(usize,usize),Vec<u8>),//(map(start,end),buffer(start,end))
    pub value:((usize,usize),(usize,usize))//(map(start,end),buffer(start,end))
}

impl Pointer{
    pub fn new()->Pointer{
        Pointer{
            pointer_type:PointerType::Corrupt,
            start_on_map_cursor:0,
            boundry:((0,0),(0,0)),
            key:((0,0),(0,0),vec![]),
            value:((0,0),(0,0))
        }
    }
    pub fn clear(&mut self){
        self.pointer_type = PointerType::Empty;
        self.key = ((0,0),(0,0),vec![]);
        self.value = ((0,0),(0,0));
    }
    pub fn point(len:usize,map_cursor:usize,buffer_cursor:usize,pt:PointerType)->Pointer{
        let mut hold = Pointer::new();
        hold.start_on_map_cursor = map_cursor;
        hold.boundry.0.0 = map_cursor;
        hold.boundry.0.1 = map_cursor+len-1;
        hold.boundry.1.0 = buffer_cursor;
        hold.boundry.1.1 = buffer_cursor+len-1;
        hold.pointer_type = pt;
        return hold;
    }
    pub fn data(
        len:usize,
        map_cursor:usize,
        start:usize,end:usize,
        key_start:usize,key_end:usize,key:Vec<u8>,
        value_start:usize,value_end:usize
    )->Pointer{
        let mut hold = Pointer::new();
        //pointer
        hold.pointer_type = PointerType::Data;
        hold.start_on_map_cursor = map_cursor;
        //boundry
        hold.boundry.0.0 = map_cursor;
        hold.boundry.0.1 = map_cursor+len-1;
        hold.boundry.1.0 = start;
        hold.boundry.1.1 = end;
        //key
        let key_len = key_end - key_start;
        hold.key.0.0 = map_cursor + key_start;
        hold.key.0.1 = map_cursor + key_start + key_len;
        hold.key.1.0 = key_start;
        hold.key.1.1 = key_end;
        hold.key.2 = key;
        //value
        let value_len = value_end - value_start;
        hold.value.0.0 = map_cursor + value_start;
        hold.value.0.1 = map_cursor + value_start + value_len - 1;
        hold.value.1.0 = value_start;
        hold.value.1.1 = value_end;
        // println!("p : {:?}",hold);
        return hold;
    }
}

#[derive(Debug)]
pub enum ReaderFunc{
    Flag,Key,Value
}

#[derive(Debug)]
pub struct Reader{
    pub no_flag_in_buffer:bool,
    pub map:Vec<Pointer>,
    pub build_map:bool,
    pub to_find:Vec<Vec<u8>>,
    pub find_values:bool,
    pub get_values_controll:bool,
    pub values:Vec<(Vec<u8>,Vec<u8>)>,
    pub map_cursor:usize,
    pub buffer_cursor:usize,
    pub buffer:Vec<u8>,
    pub func:ReaderFunc,
    pub flag:(bool,usize,usize),//found,start,end
    pub key:(bool,u8,u64,(usize,usize),Vec<u8>),//found,byte_len,len,(start,end)
    pub value:(bool,u8,u64,(usize,usize)),//found,byte_len,len,(start,end)
    pub end:(bool,usize,usize)//foudn start,end
}

impl Reader{
    pub fn get_values(&mut self,b:bool){
        self.get_values_controll = b;
    }
    pub fn find_key(&mut self,i:Vec<u8>){
        if !self.find_values{self.find_values = true;}
        self.to_find.push(i);
    }
    pub fn new()->Reader{
        Reader{
            no_flag_in_buffer:false,
            map:vec![],
            build_map:true,
            to_find:Vec::new(),
            find_values:false,
            get_values_controll:false,
            values:Vec::new(),
            map_cursor:0,
            buffer_cursor:0,
            buffer:vec![],
            func:ReaderFunc::Flag,
            flag:(false,0,0),
            key:(false,0,0,(0,0),vec![]),
            value:(false,0,0,(0,0)),
            end:(false,0,0)
        }
    }
    pub fn with_capacity(map_capacity:usize,buffer_capacity:usize)->Reader{
        Reader{
            no_flag_in_buffer:false,
            map:Vec::with_capacity(map_capacity),
            build_map:true,
            to_find:Vec::new(),
            find_values:false,
            get_values_controll:false,
            values:Vec::new(),
            map_cursor:0,
            buffer_cursor:0,
            buffer:Vec::with_capacity(buffer_capacity),
            func:ReaderFunc::Flag,
            flag:(false,0,0),
            key:(false,0,0,(0,0),vec![]),
            value:(false,0,0,(0,0)),
            end:(false,0,0)
        }
    }
    pub fn fill(&mut self,key:Vec<u8>,value_size:usize)->Result<Write,&'static str>{

        let total_len = 39+key.len()+value_size;
        //zero alligned
        let key_start_at = 18;
        let key_end_at = 18+key.len()-1;
        let value_start_at = 36+key.len();
        let value_end_at = 36+key.len()+value_size-1;

        //find a empty space
        let mut index:usize = 0;
        let mut filler_space_found = false;
        let mut empty_space_len:usize = 0;
        for pointer in self.map.iter(){
            match pointer.pointer_type{
                PointerType::Empty=>{
                    let len = pointer.boundry.0.1 - pointer.boundry.0.0 + 1;
                    empty_space_len = len;
                    if len >= total_len{
                        filler_space_found = true;
                        break;
                    }
                },
                _=>{}
            }
            index += 1;
        }

        if !filler_space_found{
            return Err("full");
        }

        //replace the empty space
        if empty_space_len == total_len{
            //edit pointer to be of data type
            match self.map.get_mut(index){
                Some(pointer)=>{
                    pointer.pointer_type = PointerType::Data;
                    pointer.key.0.0 = pointer.boundry.0.0 + key_start_at;
                    pointer.key.0.1 = pointer.boundry.0.0 + key_end_at;
                    pointer.key.2 = key;
                    pointer.value.0.0 = pointer.boundry.0.0 + value_start_at;
                    pointer.value.0.1 = pointer.boundry.0.0 + value_end_at;
                    return Ok(Write{
                        start:pointer.boundry.0.0,
                        end:pointer.boundry.0.1
                    });
                },
                None=>{
                    return Err("failed-get-empty_space");
                }
            }
        } else

        if empty_space_len > total_len{
            let map_cursor:usize;
            match self.map.get_mut(index){
                Some(pointer)=>{
                    map_cursor = pointer.boundry.0.0;
                    pointer.boundry.0.0 = pointer.boundry.0.0 + total_len;
                },
                None=>{
                    return Err("failed-get-empty_space");
                }
            }
            //add new pointer
            self.map.insert(index,Pointer::data(
                total_len,
                map_cursor,
                map_cursor,
                map_cursor+total_len-1, 
                map_cursor+key_start_at, 
                map_cursor+key_end_at, 
                key, 
                value_start_at, 
                value_end_at
            ));
            return Ok(Write{
                start:map_cursor,
                end:map_cursor+total_len-1
            });
        } else {
            return Err("unhandled-error");
        }

    }
    pub fn clear(&mut self,point:usize)->Result<(),&'static str>{

        if point >= self.map.len(){
            return Err("point is out of scope");
        }

        let mut opp:u8 = 0;
        //combine previous to current
        if point > 0{
            match self.map.get(point-1){
                Some(v)=>{
                    match v.pointer_type{
                        PointerType::Empty=>{
                            opp = 1;//combine with previous
                        },
                        _=>{}
                    }
                },
                None=>{return Err("failed-read_previous_pointer");}
            }
        }
        //combine current to next
        if point < self.map.len()-1{
            match self.map.get(point+1){
                Some(v)=>{
                    match v.pointer_type{
                        PointerType::Empty=>{
                            if opp == 1{
                                opp = 3;//combine with previous and next
                            } else {
                                opp = 2;//combine with next
                            }
                        },
                        _=>{}
                    }
                },
                None=>{return Err("failed-read_previous_pointer");}
            }
        }

        if opp == 0{//only clear data
            match self.map.get_mut(point){
                Some(v)=>{
                    v.clear();
                    return Ok(());
                },
                None=>{
                    return Err("point lock failed");
                }
            }
        } else//only clear data
        if opp == 1{//combine previos and current
            let map_boundry_end:usize;
            let buffer_boundry_end:usize;
            match self.map.get(point){
                Some(current)=>{
                    map_boundry_end = current.boundry.0.1;
                    buffer_boundry_end = current.boundry.1.1;
                },
                None=>{
                    return Err("point lock failed");
                }
            }
            match self.map.get_mut(point-1){
                Some(previous)=>{
                    previous.boundry.0.1 = map_boundry_end;
                    previous.boundry.1.1 = buffer_boundry_end;
                },
                None=>{
                    return Err("point lock failed");
                }
            }
            self.map.remove(point);
        } else //combine previos and current
        if opp == 2{//combine current and next
            let map_boundry_start:usize;
            let buffer_boundry_start:usize;
            match self.map.get(point){
                Some(current)=>{
                    map_boundry_start = current.boundry.0.0;
                    buffer_boundry_start = current.boundry.1.0;
                },
                None=>{
                    return Err("point lock failed");
                }
            }
            match self.map.get_mut(point+1){
                Some(previous)=>{
                    previous.boundry.0.0 = map_boundry_start;
                    previous.boundry.1.0 = buffer_boundry_start;
                },
                None=>{
                    return Err("point lock failed");
                }
            }
            self.map.remove(point);
        } else //combine current and next
        if opp == 3{//combine current, previous and next
            let map_boundry_start:usize;
            let buffer_boundry_start:usize;
            let map_boundry_end:usize;
            let buffer_boundry_end:usize;
            //next
            match self.map.get(point+1){
                Some(next)=>{
                    map_boundry_end = next.boundry.0.1;
                    buffer_boundry_end = next.boundry.1.1;
                },
                None=>{
                    return Err("point lock failed");
                }
            }
            //previous
            match self.map.get(point-1){
                Some(previous)=>{
                    map_boundry_start = previous.boundry.0.0;
                    buffer_boundry_start = previous.boundry.1.0;
                },
                None=>{
                    return Err("point lock failed");
                }
            }
            //current
            match self.map.get_mut(point){
                Some(current)=>{
                    current.clear();
                    current.boundry.0.0 = map_boundry_start;
                    current.boundry.0.1 = map_boundry_end;
                    current.boundry.1.0 = buffer_boundry_start;
                    current.boundry.1.1 = buffer_boundry_end;
                },
                None=>{
                    return Err("point lock failed");
                }
            }
            self.map.remove(point+1);
            self.map.remove(point-1);
        }//combine current, previous and next
        
        return Ok(());

    }//clear
    pub fn find(&self,key:&Vec<u8>)->Result<usize,()>{
        let mut index = 0;
        for i in self.map.iter(){
            match i.pointer_type{
                PointerType::Data=>{
                    if &i.key.2 == key{
                        return Ok(index);
                    }
                },
                _=>{}
            }
            index += 1;
        }
        return Err(());
    }
    pub fn map(&mut self,buffer:&mut Vec<u8>)->Result<(),&'static str>{
        loop{
            match read(self,buffer){
                Ok(v)=>{
                    //important line
                    let pointer_len = v.boundry.0.1-v.boundry.0.0+1;
                    let new_buffer = self.buffer.split_off(v.boundry.1.1+1);
                    let old_buffer = &self.buffer;
                    self.buffer_cursor = 0;
                    self.map_cursor += pointer_len;
                    if self.find_values && !self.get_values_controll{
                        for i in self.to_find.iter(){
                            if i == &v.key.2{
                                let mut collect_value:Vec<u8> = vec![];
                                for i in v.value.1.0..v.value.1.1{
                                    collect_value.push(old_buffer[i]);
                                }
                                self.values.push((v.key.2.clone(),collect_value));
                            }
                        }
                    }
                    if self.get_values_controll{
                        let mut collect_value:Vec<u8> = vec![];
                        for i in v.value.1.0..v.value.1.1{
                            collect_value.push(old_buffer[i]);
                        }
                        self.values.push((v.key.2.clone(),collect_value));
                    }
                    if self.build_map{
                        self.map.push(v);
                    }
                    self.buffer = new_buffer;
                },
                Err(e)=>{
                    return Err(e);
                }
            }
        }
        // return Ok(());
    }
    pub fn end(&mut self)->Result<(),&'static str>{
        if self.buffer.len() > 0{
            loop{
                match read(self,&mut vec![]){
                    Ok(v)=>{
                        let pointer_len = v.boundry.0.1-v.boundry.0.0+1;
                        self.buffer = self.buffer.split_off(v.boundry.1.1+1);
                        self.buffer_cursor = 0;
                        self.map_cursor += pointer_len;
                        self.map.push(v);
                    },
                    Err(_)=>{
                        break;
                    }
                }
            }
        }
        if self.buffer.len() > 0{
            unhandled::init(self,self.buffer.len());
        }
        return Ok(());
    }
    pub fn reset(&mut self){
        //flag
        self.flag.0 = false;
        self.flag.1 = 0;
        self.flag.2 = 0;
        //key
        self.key.0 = false;
        self.key.1 = 0;
        self.key.2 = 0;
        self.key.3 = (0,0);
        //value
        self.value.0 = false;
        self.value.1 = 0;
        self.value.2 = 0;
        self.value.3 =(0,0);
        //end
        self.end.0 = false;
        self.end.1 = 0;
        self.end.2 = 0;
    }
    pub fn flush(&mut self){
        unhandled::init(self,self.buffer.len());
        self.reset();
    }
    pub fn expand(&mut self,num_of_bytes:usize)->Result<(),&'static str>{

        //check if last is empty
        let map_len = self.map.len()-1;
        match self.map.get_mut(map_len){
            Some(v)=>{
                match v.pointer_type{
                    PointerType::Empty=>{
                        v.boundry.0.1 += num_of_bytes;
                        return Ok(());
                    },
                    _=>{}
                }
            },
            None=>{
                return Err("failed-get-tail");
            }
        }

        //make new empty pointer
        let mut build = Pointer::new();
        build.boundry.0.0 = self.map_cursor;
        build.boundry.0.1 = self.map_cursor+num_of_bytes-1;
        build.pointer_type = PointerType::Empty;
        self.map.push(build);
        return Ok(());

    }
}

fn read(reader:&mut Reader,buffer:&mut Vec<u8>)->Result<Pointer,&'static str>{

    if buffer.len() > 0{
        reader.buffer.append(buffer);
    }

    // buff_print(&reader.buffer);

    // println!("reading at : {:?}",reader.buffer_cursor);
        
    //find flag
    if reader.flag.0 == false {
        match vector_in_vector(&reader.buffer,&vec![0,1,0],reader.buffer_cursor){
            Ok(l)=>{
                // println!("l : {:?}",l);
                reader.no_flag_in_buffer = false;
                if l.0 > 0{
                    unhandled::init(reader,l.0);
                }
                // println!("unhandled completed");
                if true {
                    reader.flag.0 = true;
                    reader.flag.1 = reader.buffer_cursor;
                    reader.flag.2 = reader.buffer_cursor+2;
                    reader.buffer_cursor = 3;
                }
            },
            Err(_)=>{
                // println!("no flag found : {:?}",reader.buffer);
                if reader.buffer.len() > 6{
                    reader.buffer_cursor = reader.buffer.len() - 5;
                } else {
                    reader.buffer_cursor = reader.buffer.len();
                }
                return Err("not_found-flag");
            }
        }
    }

    if true {
        if reader.flag.0 == true && reader.key.0 == false{
            match key::init(reader){
                Ok(_)=>{},
                Err(_)=>{
                    return Err("not_found-key");
                }
            }
        }
        if reader.flag.0 == true && reader.key.0 == true && reader.value.0 == false{
            match value::init(reader){
                Ok(_)=>{},
                Err(_)=>{
                    return Err("not_found-value");
                }
            }
        }
        if 
            reader.flag.0 == true && 
            reader.key.0 == true && 
            reader.value.0 == true &&
            reader.end.0 == true
        {
            let build = Pointer::data(
                reader.end.2-reader.flag.1+1,
                reader.map_cursor,
                reader.flag.1,reader.end.2,
                reader.key.3.0, reader.key.3.1, reader.key.4.split_off(0),
                reader.value.3.0, reader.value.3.1,
            );
            reader.reset();
            return Ok(build);
        }
    }

    return Err("no_error");

}

fn vector_in_vector(v1:&Vec<u8>,v2:&Vec<u8>,start_cursor:usize)->Result<(usize,usize),()>{

    // println!("check vector : {:?} {:?}",start_cursor,v1.len());

    let mut v2_cursor:usize = 0;
    let mut found_some = false;
    let mut final_found = false;
    let mut v1_index = start_cursor;
    let mut start_index = 0;
    let mut end_index = 0;

    for n in start_cursor..v1.len(){
        let i = &v1[n];
        if !found_some{
            if i == &v2[0]{
                found_some = true;
                v2_cursor = 1;
                start_index = v1_index;
            }
        } else {
            if i != &v2[v2_cursor]{
                if i == &v2[0]{
                    found_some = true;
                    v2_cursor = 1;
                    start_index = v1_index;
                } else {
                    found_some = false;
                    v2_cursor = 0;
                }
            } else {
                if v2_cursor == v2.len() - 1{
                    final_found = true;
                    end_index = v1_index;
                    break;
                } else {
                    v2_cursor += 1;
                }
            }
        }
        v1_index += 1;
    }

    if final_found{
        Ok((start_index,end_index))
    } else {
        Err(())
    }

}