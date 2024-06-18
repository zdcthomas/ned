use anyhow::{anyhow, bail, Result};
use lazy_static::lazy_static;
use nvim_oxi::api::types::{
    CommandNArgs, LogLevel, WindowAnchor, WindowBorder, WindowConfig, WindowRelativeTo, WindowStyle,
};

use nvim_oxi::api::Buffer;
use nvim_oxi::conversion::{Error as ConversionError, FromObject, ToObject};
use nvim_oxi::serde::{Deserializer, Serializer};
use std::collections::HashMap;
use std::default;
use std::sync::{Arc, Mutex};

use nvim_oxi::{lua, Array, Dictionary, Function, Object};
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
    static ref FILTER_CREATOR_MAPPINGS: HashMap<String, FilterFunc> = HashMap::from([
        ("l".to_owned(), lines as FilterFunc),
        ("c".to_owned(), containing as FilterFunc),
        ("i".to_owned(), test_insert as FilterFunc),
        ("p".to_owned(), show_filter_list as FilterFunc),
        // ("<esc>".to_owned(), leave),
    ]);
    // eventually, will this be like a hashmap for filterlists?
    static ref ACTIVE_FILTERS: Mutex<FilterList> = Mutex::new(FilterList::default());
}

fn containing(_: ()) -> FilterRet {
    Ok(Filter::Containing {
        char: get_input::<String>("char>")?,
    })
}

fn show_filter_list(_: ()) -> FilterRet {
    let mut buf = nvim_oxi::api::create_buf(false, true)?;
    let filters = ACTIVE_FILTERS.try_lock().unwrap();
    let lines = filters.inner.iter().map(|f| format!("{:?}", f));

    buf.set_lines(0..lines.len(), true, lines)?;
    let mut win = nvim_oxi::api::open_win(
        &buf,
        false,
        &WindowConfig::builder()
            .relative(WindowRelativeTo::Cursor)
            .width(50)
            .height(50)
            .col(10)
            .row(10)
            .border(WindowBorder::Double)
            .anchor(WindowAnchor::SouthEast)
            .style(WindowStyle::Minimal)
            .build(),
    )?;
    // TODO: <17-06-24, zdcthomas> create autocommands that change filter list on text save

    Ok(Filter::None)
}

// TODO: <17-06-24, zdcthomas> BIG QUESTION: when do I actually run the filters/edits? Another way
// to ask this is to ask, what is the source of truth for the filter list? I guess it's the filter
// list struct. Maybe that should then also hold the whole pipeline?...

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct FilterList {
    inner: Vec<Filter>,
    buffer: Option<Buffer>,
}

impl FilterList {
    // fn new(buffer: Buffer) -> Self {
    //     Self {
    //         inner: vec![],
    //         buffer,
    //     }
    // }

    fn add(&mut self, filter: Filter) {
        self.inner.push(filter)
    }

    // fn len(&self) -> usize {
    //     self.inner.len()
    // }
}
impl From<FilterList> for Object {
    fn from(value: FilterList) -> Self {
        Object::from(Array::from_iter(value.inner))
    }
}

impl From<Filter> for Object {
    fn from(value: Filter) -> Self {
        Dictionary::from_iter(match value {
            Filter::Line { x, y } => [
                ("x", x.to_string()),
                ("y", y.to_string()),
                ("type", "Line".to_owned()),
            ],
            Filter::Between { start, end } => [
                ("left_char", start.to_string()),
                ("right_char", end.to_string()),
                ("type", "Between`".to_owned()),
            ],
            Filter::None => todo!(),
            Filter::Containing { char } => todo!(),
        })
        .into()
    }
}
impl lua::Pushable for FilterList {
    unsafe fn push(self, lstate: *mut lua::ffi::lua_State) -> Result<std::ffi::c_int, lua::Error> {
        self.to_object()
            .map_err(lua::Error::push_error_from_err::<Self, _>)?
            .push(lstate)
    }
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

    api::create_user_command("NedStart", ned_start_command, &opts)?;
    // api::create_user_command("NedEnd", ned_end_command, &opts).unwrap();

    Ok(Dictionary::from_iter([
        (
            "filter_list",
            Object::from(Function::from_fn(|args: ()| {
                format!("{:?}", ACTIVE_FILTERS.lock().unwrap())
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

// trait FilterCreator {
//     // TODO: <10-06-24, zdcthomas> this needs to get the current set of ranges I think.
//     fn create(&self) -> Vec<Filter>;
// }

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Filter {
    // TODO: <17-06-24, zdcthomas> add arity maybe?
    Line { x: usize, y: usize },
    Between { start: char, end: char },
    Containing { char: String },
    None,
}

#[derive(Debug)]
struct Line {
    index: usize,
    content: nvim_oxi::String,
}

fn get_input<T>(p: &str) -> Result<T, nvim_oxi::api::Error>
where
    T: FromObject,
{
    nvim_oxi::api::eval::<T>(format!(r#"input("{p}")"#).as_str())
}

fn lines((): ()) -> FilterRet {
    let start: usize = get_input::<String>("start>")?.parse()?;
    // Ok(s) => s,
    // Err(err) => bail!("Failed to parse input"),
    // };,
    let end: usize = get_input::<String>("end>")?.parse()?;
    let foo: Vec<Line> = nvim_oxi::api::Buffer::current()
        .get_lines(start..end, true)
        .unwrap()
        .enumerate()
        .map(|(index, content)| Line { index, content })
        // it'd be cool if this was still an iterator at the end and just a LineRange was returned
        .collect();
    nvim_oxi::print!("{:?}", foo);
    Ok(Filter::Line { x: start, y: end })
}
pub fn info(msg: impl AsRef<str>) {
    // println!("XXXXXX: {}", msg.as_ref());
    // nvim_oxi::api::notify(msg.as_ref(), LogLevel::Info, &NotifyOpts::default());
}

/// When initialized sets the keymap in nvim
/// when dropped resets to original keymapping
fn ned_start_command(_args: CommandArgs) {
    info("I'm going ned");
    let mut buffer = Buffer::current();
    info(format!("buffer: {:?}", buffer));

    let mut maps = MAPS.lock().unwrap();
    for (lhs, callback) in FILTER_CREATOR_MAPPINGS.iter() {
        maps.add(
            lhs.to_owned(),
            "",
            SetKeymapOpts::builder()
                .callback(|_: ()| {
                    match callback(()) {
                        Ok(filter) => {
                            ACTIVE_FILTERS.lock().unwrap().add(filter);
                            let lines = nvim_oxi::api::Buffer::current().get_lines(
                                0..nvim_oxi::api::Buffer::current().line_count().unwrap(),
                                false,
                            );

                            // TODO: <17-06-24, zdcthomas> run the filters
                        }
                        Err(err) => {
                            nvim_oxi::api::notify(
                                format!("{:?}", err).as_str(),
                                LogLevel::Error,
                                &NotifyOpts::builder().build(),
                            )
                            .unwrap();
                        }
                    };
                })
                .build(),
            &mut buffer,
        )
        .unwrap();
    }
}

fn leave((): ()) -> FilterRet {
    nvim_oxi::print!("Exiting Ned Mode");
    MAPS.lock().unwrap().clear();
    Ok(Filter::None)
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

fn test_insert((): ()) -> FilterRet {
    nvim_oxi::api::notify(
        "Hello there!",
        LogLevel::Warn,
        &NotifyOpts::builder().build(),
    )
    .unwrap();
    Ok(Filter::None)
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
type FilterRet = anyhow::Result<Filter>;
type FilterFunc = fn(()) -> FilterRet;
// struct Filter {}
