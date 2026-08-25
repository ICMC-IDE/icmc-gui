use super::ViewState;
use crate::{State, resources::charmap::Charmap, resources::keyboard::keycode};
use eframe::{
    egui_glow,
    glow::{self},
};
use egui_dock::egui;
use std::sync::{Arc, Mutex};

pub struct ScrCanvas {
    program: glow::Program,
    vao: glow::VertexArray,
    _vbo: glow::Buffer,
    instance_vbo: glow::Buffer,
    texture: glow::Texture,
    cols: u32,
    lines: u32,
    char_width: u32,
    char_height: u32,
    last_vram: Option<Vec<u8>>,
}

pub struct Screen {
    canvas: Arc<Mutex<ScrCanvas>>,
}

impl ScrCanvas {
    pub fn new(gl: &glow::Context, charmap: &Charmap) -> Self {
        use glow::HasContext as _;

        unsafe {
            #[cfg(target_arch = "wasm32")]
            let vertex_shader_source =
                include_str!("../../res/shaders/vertex_es.glsl");

            #[cfg(target_arch = "wasm32")]
            let fragment_shader_source =
                include_str!("../../res/shaders/fragment_es.glsl");

            #[cfg(not(target_arch = "wasm32"))]
            let vertex_shader_source =
                include_str!("../../res/shaders/vertex.glsl");

            #[cfg(not(target_arch = "wasm32"))]
            let fragment_shader_source =
                include_str!("../../res/shaders/fragment.glsl");

            let program = gl.create_program().expect("Cannot create program");
            let vs = gl
                .create_shader(glow::VERTEX_SHADER)
                .expect("Cannot create vertex shader");
            gl.shader_source(vs, vertex_shader_source);
            gl.compile_shader(vs);

            if !gl.get_shader_compile_status(vs) {
                let info = gl.get_shader_info_log(vs);
                panic!("Couldn't compile vertex shader: {}", info);
            }

            gl.attach_shader(program, vs);

            let fs = gl
                .create_shader(glow::FRAGMENT_SHADER)
                .expect("Cannot create fragment shader");
            gl.shader_source(fs, fragment_shader_source);
            gl.compile_shader(fs);

            if !gl.get_shader_compile_status(fs) {
                let info = gl.get_shader_info_log(fs);
                panic!("Coudldn't compile fragment shader: {}", info);
            }

            gl.attach_shader(program, fs);

            gl.link_program(program);
            gl.delete_shader(vs);
            gl.delete_shader(fs);

            let vao = gl.create_vertex_array().expect("Cannot create VAO");
            gl.bind_vertex_array(Some(vao));

            let vbo = gl.create_buffer().expect("Cannot create VBO");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let char_height = 8;
            let char_width = 8;
            let quad_vertices: [f32; 24] = [
                0.0,
                char_height as f32,
                0.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                char_width as f32,
                0.0,
                1.0,
                0.0,
                0.0,
                char_height as f32,
                0.0,
                1.0,
                char_width as f32,
                0.0,
                1.0,
                0.0,
                char_width as f32,
                char_height as f32,
                1.0,
                1.0,
            ];
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&quad_vertices),
                glow::STATIC_DRAW,
            );
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 4, glow::FLOAT, false, 0, 0);

            let instance_vbo =
                gl.create_buffer().expect("Cannot create instance VBO");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(instance_vbo));
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_i32(1, 2, glow::UNSIGNED_BYTE, 0, 0);
            gl.vertex_attrib_divisor(1, 1);

            let texture = gl.create_texture().expect("Cannot create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                2048,
                2048,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(charmap.pixels())),
            );

            Self {
                program,
                vao,
                _vbo: vbo,
                instance_vbo,
                texture,
                cols: 40,
                lines: 30,
                char_width: 8,
                char_height: 8,
                last_vram: None,
            }
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;

        unsafe {
            gl.delete_program(self.program);
        }
    }

    fn draw(&mut self, gl: &glow::Context, vram: &[u8], charmap: Option<&Charmap>) {
        use glow::HasContext as _;

        unsafe {
            if let Some(charmap) = charmap {
                gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_WRAP_S,
                    glow::CLAMP_TO_EDGE as i32,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_WRAP_T,
                    glow::CLAMP_TO_EDGE as i32,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::LINEAR as i32,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MAG_FILTER,
                    glow::LINEAR as i32,
                );
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA as i32,
                    2048,
                    2048,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(charmap.pixels())),
                );
            }

            gl.use_program(Some(self.program));

            gl.bind_vertex_array(Some(self.vao));

            if self.last_vram.as_deref() != Some(vram) {
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.instance_vbo));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    vram,
                    glow::DYNAMIC_DRAW,
                );
                self.last_vram = Some(vram.to_vec());
            }

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "tex").as_ref(),
                0,
            );
            gl.uniform_1_u32(
                gl.get_uniform_location(self.program, "line_cells").as_ref(),
                self.cols,
            );
            gl.uniform_2_u32(
                gl.get_uniform_location(self.program, "character_res")
                    .as_ref(),
                self.char_width,
                self.char_height,
            );

            let width = (self.cols * self.char_width) as f32;
            let height = (self.lines * self.char_height) as f32;

            let ortho = [
                2.0 / width,
                0.0,
                0.0,
                0.0,
                0.0,
                -2.0 / height,
                0.0,
                0.0,
                0.0,
                0.0,
                2.0 / 1000.0,
                0.0,
                -1.0,
                1.0,
                0.0,
                1.0,
            ];
            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(self.program, "projection").as_ref(),
                false,
                &ortho,
            );

            gl.draw_arrays_instanced(
                glow::TRIANGLES,
                0,
                6,
                (self.cols * self.lines) as i32,
            );
        }
    }
}

