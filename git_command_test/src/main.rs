use dirs::home_dir;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::str;

fn get_home_path_and_projects_without_dot_git_string(folder: &str) -> Option<String> {
    // remove .git from the URLs
    let folder_without_dot_git = folder.trim_end_matches("/.git/");
    // Get the home directory path
    if let Some(mut home_path) = home_dir() {
        // Append 'projects' to the home directory path
        home_path.push("projects");
        // Append folder without git to the home directory path
        home_path.push(folder_without_dot_git);
        // make it a string
        Some(home_path.to_string_lossy().to_string())
    } else {
        None
    }
}

fn get_final_path(folder: &str) -> PathBuf {
    let path_str = get_home_path_and_projects_without_dot_git_string(folder)
        .expect("Unable to determine home directory");
    PathBuf::from(path_str)
}

fn main() {
    // Use the `fd` crate to find all folders with '.git' in their names
    let output = match Command::new("fd")
        .arg(".git")
        .arg("-td")
        .arg("-H") // Only find directories (folders)
        .current_dir("/Users/jkyvetos/projects")
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            eprintln!("Error executing 'fd': {}", err);
            return;
        }
    };

    if !output.status.success() {
        eprintln!(
            "Error running 'fd': {}",
            str::from_utf8(&output.stderr).unwrap()
        );
        return;
    }

    // Convert the byte output to a string and split by newline to get each folder path
    let folders = str::from_utf8(&output.stdout)
        .unwrap()
        .trim()
        .split('\n')
        .map(|s| s.trim())
        .collect::<Vec<_>>();

    if folders.is_empty() {
        println!("No folders with '.git' found.");
        return;
    }

    // Iterate through each folder and execute 'git status'
    for folder in &folders {
        let home_path = get_final_path(folder);
        let path = Path::new(&home_path);

        // -------------------------------------------------------------------
        // PERF:
        // https://stackoverflow.com/questions/73469520/how-to-pipe-commands-in-rust
        // -------------------------------------------------------------------

        let mut git_status_cmd = Command::new("git");
        git_status_cmd
            .arg("status")
            .arg("-s")
            .current_dir(path)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit()) // Redirect output to the current process (terminal)
            .stderr(Stdio::inherit()); // Redirect error output to the current process (terminal)

        match git_status_cmd.status() {
            Ok(status) => {
                if status.success() {
                    println!("'git status' executed successfully in {:?}", path);
                } else {
                    eprintln!(
                        "Error executing 'git status' in {:?}. Status: {}",
                        path, status
                    );
                }
            }
            Err(err) => {
                eprintln!("Error executing 'git status' in {:?}: {}", path, err);
            }
        }
    }
}
