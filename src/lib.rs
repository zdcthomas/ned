use nvim_oxi::api::{self, opts::*, types::*, Window};
use nvim_oxi::{print, Dictionary, Error, Function, Result};

// #[nvim_oxi::plugin]
fn api() -> Result<Dictionary> {
    Ok(Dictionary::new())
}

#[cfg(test)]
mod tests {
    use super::*;
}
