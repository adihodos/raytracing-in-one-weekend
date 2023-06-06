use std::{ffi::c_void, mem::size_of, ptr::null};

use glfw::{CursorMode, MouseButton, WindowEvent};
use rendering::{
    create_shader_program_from_string, gl, OpenGLStateSnapshot, UniqueBuffer, UniqueBufferMapping,
    UniquePipeline, UniqueSampler, UniqueShaderProgram, UniqueTexture, UniqueVertexArray,
};

struct UiRenderBackend {
    vertex_buffer: UniqueBuffer,
    index_buffer: UniqueBuffer,
    vao: UniqueVertexArray,
    vs: UniqueShaderProgram,
    fs: UniqueShaderProgram,
    pipeline: UniquePipeline,
    font_atlas: UniqueTexture,
    sampler: UniqueSampler,
}

pub struct UiBackend {
    ctx: imgui::Context,
    last_mouse_pos: [f32; 2],
    renderer: UiRenderBackend,
}

impl UiBackend {
    const MAX_VERTICES: u32 = 16384;
    const MAX_INDICES: u32 = Self::MAX_VERTICES * 4;

    pub fn new(window: &glfw::Window) -> UiBackend {
        let mut ctx = imgui::Context::create();

        let (cx, cy) = window.get_cursor_pos();

        let vertex_buffer = UniqueBuffer::new(unsafe {
            let mut buf = 0u32;
            gl::CreateBuffers(1, &mut buf as *mut _);
            gl::NamedBufferStorage(
                buf,
                (Self::MAX_VERTICES as usize * size_of::<imgui::DrawVert>()) as isize,
                null(),
                gl::MAP_WRITE_BIT,
            );

            buf
        })
        .expect("Failed to create vertex buffer");

        let index_buffer = UniqueBuffer::new(unsafe {
            let mut buf = 0u32;
            gl::CreateBuffers(1, &mut buf as *mut _);
            gl::NamedBufferStorage(
                buf,
                Self::MAX_INDICES as isize * size_of::<imgui::DrawIdx>() as isize,
                null(),
                gl::MAP_WRITE_BIT,
            );

            buf
        })
        .expect("Failed to create index buffer");

        let vao = UniqueVertexArray::new(unsafe {
            let mut vao = 0u32;
            gl::CreateVertexArrays(1, &mut vao as *mut _);

            gl::VertexArrayAttribFormat(vao, 0, 2, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayAttribFormat(vao, 1, 2, gl::FLOAT, gl::FALSE, 8);
            gl::VertexArrayAttribFormat(vao, 2, 4, gl::UNSIGNED_BYTE, gl::TRUE, 16);

            gl::VertexArrayAttribBinding(vao, 0, 0);
            gl::VertexArrayAttribBinding(vao, 1, 0);
            gl::VertexArrayAttribBinding(vao, 2, 0);

            gl::EnableVertexArrayAttrib(vao, 0);
            gl::EnableVertexArrayAttrib(vao, 1);
            gl::EnableVertexArrayAttrib(vao, 2);

            gl::VertexArrayVertexBuffer(
                vao,
                0,
                *vertex_buffer,
                0,
                size_of::<imgui::DrawVert>() as i32,
            );
            gl::VertexArrayElementBuffer(vao, *index_buffer);

            vao
        })
        .expect("Failed to create vertex array object");

        let vs_code = include_str!("../../data/shaders/ui.vert");
        let vs = create_shader_program_from_string(vs_code, rendering::ShaderType::Vertex)
            .expect("Failed to create UI vertex shader");

        let fs_code = include_str!("../../data/shaders/ui.frag");
        let fs = create_shader_program_from_string(fs_code, rendering::ShaderType::Fragment)
            .expect("Failed to create UI fragment shader");

        let pipeline = UniquePipeline::new(unsafe {
            let mut pipeline = 0u32;
            gl::GenProgramPipelines(1, &mut pipeline as *mut _);
            gl::UseProgramStages(pipeline, gl::VERTEX_SHADER_BIT, *vs);
            gl::UseProgramStages(pipeline, gl::FRAGMENT_SHADER_BIT, *fs);

            pipeline
        })
        .expect("Failed to create graphics pipeline");

        let font_data = ctx.fonts().build_alpha8_texture();

        let font_atlas = UniqueTexture::new(unsafe {
            let mut tex = 0u32;
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut tex as *mut _);
            gl::TextureStorage2D(
                tex,
                1,
                gl::R8,
                font_data.width as i32,
                font_data.height as i32,
            );
            gl::TextureSubImage2D(
                tex,
                0,
                0,
                0,
                font_data.width as i32,
                font_data.height as i32,
                gl::RED,
                gl::UNSIGNED_BYTE,
                font_data.data.as_ptr() as *const c_void,
            );

            tex
        })
        .expect("Failed to create font atlast texture");

        ctx.fonts().tex_id = imgui::TextureId::new(*font_atlas as usize);

        let sampler = rendering::UniqueSampler::new(unsafe {
            let mut sampler = 0u32;
            gl::CreateSamplers(1, &mut sampler as *mut _);
            gl::SamplerParameteri(sampler, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::SamplerParameteri(
                sampler,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );

            sampler
        })
        .expect("Failed to create sampler");

        UiBackend {
            ctx,
            last_mouse_pos: [cx as f32, cy as f32],
            renderer: UiRenderBackend {
                vertex_buffer,
                index_buffer,
                vao,
                vs,
                fs,
                pipeline,
                font_atlas,
                sampler,
            },
        }
    }

    pub fn new_frame(&mut self, window: &glfw::Window) -> &mut imgui::Ui {
        let (dpy_width, dpy_height) = window.get_size();
        let (fb_width, fb_height) = window.get_framebuffer_size();

        self.ctx.io_mut().display_size = [dpy_width as f32, dpy_height as f32];
        if dpy_width > 0 && dpy_height > 0 {
            self.ctx.io_mut().display_framebuffer_scale = [
                fb_width as f32 / dpy_width as f32,
                fb_height as f32 / dpy_height as f32,
            ];
        }

        self.ctx.io_mut().delta_time = 1f32 / 60f32;
        self.ctx.new_frame()
    }

    pub fn event_handler(&mut self, window: &glfw::Window, event: glfw::WindowEvent) {
        match event {
            WindowEvent::Key(key, _, action, _) => {
                self.update_key_modifiers(window);
                self.ctx
                    .io_mut()
                    .add_key_event(glfw_key_to_imgui_key(key), action == glfw::Action::Press);
            }

            WindowEvent::MouseButton(button, action, _) => {
                self.update_key_modifiers(window);
                let imbtn = match button {
                    MouseButton::Button1 => Some(imgui::MouseButton::Left),
                    MouseButton::Button2 => Some(imgui::MouseButton::Right),
                    MouseButton::Button3 => Some(imgui::MouseButton::Middle),
                    _ => None,
                };

                imbtn.map(|b| {
                    self.ctx
                        .io_mut()
                        .add_mouse_button_event(b, action == glfw::Action::Press);
                });
            }

            WindowEvent::Scroll(xoffset, yoffset) => {
                self.ctx
                    .io_mut()
                    .add_mouse_wheel_event([xoffset as f32, yoffset as f32]);
            }

            WindowEvent::Focus(focus) => {
                self.ctx.io_mut().app_focus_lost = !focus;
            }

            WindowEvent::CursorPos(xpos, ypos) => {
                self.ctx
                    .io_mut()
                    .add_mouse_pos_event([xpos as f32, ypos as f32]);
            }

            WindowEvent::CursorEnter(entered) => {
                if entered {
                    self.ctx.io_mut().add_mouse_pos_event(self.last_mouse_pos);
                } else {
                    self.last_mouse_pos = self.ctx.io().mouse_pos;
                    self.ctx
                        .io_mut()
                        .add_mouse_pos_event([-std::f32::MAX, -std::f32::MAX]);
                }
            }

            WindowEvent::Char(c) => {
                self.ctx.io_mut().add_input_character(c);
            }
            _ => {}
        }
    }

    pub fn render(&mut self) {
        let draw_data = self.ctx.render();

        assert!(
            draw_data.total_vtx_count <= Self::MAX_VERTICES as i32,
            "Vertex buffer overflow"
        );
        assert!(
            draw_data.total_idx_count <= Self::MAX_INDICES as i32,
            "Index buffer overflow"
        );

        let fb_width = (draw_data.display_size[0] * draw_data.framebuffer_scale[0]) as i32;
        let fb_height = (draw_data.display_size[1] * draw_data.framebuffer_scale[1]) as i32;

        if fb_width <= 0 || fb_height <= 0 {
            return;
        }

        UniqueBufferMapping::new(
            *self.renderer.vertex_buffer,
            gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
        )
        .map(|mapping| {
            let _ = draw_data
                .draw_lists()
                .fold(0isize, |offset, draw_list| unsafe {
                    let items = draw_list.vtx_buffer().len() as isize;

                    std::ptr::copy_nonoverlapping(
                        draw_list.vtx_buffer().as_ptr(),
                        (mapping.memory() as *mut imgui::DrawVert).offset(offset),
                        items as usize,
                    );

                    offset + items
                });
        });

        UniqueBufferMapping::new(
            *self.renderer.index_buffer,
            gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
        )
        .map(|mapping| {
            let _ = draw_data
                .draw_lists()
                .fold(0isize, |offset, draw_list| unsafe {
                    let items = draw_list.idx_buffer().len() as isize;

                    std::ptr::copy_nonoverlapping(
                        draw_list.idx_buffer().as_ptr(),
                        (mapping.memory() as *mut imgui::DrawIdx).offset(offset),
                        items as usize,
                    );

                    offset + items
                });
        });

        //
        // setup state
        let prev_gl_state = OpenGLStateSnapshot::new();
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFuncSeparate(
                gl::SRC_ALPHA,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
            );
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::STENCIL_TEST);
            gl::Enable(gl::SCISSOR_TEST);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::ViewportIndexedf(0, 0f32, 0f32, fb_width as f32, fb_height as f32);
        }

