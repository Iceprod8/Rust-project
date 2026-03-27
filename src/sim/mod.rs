/// Petit descripteur utilisé pendant la toute première phase d'initialisation du projet.
pub struct AppSkeleton {
    pub module_count: usize,
    pub entrypoint: &'static str,
}

/// Construit le squelette initial avant l'arrivée de la vraie boucle de simulation.
pub fn bootstrap() -> AppSkeleton {
    // On appelle chaque module ici pour valider que l'arborescence du crate est bien connectée.
    crate::map::register();
    crate::robots::register();
    crate::base::register();
    crate::comms::register();
    crate::ui::register();

    AppSkeleton {
        module_count: 6,
        entrypoint: "src/main.rs",
    }
}
