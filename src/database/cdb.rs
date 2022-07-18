use std::{
    collections::HashMap,
    fs::File,
    io::{stdout, BufRead, BufReader, Read, Write},
    path::Path,
    process::exit,
};

use byteorder::ReadBytesExt;

use crate::error::CommandoError;

const MAGIC: [u8; 8] = [0x7F, 0x43, 0x4F, 0x4D, 0x4D, 0x44, 0x42, 0x7F];
const CDB_VERSION: u32 = 1;

// macro_rules! write_unwrap {
//     ($($write:expr),+) => {
//         $(
//             if let Err(e) = $write {
//                 eprintln!("[FATAL] couldn't write to CDB file.\nError details: {e}");
//                 exit(1);
//             }
//         )*
//     };
// }

pub fn create_cdb<P: AsRef<Path>>(
    mut map_data: HashMap<String, Vec<String>>,
    path: P,
) -> Result<(), CommandoError> {
    let mut data = Vec::with_capacity(map_data.len());

    for (command, bins) in map_data.iter_mut() {
        bins.sort_unstable();
        data.push((command, bins));
    }

    data.sort_unstable_by_key(|(k, _)| k.len());

    let path = path.as_ref();
    let mut file = File::create(path)
        .map_err(|err| CommandoError::CdbCreation(path.to_str().unwrap().to_owned(), err))?;

    file.write_all(&MAGIC)?; // Write magic_number
    file.write_all(&CDB_VERSION.to_le_bytes())?; // Write version_number
    file.sync_all()?; // Sync

    for (command, packages) in data {
        // write_unwrap!(
        file.write_all(&[command.len().clamp(0, u8::MAX as usize) as u8])?; // Write command_length
        file.write_all(command.get(..255).unwrap_or(command).as_bytes())?; // Write command_name
                                                                           // );

        let mut package = packages.join("\n");
        package.push('\n');

        if let Some(e) = package.get(..(u32::MAX - 1) as usize) {
            package = e.to_string();
            package.push('\n');
        }

        file.write_all(&(package.len().clamp(0, u32::MAX as usize) as u32).to_le_bytes())?; // Write package_length
        file.write_all(package.as_bytes())?; // Write package_name(s) + newline
        file.write_all(&[0x3])?; // Write ETX
    }

    Ok(())
}

pub fn search_in_cdb<S: AsRef<str>, P: AsRef<Path>>(
    command: S,
    path: P,
) -> Result<(), CommandoError> {
    let command = command.as_ref();
    let verbose = false;

    let mut file = BufReader::new(File::open(path).map_err(CommandoError::CdbOpen)?);

    let mut magic_buf = [0u8; 8];
    file.read_exact(&mut magic_buf)?;
    match magic_buf {
        MAGIC => Ok(()),
        _ => Err(CommandoError::BadMagic),
    }?;

    let mut version_number = [0u8, 0, 0, 0];
    file.read_exact(&mut version_number)?;
    let version_number = u32::from_le_bytes(version_number);

    #[allow(unreachable_patterns)] // CDB_VERSION might be set to zero in dev builds.
    match version_number {
        0 => log::warn!("Reading unstable CDB file, proper behaviour is not guaranteed"),
        CDB_VERSION => {}
        version => {
            return Err(CommandoError::BadVersion {
                expected: CDB_VERSION,
                got: version,
            });
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
            }
        };

        increase_address!();

        macro_rules! skip_package {
            () => {
                let mut package_length = [0u8, 0, 0, 0];
                file.read_exact(&mut package_length)?;
                increase_address!(4);

                let package_length = u32::from_le_bytes(package_length);

                file.seek_relative((package_length as i64) + 1)?;

                increase_address!(package_length + 1);
            };
        }

        if command_length > command.len() as u8 {
            println!("Command `{}` not found in commando database", command);
            exit(127);
        }

        if command_length != command.len() as u8 {
            file.seek_relative(command_length.into())?;

            increase_address!(command_length as u32);

            skip_package!();

            continue;
        }

        let mut command_name = Vec::with_capacity(command.len());
        #[allow(clippy::uninit_vec)]
        unsafe {
            command_name.set_len(command.len())
        }

        log::debug!("Reading command name at address: {current_address:#X}");

        file.read_exact(&mut command_name)?;
        let command_name = String::from_utf8(command_name).unwrap();

        log::debug!("Command name: '{command_name}'");

        if command_name != command {
            skip_package!();

            continue;
        }

        #[cfg(debug_assertions)]
        {
            log::debug!("Command found");
            let mut package_length = [0u8, 0, 0, 0];
            file.read_exact(&mut package_length)?;

            let package_length = u32::from_le_bytes(package_length);
            increase_address!(package_length);
        }

        #[cfg(not(debug_assertions))]
        file.seek_relative(4)?;

        increase_address!(4);

        let mut packages = Vec::with_capacity(20);

        let mut stdout = stdout();
        file.read_until(0x03, &mut packages)?;

        let mut packages = packages.as_slice();
        if packages.ends_with(&[0x03]) {
            packages = &packages[..packages.len() - 1];
        }

        if verbose {
            print!("Found command `{command}` in the following packages:");
            let packages = packages.split(|e| e == &b'\n');
            for package in packages {
                if !package.is_empty() || package == b"\n" {
                    stdout.write_all(b"\n\n\t")?;
                    stdout.write_all(package)?;
                }
            }
            stdout.write_all(b"\n")?;
        } else {
            stdout.write_all(packages)?;
        }

        stdout.flush()?;

        break;
    }

    Ok(())
}
