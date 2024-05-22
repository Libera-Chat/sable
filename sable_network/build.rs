use std::{env, fs, io::Write, path};

use chrono::DateTime;

fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");

    let root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out = path::Path::new(&env::var("OUT_DIR").expect("OUT_DIR not set")).join("built.rs");
    let _ = write_head_date(root, out);
}

fn write_head_date(root: String, dest: path::PathBuf) -> Result<(), git2::Error> {
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(dest)
        .expect("could not append to built.rs");
    match git2::Repository::discover(root) {
        Ok(repo) => {
            let head = repo.head()?.peel_to_commit()?.time();
            let head_secs = head.seconds() + (head.offset_minutes() as i64 * 60);
            let head_time = DateTime::from_timestamp(head_secs, 0).map(|dt| dt.to_rfc2822());
            f.write_all(
                format!(
                    "\npub const GIT_COMMIT_TIME_UTC: Option<&str> = {:?};\n",
                    head_time
                )
                .as_bytes(),
            )
            .expect("could not write to built.rs");
            Ok(())
        }
        Err(ref e)
            if e.class() == git2::ErrorClass::Repository
                && e.code() == git2::ErrorCode::NotFound =>
        {
            f.write_all(
                format!("\npub const GIT_COMMIT_TIME_UTC: Option<&str> = None;\n").as_bytes(),
            )
            .expect("could not write to built.rs");
            Ok(())
        }
        Err(e) => Err(e),
    }
}
