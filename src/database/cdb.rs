use std::{fs::File, io::{Write, Read, BufReader, BufRead, stdout}, path::Path, process::exit, collections::HashMap};

use byteorder::ReadBytesExt;

const MAGIC: [u8; 8] = [0x7F, 0x43, 0x4F, 0x4D, 0x4D, 0x44, 0x42, 0x7F];
const CDB_VERSION: u32 = 1;

macro_rules! write_unwrap {
    ($($write:expr),+) => {
        $(
            if let Err(e) = $write {
                eprintln!("[FATAL] couldn't write to CDB file.\nError details: {e}");
                exit(1);
            }
        )*
    };
}

pub fn create_cdb<P: AsRef<Path>>(mut map_data: HashMap<String, Vec<String>>, path: P) {
    let mut data = Vec::with_capacity(map_data.len());

    for (command, bins) in map_data.iter_mut() {
        bins.sort_unstable();
        data.push((command, bins));
    }

    data.sort_unstable_by_key(|(k, _)| k.len());

    let path = path.as_ref();
    let mut file = File::create(path).unwrap_or_else(|e| {
        eprintln!("[FATAL] couldn't create CDB file at path '{e}'.\nError details: {e}");
        exit(1);
    });

    write_unwrap!(
        file.write_all(&MAGIC),                     // Write magic_number
        file.write_all(&CDB_VERSION.to_le_bytes()), // Write version_number
        file.sync_all()                             // Sync
    );

    for (command, packages) in data {
        write_unwrap!(
            file.write_all(&[command.len().clamp(0, u8::MAX as usize) as u8]), // Write command_length
            file.write_all(command.get(..255).unwrap_or(command).as_bytes()) // Write command_name
        );

        let mut package = packages.join("\n");
        package.push('\n');

        if let Some(e) = package.get(..(u32::MAX - 1) as usize) {
            package = e.to_string();
            package.push('\n');
        }

        write_unwrap!(
            file.write_all(&(package.len().clamp(0, u32::MAX as usize) as u32).to_le_bytes()), // Write package_length
            file.write_all(package.as_bytes()) // Write package_name(s) + newline
        );

        write_unwrap!(file.write(&[0x3])); // Write ETX
    }
}

pub fn search_in_cdb<S: AsRef<str>, P: AsRef<Path>>(command: S, path: P, verbose: bool) {
    let command = command.as_ref();

    macro_rules! read_unwrap {
        ($($write:expr, $error:literal);+) => {
            $(
                match $write {
                    Err(e) => {
                        eprintln!("[FATAL] couldn't read from CDB file while {}.\nError details: {e}", $error);
                        exit(1);
                    },
                    Ok(v) => v
                }
            )*
        };

        ($($write:expr),+) => {
            $(
                match $write {
                    Err(e) => {
                        eprintln!("[FATAL] couldn't read from CDB file.\nError details: {e}");
                        exit(1);
                    },
                    Ok(v) => v
                }
            )*
        };
    }

    let mut file = BufReader::new(File::open(path).unwrap_or_else(|e| {
        eprintln!("[FATAL]: couldn't open cdb file.\nError details: {e}");
        exit(1);
    }));

    let mut magic_buf = [0u8,0,0,0,0,0,0,0];
    read_unwrap!(file.read_exact(&mut magic_buf), "reading magic number");
    if magic_buf != MAGIC {
        eprintln!("[FATAL]: unrecognized file format (wrong magic number). Expected CDB file.");
        exit(1);
    }
    let mut version_number = [0u8,0,0,0];
    read_unwrap!(file.read_exact(&mut version_number), "reading version number");
    let version_number = u32::from_le_bytes(version_number);

    #[allow(unreachable_patterns)] // CDB_VERSION might be set to zero in dev builds.
    match version_number {
        0 => eprintln!("[WARNING]: reading unstable CDB file, proper behaviour is not guaranteed."),
        CDB_VERSION => {},
        version => {
            eprintln!("[FATAL]: wrong CDB version, please update your database.\nExpected CDB version {CDB_VERSION}, got {version}");
            exit(1);
        }
    }

    #[cfg(debug_assertions)]
    let mut current_address: u32 = 12;

    macro_rules! increase_address {
        () => {
            #[cfg(debug_assertions)]
            #[allow(unused_assignments)]
            {
                current_address += 1;
            }
        };

        ($increment:expr) => {
            #[cfg(debug_assertions)]
            #[allow(unused_assignments)]
            {
                current_address += $increment;
            }
        };
    }

    loop {
        let command_length = match file.read_u8() {
            Ok(l) => l,
            _ => {
                println!("Command `{}` not found in commando database", command);
                exit(127);
            },
        };

        increase_address!();

        macro_rules! skip_package {
            () => {

                let mut package_length = [0u8,0,0,0];
                read_unwrap!(file.read_exact(&mut package_length), "reading package length for skip");

                increase_address!(4);

                let package_length = u32::from_le_bytes(package_length);

                file.seek_relative((package_length as i64) + 1).unwrap();

                increase_address!(package_length+1);


            };
        }

        if command_length > command.len() as u8 {
            println!("Command `{}` not found in commando database", command);
            exit(127);
        }

        if command_length != command.len() as u8 {
            file.seek_relative(command_length.into()).unwrap();

            increase_address!(command_length as u32);

            skip_package!();

            continue;
        }

        let mut command_name = Vec::with_capacity(command.len());
        #[allow(clippy::uninit_vec)]
        unsafe { command_name.set_len(command.len()) }

        #[cfg(debug_assertions)]
        eprintln!("Reading command name at address: {current_address:#X}");

        read_unwrap!(file.read_exact(&mut command_name), "reading command name");
        let command_name = String::from_utf8(command_name).unwrap();

        #[cfg(debug_assertions)]
        eprintln!("Command name: '{command_name}'");

        if command_name != command {
            skip_package!();

            continue;
        }

        #[cfg(debug_assertions)] {
            eprintln!("Command found");
            let mut package_length = [0u8,0,0,0];
            read_unwrap!(file.read_exact(&mut package_length), "reading package length for command");
            let package_length = u32::from_le_bytes(package_length);
            increase_address!(package_length);
        }
        
        #[cfg(not(debug_assertions))]
        file.seek_relative(4).unwrap();

        increase_address!(4);

        let mut packages = Vec::with_capacity(20);

        let mut stdout = stdout();
        read_unwrap!(file.read_until(0x03, &mut packages), "reading package_name(s)");

        let mut packages = packages.as_slice();

        if packages.ends_with(&[0x03]) {
            packages = &packages[..packages.len()-1];
        }

        if verbose {
            print!("Found command `{command}` in the following packages:");
            let packages = packages.split(|e| e == &b'\n');
            for package in packages {
                if !package.is_empty() || package == b"\n" {
                    stdout.write_all(b"\n\n\t").unwrap();
                    stdout.write_all(package).unwrap();
                }
            }
            stdout.write_all(b"\n").unwrap();
        } else {
            stdout.write_all(&packages).unwrap();
        }

        stdout.flush().unwrap();

        break;
    }
}
