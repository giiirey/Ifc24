use std::path::PathBuf;

fn main() {
    let src_dir = PathBuf::from("src/bindings");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let boost_available = std::path::Path::new("/usr/include/boost").exists();

    let mut builder = cc::Build::new();
    builder
        .cpp(true)
        .std("c++17")
        .include(&src_dir)
        .file(src_dir.join("ifc_openshell.cpp"));

    builder.include("/usr/include");
    builder.include("/usr/include/ifcparse");
    builder.include("/usr/include/ifcgeom");
    builder.include("/usr/include/ifcgeom/kernels/opencascade");
    builder.include("/usr/include/opencascade");
    builder.include("/usr/include/eigen3");

    if let Ok(ifc_include_dir) = std::env::var("IFCOPENSHELL_ROOT") {
        builder.include(PathBuf::from(ifc_include_dir).join("include"));
    }

    if boost_available {
        builder.define("IFC_OPENSHELL_AVAILABLE", None);
    } else {
        builder.define("IFC_OPENSHELL_STUB", None);
    }

    builder.compile("ifc_wrapper");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    if boost_available {
        println!("cargo:rustc-link-lib=dylib=IfcParse");
        println!("cargo:rustc-link-lib=dylib=IfcGeom");
        println!("cargo:rustc-link-lib=dylib=geometry_kernel_opencascade");
    }

    println!("cargo:rustc-link-search=native=/usr/lib");

    bindgen::Builder::default()
        .header(src_dir.join("ifc_openshell.h").to_string_lossy())
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Unable to write bindings");

    println!("cargo:rerun-if-changed=src/bindings/");
}
