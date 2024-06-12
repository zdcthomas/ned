use lazy_static::lazy_static;
use nvim_oxi::api::types::{CommandNArgs, LogLevel};
use nvim_oxi::conversion::{Error as ConversionError, FromObject, ToObject};
use nvim_oxi::serde::{Deserializer, Serializer};
use std::collections::HashMap;
use std::sync::Mutex;

use nvim_oxi::{Dictionary, Function, Object};
use serde::{self, Deserialize, Serialize};

use nvim_oxi::api::{self, opts::*, types::CommandArgs};

use crate::temp_key::Mappings;
mod temp_key;

mod range;

lazy_static! {
    static ref CACHE_PATTERN: Mutex<Vec<Config>> = Mutex::new(vec![Config {
        manufacturer: "Hello!".to_string(),
        miles: 30
    }]);
    static ref MAPS: Mutex<Mappings> = Mutex::new(Mappings::new());
    static ref FILTER_MAPPINGS: HashMap<String, fn(())> = HashMap::from([
        ("l".to_owned(), lines as fn(())),
        ("i".to_owned(), test_insert as fn(())),
        ("<esc>".to_owned(), ned_end_command as fn(())),
    ]);
    static ref ACTIVE_FILTERS: Vec<Box<dyn FilterCreator + Send + Sync>> = vec![];
}

/// N(ew)ED(itor)
///
/// Hey kid, you like the unix philosophy? Sure you do! Look at that bashrc! You like pointfree
/// languages like APL or Uiua? Yeah... Sure...! Well, have I got the editor for you!
///
/// Vim uses Verb -> (count)Object syntax Helix/Kakoune uses (count)Object -> Verb VsCode uses...
/// Nano...
///
/// But these are all just pipelines that have been artificially limited to two steps! What if
/// every bash command was only allowed one `|`!? Madness!
///
/// The idea behind Ned is that you should be able to control the build up of your pipelines! You
/// should be able to preview, rearrange, add, delete, and save the filters and edits you want to
/// execute!
///
/// WARNING: This has no functionality right now! Don't use it!
#[nvim_oxi::plugin]
fn ned() -> nvim_oxi::Result<Dictionary> {
    nvim_oxi::print!("Hello rust meetup!@@");
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
        // ("c", Object::from(Function::from(get_miles))),
        // ("lines", Object::from(Function::from_fn(lines))),
        // ("test_insert", Object::from(Function::from_fn(test_insert))),
        // (
        //     "close_ned",
        //     Object::from(Function::from_fn(ned_end_command)),
        // ),
    ]))
}

/*

What might a filter look like

If we have the text:

>>>

1  use lazy_static::lazy_static;
2  use nvim_oxi::api::types::{CommandNArgs, LogLevel};
3  use nvim_oxi::conversion::{Error as ConversionError, FromObject, ToObject};
4  use nvim_oxi::mlua::{Function as LuaFunction, Table};
5  use nvim_oxi::serde::{Deserializer, Serializer};
6  use std::cell::RefCell;
7  use std::collections::HashMap;
8  use std::fs::File;
9  use std::io::Write as _;
10 use std::rc::Rc;
11 use std::sync::Mutex;

<<<

I think the ranges are just a vec of pairs like:
Point {x, y} ... Point {a, b} where a >= x && b >= y

So the process looks like:

enter Ned mode
define a filter, like:
    hit l (line filter)
        type "5<CR>" for the start
        type "9<CR>" for the end

    Ned waits for next input in filter-make mode until <CR>(?) is hit
    and then edit mode is begun

    Edit mode might just have one exit point allowed

So through keymaps we want to build up a list of filters that we can pass the current reference to

// TODO: <10-06-24, zdcthomas> How to edit

// TODO: <10-06-24, zdcthomas> do we pass the actual buffer or some handler or some secret third
// thing


-----

I have a bunch of functions that create abstract filters

FilterCreator
│
▼
Filter::Line{2..7}
Filter::StartsWith{txt: '('}
Filter::StartsWith{txt: '('}
│
│   Which then become a pipeline getting the whole buffer fed in
│
▼








*/

trait FilterCreator {
    // TODO: <10-06-24, zdcthomas> this needs to get the current set of ranges I think.
    fn create(&self) -> Vec<Filter>;
}

enum Filter {
    Line { x: usize, y: usize },
    Between { start: char, end: char },
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

fn get_input(p: &str) -> Result<String, nvim_oxi::api::Error> {
    nvim_oxi::api::eval(format!(r#"input("{p}")"#).as_str())
}

fn lines((): ()) {
    let start: usize = get_input("start>").unwrap().parse().unwrap();
    let end = get_input("end>").unwrap().parse().unwrap();
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
    for (lhs, callback) in FILTER_MAPPINGS.iter() {
        maps.add(
            lhs.to_owned(),
            "",
            SetKeymapOpts::builder().callback(callback).build(),
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

type FilterFunc = fn(());
// struct Filter {}
