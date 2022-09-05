# Commando

A fast utility to search which packages provide a specific command on
Arch and Arch based systems.

## Why

Because `pacman -F` was too slow to be set as fallback for a "command
not found" exception in a shell.

## How to use it

### Database creation/update

Right after installing it, create your database with:

``` bash
$ commando -u

# for verbose output (recommended outside of scripts)
$ commando -vu
```

or you could try the `-a` flag, if want to include AUR packages (trough
the use of chaotic aur repos).

``` bash
$ commando -ua

# for verbose output (recommended outside of scripts)
$ commando -vua
```

It may take a while depending on your connection speed and your
computer, but you only need to create/update your database the first
time and every once in a while. I'd suggest perhaps once every one or
two months, but it's entirely up to you.

Also do note that (in case you were using the AUR flag) chaotic aur
repos might take a while to update, so don't worry if you can't
instantly find some new software. Do mind though that AUR updates way
more frequently than normal repos, so you might consider updating your
database more often.

### Database search

Now you're ready to search with `commando`!

Just search your command with:

``` bash
$ commando command-name
```

For example, if I'd want to see which packages provide the `ls` command,
I would just do:

``` bash
$ commando ls

# for verbose output (recommended outside of scripts)
$ commando -v ls
```

For further usage instructions, please execute:

``` bash
$ commando --help
```

*Note: command search is case sensitive, this means that, for example,
searching for `LS` won't give the same results as `ls`*

## How to install

### AUR

You can install commando directly from the Arch User Repository.

You can find the binary package here:
<https://aur.archlinux.org/packages/commando-bin>

Or use any aur helper such as `paru` and `yay` to install it.

``` bash
# With paru
$ paru -S commando-bin

# With yay
$ yay -S commando-bin
```

There is also an AUR package to built commando yourself:
<https://aur.archlinux.org/packages/commando>

### Cargo

If you have cargo installed and your `$PATH` is set up properly you can
use

``` bash
$ cargo install arch-commando
```

to download, build and install `commando`

### Building

You can compile it by cloning this repo and then executing:

``` bash
$ cargo build --release
```

Then you'll have your binary placed in `./target/release/commando`,
simply move it to somewhere in your `$PATH`.

## Credits

Thanks to [BRA1L0R](https://github.com/BRA1L0R) for the refactor.
