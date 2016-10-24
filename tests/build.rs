#[macro_use]
extern crate glossy;

fn setup() {
    use std::env;
    use std::fs;

    // Make output dir, as this isn't a real build script
    fs::create_dir_all(env::var("OUT_DIR").unwrap()).ok();
}

#[test]
fn common() {
    setup();

    // Test 1 (ordinary include use-case with a bunch of comment tests, no optimization, id map)
    glossy::Config::new(glossy::Language::OpenGl)
        .vertex("tests/common.glsl")
        .include("tests/include*.glsl")
        .build();
}

#[test]
fn optimize() {
    setup();

    // Test 2 (ordinary include use-case with a bunch of comment tests, optimized output)
    glossy::Config::new(glossy::Language::OpenGl)
        .vertex("tests/common.glsl")
        .include("tests/include*.glsl")
        .optimize()
        .build();
}

#[test]
#[should_panic]
fn recurse() {
    setup();

    // Test 3 (recursive inclusion)
    glossy::Config::new(glossy::Language::OpenGl)
        .source("tests/recurse.glsl")
        .include("tests/badinclude*.glsl")
        .build();
}

#[test]
#[should_panic]
fn version() {
    setup();

    // Test 4 (version validation)
    glossy::Config::new(glossy::Language::OpenGl)
        .source("tests/version.glsl")
        .include("tests/include*.glsl")
        .build();
}
