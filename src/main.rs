fn main() {
    // Cet appel garde un point d'entrée minimal tout en vérifiant que tous les modules sont reliés.
    let app = rust_project::sim::bootstrap();

    println!(
        "Squelette Rust prêt : {} modules, point d'entrée '{}'.",
        app.module_count, app.entrypoint
    );
}
