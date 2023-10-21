## Geo Converter

**This is a training project**

But that does not mean it's not working

converts `geo`/`bgeo` geometry formats into some random other formats

### Usage

Input file can be piped in, like 

`cat file.geo | geoconverter -t obj file_out.obj`

Or specified as a file, like

`geoconverter file_in.bgeo file_out.obj`

(if `-t` is not provided - assumed output format is `obj`)


#### Note on bgeo.sc, bgeo.gz

Formats `bgeo.sc`, `bgeo.gz` are simple bgeos, but additionally compressed with c-blosc and gzip correspondingly.

To convert them with this tool you need to first pipe the `bgeo.sc` file through a blosc decompressing tool into geoconverter

#### Expressions

You can run simple expressions over attributes (just float/vector point attributes for now).

Syntax is somewhat inspired by vex, but **it's not vex by any means**.  
There are no functions for now, and the "language" is just interpreted postfix notation, nothing is truly compiled.  
There are also no built-in functions, for now at least.

for example:
* `@P = @P + 0.5*@mask*@N` will add offset along normal `N` to `P` based on mask attribute `mask`. All attributes MUST exist beforehand in the cache file.
* `@P=@P + {0,1,0}*@mask` will offset `P` along vertical axis, multiplied by `mask` attribute
* `@mask=(@P.z+1)/2` will set `mask` to be that value evaluated from z component of `P`

Full command line example would look like this
```shell
geoconverter -t bgeo -e "@P = @P + 0.5*@mask*@N" file_in.bgeo file_out.bgeo
```

### supported output formats:
* [x] obj
* [x] stl
* [x] geo
* [x] bgeo

supporter input format:
* [x] geo
* [x] bgeo

### Embedding transparently into Houdini

If you want to define your own file format, so you can save it transparently through standard nodes like `rop_geometry` or `file`,
you can copy the file `GEOio.json` into your houdini home dir,  
for example to `~/houdini19.5/GEOio.json`

You might need to adapt the json file, replace `geoconverter` with full path to `geoconverter` binary downloaded/built from this repository.  
Or, instead, you can add `geoconverter` binary to `PATH` env variable so you can call it without full path, then you do not need to change `GEOio.json`
