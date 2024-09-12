# What is this?

## N(ew)ED(itor)

Hey kid, you like the unix philosophy? Sure you do! Look at that bashrc! You like pointfree
languages like APL or Uiua? Yeah... Sure...! Well, have I got the editor for you!

Vim uses Verb -> (count)Object syntax Helix/Kakoune uses (count)Object -> Verb VsCode uses...
Nano...

But these are all just pipelines that have been artificially limited to two steps! What if
every bash command was only allowed one `|`!? Madness!

The idea behind Ned is that you should be able to control the build up of your pipelines! You
should be able to preview, rearrange, add, delete, and save the filters and edits you want to
execute!

What this actually means in practice is that you should be able to be in a
buffer, looking at a function that has a parameter you want to change the name
of, and create a filter first to select the function (either by line numbers or
with treesitter), pipe that to a second filter that specifies the name of the
variable, and then hit the equivalent of vim's `c` to change the name. Then you
head to a different function, and instead of having to create that same
pipeline, you just hit the equivalent of `<up>` and edit the filter you used, all
those many years ago at the top of this paragraph, to grab the new parameter in
this new function and change it.

OR

Say you have a long url that you might be using to build, for example, an anki
vocab builder automation tool for learning Dutch and you have a url like this
one:

```
https://www.google.com/search?as_st=y&as_q=praten&as_epq=&as_oq=&as_eq=&imgar=&imgcolor=&imgtype=&cr=countryNL&as_sitesearch=&as_filetype=&tbs=&udm=2
```

that you want to turn into a url creation with the Reqwest Rust crate that looks
like this:

```
    let mut image_search_url: Url = "https://www.google.com/search?".parse()?;
    image_search_url
        .query_pairs_mut()
        .append_pair("as_st", "y")
        .append_pair("as_q", cli.word.as_str())
        .append_pair("as_epq", "")
        .append_pair("as_oq", "")
        .append_pair("as_eq", "")
        .append_pair("imgar", "")
        .append_pair("imgcolor", "")
        .append_pair("imgtype", "")
        .append_pair("cr", "countryNL")
        .append_pair("as_sitesearch", "")
        .append_pair("as_filetype", "")
        .append_pair("tbs", "")
        .append_pair("udm", "2");
```

usually, in plain old vim, I'd write a recursive macro, such as

```
qq
i.append_pair("f=��5i"lr,��5a"A")jI@q
```

which works great, but is nearly the definition of delicate and finicky and if
I, instead, want to ignore the query parameters that have no value, I'm in a bit
of hot water unless I'm feeling up to manually editing the macro buffer with

```
"qp
...change stuff...
V"y
```

or by using [an ancient plugin I made once.](https://github.com/zdcthomas/medit)

Instead, you should be able to make a filter that narrows down a range to two
values on either side of an equals that's also between two `&`. sort of like the
regex `[?&](\w+)=(\w+)&` so that you can use those two "captures" later in the
mutation step of the pipeline, like `.append_pair("\1", "\2")\n`. You can do
this right now in vim regex or vim macros, which is awesome! But with Ned you
should be able to

1. build these super easily with only a few keystrokes
2. edit them during and after creation
3. reuse/reference pipelines in other pipelines
4. all from within a small pipeline editor

And eventually, if things go very well:

- treat pipelines like functions, with parameterized inputs
- publish pipelines, maybe serialized in a very clever spec?
- include lsp completions/code actions in pipelines
- pipeline preview
- user creatable filter types

## Why is this in Neovim?

Because it's way easier to integrate into neovim through the lua spec than it is
to create my own editor! But maybe someday!

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

### Lazy

You don't need to use Lazy as a package manager to use these steps above, I just
use it here as an automatic to include a lua file when I'm in this directory
