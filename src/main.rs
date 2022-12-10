use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// Autopull a specified branch in all subdirectories of a specified path
#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    /// Path to the folder containing multiple github repos
    path: PathBuf,
    /// Name of the branch that should be pulled
    branch_name: String,
}

const SPACER: &str = "               ";
const COUNTING: &str = "[counting]     ";
const CHECK_BRANCH: &str = "[git branch]   ";
const ERROR: &str = " ! [error]     ";
const GIT_CHECKOUT: &str = "[git checkout] ";
const GIT_PULL: &str = "[git pull]     ";
const OUTPUT: &str = " > [output]      | ";

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path: PathBuf = cli.path;
    let branch: String = cli.branch_name;

    let dir = fs::read_dir(&path)
        .with_context(|| format!(" ! [error] Failed to read path: {:?}", &path))?;
    let dir: Vec<_> = dir.collect();

    let mut count = 0;

    println!("{} Counting paths", COUNTING);
    for entry in dir.iter() {
        let entry = entry.as_ref().unwrap();

        let metadata = entry.metadata()?;

        println!("{}  |  Detected path: {:?}", SPACER, entry.path());
        if metadata.is_dir() {
            count += 1;
            println!("{}  |  Added - current count: {}", SPACER, count);
        }
    }

    for (i, entry) in dir.into_iter().enumerate() {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            let path = entry.path();

            println!("\n");
            println!("[{} / {}]         Continuing with next repo:", i + 1, count);
            let branch_exists = git_branch_exists(&path, &branch)?;
            if branch_exists {
                let output = git_checkout(&path, &branch)?;
                handle_output(&output)?;

                let output = git_pull(&path)?;
                handle_output(&output)?;
            }
        }
    }

    Ok(())
}

fn git_branch_exists(path: &PathBuf, branch: &String) -> Result<bool> {
    let mut git_branch = Command::new("git");
    git_branch.args(["branch"]);
    git_branch.current_dir(path);

    println!("{} Checking branches of {:?}", CHECK_BRANCH, path);
    let output = git_branch.output().with_context(|| {
        format!(
            "{} Failed to execute process: {:?}, in {:?}",
            ERROR, git_branch, path
        )
    })?;

    let output = String::from_utf8(output.stdout)?;

    let branch_to_check = format!("{}\n", branch);
    let branch_to_check = branch_to_check.as_str();

    let check = output.contains(branch_to_check);

    println!(
        "{} Branch '{}' {}found",
        CHECK_BRANCH,
        branch,
        match check {
            false => "not ",
            true => "",
        }
    );

    Ok(check)
}

fn git_checkout(path: &PathBuf, branch: &String) -> Result<Output> {
    let mut git_checkout = Command::new("git");
    git_checkout.args(["checkout", branch.as_str()]);
    git_checkout.current_dir(path);

    println!("{} Checking out: {:?}", GIT_CHECKOUT, path);
    let output = git_checkout.output().with_context(|| {
        format!(
            "{} Failed to execute process: {:?}, in {:?}",
            ERROR, git_checkout, path
        )
    })?;

    Ok(output)
}

fn git_pull(path: &PathBuf) -> Result<Output> {
    let mut git_pull = Command::new("git");
    git_pull.args(["pull"]);
    git_pull.current_dir(path);

    println!("{} Pulling: {:?}", GIT_PULL, path);
    let output = git_pull.output().with_context(|| {
        format!(
            "{} Failed to execute process: {:?}, in {:?}",
            ERROR, git_pull, path
        )
    })?;

    Ok(output)
}

fn handle_output(output: &Output) -> Result<()> {
    println!("{} {}", OUTPUT, output.status);

    let out = std::str::from_utf8(&output.stdout)?;
    out.lines().for_each(|line| {
        println!("{} {}", OUTPUT, line);
    });

    let err = std::str::from_utf8(&output.stderr)?;
    err.lines().for_each(|line| {
        println!("{} {}", OUTPUT, line);
    });

    Ok(())
}
