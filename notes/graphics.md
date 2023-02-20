# Graphics

For the initial experiments we are using the WebGl2 bindings from web-sys for rendering.

## Rendering pipeline

## Resources

-  [Uniform function listing](https://webgl2fundamentals.org/webgl/lessons/webgl-shaders-and-glsl.html#:~:text=Uniforms%20can%20be%20many%20types)
- [Some basic composition strategy](https://thebookofshaders.com/)
    - Distance fields seem like a fun choice, but can we draw them on arbitrary polygons?


## Uniforms

In web-sys they'll have another suffix indicating the argument types,
since we don't have function overloading.
We're using the `<method>_with_<type>_array` for now, e.g. `uniform4fv_with_f32_array`,
which just take the uniform location and an `&[f32]`.