        let left = draw_data.display_pos[0];
        let right = draw_data.display_pos[0] + draw_data.display_size[0];
        let top = draw_data.display_pos[1];
        let bottom = draw_data.display_pos[1] + draw_data.display_size[1];

        let ortho_projection: [f32; 16] = [
            //
            2f32 / (right - left),
            0f32,
            0f32,
            0f32,
            //
            0f32,
            2f32 / (top - bottom),
            0f32,
            0f32,
            //
            0f32,
            0f32,
            -1f32,
            0f32,
            //
            (left + right) / (left - right),
            (bottom + top) / (bottom - top),
            0f32,
            1f32,
        ];

        unsafe {
            gl::BindProgramPipeline(*self.renderer.pipeline);

            gl::ProgramUniformMatrix4fv(
                *self.renderer.vs,
                0,
                1,
                gl::FALSE,
                ortho_projection.as_ptr(),
            );

            gl::BindVertexArray(*self.renderer.vao);
            gl::BindTextureUnit(0, *self.renderer.font_atlas);
            gl::BindSampler(0, *self.renderer.sampler);
        }

        let [clip_off_x, clip_off_y] = draw_data.display_pos;
        let [clip_scale_x, clip_scale_y] = draw_data.framebuffer_scale;

        draw_data
            .draw_lists()
            .fold((0, 0), |(index_offset, vertex_offset), draw_list| {
                for draw_cmd in draw_list.commands() {
                    match draw_cmd {
                        imgui::DrawCmd::Elements { count, cmd_params } => {
                            let [clip_rect_x, clip_rect_y, clip_rect_z, clip_rect_w] =
                                cmd_params.clip_rect;

                            let [clip_min_x, clip_min_y] = [
                                (clip_rect_x - clip_off_x) * clip_scale_x,
                                (clip_rect_y - clip_off_y) * clip_scale_y,
                            ];

                            let [clip_max_x, clip_max_y] = [
                                (clip_rect_z - clip_off_x) * clip_scale_x,
                                (clip_rect_w - clip_off_y) * clip_scale_y,
                            ];

                            if clip_max_x <= clip_min_x || clip_max_y <= clip_min_y {
                                continue;
                            }

                            unsafe {
                                // Apply scissor/clipping rectangle (Y is inverted in OpenGL)
                                gl::Scissor(
                                    clip_min_x as i32,
                                    (fb_height as f32 - clip_max_y) as i32,
                                    (clip_max_x - clip_min_x) as i32,
                                    (clip_max_y - clip_min_y) as i32,
                                );
                                gl::BindTextureUnit(0, cmd_params.texture_id.id() as u32);
                                gl::DrawElementsBaseVertex(
                                    gl::TRIANGLES,
                                    count as i32,
                                    gl::UNSIGNED_SHORT,
                                    ((cmd_params.idx_offset + index_offset)
                                        * size_of::<imgui::DrawIdx>())
                                        as *const c_void,
                                    (cmd_params.vtx_offset + vertex_offset) as i32,
                                );
                            }
                        }

                        imgui::DrawCmd::ResetRenderState => {
                            unimplemented!("fix this")
                        }
                        _ => unimplemented!("fix this"),
                    }
                }

                (
                    index_offset + draw_list.idx_buffer().len(),
                    vertex_offset + draw_list.vtx_buffer().len(),
                )
            });
    }

    fn update_key_modifiers(&mut self, window: &glfw::Window) {
        let io = self.ctx.io_mut();

        [
            (
                imgui::Key::ModCtrl,
                glfw::Key::LeftControl,
                glfw::Key::RightControl,
            ),
            (
                imgui::Key::ModShift,
                glfw::Key::LeftShift,
                glfw::Key::RightShift,
            ),
            (imgui::Key::ModAlt, glfw::Key::LeftAlt, glfw::Key::RightAlt),
            (
                imgui::Key::ModSuper,
                glfw::Key::LeftSuper,
                glfw::Key::RightSuper,
            ),
        ]
        .iter()
        .for_each(|&(im_key, left_key, right_key)| {
            io.add_key_event(
                im_key,
                window.get_key(left_key) == glfw::Action::Press
                    || window.get_key(right_key) == glfw::Action::Press,
            );
        });
    }

    // fn update_mouse_data(&mut self, window: &mut glfw::Window) {
    //     if window.get_cursor_mode() == CursorMode::Disabled {
    //         self.ctx
    //             .io_mut()
    //             .add_mouse_pos_event([-std::f32::MAX, -std::f32::MAX]);
    //         return;
    //     }

    //     if window.is_focused() {
    //         if self.ctx.io().want_set_mouse_pos {
    //             let mouse_pos = self.ctx.io().mouse_pos;
    //             window.set_cursor_pos(mouse_pos[0] as f64, mouse_pos[1] as f64);
    //         }
    //     }
    // }
}

