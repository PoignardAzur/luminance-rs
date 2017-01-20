//! Dynamic rendering pipelines.
//!
//! This module gives you materials to build *dynamic* rendering **pipelines**. A `Pipeline`
//! represents a functional stream that consumes geometric data and rasterizes them.

use gl;
use gl::types::*;

use buffer::RawBuffer;
use blending;
use framebuffer::{ColorSlot, DepthSlot, Framebuffer};
use shader::program::Program;
use tess::Tess;
use texture::{Dimensionable, Layerable, RawTexture};

/// A dynamic rendering pipeline. A *pipeline* is responsible of rendering into a `Framebuffer`.
///
/// `L` refers to the `Layering` of the underlying `Framebuffer`.
///
/// `D` refers to the `Dim` of the underlying `Framebuffer`.
///
/// `CS` and `DS` are – respectively – the *color* and *depth* `Slot` of the underlying
/// `Framebuffer`.
pub struct Pipeline<'a, L, D, CS, DS>
    where L: 'a + Layerable,
          D: 'a + Dimensionable,
          D::Size: Copy,
          CS: 'a + ColorSlot<L, D>,
          DS: 'a + DepthSlot<L, D> {
  /// The embedded framebuffer.
  framebuffer: &'a Framebuffer<L, D, CS, DS>,
  /// The color used to clean the framebuffer when  executing the pipeline.
  clear_color: [f32; 4],
  /// Texture set.
  texture_set: &'a[&'a RawTexture],
  /// Buffer set.
  buffer_set: &'a[&'a RawBuffer],
  /// Shading commands to render into the embedded framebuffer.
  shading_commands: Vec<Pipe<'a, ShadingCommand<'a>>>
}

impl<'a, L, D, CS, DS> Pipeline<'a, L, D, CS, DS>
    where L: 'a + Layerable,
          D: 'a + Dimensionable,
          D::Size: Copy,
          CS: 'a + ColorSlot<L, D>,
          DS: 'a + DepthSlot<L, D> {
  /// Create a new pipeline.
  pub fn new(framebuffer: &'a Framebuffer<L, D, CS, DS>, clear_color: [f32; 4],
             texture_set: &'a[&'a RawTexture], buffer_set: &'a[&'a RawBuffer],
             shading_commands: Vec<Pipe<'a, ShadingCommand<'a>>>) -> Self {
    Pipeline {
      framebuffer: framebuffer,
      clear_color: clear_color,
      texture_set: texture_set,
      buffer_set: buffer_set,
      shading_commands: shading_commands
    }
  }

  /// Run a `Pipeline`.
  pub fn run(&self) {
    let clear_color = self.clear_color;

    unsafe {
      gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer.handle());
      gl::Viewport(0, 0, self.framebuffer.width() as GLint, self.framebuffer.height() as GLint);
      gl::ClearColor(clear_color[0], clear_color[1], clear_color[2], clear_color[3]);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

      // traverse the texture set and bind required textures
      for (unit, tex) in self.texture_set.iter().enumerate() {
        gl::ActiveTexture(gl::TEXTURE0 + unit as GLenum);
        gl::BindTexture(tex.target(), tex.handle());
      }

      // traverse the buffer set and bind required buffers
      for (index, buf) in self.buffer_set.iter().enumerate() {
        gl::BindBufferBase(gl::UNIFORM_BUFFER, index as GLuint, buf.handle());
      }
    }

    for piped_shading_cmd in &self.shading_commands {
      Self::run_shading_command(piped_shading_cmd);
    }
  }

  fn run_shading_command(piped: &Pipe<'a, ShadingCommand>) {
    let update_program = &piped.update_program;
    let shading_cmd = &piped.next;

    unsafe { gl::UseProgram(shading_cmd.program.handle()) };

    update_program(&shading_cmd.program);

    for piped_render_cmd in &shading_cmd.render_commands {
      Self::run_render_command(&shading_cmd.program, piped_render_cmd);
    }
  }

  fn run_render_command(program: &Program, piped: &Pipe<'a, RenderCommand<'a>>) {
    let update_program = &piped.update_program;
    let render_cmd = &piped.next;

    update_program(program);

    set_blending(render_cmd.blending);
    set_depth_test(render_cmd.depth_test);

    for piped_tess in &render_cmd.tessellations {
      let tess_update_program = &piped_tess.update_program;
      let tess = &piped_tess.next;

      tess_update_program(program);

      tess.render(render_cmd.rasterization_size, render_cmd.instances);
    }
  }
}

