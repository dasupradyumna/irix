# Core primitives and semantics

**TODO: add crate stability and other details**

## Normative Semantics

This section establishes the **authoritative semantic foundation** of the **irix** ecosystem.  
No higher-tier crate is allowed to reinterpret, relax, or silently override these meanings.  
The structs in this crate must be designed such that semantic violation is a compile-time error.

### Stability Guarantees

All semantics in this document are **frozen from the first minor release `vX.0.0`**, with one
exception - `v0.Y.Z`.

- Any deviation is a *breaking* semantic change and requires a *major version* bump.
- Future minor releases are allowed to add new primitive structs and semantic markers
- They are also allowed to optimize and modify implementations
- They are **not** allowed to:
  - reinterpret defined semantics
  - break API or function signatures
  - change coordinate or layout meaning
  - weaken existing invariants

#### `v0.Y.Z` Semantic Stability
<!-- TODO: remove this section for v1.0.0 release -->

[SemVer](https://semver.org/#spec-item-4) allows flexibility for **v0** to not provide any API
stability guarantees since it is meant for initial development. **irix** will adopt a moderately
stricter approach, by allowing changes to the core semantics but it must be **tightly documented and
justified**. Additionally, all feature development must be suspended and the proposed changes must
be integrated on priority into all feature branches as soon as they are available in **main**.

---
### 1. Tensor Semantics

#### 1.1 Axis Ordering
- A tensor has ordered positional axes with **no** intrinsic semantic meaning.
- Axis indices are interpreted *strictly* by position:
  - Axis `0`: outermost dimension
  - Axis `N-1`: innermost dimension

#### 1.2 Layout Semantics
- Layout markers simply define how multi-dimensional indices map to linear memory.
- Memory layout can either be **row-major** (C-order) or **column-major** (Fortran-order).
  - Row major format implies axis `0` changes slowest and axis `N-1` changes fastest.
  - Column major format implies axis `N-1` changes slowest and axis `0` changes fastest.
- Layout meaning is purely semantic, not an optimization hint.

#### 1.3 Shape, Offset, and Strides
- `shape`, `offset`, and `strides` together define the tensor’s **memory interpretation**.
- `shape` specifies the extent of each axis in index space.
- `offset` is a non-negative index into the underlying buffer indicating where the first logical
  element `(0, ..., 0)` is located.
- `strides` specify the increment in buffer indices required to advance by one element along each
  axis.
- A tensor is **contiguous** iff its strides exactly match those implied by its declared layout and
  shape.
- APIs that require contiguity must **explicitly check** for it and reject non-conforming tensors.

#### 1.4 Mutability and Aliasing

- Tensors may *share underlying storage* via views created by slicing, reshaping, or transposition.
- Shared storage is managed via **explicit buffer ownership** semantics; memory remains valid as
  long as any tensor references it.
- Aliasing is a *defined* and *intentional* property of tensor views, never an implicit side effect.
- Mutation through aliased tensors is **restricted**:
  - APIs must explicitly document whether mutation is permitted on shared storage.
  - Operations requiring exclusive access must either enforce uniqueness or perform a copy-on-write.
  - No API may assume exclusive ownership of the underlying buffer unless this is explicitly
    guaranteed.

#### 1.5 Construction Rules
- Safe constructors must validate all layout and shape invariants.
- Unsafe constructors may assume correctness but must be **explicitly** marked.
- No API may silently reinterpret tensor layout, axes, or strides.

---
### 2. Image Semantics

#### 2.1 Image Coordinate System
- Images use a **2D Cartesian** coordinate system, with *only* non-negative numbers.
- The origin `(0, 0)` is located at the **top-left corner** of the image domain.
- **X-axis** increases rightward, while **Y-axis** increases downward.

#### 2.2 Discrete vs Continuous Coordinates
- Coordinates can be expressed in either *discrete* `[i, j]` or *continuous* spaces `(x, y)`.
- Integer coordinates represent **pixel centers** (not corners) and are used for read-write storage.
  - A discrete pixel coordinate has its center at continuous coordinate `x = i + 0.5, y = j + 0.5`.
- Floating-point coordinates represent **continuous image space**, mainly used in image operations.
  - A continuous coordinate belongs to the pixel at `i = round(x - 0.5), j = round(y - 0.5)`.
- All geometric operations (*sampling, projection, feature detection*) assume this convention.
- Conversions between discrete and continuous space must be explicit and documented.

#### 2.3 Color Channel Semantics
- **Channel order** and **color space** are independent semantic dimensions.
- Channel order defines memory layout (e.g. RGB vs BGR) - applies only to RGB spaces.
- Color space defines meaning of each pixel value (e.g. RGB, RGBA, HSV, YUV, Grayscale).
- No API may conflate or silently convert between color spaces.
- Color conversion is always explicit and loss-aware.

#### 2.4 Image Axes Meaning
- Image pixels are conceptually indexed as `(y,x,c)` or `(H,W,C)`
  - `y`: vertical spatial axis - height `H`
  - `x`: horizontal spatial axis - width `W`
  - `c`: channel axis
- Any alternative ordering must be encoded in the type system and documented.

#### 2.5 Construction Rules
- Public constructors of image structs are **safe** and exposed *only* via semantic wrappers like
  `RgbImage`, `GrayImage`.
- Construction succeeds **iff** all image invariants hold:
  - channel count matches the color space.
  - spatial dimensions are positive.
  - axis meaning and layout conform to image semantics
- Image semantics are preserved across operations unless explicitly documented otherwise.

---
### 3. Image–Tensor Relationship

#### 3.1 Shared Storage Model
- Images are a **semantic specialization of tensors**.
- An image internally stores pixel data using a constrained tensor representation.

#### 3.2 Lowering - `Image >> Tensor`
- Lowering is an explicit *semantic erasure* operation, where image-specific guarantees are dropped.
- In the context of the underlying data, this operation preserves:
  - memory layout
  - coordinate meaning
  - channel interpretation

#### 3.3 Lifting - `Tensor >> Image`
- Lifting is an explicit, checked, and *fallible* operation.
- A tensor may be lifted iff it is rank-3 with axes `(y, x, c)`, and channel count matches the
  target color space.
- Lifting must target a concrete image wrapper like `to_rgb_image()` or `to_gray_image()`.
  - A generic `to_image()` API is **forbidden**, since users must not be exposed to `Image`.

---
### 4. Coordinate & Geometry Semantics

#### 4.1 Coordinate Frames
- All geometric quantities are expressed in named coordinate frames.
- Frame identity must be part of the type system.
- No implicit frame conversion is permitted.
- All coordinate systems are **right-handed**.

#### 4.2 Pose Semantics
- A pose represents a **rigid body transform** in `SE(3)`.
- Pose direction is explicit: `Pose<A, B>` transforms coordinates from frame `A` to frame `B`.
- Pose composition order is `Pose<A, B> . Pose<B, C> = Pose<A, C>`

#### 4.3 Units
- Default translation units are *meters*, while rotation units are *radians*.
- Other units must be specified **explicitly** by the type system.
- Mixed or unit-less geometry is forbidden.

---
### 5. Camera Semantics

#### 5.1 Camera Model
- Camera models must describe how they project points to pixel coordinates, and vice versa.
- Camera model objects are **immutable** once they are constructed.
  - Invalid parameter ranges are rejected at construction.
- Parameter meaning (focal length, distortion coefficients) is explicitly documented with the type.

#### 5.2 Camera Coordinate Frame
- Camera frame origin is the principal point of focus in the camera model.
- Camera frame axes directions:
  - **+Z** forward (from origin into the scene)
  - **+X** right and **+Y** down (when viewing along +Z)
- The image plane (*parallel to XY plane*) is placed at `Z = f`, where `f` is the focal length.
  - Image plane coordinate XY axes are parallel to XY axes of the camera coordinate frame.

#### 5.3 Transform Semantics
```
World frame (3D) <=A=> Camera frame (3D) <=B=> Image (2D continuous) <=C=> Image (2D discrete)
```
- `A` is an *invertible* 3D transformation, changing the coordinate frame.
  - 3D World frame to 3D Camera frame is called **camera extrinsics**.
  - 3D Camera frame to 3D World frame is called **camera pose**.
- `B` is a *projective* transformation from 3D to 2D, while the inverse is unique only upto a scale.
- `C` is a pixel *quantization* policy between discrete and continuous spaces described in
  [Section 2.2](#22-discrete-vs-continuous-coordinates)

---
### 6. Feature & Keypoint Semantics

#### 6.1 Keypoint Semantics
- Keypoints are expressed in **continuous** image coordinates.
- Coordinates follow the image pixel-center convention.
- Subpixel values represent continuous spatial location, *not* interpolation hints.
- Keypoints must also specify scale and orientation values, if and when applicable.

#### 6.2 Feature Semantics
- Feature descriptor **dimensionality** is part of the type.
- Descriptor values have no assumed normalization unless explicitly documented.
  - Distance metrics must match descriptor semantics.
- Keypoints and descriptors must reference the same image semantics.
- Mixing coordinate systems or image domains is forbidden.

---
### 7. Error Semantics

#### 7.1 Invariant Violations
- Violations of semantic invariants are **explicit errors**, not undefined behavior.
- Errors are categorized as:
  - construction-time errors
  - conversion-time errors
  - runtime precondition failures

#### 7.2 Panic Policy
- Panics are reserved for:
  - internal logic errors
  - violations of documented unsafe contracts
- User-facing APIs must not panic on invalid input.

#### 7.3 Error Transparency
- Errors must be descriptive and non-generic.
- No error may silently mask semantic violations.
