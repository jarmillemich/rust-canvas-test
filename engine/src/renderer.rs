use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};
use wasm_bindgen::prelude::*;


pub fn init_renderer(canvas: &web_sys::HtmlCanvasElement) -> Option<Renderer> {
    let context = canvas
        .get_context("webgl2")
        .unwrap()?
        .dyn_into::<WebGl2RenderingContext>()
        .unwrap();

    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es
 
        in vec4 position;
        in vec4 color;

        out vec4 v_color;

        void main() {
        
            gl_Position = position;
            v_color = color;
        }
        "##,
    ).unwrap();

    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
    
        precision highp float;
        out vec4 outColor;
        in vec4 v_color;
        
        void main() {

            //outColor = vec4(1, 0.5, 0.0, 0.5);
            outColor = v_color;
        }
        "##,
    ).unwrap();

    let program = link_program(&context, &vert_shader, &frag_shader).unwrap();

    context.use_program(Some(&program));

    let position_attribute_location = context.get_attrib_location(&program, "position") as u32;
    let color_attribute_location = context.get_attrib_location(&program, "color") as u32;
    
    Some(Renderer {
        context,
        program,

        position_attribute_location,
        color_attribute_location,
    })
}

pub struct Renderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,

    // Shader attributes
    position_attribute_location: u32,
    color_attribute_location: u32,
}

impl Renderer {
    pub fn draw(&mut self, vertices: [f32; 6], color: [f32; 4]) {
        let context = &self.context;

        
        let color_buffer = context.create_buffer().ok_or("Failed to create buffer").unwrap();
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&color_buffer));

        unsafe {
            let color_array_buf_view = js_sys::Float32Array::view(&color);

            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &color_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        

        
        let buffer = context.create_buffer().ok_or("Failed to create buffer").unwrap();
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let positions_array_buf_view = js_sys::Float32Array::view(&vertices);

            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &positions_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        let vao = context
            .create_vertex_array()
            .ok_or("Could not create vertex array object")
            .unwrap();
        context.bind_vertex_array(Some(&vao));

        context.enable_vertex_attrib_array(self.position_attribute_location);
        context.vertex_attrib_pointer_with_i32(
            self.position_attribute_location,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        
        let vao = context
            .create_vertex_array()
            .ok_or("Could not create vertex array object")
            .unwrap();
        context.bind_vertex_array(Some(&vao));

        context.enable_vertex_attrib_array(self.color_attribute_location);
        context.vertex_attrib_pointer_with_i32(self.color_attribute_location, 4, WebGl2RenderingContext::FLOAT, true, 0, 0);
        
        
        
        //context.clear_color(0.0, 0.0, 0.0, 1.0);
        //context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    
        context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);
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