fn glfw_key_to_imgui_key(key: glfw::Key) -> imgui::Key {
    match key {
        glfw::Key::Tab => imgui::Key::Tab,
        glfw::Key::Left => imgui::Key::LeftArrow,
        glfw::Key::Right => imgui::Key::RightArrow,
        glfw::Key::Up => imgui::Key::UpArrow,
        glfw::Key::Down => imgui::Key::DownArrow,
        glfw::Key::PageUp => imgui::Key::PageUp,
        glfw::Key::PageDown => imgui::Key::PageDown,
        glfw::Key::Home => imgui::Key::Home,
        glfw::Key::End => imgui::Key::End,
        glfw::Key::Insert => imgui::Key::Insert,
        glfw::Key::Delete => imgui::Key::Delete,
        glfw::Key::Backspace => imgui::Key::Backspace,
        glfw::Key::Space => imgui::Key::Space,
        glfw::Key::Enter => imgui::Key::Enter,
        glfw::Key::Escape => imgui::Key::Escape,
        glfw::Key::Apostrophe => imgui::Key::Apostrophe,
        glfw::Key::Comma => imgui::Key::Comma,
        glfw::Key::Minus => imgui::Key::Minus,
        glfw::Key::Period => imgui::Key::Period,
        glfw::Key::Slash => imgui::Key::Slash,
        glfw::Key::Semicolon => imgui::Key::Semicolon,
        glfw::Key::Equal => imgui::Key::Equal,
        glfw::Key::LeftBracket => imgui::Key::LeftBracket,
        glfw::Key::Backslash => imgui::Key::Backslash,
        glfw::Key::RightBracket => imgui::Key::RightBracket,
        glfw::Key::GraveAccent => imgui::Key::GraveAccent,
        glfw::Key::CapsLock => imgui::Key::CapsLock,
        glfw::Key::ScrollLock => imgui::Key::ScrollLock,
        glfw::Key::NumLock => imgui::Key::NumLock,
        glfw::Key::PrintScreen => imgui::Key::PrintScreen,
        glfw::Key::Pause => imgui::Key::Pause,
        glfw::Key::Kp0 => imgui::Key::Keypad0,
        glfw::Key::Kp1 => imgui::Key::Keypad1,
        glfw::Key::Kp2 => imgui::Key::Keypad2,
        glfw::Key::Kp3 => imgui::Key::Keypad3,
        glfw::Key::Kp4 => imgui::Key::Keypad4,
        glfw::Key::Kp5 => imgui::Key::Keypad5,
        glfw::Key::Kp6 => imgui::Key::Keypad6,
        glfw::Key::Kp7 => imgui::Key::Keypad7,
        glfw::Key::Kp8 => imgui::Key::Keypad8,
        glfw::Key::Kp9 => imgui::Key::Keypad9,
        glfw::Key::KpDecimal => imgui::Key::KeypadDecimal,
        glfw::Key::KpDivide => imgui::Key::KeypadDivide,
        glfw::Key::KpMultiply => imgui::Key::KeypadMultiply,
        glfw::Key::KpSubtract => imgui::Key::KeypadSubtract,
        glfw::Key::KpAdd => imgui::Key::KeypadAdd,
        glfw::Key::KpEnter => imgui::Key::KeypadEnter,
        glfw::Key::KpEqual => imgui::Key::KeypadEqual,
        glfw::Key::LeftShift => imgui::Key::LeftShift,
        glfw::Key::LeftControl => imgui::Key::LeftCtrl,
        glfw::Key::LeftAlt => imgui::Key::LeftAlt,
        glfw::Key::LeftSuper => imgui::Key::LeftSuper,
        glfw::Key::RightShift => imgui::Key::RightShift,
        glfw::Key::RightControl => imgui::Key::RightCtrl,
        glfw::Key::RightAlt => imgui::Key::RightAlt,
        glfw::Key::RightSuper => imgui::Key::RightSuper,
        glfw::Key::Menu => imgui::Key::Menu,
        glfw::Key::Num0 => imgui::Key::Alpha0,
        glfw::Key::Num1 => imgui::Key::Alpha1,
        glfw::Key::Num2 => imgui::Key::Alpha2,
        glfw::Key::Num3 => imgui::Key::Alpha3,
        glfw::Key::Num4 => imgui::Key::Alpha4,
        glfw::Key::Num5 => imgui::Key::Alpha5,
        glfw::Key::Num6 => imgui::Key::Alpha6,
        glfw::Key::Num7 => imgui::Key::Alpha7,
        glfw::Key::Num8 => imgui::Key::Alpha8,
        glfw::Key::Num9 => imgui::Key::Alpha9,
        glfw::Key::A => imgui::Key::A,
        glfw::Key::B => imgui::Key::B,
        glfw::Key::C => imgui::Key::C,
        glfw::Key::D => imgui::Key::D,
        glfw::Key::E => imgui::Key::E,
        glfw::Key::F => imgui::Key::F,
        glfw::Key::G => imgui::Key::G,
        glfw::Key::H => imgui::Key::H,
        glfw::Key::I => imgui::Key::I,
        glfw::Key::J => imgui::Key::J,
        glfw::Key::K => imgui::Key::K,
        glfw::Key::L => imgui::Key::L,
        glfw::Key::M => imgui::Key::M,
        glfw::Key::N => imgui::Key::N,
        glfw::Key::O => imgui::Key::O,
        glfw::Key::P => imgui::Key::P,
        glfw::Key::Q => imgui::Key::Q,
        glfw::Key::R => imgui::Key::R,
        glfw::Key::S => imgui::Key::S,
        glfw::Key::T => imgui::Key::T,
        glfw::Key::U => imgui::Key::U,
        glfw::Key::V => imgui::Key::V,
        glfw::Key::W => imgui::Key::W,
        glfw::Key::X => imgui::Key::X,
        glfw::Key::Y => imgui::Key::Y,
        glfw::Key::Z => imgui::Key::Z,
        glfw::Key::F1 => imgui::Key::F1,
        glfw::Key::F2 => imgui::Key::F2,
        glfw::Key::F3 => imgui::Key::F3,
        glfw::Key::F4 => imgui::Key::F4,
        glfw::Key::F5 => imgui::Key::F5,
        glfw::Key::F6 => imgui::Key::F6,
        glfw::Key::F7 => imgui::Key::F7,
        glfw::Key::F8 => imgui::Key::F8,
        glfw::Key::F9 => imgui::Key::F9,
        glfw::Key::F10 => imgui::Key::F10,
        glfw::Key::F11 => imgui::Key::F11,
        glfw::Key::F12 => imgui::Key::F12,
        _ => imgui::Key::Escape,
    }
}
