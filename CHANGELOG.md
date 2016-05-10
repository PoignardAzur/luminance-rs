## 0.4.0

- Changed the whole `Program` API to make it safer. Now, it closely looks like the Haskell version
  of `luminance`. The idea is that the user cannot have `Uniform`s around anymore as they’re held by
  `Program`s. Furthermore, the *uniform interface* is introduced. Because Rust doesn’t have a
  **“rank-2 types”** mechanism as Haskell, `ProgramProxy` is introduced to emulate such a behavior.
- Added a bit of documentation for shader programs.

### 0.3.1

- Removed `rw`.

## 0.3.0

- Removed A type parameter form `Buffer`. It was unnecessary safety that was never actually used.
- Added documentation around.

### 0.2.1

- Exposed `Vertex` directly in `luminance`.

## 0.2.0

- Changed `Negative*` blending factors to `*Complement`.

## 0.1.0

- Initial revision.