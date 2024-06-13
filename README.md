# Hi!!

## Running locally

Neovim looks for a directory called `lua` with a file called
`<library-name>.[so|dylib]` so when you build, run

```
mkdir lua
cargo build
cp ./target/debug/libned.so ./lua/ned.so
```

Then, inside of neovim, set the runtime path to include ned, by running
`vim.opt.runtimepath:append("<path to ned on your system")`

You can run this either in EX mode (by hitting `:` while inside neovim, but
remember to prepend `lua` to the above command)
OR
In any lua file you already load when neovim starts.

Then, inside your running Neovim instance, you can run `require("ned")`.

### Pro Tip!

You can run AND print lua code from EX mode by prefixing with `:=`
e.g

```
:=require("ned")
```
