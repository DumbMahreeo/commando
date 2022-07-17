# Commando

A fast utility to search which packages provide a specific command on
Arch and Arch based systems.

## Why

Because `pacman -F` was too slow to be set as fallback for a "command
not found" exception in a shell.

## How to use it

### Database creation/update

Right after installing it create your database with:

``` bash
$ commando -u
```

It may take a while depending on your connection speed and your
computer, but you only need to create/update your database the first
time and every once in a while. I'd suggest perhaps once every one or
two months, but it's entirely up to you.

### Database search

Now you're ready to search with `commando`!

Just search your command with:

``` bash
$ commando command-name
```

For example, if I wanted to see which packages provide the `ls` command,
I'd just do:

``` bash
$ commando ls
```

For further usage instructions, please execute:

``` bash
$ commando --help
```

*Note: command search is case sensitive, this means that, for example,
searching for `LS` won't give the same results of `ls`*

## How to install

(AUR support is coming soon)

As of now, you'll need to compile it yourself with:

``` bash
$ cargo build --release
$ strip ./target/release/commando
```

Then you'll have your binary placed in `./target/release/commando`,
simply move it to somewhere in your `$PATH`.
