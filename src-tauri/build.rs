fn main() {
    // Se o manifesto mudar, refaz o build (senao o exe antigo, com manifesto corrompido, fica em cache).
    println!("cargo:rerun-if-changed=windows-app-manifest.xml");
    // Embute um manifesto Windows exigindo elevação de administrador no build de release.
    let manifest = include_str!("windows-app-manifest.xml");
    tauri_build::try_build(
        tauri_build::Attributes::new().windows_attributes(
            tauri_build::WindowsAttributes::new().app_manifest(manifest),
        ),
    )
    .expect("falha ao rodar tauri-build");
}
