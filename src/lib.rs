use lazy_static::lazy_static;
use nvim_oxi::api::types::{CommandNArgs, KeymapInfos, Mode};
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

#[derive(Debug)]
struct Line {
    index: usize,
    content: nvim_oxi::String,
}

// Range type, gauranteed contiguous?

fn lines(start: usize, end: usize) -> Vec<Line> {
    nvim_oxi::api::Buffer::current()
        .get_lines(start..end, true)
        .unwrap()
        .enumerate()
        .map(|(index, content)| Line { index, content })
        // it'd be cool if this was still an iterator at the end and just a LineRange was returned
        .collect()
}

/// gets a keymap based on the mode and lhs.
/// doesn't account for buffer vs global and is currently implicit based on the call context
fn get_mapping(mode: Mode, lhs: String) -> Option<api::types::KeymapInfos> {
    nvim_oxi::api::get_keymap(mode).find(|keymap| keymap.lhs == lhs)
}

mod temp_key {
    use nvim_oxi::api::{opts::SetKeymapOpts, types::KeymapInfos};

    struct TempKeyBind {
        lhs: String,
        original_keybind: KeymapInfos,
    }

    /// starts a new keymap in the buffer currently in
    pub fn new(lhs: String, rhs: String, original_keybind: KeymapInfos) {}

    impl Drop for TempKeyBind {
        fn drop(&mut self) {
            let mut builder = SetKeymapOpts::builder();
            builder.noremap(self.original_keybind.noremap);
            if let Some(callback) = self.original_keybind.callback.clone() {
                builder.callback(callback);
            }
            builder.expr(self.original_keybind.expr);
            builder.nowait(self.original_keybind.nowait);
            builder.script(self.original_keybind.script);
            builder.silent(self.original_keybind.silent);
            if self.original_keybind.buffer {
                // TODO: <08-06-24, zdcthomas> this is wrong, it's not neccesarily the current
                // buffer I don't think...
                // Solution:
                // Get buffer of original buffer keymap
                // But I don't see how... Might not be a thing...

                nvim_oxi::api::Buffer::current().set_keymap(
                    self.original_keybind.mode,
                    &self.original_keybind.lhs,
                    self.original_keybind
                        .rhs
                        .clone()
                        .unwrap_or_default()
                        .as_str(),
                    &builder.build(),
                )
            } else {
                nvim_oxi::api::set_keymap(
                    self.original_keybind.mode,
                    &self.original_keybind.lhs,
                    self.original_keybind
                        .rhs
                        .clone()
                        .unwrap_or_default()
                        .as_str(),
                    &builder.build(),
                )
            }
            .unwrap()
        }
    }
}
/// When initialized sets the keymap in nvim
/// when dropped resets to original keymapping

fn ned_command(args: CommandArgs) {
    print!("{:?}", args.fargs);
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
