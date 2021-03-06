use std::collections::HashSet;

use std::fs::{self, ReadDir};
use std::io;

use std::cmp;

use dust::{DirEnt, Node};

mod platform;
use self::platform::*;

extern crate ansi_term;
use self::ansi_term::Colour::Fixed;

pub fn get_dir_tree(filenames: &Vec<&str>, apparent_size: bool) -> (bool, Vec<Node>) {
    let mut permissions = true;
    let mut results = vec![];
    for &b in filenames {
        let mut new_name = String::from(b);
        while new_name.chars().last() == Some('/') && new_name.len() != 1 {
            new_name.pop();
        }
        let (hp, data) = examine_dir_str(&new_name, apparent_size);
        permissions = permissions && hp;
        results.push(data);
    }
    (permissions, results)
}

fn examine_dir_str(loc: &str, apparent_size: bool) -> (bool, Node) {
    let mut inodes: HashSet<(u64, u64)> = HashSet::new();
    let (hp, result) = examine_dir(fs::read_dir(loc), apparent_size, &mut inodes);

    // This needs to be folded into the below recursive call somehow
    let new_size = result.iter().fold(0, |a, b| a + b.entry().size());
    (hp, Node::new(DirEnt::new(loc, new_size), result))
}

fn examine_dir(
    a_dir: io::Result<ReadDir>,
    apparent_size: bool,
    inodes: &mut HashSet<(u64, u64)>,
) -> (bool, Vec<Node>) {
    let mut result = vec![];
    let mut have_permission = true;

    if a_dir.is_ok() {
        let paths = a_dir.unwrap();
        for dd in paths {
            match dd {
                Ok(d) => {
                    let file_type = d.file_type().ok();
                    let maybe_size_and_inode = get_metadata(&d, apparent_size);

                    match (file_type, maybe_size_and_inode) {
                        (Some(file_type), Some((size, inode))) => {
                            let s = d.path().to_string_lossy().to_string();
                            if let Some(inode_dev_pair) = inode {
                                if inodes.contains(&inode_dev_pair) {
                                    continue;
                                }
                                inodes.insert(inode_dev_pair);
                            }

                            if d.path().is_dir() && !file_type.is_symlink() {
                                let (hp, recursive) =
                                    examine_dir(fs::read_dir(d.path()), apparent_size, inodes);
                                have_permission = have_permission && hp;
                                let new_size =
                                    recursive.iter().fold(size, |a, b| a + b.entry().size());
                                result.push(Node::new(DirEnt::new(&s, new_size), recursive))
                            } else {
                                result.push(Node::new(DirEnt::new(&s, size), vec![]))
                            }
                        }
                        (_, None) => have_permission = false,
                        (_, _) => (),
                    }
                }
                Err(_) => (),
            }
        }
    } else {
        have_permission = false;
    }
    (have_permission, result)
}

// We start with a list of root directories - these must be the biggest folders
// We then repeadedly merge in the children of the biggest directory - Each iteration
// the next biggest directory's children are merged in.
pub fn find_big_ones<'a>(l: &'a Vec<Node>, max_to_show: usize) -> Vec<&Node> {
    let mut new_l: Vec<&Node> = l.iter().map(|a| a).collect();
    new_l.sort();

    for processed_pointer in 0..max_to_show {
        if new_l.len() == processed_pointer {
            break;
        }
        // Must be a list of pointers into new_l otherwise b_list will go out of scope
        // when it is deallocated
        let mut b_list: Vec<&Node> = new_l[processed_pointer]
            .children()
            .iter()
            .map(|a| a)
            .collect();
        new_l.extend(b_list);
        new_l.sort();
    }

    if new_l.len() > max_to_show {
        new_l[0..max_to_show + 1].to_vec()
    } else {
        new_l
    }
}

pub fn display(permissions: bool, to_display: &Vec<&Node>) -> () {
    if !permissions {
        eprintln!("Did not have permissions for all directories");
    }

    display_node(to_display[0], &to_display, true, 1, "")
}

fn display_node<S: Into<String>>(
    node_to_print: &Node,
    to_display: &Vec<&Node>,
    is_first: bool,
    depth: u8,
    indentation_str: S,
) {
    let mut is = indentation_str.into();
    print_this_node(node_to_print, is_first, depth, is.as_ref());

    is = is.replace("└─┬", "  ");
    is = is.replace("└──", "  ");
    is = is.replace("├──", "│ ");
    is = is.replace("├─┬", "│ ");

    let printable_node_slashes = node_to_print.entry().name().matches('/').count();

    let mut num_siblings = to_display.iter().fold(0, |a, b| {
        if node_to_print.children().contains(b)
            && b.entry().name().matches('/').count() == printable_node_slashes + 1
        {
            a + 1
        } else {
            a
        }
    });

    let mut is_biggest = true;
    for node in to_display {
        if node_to_print.children().contains(node) {
            let has_display_children = node.children()
                .iter()
                .fold(false, |has_kids, n| has_kids || to_display.contains(&n));

            let has_children = node.children().len() > 0 && has_display_children;
            if node.entry().name().matches('/').count() == printable_node_slashes + 1 {
                num_siblings -= 1;

                let tree_chars = {
                    if num_siblings == 0 {
                        if has_children {
                            "└─┬"
                        } else {
                            "└──"
                        }
                    } else {
                        if has_children {
                            "├─┬"
                        } else {
                            "├──"
                        }
                    }
                };
                display_node(
                    &node,
                    to_display,
                    is_biggest,
                    depth + 1,
                    is.to_string() + tree_chars,
                );
                is_biggest = false;
            }
        }
    }
}

fn print_this_node(node_to_print: &Node, is_biggest: bool, depth: u8, indentation_str: &str) {
    let padded_size = format!("{:>5}", human_readable_number(node_to_print.entry().size()),);
    println!(
        "{} {} {}",
        if is_biggest {
            Fixed(196).paint(padded_size)
        } else {
            Fixed(7).paint(padded_size)
        },
        indentation_str,
        Fixed(7)
            .on(Fixed(cmp::min(8, (depth) as u8) + 231))
            .paint(node_to_print.entry().name().to_string())
    );
}

fn human_readable_number(size: u64) -> (String) {
    let units = vec!["T", "G", "M", "K"]; //make static

    //return format!("{}B", size);

    for (i, u) in units.iter().enumerate() {
        let marker = 1024u64.pow((units.len() - i) as u32);
        if size >= marker {
            if size / marker < 10 {
                return format!("{:.1}{}", (size as f32 / marker as f32), u);
            } else {
                return format!("{}{}", (size / marker), u);
            }
        }
    }
    return format!("{}B", size);
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_human_readable_number() {
        assert_eq!(human_readable_number(1), "1B");
        assert_eq!(human_readable_number(956), "956B");
        assert_eq!(human_readable_number(1004), "1004B");
        assert_eq!(human_readable_number(1024), "1.0K");
        assert_eq!(human_readable_number(1536), "1.5K");
        assert_eq!(human_readable_number(1024 * 512), "512K");
        assert_eq!(human_readable_number(1024 * 1024), "1.0M");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 - 1), "1023M");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 * 20), "20G");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 * 1024), "1.0T");
    }
}
