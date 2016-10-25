glossy
======

A GLSL source loading crate for Rust which supports the `#include`
directive and shader optimization at compile time via
[glsl-optimizer](https://github.com/aras-p/glsl-optimizer).

Links
-----

* [Documentation](https://mathewv.github.io/doc/glossy/glossy/index.html)
* [glossy on crates.io](https://crates.io/crates/glossy)
* [glossy_codegen on crates.io](https://crates.io/crates/glossy_codegen)

Example Usage
-------------

In build script `build.rs`:

```
extern crate glossy_codegen as glsl;

void main() {
    glsl::Config::new(glsl::Language::OpenGl)
        .vertex("shaders/*.vert")
        .fragment("shaders/*.frag")
        .include("shaders/include/*")
        .optimize()
        .build();
}
```

In Rust source file `main.rs`:

```
#[macro_use]
extern crate glossy;
extern crate glium;

void main() {
    // ...
    glium::Program::from_source(gl,
                                shader!("sprite.vert"),
                                shader!("sprite.frag"),
                                None)
        .unwrap();
    // ...
}
```

In shader source file `shader.frag`:

```
#version 120
#include "common.glsl"

void main() {
    float v = common_func(common_uniform);
    // ...
}
```
