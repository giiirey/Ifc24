use std::path::PathBuf;

fn main() {
    let src_dir = PathBuf::from("src/bindings");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let mut builder = cc::Build::new();
    builder
        .cpp(true)
        .std("c++17")
        .include(&src_dir)
        .file(src_dir.join("ifc_openshell.cpp"));

    // Arch Linux ifcopenshell installs includes in subdirectories
    builder.include("/usr/include");
    builder.include("/usr/include/ifcparse");
    builder.include("/usr/include/ifcgeom");

    // IFCOPENSHELL_ROOT override for custom builds (e.g. ~/src/ifcopenshell)
    if let Ok(ifc_include_dir) = std::env::var("IFCOPENSHELL_ROOT") {
        builder.include(PathBuf::from(ifc_include_dir).join("include"));
    }

    builder.compile("ifc_wrapper");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    // Arch Linux ifcopenshell package provides only these two shared libs
    for lib in &["IfcParse", "IfcGeom"] {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }

    bindgen::Builder::default()
        .header(src_dir.join("ifc_openshell.h").to_string_lossy())
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Unable to write bindings");

    println!("cargo:rerun-if-changed=src/bindings/");
}
