# sheet-shark

[![CI](https://github.com/literalplus/sheet-shark/workflows/CI/badge.svg)](https://github.com/literalplus/sheet-shark/actions)

```
___/|       
\ * ~~~
 ≈≈_ __     Funny CLI fish to help with timesheets 
    \  
```

## Installation

**Important:** Ensure that your `PATH` contains `$HOME/.cargo/bin/` !

```bash
git clone git@github.com:literalplus/sheet-shark.git
cd sheet-shark
./install.sh
```

You can then run via the `sheet-shark` application added to your launcher
(or manually with `gtk-launch sheet-shark`)
(or from the terminal with `sheet-shark`).

## Configuration

You can open the config directory in the calendar view by pressing `Shift+F`.
The data directory is accessible using `F`.

## Development

For the `diesel` CLI, you can use `export DATABASE_URL=~/.local/share/sheet-shark/sharkdb.sqlite`.

If you need more logs run `RUST_LOG=debug cargo run` and check
`~/.local/share/sheet-shark/sheet-shark.log`.
