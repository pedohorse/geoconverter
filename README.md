## Geo Converter

**This is a training project**

But that does not mean it's not working


### Usage

Input file has to be piped in, like 

`cat file.geo | geoconverter -t obj file_out.obj`

supported output formats:
* [x] obj
* [x] stl

supporter input format:
* [x] geo
* [ ] bgeo

### Embedding transparently into Houdini

If you want to define your own file format, so you can save it transparently through standard nodes like `rop_geometry` or `file`,
you can copy the file `GEOio.json` into your houdini home dir,  
for example to `~/houdini19.5/GEOio.json`

You might need to adapt the json file, replace `geoconverter` with full path to `geoconverter` binary downloaded/built from this repository.  
Or, instead, you can add `geoconverter` binary to `PATH` env variable so you can call it without full path, then you do not need to change `GEOio.json`