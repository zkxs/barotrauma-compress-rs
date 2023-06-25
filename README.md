# Barotrauma Compress

A simple CLI utility to compress and decompress Barotrauma save files using fully cross-platform native code.

## Installation

**Manual:** download an artifact from the
[latest release](https://github.com/zkxs/barotrauma-compress-rs/releases/latest)

**Cargo**: `cargo install barotrauma-compress`

## Usage

Whether to compress or decompress will be chosen automatically based on what `<INPUT>` is: files will be decompressed,
and directories will be compressed.

```
Usage: barotrauma-compress <INPUT>

Arguments:
  <INPUT>  input file or directory.

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Quick Guide to Save Editing

1.  Get a .save file from `%localappdata%\Daedalic Entertainment GmbH\Barotrauma\Multiplayer`
2.  **Don't forget to back up your save somewhere in case you blunder!**
3.  Run `barotrauma-compress` on the .save file
4.  It will create a folder next to the save file. Look inside and copy the filename of the .sub file, minus the 
    extension. For example, given `My Sub Name.sub` you should now have `My Sub Name` in your clipboard.
5.  Create a new empty submarine in the sub editor, paste the sub name from your clipboard, and save. This will generate
    a new folder at `C:\Steam\steamapps\common\Barotrauma\LocalMods\My Sub Name` that contains a `My Sub Name.sub` and a
    `filelist.xml` file. This whole step was an exercise to generate that `filelist.xml` with the correct contents.
6.  Close the submarine editor.
7.  Copy the .sub file from your decompressed save over the
    `C:\Steam\steamapps\common\Barotrauma\LocalMods\My Sub Name\My Sub Name.sub` file.
8.  Reopen the sub in the submarine editor. You should see your campaign sub with all its contents. Make your edits, and
    save.
9.  Copy the edited `C:\Steam\steamapps\common\Barotrauma\LocalMods\My Sub Name\My Sub Name.sub` file back into your
    decompressed save, and use Barotrauma Save Decompressor to recompress the directory.
10. If all went well you can now load that campaign up and it'll work properly.

### Removing the Hull Gunk

You may notice weird gunk on your walls that you cannot select. You don't *need* to remove this, but if it bothers you
here's how. It's some extra metadata in the hulls that the sub editor doesn't know how to deal with: specifically in the
xml, the `<Hull>` entries have a `backgroundsections` attribute that defines the gunk. If you just select all your
hulls and move them a bit, then Ctrl+Z them back it nukes all the background gunk. This saves you some effort over
gunzipping the .sub file and editing the XML directly.

## Building from Source

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Clone the project
3. `cargo build --release`

## License

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License.
See [LICENSE](LICENSE) for the full license text.

## Credits

This project was inspired by
[Jlobblet/Barotrauma-Save-Decompressor](https://github.com/Jlobblet/Barotrauma-Save-Decompressor), which is a GUI that
does the same thing as this project. I wanted to make a CLI version that doesn't depend on .NET, and thus this project
was born.
