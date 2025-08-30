use std::path::{Path, PathBuf};

/// Wrapper to override bindgen callbacks so that link names for items ending in
/// `_bindgen_wrapper` can be intercepted and mapped.
#[derive(Debug)]
struct BindgenWrapperCallbacks {
    /// Stores the list of names mapped during the generate name override phase
    /// so that they can be correctly emitted during link name override
    /// phase.
    mapped_names: std::cell::RefCell<Vec<String>>,
}

impl BindgenWrapperCallbacks {
    /// Create a new wrapper with no mapped names.
    pub fn new() -> Self {
        Self {
            mapped_names: Default::default(),
        }
    }
}

impl bindgen::callbacks::ParseCallbacks for BindgenWrapperCallbacks {
    fn generated_name_override(
        &self,
        item_info: bindgen::callbacks::ItemInfo<'_>,
    ) -> Option<String> {
        match (
            &item_info.kind,
            item_info.name.rsplit_once("_bindgen_wrapper"),
        ) {
            (bindgen::callbacks::ItemKind::Function, Some((start, _))) => {
                let name = start.to_string();
                self.mapped_names.borrow_mut().push(name.clone());
                Some(name)
            }
            _ => None,
        }
    }

    fn generated_link_name_override(
        &self,
        item_info: bindgen::callbacks::ItemInfo<'_>,
    ) -> Option<String> {
        let name = item_info.name.into();

        match item_info.kind {
            bindgen::callbacks::ItemKind::Function
                if self.mapped_names.borrow().contains(&name) =>
            {
                Some(name)
            }
            _ => None,
        }
    }
}

/// Fix the exported bindings dependings on the platform.
fn platform_fixup(target: &Target, bindings_path: &Path) {
    if target.os == "macos" {
        let contents = std::fs::read_to_string(bindings_path).expect("could not read bindings.rs");
        // Fixes an issue under macOS in bindgen: https://github.com/rust-lang/rust-bindgen/issues/1725
        let cleaned = contents.replace("#[link_name = \"\\u{1}dds", "#[link_name = \"dds");
        std::fs::write(bindings_path, cleaned).expect("could not write cleaned bindings.rs");
    }
}

fn env(var: &'static str) -> Result<String, BuildError> {
    std::env::var(var).map_err(|source| BuildError::Env { var, source })
}

pub struct Target {
    pub os: String,
    pub arch: String,
}

impl Target {
    pub fn from_env() -> Result<Self, BuildError> {
        let os = env("CARGO_CFG_TARGET_OS")?;
        let arch = env("CARGO_CFG_TARGET_ARCH")?;

        Ok(Self { os, arch })
    }
}

#[derive(Debug)]
pub enum BuildError {
    Env {
        var: &'static str,
        source: std::env::VarError,
    },
    Bindgen {
        source: bindgen::BindgenError,
    },
}

pub fn generate_bindings(
    mut bindgen: bindgen::Builder,
    _target: &Target,
) -> Result<bindgen::Bindings, BuildError> {
    // Check for the CYCLONEDDS_HOME variable which has priority over the system
    // paths.
    println!("cargo:rerun-if-env-changed=CYCLONEDDS_HOME");

    // If the environment variable exists, search it for the libraries and the
    // headers.
    if let Ok(cyclonedds_home) = std::env::var("CYCLONEDDS_HOME") {
        let cyclonedds_home = Path::new(&cyclonedds_home);

        for lib_dir in &["lib", "lib64"] {
            let path = cyclonedds_home.join(lib_dir);
            if path.exists() {
                println!("cargo:rustc-link-search={}", path.display());
            }
        }

        bindgen = bindgen.clang_arg(format!("-I{}", cyclonedds_home.join("include").display()));
    }

    // Prepare the bindings.
    bindgen
        .generate()
        .map_err(|source| BuildError::Bindgen { source })
}

fn main() -> Result<(), BuildError> {
    // Get the build target properties.
    let target = Target::from_env()?;
    // Prepare the output directory.
    let out_path = PathBuf::from(env("OUT_DIR")?);

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rustc-link-lib=ddsc");

    // Prepare the base bindgen configuration.
    let mut bindgen = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(BindgenWrapperCallbacks::new()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .wrap_static_fns(true)
        .generate_cstr(true)
        .derive_default(true)
        .disable_untagged_union()
        .allowlist_item("dds_.*")
        .allowlist_item("ddsi_.*")
        .allowlist_item("ddsrt_.*")
        .allowlist_var("DDS_.*")
        .allowlist_var("DOMAIN_DEFAULT")
        .allowlist_var("DURATION_INFINITE")
        .allowlist_var("TIME_NEVER")
        .formatter(bindgen::Formatter::Prettyplease)
        .layout_tests(true);

    if target.os == "windows" {
        // NOTE: Windows does not have the `dds_sertype_v0` constant exposed and the
        // underlying value that v0 expects in the C lib for that
        // platform is 0x01 cast to a function pointer.
        //
        // This cannot be created in Rust as a function pointer because the const
        // evaluator rejects 0x01 as a valid function pointer so the `-sys` crate
        // intercepts the type under Windows and exposes a compatible type definition
        // with the same niche-optimization as Option<fn()> i.e. Option<NonZeroUsize>.
        //
        // I can then safely (internally) create this value which should be interpreted
        // equivalently on the Windows side as if I had passed the 0x01 function
        // pointer.
        bindgen = bindgen
            .blocklist_type("ddsi_sertype_v0_t")
            .raw_line("pub type ddsi_sertype_v0_t = Option<std::num::NonZeroUsize>;");

        // Windows native libs that are needed.
        println!("cargo:rustc-link-lib=iphlpapi");
        println!("cargo:rustc-link-lib=dbghelp");
        println!("cargo:rustc-link-lib=bcrypt");
    }

    // Prepare the bindings.
    let bindings = generate_bindings(bindgen, &target)?;

    // Write the preliminary bindings.
    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("could not write bindings");

    // Fix up the bindings as needed for the target.
    platform_fixup(&target, &bindings_path);

    Ok(())
}
