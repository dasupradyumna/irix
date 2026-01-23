# `irix-core` - Core primitives and semantics



## Normative Semantics

The following section establishes the **authoritative semantic foundation** of the *irix* ecosystem.  
No higher-tier crate is allowed to reinterpret, relax, or silently override these meanings.  
The structs in this crate must be designed such that semantic violation is a compile-time error.

### Stability Guarantees

- All semantics in this document are **frozen from the first minor release vX.0.0**, with
  [exceptions]().
  - Any deviation constitutes a breaking semantic change, i.e. a major version bump.
- Future minor releases are bound by the following rules.
  - They are allowed to add:
    - add new semantic types
    - add new markers
  - They may **not**:
    - reinterpret existing semantics
    - weaken invariants
    - change coordinate or layout meaning

##### IMPORTANT: v0.X.Y Semantic Stability

---

### 1. Image Semantics

#### 1.1 Image Coordinate System
- Images use a **2D Cartesian coordinate system**.
- The origin `(0, 0)` is located at the **top-left corner of the image domain**.
- The **x-axis** increases to the **right**.
- The **y-axis** increases **downward**.
- Coordinates are expressed in **pixel units**.

#### 1.2 Pixel Center Convention
- Pixel coordinates refer to **pixel centers**, not corners.
- The pixel at integer index `(i, j)` has its center at:
  ```
  (x = i + 0.5, y = j + 0.5)
  ```
- All geometric operations (sampling, projection, feature detection) assume this convention.

#### 1.3 Discrete vs Continuous Coordinates
- Integer coordinates index pixels.
- Floating-point coordinates represent **continuous image space**.
- Conversions between discrete and continuous space must be explicit and documented.

#### 1.4 Channel and Color Semantics
- **Channel order** and **color space** are independent semantic dimensions.
- Channel order defines memory layout (e.g. RGB vs BGR).
- Color space defines meaning (e.g. linear RGB, sRGB, grayscale).
- No API may conflate or silently convert between color spaces.
- Color conversion is always explicit and loss-aware.

#### 1.5 Image Axes Meaning - Images are conceptually indexed as `(y, x, c)`:
  - `y`: vertical spatial axis
  - `x`: horizontal spatial axis
  - `c`: channel axis
- Any alternative ordering must be encoded in the type system and documented.

#### 1.6 Image Validity Invariants
- Channel count must match the declared color space.
- Spatial dimensions must be positive.
- Image semantics must be preserved across all operations unless explicitly stated.

---

### 2. Tensor Semantics

#### 2.1 Axis Ordering
- A `Tensor<N, T>` has **ordered positional axes**.
- Axis indices are interpreted **strictly by position**:
  - Axis `0`: outermost, slowest-changing
  - Axis `N-1`: innermost, fastest-changing
- Axes have **no intrinsic semantic meaning** unless encoded via markers.

#### 2.2 Layout Semantics
- Default layout is **row-major (C-order)**.
- Layout meaning is semantic, not an optimization hint.
- Layout markers define how multi-dimensional indices map to linear memory.

#### 2.3 Shape and Strides
- `shape` and `strides` are part of the tensor’s semantic state.
- Strides are observable and meaningful.
- A tensor is **contiguous** iff its strides exactly match the declared layout.
- APIs may explicitly require contiguity and must reject non-conforming tensors.

#### 2.4 Mutability and Aliasing
- Tensor views may alias underlying storage.
- Aliasing is explicit and never implicit.
- Mutation through aliased views is only permitted where explicitly documented.

#### 2.5 Construction Rules
- Safe constructors must validate all layout and shape invariants.
- Unsafe constructors may assume correctness but must be explicitly marked.
- No API may silently reinterpret tensor layout, axes, or strides.

---

### 3. Image–Tensor Relationship

#### 3.1 Shared Storage Model
- Images are a **semantic specialization of tensors**.
- An image may internally store data using a tensor representation.

#### 3.2 Lowering (Image → Tensor)
- Lowering preserves:
  - memory layout
  - coordinate meaning
  - channel interpretation
- Axis markers must be attached during lowering.

#### 3.3 Lifting (Tensor → Image)
- Lifting is **checked and fallible**.
- A tensor may be lifted to an image iff:
  - axis semantics match `(y, x, c)`
  - layout is compatible
  - channel count matches the target color space
- All invariant violations must be reported explicitly.

---

### 4. Coordinate & Geometry Semantics

#### 4.1 Coordinate Frames
- All geometric quantities are expressed in named coordinate frames.
- Frame identity is part of the type system.
- No implicit frame conversion is permitted.

#### 4.2 Handedness
- The global coordinate system is **right-handed**.
- Handedness is explicit and immutable.

#### 4.3 Pose Semantics
- A pose represents a **rigid body transform in SE(3)**.
- Pose direction is explicit:
  - A pose `Pose<A, B>` transforms coordinates from frame `A` to frame `B`.
- Pose composition order is:
  ```
  Pose<A, B> . Pose<B, C> = Pose<A, C>
  ```

#### 4.4 Units
- Translation units are meters.
- Rotation units are radians.
- Mixed or unit-less geometry is forbidden.

---

### 5. Camera Semantics

#### 5.1 Camera Model
- Cameras are modeled as **pinhole projections unless explicitly stated otherwise**.
- Intrinsics define a mapping from camera frame to image plane.

#### 5.2 Camera Coordinate Frame
- Camera frame conventions:
  - +X: right
  - +Y: down
  - +Z: forward (into the scene)

#### 5.3 Projection Semantics
- Projection maps 3D points in the camera frame to continuous image coordinates.
- Back-projection is the inverse operation up to scale.

#### 5.4 Intrinsic Parameters
- Intrinsics are immutable once constructed.
- Parameter meaning (focal length, principal point) is explicitly documented.
- Invalid parameter ranges are rejected at construction.

---

### 6. Feature & Keypoint Semantics

#### 6.1 Keypoint Coordinates
- Keypoints are expressed in **continuous image coordinates**.
- Coordinates follow the image pixel-center convention.

#### 6.2 Subpixel Meaning
- Subpixel values represent continuous spatial location, not interpolation hints.

#### 6.3 Descriptor Semantics
- Descriptor dimensionality is part of the type.
- Descriptor values have no assumed normalization unless explicitly stated.
- Distance metrics must match descriptor semantics.

#### 6.4 Consistency Rules
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
