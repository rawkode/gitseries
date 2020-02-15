use dirs;
use git2::{Repository, Revwalk};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let user_home = match dirs::home_dir() {
        Some(home) => home,
        None => {
            println!("Failed to establish users home directory");
            return;
        }
    };

    let config_dir = user_home.join(".config/gitseries");

    fs::create_dir_all(&config_dir);

    let url = "https://github.com/influxdata/influxdb";
    let clone_dir = config_dir.join("influxdb");

    let mut repo: git2::Repository;
    if !clone_dir.exists() {
        repo = match Repository::clone(url, clone_dir) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };
    } else {
        repo = match Repository::open(clone_dir) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };
    }

    let object_database = repo.odb().unwrap();
    let (mut total, mut unknowns, mut anys, mut commits, mut trees, mut blobs, mut tags) =
        (0, 0, 0, 0, 0, 0, 0);

    match object_database.foreach(|&oid| {
        use git2::ObjectType::*;
        let object = repo.find_object(oid, None).expect("no error");

        match object.kind() {
            Some(Tag) => tags += 1,
            Some(Commit) => {
                commits += 1;
                process_commit(&repo, object.as_commit().unwrap());
            }
            Some(Tree) => trees += 1,
            Some(Blob) => blobs += 1,
            Some(Any) => anys += 1,
            None => unknowns += 1,
        };
        total += 1;
        true
    }) {
        Ok(_) => (),
        Err(e) => {
            println!("Something went wrong");
            return;
        }
    }
}

fn process_commit(repo: &git2::Repository, commit: &git2::Commit) -> bool {
    let author = commit.committer();

    let mut tags: Vec<String> = Vec::new();
    let mut fields: Vec<String> = Vec::new();

    tags = tag_push(
        tags,
        String::from("repository"),
        String::from("github.com/influxdata/influxdb"),
    );

    tags = tag_push(
        tags,
        String::from("author_email"),
        String::from(author.email().unwrap()),
    );

    tags = tag_push(
        tags,
        String::from("author_name"),
        String::from(author.name().unwrap()),
    );

    // fields = field_push_s(
    //     fields,
    //     String::from("message"),
    //     String::from(commit.message().unwrap()),
    // );

    let mut diff_opts = git2::DiffOptions::new();
    let diff = repo.diff_tree_to_tree(
        Some(&commit.tree().unwrap()),
        Some(&commit.parent(0).unwrap().tree().unwrap()),
        Some(&mut diff_opts),
    );

    let stats = diff.unwrap().stats().unwrap();

    fields = field_push_u(
        fields,
        String::from("files_modified"),
        stats.files_changed(),
    );
    fields = field_push_u(fields, String::from("deletions"), stats.deletions());
    fields = field_push_u(fields, String::from("insertions"), stats.insertions());

    write_line_protocol("commit", &tags, &fields, &commit.time().seconds());

    return true;
}

fn tag_push(mut vec: Vec<String>, key: String, val: String) -> Vec<String> {
    vec.push(format!("{}={}", key, val.replace(" ", "\\ ")));
    return vec;
}

fn field_push_i(mut vec: Vec<String>, key: String, val: i64) -> Vec<String> {
    vec.push(format!("{}={}i", key, val));
    return vec;
}

fn field_push_u(mut vec: Vec<String>, key: String, val: usize) -> Vec<String> {
    vec.push(format!("{}={}i", key, val));
    return vec;
}

fn field_push_s(mut vec: Vec<String>, key: String, val: String) -> Vec<String> {
    vec.push(format!("{}=\"{}\"", key, val.replace("\n", "\\n")));
    return vec;
}

fn write_line_protocol(
    measurement: &str,
    tags: &Vec<String>,
    fields: &Vec<String>,
    timestamp: &i64,
) -> bool {
    let tags_join = tags.join(",");
    let fields_join = fields.join(",");

    println!(
        "{},{} {} {}000000000",
        measurement, tags_join, fields_join, timestamp
    );
    return true;
}
