//! GPU render state.
//!
//! Such a state controls how the GPU must operate some fixed pipeline functionality, such as the
//! blending, depth test or face culling operations.

use crate::blending::{Blending, BlendingMode};
use crate::depth_test::{DepthComparison, DepthWrite};
use crate::face_culling::FaceCulling;

/// GPU render state.
///
/// You can get a default value with `RenderState::default` and set the operations you want with the
/// various `RenderState::set_*` methods.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderState {
  /// Blending configuration.
  blending: Option<BlendingMode>,
  /// Depth test configuration.
  depth_test: Option<DepthComparison>,
  /// Depth write configuration.
  depth_write: DepthWrite,
  /// Face culling configuration.
  face_culling: Option<FaceCulling>,
}

impl RenderState {
  /// Override the blending configuration.
  pub fn set_blending<B>(self, blending: B) -> Self
  where
    B: Into<Option<Blending>>,
  {
    RenderState {
      blending: blending.into().map(|x| x.into()),
      ..self
    }
  }

  /// Override the blending configuration using separate blending.
  pub fn set_blending_separate(self, blending_rgb: Blending, blending_alpha: Blending) -> Self {
    RenderState {
      blending: Some(BlendingMode::Separate {
        rgb: blending_rgb,
        alpha: blending_alpha,
      }),
      ..self
    }
  }

  /// Blending configuration.
  pub fn blending(&self) -> Option<BlendingMode> {
    self.blending
  }

  /// Override the depth test configuration.
  pub fn set_depth_test<D>(self, depth_test: D) -> Self
  where
    D: Into<Option<DepthComparison>>,
  {
    let depth_test = depth_test.into();
    RenderState { depth_test, ..self }
  }

  /// Depth test configuration.
  pub fn depth_test(&self) -> Option<DepthComparison> {
    self.depth_test
  }

  /// Override the depth write configuration.
  pub fn set_depth_write(self, depth_write: DepthWrite) -> Self {
    RenderState {
      depth_write,
      ..self
    }
  }

  /// Depth write configuration.
  pub fn depth_write(&self) -> DepthWrite {
    self.depth_write
  }

  /// Override the face culling configuration.
  pub fn set_face_culling<FC>(self, face_culling: FC) -> Self
  where
    FC: Into<Option<FaceCulling>>,
  {
    RenderState {
      face_culling: face_culling.into(),
      ..self
    }
  }

  /// Face culling configuration.
  pub fn face_culling(&self) -> Option<FaceCulling> {
    self.face_culling
  }
}

impl Default for RenderState {
  /// The default `RenderState`.
  ///
  ///   - `blending`: `None`
  ///   - `depth_test`: `Some(DepthComparison::Less)`
  ///   - `depth_write`: `DepthWrite::On`
  ///   - `face_culling`: `None`
  fn default() -> Self {
    RenderState {
      blending: None,
      depth_test: Some(DepthComparison::Less),
      depth_write: DepthWrite::On,
      face_culling: None,
    }
  }
}
