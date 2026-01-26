use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use crate::derived::card_lookup::CardLookupFunction;
use crate::derived::derived_types::DerivedFunction;
use crate::derived::image_derived::ImageDerivedFunction;
use crate::derived::image_url::ImageUrlFunction;
use crate::images::image_cache::ImageCache;

/// Global function registry storing all registered derived functions.
static GLOBAL_REGISTRY: OnceLock<FunctionRegistry> = OnceLock::new();

/// Registry for derived column functions, providing lookup by function name.
pub struct FunctionRegistry {
    functions: RwLock<HashMap<&'static str, Box<dyn DerivedFunction>>>,
}

impl FunctionRegistry {
    /// Creates a new empty function registry.
    pub fn new() -> Self {
        Self { functions: RwLock::new(HashMap::new()) }
    }

    /// Registers a derived function with this registry.
    ///
    /// Panics if a function with the same name is already registered.
    pub fn register(&self, function: Box<dyn DerivedFunction>) {
        let name = function.name();
        let mut functions = self.functions.write().expect("Registry lock poisoned");
        if functions.contains_key(name) {
            panic!("Derived function '{}' is already registered", name);
        }
        tracing::debug!(
            component = "tv.derived.registry",
            function_name = name,
            "Registered derived function"
        );
        functions.insert(name, function);
    }

    /// Invokes a callback with the derived function if it exists.
    ///
    /// Returns `Some(result)` if the function was found, `None` otherwise.
    pub fn with_function<F, R>(&self, name: &str, callback: F) -> Option<R>
    where
        F: FnOnce(&dyn DerivedFunction) -> R,
    {
        let functions = self.functions.read().expect("Registry lock poisoned");
        functions.get(name).map(|f| callback(f.as_ref()))
    }

    /// Returns whether a function with the given name is registered.
    pub fn contains(&self, name: &str) -> bool {
        let functions = self.functions.read().expect("Registry lock poisoned");
        functions.contains_key(name)
    }

    /// Returns a list of all registered function names.
    pub fn list_functions(&self) -> Vec<&'static str> {
        let functions = self.functions.read().expect("Registry lock poisoned");
        functions.keys().copied().collect()
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Initializes the global function registry with all built-in derived functions.
///
/// This function should be called once at application startup before any
/// derived column computations are requested.
pub fn initialize_global_registry() {
    let registry = GLOBAL_REGISTRY.get_or_init(|| {
        let registry = FunctionRegistry::new();
        registry.register(Box::new(CardLookupFunction::new()));
        registry.register(Box::new(ImageUrlFunction::new()));
        tracing::info!(
            component = "tv.derived.registry",
            function_count = registry.list_functions().len(),
            functions = ?registry.list_functions(),
            "Initialized global function registry"
        );
        registry
    });
    let _ = registry;
}

/// Returns a reference to the global function registry.
///
/// Panics if the registry has not been initialized via `initialize_global_registry`.
pub fn global_registry() -> &'static FunctionRegistry {
    GLOBAL_REGISTRY.get().expect("Global function registry not initialized. Call initialize_global_registry() first.")
}

/// Registers the image derived function with the global registry.
///
/// This must be called after the image cache is initialized, since the
/// function requires access to the cache for fetching and storing images.
pub fn register_image_derived_function(cache: Arc<ImageCache>) {
    let registry = global_registry();
    if registry.contains("image_derived") {
        tracing::debug!(
            component = "tv.derived.registry",
            "Image derived function already registered"
        );
        return;
    }
    registry.register(Box::new(ImageDerivedFunction::new(cache)));
    tracing::info!(
        component = "tv.derived.registry",
        "Registered image derived function"
    );
}