fn set_blending(blending: Option<(blending::Equation, blending::Factor, blending::Factor)>) {
  match blending {
    Some((equation, src_factor, dest_factor)) => {
      unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendEquation(opengl_blending_equation(equation));
        gl::BlendFunc(opengl_blending_factor(src_factor), opengl_blending_factor(dest_factor));
      }
    },
    None => {
      unsafe { gl::Disable(gl::BLEND) };
    }
  }
}

fn set_depth_test(test: bool) {
  unsafe {
    if test {
      gl::Enable(gl::DEPTH_TEST);
    } else {
      gl::Disable(gl::DEPTH_TEST);
    }
  }
}

fn opengl_blending_equation(equation: blending::Equation) -> GLenum {
  match equation {
    blending::Equation::Additive => gl::FUNC_ADD,
    blending::Equation::Subtract => gl::FUNC_SUBTRACT,
    blending::Equation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
    blending::Equation::Min => gl::MIN,
    blending::Equation::Max => gl::MAX
  }
}

fn opengl_blending_factor(factor: blending::Factor) -> GLenum {
  match factor {
    blending::Factor::One => gl::ONE,
    blending::Factor::Zero => gl::ZERO,
    blending::Factor::SrcColor => gl::SRC_COLOR,
    blending::Factor::SrcColorComplement => gl::ONE_MINUS_SRC_COLOR,
    blending::Factor::DestColor => gl::DST_COLOR,
    blending::Factor::DestColorComplement => gl::ONE_MINUS_DST_COLOR,
    blending::Factor::SrcAlpha => gl::SRC_ALPHA,
    blending::Factor::SrcAlphaComplement => gl::ONE_MINUS_SRC_ALPHA,
    blending::Factor::DstAlpha => gl::DST_ALPHA,
    blending::Factor::DstAlphaComplement => gl::ONE_MINUS_DST_ALPHA,
    blending::Factor::SrcAlphaSaturate => gl::SRC_ALPHA_SATURATE
  }
}

/// A dynamic *shading command*. A shading command gathers *render commands* under a shader
/// `Program`.
pub struct ShadingCommand<'a> {
  /// Embedded program.
  pub program: &'a Program,
  /// Render commands to execute for this shading command.
  pub render_commands: Vec<Pipe<'a, RenderCommand<'a>>>
}

impl<'a> ShadingCommand<'a> {
  /// Create a new shading command.
  pub fn new(program: &'a Program, render_commands: Vec<Pipe<'a, RenderCommand<'a>>>) -> Self {
    ShadingCommand {
      program: program,
      render_commands: render_commands
    }
  }
}

/// A render command, which holds information on how to rasterize tessellations.
pub struct RenderCommand<'a> {
  /// Color blending configuration. Set to `None` if you don’t want any color blending. Set it to
  /// `Some(equation, source, destination)` if you want to perform a color blending with the
  /// `equation` formula and with the `source` and `destination` blending factors.
  pub blending: Option<(blending::Equation, blending::Factor, blending::Factor)>,
  /// Should a depth test be performed?
  pub depth_test: bool,
  /// The embedded tessellations.
  pub tessellations: Vec<Pipe<'a, &'a Tess>>,
  /// Number of instances of the tessellation to render.
  pub instances: u32,
  /// Rasterization size for points and lines.
  pub rasterization_size: Option<f32>
}

impl<'a> RenderCommand<'a> {
  /// Create a new render command.
  pub fn new(blending: Option<(blending::Equation, blending::Factor, blending::Factor)>,
             depth_test: bool, tessellations: Vec<Pipe<'a, &'a Tess>>, instances: u32,
             rasterization_size: Option<f32>) -> Self {
    RenderCommand {
      blending: blending,
      depth_test: depth_test,
      tessellations: tessellations,
      instances: instances,
      rasterization_size: rasterization_size
    }
  }
}

/// A pipe used to build up a `Pipeline` by connecting its inner layers.
pub struct Pipe<'a, T> {
  pub update_program: Box<Fn(&Program) + 'a>,
  pub next: T
}

impl<'a, T> Pipe<'a, T> {
  pub fn new<F>(update_program: F, next: T) -> Self where F: Fn(&Program) + 'a {
    Pipe {
      update_program: Box::new(update_program),
      next: next
    }
  }
}
