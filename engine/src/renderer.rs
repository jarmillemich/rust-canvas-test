use wasm_bindgen::prelude::*;
use web_sys::{
    console, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlUniformLocation,
    WebGlVertexArrayObject,
};

pub fn init_renderer(canvas: &web_sys::HtmlCanvasElement) -> Option<Renderer> {
    let context = canvas
        .get_context("webgl2")
        .unwrap()?
        .dyn_into::<WebGl2RenderingContext>()
        .unwrap();

    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        include_str!("./shaders/circle_test.vert"),
    )
    .expect("Vertex shader should compile");

    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        include_str!("./shaders/circle_test.frag"),
    )
    .expect("Fragment shader should compile");

    let program = link_program(&context, &vert_shader, &frag_shader).unwrap();

    context.use_program(Some(&program));

    // Enable alpha blending
    context.enable(WebGl2RenderingContext::BLEND);
    context.blend_func(
        WebGl2RenderingContext::SRC_ALPHA,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );

    // Setup attributes
    let position_attribute_location = context.get_attrib_location(&program, "position") as u32;
    let color_uniform_location = get_uniform_location(&context, &program, "u_color");
    let resolution_uniform_location = get_uniform_location(&context, &program, "u_resolution");
    let center_uniform_location = get_uniform_location(&context, &program, "u_center");
    let radius_uniform_location = get_uniform_location(&context, &program, "u_radius");

    context.uniform2fv_with_f32_array(
        Some(&resolution_uniform_location),
        &[canvas.width() as f32, canvas.height() as f32],
    );

    console::log_1(&format!("{}, {}", canvas.width(), canvas.height()).into());

    // Setup VAO
    let vao = context
        .create_vertex_array()
        .ok_or("Could not create vertex array object")
        .unwrap();
    context.bind_vertex_array(Some(&vao));

    // Attach buffers to VAO
    attach_buffer(
        &context,
        position_attribute_location,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );

    Some(Renderer {
        context,
        program,

        position_attribute_location,
        color_uniform_location,
        center_uniform_location,
        radius_uniform_location,
        vao,
    })
}

/// Creates and attaches a buffer to the currently active VAO
fn attach_buffer(
    context: &WebGl2RenderingContext,
    location: u32,
    size: i32,
    field_type: u32,
    normalized: bool,
    stride: i32,
    offset: i32,
) -> WebGlBuffer {
    let buffer = context
        .create_buffer()
        .ok_or("Failed to create buffer")
        .unwrap();
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    context.enable_vertex_attrib_array(location);
    context.vertex_attrib_pointer_with_i32(location, size, field_type, normalized, stride, offset);
    buffer
}

fn get_uniform_location(
    context: &WebGl2RenderingContext,
    program: &WebGlProgram,
    name: &str,
) -> WebGlUniformLocation {
    context
        .get_uniform_location(program, name)
        .expect(&("Should have ".to_owned() + name))
}

pub struct Renderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,

    // Shader attributes
    position_attribute_location: u32,
    color_uniform_location: WebGlUniformLocation,
    center_uniform_location: WebGlUniformLocation,
    radius_uniform_location: WebGlUniformLocation,
    vao: WebGlVertexArrayObject,
}

fn write_to_buffer(context: &WebGl2RenderingContext, vertices: &[f32]) {
    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(vertices);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }
}

impl Renderer {
    pub fn draw(&mut self, x: f32, y: f32, radius: f32, color: [f32; 4]) {
        // Create our vertices
        let d = 1.414 * radius;
        let vertices = [x - d, y - d, x - d, y + d, x + d, y - d, x + d, y + d];

        let context = &self.context;

        context.use_program(Some(&self.program));
        context.bind_vertex_array(Some(&self.vao));

        // Write to uniforms
        // TODO update this uniform on canvas resizes
        //context.uniform2fv_with_f32_array(Some(&self.resolution_uniform_location), &[context.canvas().unwrap().])
        context.uniform4fv_with_f32_array(Some(&self.color_uniform_location), &color);
        context.uniform2fv_with_f32_array(Some(&self.center_uniform_location), &[x, y]);
        context.uniform1f(Some(&self.radius_uniform_location), radius);

        // Write to vertices
        // We may need to re-select the position buffer here, but for the moment we only have the one
        write_to_buffer(context, &vertices);

        context.enable_vertex_attrib_array(self.position_attribute_location);
        context.vertex_attrib_pointer_with_i32(
            self.position_attribute_location,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );

        context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);
    }
}

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
