fn main() {
    // https://docs.rs/diesel_migrations/2.2.0/diesel_migrations/macro.embed_migrations.html#automatic-rebuilds
    println!("cargo:rerun-if-changed=migrations/");

    built::write_built_file().expect("Failed to acquire build-time information");
}
