use std::collections::{HashMap};
use std::io::{Read};
use std::mem;

use Config;
use Value;
use Buffer;
use Field;
// use Proto;
use RpResult;
use ErrorKind;
use {TYPE_NIL, TYPE_U8, TYPE_I8, TYPE_U16, TYPE_I16, TYPE_U32, TYPE_I32, TYPE_FLOAT, TYPE_STR, TYPE_RAW, TYPE_MAP,};
use {TYPE_AU8, TYPE_AI8, TYPE_AU16, TYPE_AI16, TYPE_AU32, TYPE_AI32, TYPE_AFLOAT,TYPE_ASTR, TYPE_ARAW, TYPE_AMAP,};
use {get_name_by_type, get_type_by_name};

pub fn decode_number(buffer : &mut Buffer, pattern : u16) -> RpResult<Value>  {
    match pattern {
        TYPE_U8 => {
            let data: &mut [u8; 2] = &mut [0, 0];
            try_read!(buffer.read(data), data.len());
            Ok(Value::from(data[0]))
        }
        TYPE_I8 => {
            let data: &mut [u8; 2] = &mut [0, 0];
            try_read!(buffer.read(data), data.len());
            Ok(Value::from(data[0] as i8))
        }
        TYPE_U16 => {
            let data: &mut [u8; 2] = &mut [0, 0];
            try_read!(buffer.read(data), data.len());
            let val = unsafe { mem::transmute::<[u8;2], u16>(*data) };
            Ok(Value::from(u16::from_le(val)))
        }
        TYPE_I16 => {
            let data: &mut [u8; 2] = &mut [0, 0];
            try_read!(buffer.read(data), data.len());
            let val = unsafe { mem::transmute::<[u8;2], i16>(*data) };
            Ok(Value::from(i16::from_le(val)))
        }
        TYPE_U32 => {
            let data: &mut [u8; 4] = &mut [0, 0, 0, 0];
            try_read!(buffer.read(data), data.len());
            let val = unsafe { mem::transmute::<[u8;4], u32>(*data) };
            Ok(Value::from(u32::from_le(val)))
        }
        TYPE_I32 => {
            let data: &mut [u8; 4] = &mut [0, 0, 0, 0];
            try_read!(buffer.read(data), data.len());
            let val = unsafe { mem::transmute::<[u8;4], i32>(*data) };
            Ok(Value::from(i32::from_le(val)))
        }
        TYPE_FLOAT => {
            let data: &mut [u8; 4] = &mut [0, 0, 0, 0];
            try_read!(buffer.read(data), data.len());
            let val = unsafe { mem::transmute::<[u8;4], i32>(*data) };
            Ok(Value::from(val as f32 / 1000.0))
        }
        _ => {
            unreachable!("not other numbers");
        }
    }
}


pub fn decode_str_raw(buffer : &mut Buffer, pattern : u16) -> RpResult<Value>  {
    match pattern {
        TYPE_STR => {
            let len : u16 = try!(decode_number(buffer, TYPE_U16)).into();
            let mut rv = vec![0; len as usize];
            try_read!(buffer.read(&mut rv[..]), len as usize);
            let val = String::from_utf8(rv);
            if val.is_err() {
                fail!((ErrorKind::StringFormatError, "string format error"));
            }
            Ok(Value::from(val.ok().unwrap()))
        }
        TYPE_RAW => {
            let len : u16 = try!(decode_number(buffer, TYPE_U16)).into();
            let mut rv = vec![0; len as usize];
            try_read!(buffer.read(&mut rv[..]), len as usize);
            Ok(Value::from(rv))
        }
        _ => {
            unreachable!("not other str");
        }
    }
}

pub fn decode_map(buffer : &mut Buffer, config : &Config) -> RpResult<Value>  {
    let mut map = HashMap::<String, Value>::new();
    loop {
        let field = try!(read_field(buffer));
        if field.is_nil_type() {
            return Ok(Value::from(map));
        }
        let sub_value = try!(decode_field(buffer, config));
        let name = config.get_field_index_name(&field.index);
        if name.is_none() {
            continue;
        }
        let name = name.map(|s| s.clone()).unwrap();
        map.insert(name, sub_value);
    }
}

pub fn read_field(buffer : &mut Buffer) -> RpResult<Field> {
    let index = try!(decode_number(buffer, TYPE_U16)).into();
    let pattern = try!(decode_number(buffer, TYPE_U16)).into();
    Ok(Field { index : index, pattern : get_name_by_type(pattern).to_string()})
}

fn decode_by_field(buffer : &mut Buffer, config : &Config, field : &Field) -> RpResult<Value> {
    let t = get_type_by_name(&*field.pattern);
    match t {
        TYPE_U8 | TYPE_I8 | TYPE_U16 | TYPE_I16 | TYPE_U32 | TYPE_I32 | TYPE_FLOAT => {
            decode_number(buffer, t)
        }
        TYPE_STR | TYPE_RAW => {
            decode_str_raw(buffer, t)
        }
        TYPE_MAP => {
            decode_map(buffer, config)
        }
        TYPE_AU8 => {
            decode_array!(decode_field(buffer, config), Value::AU8, Value::U8)
        }
        TYPE_AI8 => {
            decode_array!(decode_field(buffer, config), Value::AI8, Value::I8)
        }
        TYPE_AU16 => {
            decode_array!(decode_field(buffer, config), Value::AU16, Value::U16)
        }
        TYPE_AI16 => {
            decode_array!(decode_field(buffer, config), Value::AI16, Value::I16)
        }
        TYPE_AU32 => {
            decode_array!(decode_field(buffer, config), Value::AU32, Value::U32)
        }
        TYPE_AI32 => {
            decode_array!(decode_field(buffer, config), Value::AI32, Value::I32)
        }
        TYPE_AFLOAT => {
            decode_array!(decode_field(buffer, config), Value::AFloat, Value::Float)
        }
        TYPE_ASTR => {
            decode_array!(decode_field(buffer, config), Value::AStr, Value::Str)
        }
        TYPE_ARAW => {
            decode_array!(decode_field(buffer, config), Value::ARaw, Value::Raw)
        }
        TYPE_AMAP => {
            decode_array!(decode_field(buffer, config), Value::AMap, Value::Map)
        }
        TYPE_NIL => {
            Ok(Value::Nil)
        }
        _ => {
            fail!((ErrorKind::TypeNotMatchError, "must match type"))
        }
    }
}


pub fn decode_field(buffer : &mut Buffer, config : &Config) -> RpResult<Value> {
    let field = try!(read_field(buffer));
    if field.is_nil_type() {
        return Ok(Value::Nil);
    }
    decode_by_field(buffer, config, &field)
}

pub fn decode_proto(buffer : &mut Buffer, config : &Config) -> RpResult<(String, Vec<Value>)> {
    let name = try!(decode_str_raw(buffer, TYPE_STR)).into();
    //TODO check proto choose to transfer
    let mut value : Vec<Value> = vec![];
    loop {
        let sub_value = try!(decode_field(buffer, config));
        match sub_value {
            Value::Nil => break,
            _ => (),
        }
        value.push(sub_value);
    }
    let proto = config.get_proto_by_name(&name);
    match proto {
        Some(val) => {
            if val.args.len() != value.len() {
                fail!((ErrorKind::TypeNotMatchError, "must match type"));
            }
        },
        _ => {
            fail!((ErrorKind::TypeNotMatchError, "must match type"));
        }
    }
    Ok((name, value))
}
