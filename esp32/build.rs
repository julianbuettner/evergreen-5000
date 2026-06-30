fn main() {
    // Load .env file at compile time so env!("...") works without manually
    // exporting variables before every build.
    if let Err(e) = dotenvy::dotenv() {
        println!("cargo:warning=Could not load .env file: {e}");
    }

    embuild::espidf::sysenv::output();
}
