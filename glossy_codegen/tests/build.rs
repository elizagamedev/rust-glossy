extern crate glossy_codegen;
use glossy_codegen::{Config, Language};

fn setup() {
    use std::{env, fs};

    // Make output dir, as this isn't a real build script
    fs::create_dir_all(env::var("OUT_DIR").unwrap()).ok();
}

#[test]
fn common() {
    setup();

    // Test 1 (ordinary include use-case with a bunch of comment tests, no optimization, id map)
    Config::new(Language::OpenGl)
        .vertex("tests/common.glsl")
        .include("tests/include*.glsl")
        .build();
}

#[test]
fn optimize() {
    setup();

    // Test 2 (ordinary include use-case with a bunch of comment tests, optimized output)
    Config::new(Language::OpenGl)
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
    Config::new(Language::OpenGl)
        .source("tests/recurse.glsl")
        .include("tests/badinclude*.glsl")
        .build();
}

#[test]
#[should_panic]
fn version() {
    setup();

    // Test 4 (version validation)
    Config::new(Language::OpenGl)
        .source("tests/version.glsl")
        .include("tests/include*.glsl")
        .build();
}
