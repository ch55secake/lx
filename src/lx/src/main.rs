mod args;
mod output;

use crate::output::output_with_permissions;
use args::Args;
use clap::Parser;
use lx_lib::DirectoryItem;
use output::output;

fn main() {
    let args = Args::parse();

    let dir_items: Vec<DirectoryItem> = if args.recursive {
        lx_lib::find_directory_items_recursive(&args.directory)
    } else {
        lx_lib::find_directory_items(&args.directory)
    };

    let mut filtered_items: Vec<DirectoryItem> = dir_items
        .into_iter()
        .filter(|item| matches_pattern(&item.name, &args.find))
        .collect();

    let output = match args {
        args if args.all && args.permissions => output_with_permissions(&mut filtered_items, show_all),
        args if args.all => output(&filtered_items, show_all),
        args if args.permissions => output_with_permissions(&mut filtered_items, hide_hidden),
        _ => output(&filtered_items, hide_hidden),
    };

    println!("{}", output);
}

/// Include all items in output including hidden files
fn show_all(_: &DirectoryItem) -> bool {
    true
}

/// Remove hidden files from the output
fn hide_hidden(item: &DirectoryItem) -> bool {
    !item.is_hidden
}

/// Check if a filename matches the given pattern
/// Supports simple wildcard patterns with '*' matching any sequence of characters
fn matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern.is_empty() {
        return true;
    }

    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 && !pattern.contains('*') {
        return filename == pattern;
    }

    let mut remaining = filename;

    if pattern.starts_with('*') {
        if let Some(value) = find_remaining_parts(parts, remaining) {
            return value;
        }
        return pattern.ends_with('*') || remaining.is_empty();
    } else {
        if !filename.starts_with(parts[0]) {
            return false;
        }

        remaining = &remaining[parts[0].len()..];

        if let Some(value) = find_remaining_parts(parts, remaining) {
            return value;
        }

        pattern.ends_with('*') || remaining.is_empty()
    }
}

/// Find the remaining matching parts of a string
fn find_remaining_parts(parts: Vec<&str>, mut remaining: &str) -> Option<bool> {
    for &part in parts.iter().skip(1) {
        if part.is_empty() {
            continue;
        }

        match remaining.find(part) {
            Some(pos) => {
                remaining = &remaining[pos + part.len()..];
            }
            None => return Some(false),
        }
    }
    None
}