impl Screen {
    pub fn new(cc: &eframe::CreationContext<'_>, charmap: &Charmap) -> Self {
        let gl = cc.gl.as_ref().expect("Context error");

        Self {
            canvas: Arc::new(Mutex::new(ScrCanvas::new(gl, charmap))),
        }
    }

    pub fn destroy(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.canvas.lock().unwrap().destroy(gl);
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui, state: &mut State) {
        let size = ui.available_width().min(ui.available_height()) - 25.0;
        let square_size = egui::Vec2::splat(size);
        ui.set_min_size(square_size);
        ui.set_max_size(square_size);

        let (_, rect) = ui.allocate_space(square_size);
        let canvas = self.canvas.clone();
        let emulator = state.emulator.clone();

        let charmap = if state.settings.charmap.needs_reload {
            state.settings.charmap.needs_reload = false;
            Some(state.settings.charmap.clone())
        } else {
            None
        };

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(
                move |_info, painter| {
                    let emu = emulator.lock().unwrap();
                    let vram: &[u8] = bytemuck::cast_slice(emu.vram());

                    canvas.lock().expect("Couldn't unlock canvas").draw(
                        painter.gl(),
                        vram,
                        charmap.as_ref(),
                    );
                },
            )),
        };
        ui.painter().add(callback);
    }
}

/* todo: render charmap into canvas */
impl ViewState for Screen {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        /* handle keyboard input */
        if state.settings.input_enabled {
            ui.input(|i| {
                for key in egui::Key::ALL {
                    if i.key_pressed(*key) {
                        let keycode = keycode(key);
                        let mut emu = state.emulator.lock().unwrap();

                        *emu.ireg_as_mut_ref(0x4) = if !i.modifiers.shift
                            && keycode >= 65
                            && keycode <= 90
                        {
                            keycode + 32
                        } else {
                            keycode
                        }
                    }

                    if i.key_released(*key) {
                        let _keycode = keycode(key);
                        let mut emu = state.emulator.lock().unwrap();

                        *emu.ireg_as_mut_ref(0x4) = 0xff;
                    }
                }
            });
        }

        ui.add_space(10.0);
        ui.vertical_centered(|ui| {
            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                self.draw(ui, state);
            });
        });
    }
}
