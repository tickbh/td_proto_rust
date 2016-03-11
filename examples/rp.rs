extern crate td_rp;

use std::fs::File;
use std::io::prelude::*;
use std::io;

fn read_all(name : &str) -> io::Result<String> {
    let mut f = try!(File::open(name));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}

fn main()
{
    println!("welcome to tickdream rust protocol");
    let value = td_rp::Value::from(td_rp::get_type_by_name("u8") as u8);
    println!("value = {:?}", value);

    let config = td_rp::Config::new("{ \"name\" : { \"index\" :    1, \"pattern\" : \"string\" } }",
        "{\"cmd_achieve_op\"        : { \"index\" :    1, \"args\" : [ \"map\" ] }}");
    println!("config is {:?}", config);

    println!("read_all = {:?}", read_all("doc/protocol.txt"));

    match read_all("doc/protocol.txt") {
        Ok(ref string) => {
            let config = td_rp::Config::new_by_full_str(&string);
            println!("config is {:?}", config);
        },
        _ => (),
    };

}
