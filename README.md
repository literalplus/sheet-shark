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

Locate the config directory using:

```
$ cargo run -- --version
sheet-shark 0.1.0- (2025-09-21)

Config directory: /home/lit/.config/sheet-shark <---
Data directory: /home/lit/.local/share/sheet-shark
```

In that directory, create a file `config.yaml` that defines your projects, like this:

```yaml
default_project_key: "M" # empty project cells will use this project
projects:
  M: # This is what you enter in the table
    internal_name: "My main project" # This is used for data export to the internal time booking tool
    jira_url: "https://your-customer.atlassian.net/" # optional
```

## Development

For the `diesel` CLI, you can use `export DATABASE_URL=~/.local/share/sheet-shark/sharkdb.sqlite`.
