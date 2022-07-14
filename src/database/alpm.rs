use std::{
    collections::HashMap,
    fs::{create_dir_all, read_dir, File},
    io::{BufRead, BufReader, Write, stderr},
    path::Path,
    process::{exit, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use lazy_regex::regex;

//use compress_tools::{uncompress_archive, Ownership};


/// Extracts libalpm file database into `path`
pub fn extract_alpm_db<P: AsRef<Path>>(path: P, data: Vec<(String, Vec<u8>)>) {
    let path = path.as_ref().to_path_buf();

    //let files = read_dir(path.join("sync")).unwrap_or_else(|e| {
    //    eprintln!(
    //        "[FATAL]: couldn't read directory '{}'.\nError details: {e}",
    //        path.display()
    //    );
    //    exit(1);
    //});

    let mut handles = Vec::with_capacity(3);
    let extract_path = Arc::new(path);
    for file in data {
        let extract_path = extract_path.clone();

        handles.push(thread::spawn(move || {
            let extract_path = extract_path.join(file.0.clone());
            if let Err(e) = create_dir_all(&*extract_path) {
                eprintln!("[FATAL] couldn't create extraction directory at path '{}'.\nError details: '{e}'", extract_path.display());
                exit(1);
            }

            // Good enough until this gets solved
            // https://github.com/OSSystems/compress-tools-rs/issues/85
            let mut tar = match Command::new("bsdtar")
                .arg("x")
                .arg("--directory").arg(extract_path)
                .stderr(Stdio::piped()).stdout(Stdio::inherit()).stdin(Stdio::piped())
                .spawn() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("[FATAL]: couldn't execute bsdtar command.\nError details: '{e}'");
                        exit(1);
                    },
                };

            //let reader = BufReader::new(file.1);
            let mut write_failed = false;
            if let Err(e) = tar.stdin.take().unwrap().write_all(&file.1) {
                eprintln!("[FATAL]: couldn't send file '{}' to bsdtar command.\nError details: {e}", file.0);
                write_failed = true;
            }

            let output = tar.wait_with_output().unwrap();
            if !output.status.success() {
                if let Ok(error) = String::from_utf8(output.stderr) {
                    eprintln!("[BEGIN TAR ERROR]\n{error}\n[END TAR ERROR]");
                }
                eprintln!("\n[FATAL]: bsdtar command errored");
                exit(1)
            }

            if write_failed {
                exit(1);
            }

            //if let Err(e) = uncompress_archive(
            //    File::open(file.path()).unwrap(),
            //    &extract_path.join(file.file_name()),
            //    Ownership::Ignore,
            //) {
            //    println!("{e:?}");
            //}
        }));
    }

    for h in handles {
        if let Err(_) = h.join() {
            eprintln!("[FATAL]: thread errored in extract_alpm_db");
            exit(1);
        }
    }
}

fn read_package_name<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref().join("desc");

    let mut file = BufReader::new(File::open(path).unwrap_or_else(|e| {
        eprintln!("[FATAL]: couldn't open 'desc' file for reading.\nError details: {e}");
        exit(1);
    }));

    let mut buf = String::with_capacity(6);
    while file.read_line(&mut buf).unwrap_or_else(|e| {
        eprintln!("[FATAL]: couldn't read lines from 'desc' file.\nError details: {e}");
        exit(1);
    }) != 0
    {
        if buf.trim_end() == "%NAME%" {
            buf.clear();
            file.read_line(&mut buf).unwrap_or_else(|e| {
                eprintln!("[FATAL]: couldn't read lines from 'desc' file.\nError details: {e}");
                exit(1);
            });
            return buf.trim_end().to_string();
        }
        buf.clear();
    }

    eprintln!("[FATAL]: couldn't find package name in 'desc' file");
    exit(1);
}

fn read_package_bins<P: AsRef<Path>>(path: P) -> Vec<String> {
    let re = regex!(r"/bin/([^\s/.]+)");

    let path = path.as_ref().join("files");

    let mut file = BufReader::new(File::open(path).unwrap_or_else(|e| {
        eprintln!("[FATAL]: couldn't open 'files' file for reading.\nError details: {e}");
        exit(1);
    }));

    let mut buf = String::new();
    let mut commands = Vec::new();
    while file.read_line(&mut buf).unwrap_or_else(|e| {
        eprintln!("[FATAL]: couldn't read lines from 'files' file.\nError details: {e}");
        exit(1);
    }) != 0
    {
        if let Some(caps) = re.captures(&buf) {
            let bin_name = (&caps[1]).to_string();
            if !buf.contains("node_modules") && !commands.contains(&bin_name) {
                commands.push(bin_name);
            }
        }
        buf.clear();
    }

    commands
}

pub fn parse_alpm_db<P: AsRef<Path>>(path: P) -> HashMap<String, Vec<String>> {
    let parsed_data = Arc::new(Mutex::new(Some(HashMap::<String, Vec<String>>::new())));

    let path = path.as_ref().to_path_buf();
    let mut handles = Vec::with_capacity(3);
    for repo_dir in read_dir(path).unwrap_or_else(|e| {
        eprintln!("[FATAL]: couldn't read the extracted directory.\nError details: {e}");
        exit(1);
    }) {
        let repo_dir = repo_dir.unwrap_or_else(|e| {
            eprintln!(
                "[FATAL]: couldn't iterate over dir 'extracted' content.\nError details: {e}"
            );
            exit(1);
        });

        let parsed_data = parsed_data.clone();
        handles.push(thread::spawn(move || {
            for package in read_dir(repo_dir.path()).unwrap_or_else(|e| {
                println!("[FATAL]: couldn't read repo dir.\nError details: {e}");
                exit(1)
            }) {
                let package = package.unwrap_or_else(|e| {
                    eprintln!(
                        "[FATAL]: couldn't iterate over package dir content.\nError details: {e}"
                    );
                    exit(1);
                });

                let package = package.path();
                let package_name = thread::spawn({
                    let package = package.clone();
                    move || read_package_name(package)
                });

                let commands = thread::spawn(move || read_package_bins(package));

                let commands = commands.join().unwrap_or_else(|_| {
                    eprintln!("[FATAL]: nested thread errored in parse_alpm_db.");
                    exit(1);
                });

                if !commands.is_empty() {
                    let package_name = package_name.join().unwrap_or_else(|_| {
                        eprintln!("[FATAL]: nested thread errored in parse_alpm_db");
                        exit(1);
                    });

                    let mut parsed_data = parsed_data.lock().unwrap_or_else(|e| {
                        eprintln!("[FATAL]: thread holding mutex panicked.\nError details: {e}");
                        exit(1);
                    });

                    let parsed_data = parsed_data.as_mut().unwrap();

                    for command in commands {
                        match parsed_data.get_mut(&command) {
                            Some(cmd) => cmd.push(package_name.clone()),
                            None => {
                                let _ = parsed_data.insert(command, vec![package_name.clone()]);
                            }
                        }
                    }
                }
            }
        }));
    }

    for h in handles {
        if h.join().is_err() {
            eprintln!("[FATAL]: thread errored in parse_alpm_db");
            exit(1);
        }
    }

    let data = parsed_data.lock().unwrap().take().unwrap();
    data
}
