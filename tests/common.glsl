#version 120
/* #include "block_dummy.glsl" */
#include "include1.glsl"
// #include "line_dummy.glsl"
#include "include2.glsl" /*
someone made a stupid block comment
#include "block_dummy_multiline.glsl"
*/

void main() {
    float f = common_func();
    gl_Position = vec4(f, f, 0.0, 1.0);
}
