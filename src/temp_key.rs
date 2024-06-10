use std::collections::{HashMap, HashSet};

use nvim_oxi::{
    api::{
        opts::SetKeymapOpts,
        types::{KeymapInfos, Mode},
    },
    Result,
};

pub struct Mappings {
    // TODO: <09-06-24, zdcthomas> inner should probably just be a HashSet to not duplication the
    // lhs
    // mappings: HashMap<String, TempKeyBind>,
    mappings: HashSet<TempKeyBind>,
}

impl Mappings {
    pub fn new() -> Self {
        Self {
            mappings: HashSet::new(),
        }
    }
    pub fn add(&mut self, lhs: String, rhs: &str, opts: SetKeymapOpts) -> Result<()> {
        // remove first to drop
        // self.mappings.remove(&);

        self.mappings.insert(TempKeyBind::new(lhs, rhs, opts)?);
        Ok(())
    }

    /// Clears the mappings, dropping them, and restoring the original mappings
    pub fn clear(&mut self) {
        for mapping in self.mappings.drain() {
            nvim_oxi::print!("Removing {}", mapping.lhs);
            mapping.remove();
        }
    }
}

/// gets a keymap based on the mode and lhs.
/// doesn't account for buffer vs global and is currently implicit based on the call context
pub fn get_mapping(mode: Mode, lhs: &str) -> Option<nvim_oxi::api::types::KeymapInfos> {
    nvim_oxi::api::get_keymap(mode).find(|keymap| keymap.lhs.as_str() == lhs)
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct TempKeyBind {
    lhs: String,
    // opts: SetKeymapOpts,
    original_keybind: Option<KeymapInfos>,
}

impl TempKeyBind {
    fn remove(self) {
        nvim_oxi::print!("{:?}", self.original_keybind);

        let mut buf = nvim_oxi::api::Buffer::current();
        let _ = buf.del_keymap(Mode::Normal, self.lhs.as_str());
        let Some(original_keybind) = self.original_keybind.clone() else {
            return;
        };
        nvim_oxi::print!("ok");
        nvim_oxi::print!("Removing keymapping for {}", original_keybind.lhs);
        let mut builder = SetKeymapOpts::builder();
        builder.noremap(original_keybind.noremap);
        if let Some(callback) = original_keybind.callback.clone() {
            builder.callback(callback);
        }
        builder.expr(original_keybind.expr);
        builder.nowait(original_keybind.nowait);
        builder.script(original_keybind.script);
        builder.silent(original_keybind.silent);
        if original_keybind.buffer {
            nvim_oxi::print!("local");
            // TODO: <08-06-24, zdcthomas> this is wrong, it's not neccesarily the current
            // buffer I don't think...
            // Solution:
            // Get buffer of original buffer keymap
            // But I don't see how... Might not be a thing...

            buf.set_keymap(
                original_keybind.mode,
                &original_keybind.lhs,
                original_keybind.rhs.clone().unwrap_or_default().as_str(),
                &builder.build(),
            )
        } else {
            nvim_oxi::print!("global");
            nvim_oxi::api::set_keymap(
                original_keybind.mode,
                &original_keybind.lhs,
                original_keybind.rhs.clone().unwrap_or_default().as_str(),
                &builder.build(),
            )
        }
        .unwrap()
    }
    fn new(lhs: String, rhs: &str, opts: SetKeymapOpts) -> Result<Self> {
        let original_keybind = get_mapping(Mode::Normal, lhs.as_str());

        match nvim_oxi::api::Buffer::current().set_keymap(Mode::Normal, &lhs, rhs, &opts) {
            Ok(()) => Ok(Self {
                lhs,
                original_keybind,
            }),
            Err(err) => Err(err.into()),
        }
    }
}
