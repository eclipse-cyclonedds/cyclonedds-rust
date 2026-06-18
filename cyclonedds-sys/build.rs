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
    Vendoring {
        message: String,
        source: VendoringError,
    },
}

#[derive(Debug)]
pub enum VendoringError {
    Io(std::io::Error),
    StripPrefix(std::path::StripPrefixError),
}

#[cfg(not(feature = "vendored"))]
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

#[cfg(feature = "vendored")]
pub fn generate_bindings(
    mut bindgen: bindgen::Builder,
    target: &Target,
) -> Result<bindgen::Bindings, BuildError> {
    fn for_each_file(
        path: &Path,
        f: &impl Fn(&Path) -> Result<(), BuildError>,
    ) -> Result<(), BuildError> {
        for entry in std::fs::read_dir(path).map_err(|source| BuildError::Vendoring {
            message: format!("failed to read directory: {path:?}"),
            source: VendoringError::Io(source),
        })? {
            let entry = entry.map_err(|source| BuildError::Vendoring {
                message: "encountered invalid directory entry".to_string(),
                source: VendoringError::Io(source),
            })?;
            if entry
                .file_type()
                .map_err(|source| BuildError::Vendoring {
                    message: "unable to determine file type for directory entry".to_string(),
                    source: VendoringError::Io(source),
                })?
                .is_dir()
            {
                for_each_file(&entry.path(), f)?;
            } else {
                f(&entry.path())?;
            }
        }
        Ok(())
    }

    // Build Cyclone DDS from vendored sources.
    println!("cargo:rerun-if-changed=vendor/cyclonedds-c");
    let cyclonedds_c_out_path = PathBuf::from(env("OUT_DIR")?).join("cyclonedds-c-build");

    let tempdir = tempfile::tempdir().map_err(|source| BuildError::Vendoring {
        message: "failed to create tempdir".to_string(),
        source: VendoringError::Io(source),
    })?;

    let original_vendored_src = Path::new("vendor/cyclonedds-c");
    let tempdir_vendored_src = tempdir.path().join("cyclonedds-c-src");

    for_each_file(original_vendored_src, &|path| {
        let relative =
            path.strip_prefix(original_vendored_src)
                .map_err(|source| BuildError::Vendoring {
                    message: format!(
                        "could not strip prefix: {path:?} from {original_vendored_src:?}"
                    ),
                    source: VendoringError::StripPrefix(source),
                })?;
        let destination = tempdir_vendored_src.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).map_err(|source| {
            BuildError::Vendoring {
                message: format!("could not create directory for {destination:?}"),
                source: VendoringError::Io(source),
            }
        })?;
        std::fs::copy(path, &destination).map_err(|source| BuildError::Vendoring {
            message: format!("could not copy sources from {path:?} to {destination:?}"),
            source: VendoringError::Io(source),
        })?;
        Ok(())
    })?;

    let cross_compiling = env("HOST")? != env("TARGET")?;

    if cross_compiling && target.os == "windows" {
        // Fixup cross-compile issues when targeting Windows.
        let cmakelists = tempdir_vendored_src.join("CMakeLists.txt");
        let contents =
            std::fs::read_to_string(&cmakelists).map_err(|source| BuildError::Vendoring {
                message: format!("could not read CMakeLists.txt from tempdir: {cmakelists:?}"),
                source: VendoringError::Io(source),
            })?;
        let patched = contents.replace("include(CMakeCPack.cmake)", "");

        std::fs::write(&cmakelists, patched).map_err(|source| BuildError::Vendoring {
            message: format!("could not write cross-compile changes to {cmakelists:?}"),
            source: VendoringError::Io(source),
        })?;
    }

    let mut cyclonedds_cmake = cmake::Config::new(&tempdir_vendored_src);
    let mut cyclonedds_cmake = cyclonedds_cmake.out_dir(&cyclonedds_c_out_path);

    cyclonedds_cmake = cyclonedds_cmake
        .out_dir(&cyclonedds_c_out_path)
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_IDLC", "OFF")
        .define("BUILD_DDSPERF", "OFF")
        .define("ENABLE_SSL", "NO")
        .define("ENABLE_SECURITY", "NO")
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        // NOTE: this is to keep the symbols when building with `--release`.
        .define("ENABLE_LTO", "NO");

    if cross_compiling {
        cyclonedds_cmake = cyclonedds_cmake.define("CMAKE_CROSSCOMPILING", "ON");
    }

    let cyclonedds_c = cyclonedds_cmake.build();

    // Fix rebuilds of the -sys crate.
    let time = cyclonedds_c_out_path
        .metadata()
        .map(|metadata| filetime::FileTime::from_creation_time(&metadata))
        .ok()
        .flatten()
        .unwrap_or(filetime::FileTime::zero());
    for_each_file(&cyclonedds_c_out_path.join("include"), &|path| {
        filetime::set_file_mtime(path, time).map_err(|source| BuildError::Vendoring {
            message: format!("could not reset mtime (to reduce rebuilds) on {path:?}"),
            source: VendoringError::Io(source),
        })
    })?;

    println!(
        "cargo:rustc-link-search={}",
        cyclonedds_c.join("lib").display()
    );
    bindgen = bindgen.clang_arg(format!("-I{}", cyclonedds_c.join("include").display()));

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
