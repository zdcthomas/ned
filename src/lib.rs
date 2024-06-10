use lazy_static::lazy_static;
use nvim_oxi::api::types::{CommandNArgs, LogLevel};
use nvim_oxi::conversion::{Error as ConversionError, FromObject, ToObject};
use nvim_oxi::mlua::{Function as LuaFunction, Table};
use nvim_oxi::serde::{Deserializer, Serializer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write as _;
use std::rc::Rc;
use std::sync::Mutex;

use nvim_oxi::{Dictionary, Function, Object};
use serde::{self, Deserialize, Serialize};

use nvim_oxi::api::{self, opts::*, types::CommandArgs};

use crate::temp_key::Mappings;
mod temp_key;

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

fn get_miles(_foo: ()) -> Vec<u32> {
    CACHE_PATTERN
        .lock()
        .unwrap()
        .iter()
        .map(|c| c.miles)
        .collect()
}

#[derive(Debug)]
struct Line {
    index: usize,
    content: nvim_oxi::String,
}

fn lines((): ()) {
    let start = 1;
    let end = 2;
    let foo: Vec<Line> = nvim_oxi::api::Buffer::current()
        .get_lines(start..end, true)
        .unwrap()
        .enumerate()
        .map(|(index, content)| Line { index, content })
        // it'd be cool if this was still an iterator at the end and just a LineRange was returned
        .collect();
    nvim_oxi::print!("{:?}", foo);
}

/// When initialized sets the keymap in nvim
/// when dropped resets to original keymapping
fn ned_start_command(_args: CommandArgs) {
    // nvim_oxi::print!("Entering Ned Mode");

    let mut maps = MAPS.lock().unwrap();
    for (lhs, func_name) in FILTERS.iter() {
        maps.add(
            lhs.to_owned(),
            format!(r#"<CMD>lua require("ned").{}()<CR>"#, func_name).as_str(),
            SetKeymapOpts::builder().build(),
        )
        .unwrap();
    }

    // nvim_oxi::api::set_keymap(Mode::Normal, l, rhs, opts)
    // lines(0, 5);
    // let foo: Result<i32, nvim_oxi::api::Error> = nvim_oxi::api::eval("getchar()");
    // nvim_oxi::print!("{:?}", foo);
}

fn ned_end_command((): ()) {
    nvim_oxi::print!("Exiting Ned Mode");

    // let foo: String = add.to_object().into();

    MAPS.lock().unwrap().clear();

    // nvim_oxi::api::set_keymap(Mode::Normal, l, rhs, opts)
    // lines(0, 5);
    // let foo: Result<i32, nvim_oxi::api::Error> = nvim_oxi::api::eval("getchar()");

    // nvim_oxi::print!("{:?}", foo);
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

fn test_insert((): ()) {
    nvim_oxi::api::notify(
        "Hello there!",
        LogLevel::Warn,
        &NotifyOpts::builder().build(),
    )
    .unwrap();
    // let gcs = nvim_oxi::mlua::lua()
    //     .globals()
    //     .get::<_, Table>("vim")
    //     .unwrap()
    //     .get::<_, Table>("fn")
    //     .unwrap()
    //     .get::<_, LuaFunction>("getcharstr")
    //     .unwrap();
    //
    // let answer: String = gcs.call("").unwrap();
    // nvim_oxi::print!("{}", answer);

    // let mut file = File::create("foo.txt").unwrap();
    // file.write_all(answer.as_bytes()).unwrap();
}

lazy_static! {
    static ref CACHE_PATTERN: Mutex<Vec<Config>> = Mutex::new(vec![Config {
        manufacturer: "Hello!".to_string(),
        miles: 30
    }]);
    static ref MAPS: Mutex<Mappings> = Mutex::new(Mappings::new());
    static ref FILTERS: HashMap<String, String> = HashMap::from([
        ("l".to_owned(), "lines".to_owned()),
        ("i".to_owned(), "test_insert".to_owned()),
        ("<esc>".to_owned(), "close_ned".to_owned()),
    ]);
}

#[nvim_oxi::plugin]
fn ned() -> nvim_oxi::Result<Dictionary> {
    //
    // let compute = Function::from_fn(
    //     |(fun, a, b): (Function<(i32, i32), i32>, i32, i32)| -> Result<i32, nvim_oxi::Error> {
    //         Ok(fun.call((a, b))?)
    //     },
    // );
    // let config: Rc<RefCell<Option<Config>>> = Rc::default();
    // let c = Rc::clone(&config);
    let opts = CreateCommandOpts::builder()
        .desc("Ned entry point")
        .nargs(CommandNArgs::ZeroOrOne)
        .build();

    api::create_user_command("NedStart", ned_start_command, &opts).unwrap();
    // api::create_user_command("NedEnd", ned_end_command, &opts).unwrap();

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
        ("lines", Object::from(Function::from_fn(lines))),
        ("test_insert", Object::from(Function::from_fn(test_insert))),
        (
            "close_ned",
            Object::from(Function::from_fn(ned_end_command)),
        ),
    ]))
}
