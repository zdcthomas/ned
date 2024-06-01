use lazy_static::lazy_static;
use nvim_oxi::api::types::CommandNArgs;
use nvim_oxi::conversion::{Error as ConversionError, FromObject, ToObject};
use nvim_oxi::serde::{Deserializer, Serializer};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;

use nvim_oxi::{Dictionary, Function, Object};
use serde::{self, Deserialize, Serialize};

use nvim_oxi::api::{self, opts::*, types::CommandArgs, Window};
use nvim_oxi::print;

// - could hash before and after filter creation to ensure buffer hasn't changed, or maybe swap
//   buffer to readonly while making filters, then turn that off to apply changes
//
//
// create filter                                      ◄┐
// add filter to list                                  │
// apply filter to existing selection for caching     ─┘
//
// Change selection


enum Filter {
    Line { l_index: u32, r_index: u32 },
}

fn multiply((a, b): (i32, i32)) -> i32 {
    a * b
}

fn get_miles(_foo: ()) -> Vec<u32> {
    CACHE_PATTERN
        .lock()
        .unwrap()
        .iter()
        .map(|c| c.miles)
        .collect()
}

fn ned_command(args: CommandArgs) {
    print!("{:?}", args.fargs);
    print!("Yo YO YO YOY OY OIU OHSDFKJHD!");
    let foo = nvim_oxi::api::Buffer::current()
        .get_lines(0..5, true)
        .unwrap();
}

#[derive(Serialize, Deserialize)]
struct Config {
    manufacturer: String,
    miles: u32,
}
impl FromObject for Config {
    fn from_object(obj: Object) -> Result<Self, ConversionError> {
        Config::deserialize(Deserializer::new(obj)).map_err(Into::into)
    }
}

impl ToObject for &Config {
    fn to_object(self) -> Result<Object, ConversionError> {
        self.serialize(Serializer::new()).map_err(Into::into)
    }
}

lazy_static! {
    static ref CACHE_PATTERN: Mutex<Vec<Config>> = Mutex::new(vec![Config {
        manufacturer: "Hello!".to_string(),
        miles: 30
    }]);
}

#[nvim_oxi::plugin]
fn ned() -> nvim_oxi::Result<Dictionary> {
    // let add = Function::from_fn(|(a, b): (i32, i32)| -> Result<i32, nvim_oxi::Error> { Ok(a + b) });
    //
    // let compute = Function::from_fn(
    //     |(fun, a, b): (Function<(i32, i32), i32>, i32, i32)| -> Result<i32, nvim_oxi::Error> {
    //         Ok(fun.call((a, b))?)
    //     },
    // );
    let config: Rc<RefCell<Option<Config>>> = Rc::default();
    let c = Rc::clone(&config);
    let opts = CreateCommandOpts::builder()
        .desc("Ned entry point")
        .nargs(CommandNArgs::ZeroOrOne)
        .build();

    api::create_user_command("Ned", ned_command, &opts);

    Ok(Dictionary::from_iter([
        (
            "drive_all",
            Object::from(Function::from_fn(|args: Option<u32>| {
                for ele in CACHE_PATTERN.lock().unwrap().iter_mut() {
                    ele.miles += args.unwrap_or(5)
                }
            })),
        ),
        ("c", Object::from(Function::from(get_miles))),
        ("multiply", Object::from(Function::from_fn(multiply))), // ("compute", Object::from(compute)),
    ]))
}
