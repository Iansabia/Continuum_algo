# Camera Integration - Future Implementation (Phase 9.5)

**Status**: Deferred until software foundation is complete
**Priority**: Low (infrastructure not yet in place)
**Created**: 2025-10-19

## Overview

Phase 9.5 involves integrating a camera system to capture actual (x,y) coordinates of ball landings on the green. This enables the BVN (Bivariate Normal) distribution to use real 2D data instead of simulated 1D radial distances.

## Prerequisites (Currently Missing)

Before camera integration can begin, we need:

1. **Physical Hardware Setup**
   - Camera(s) mounted above green with clear view
   - Calibrated camera intrinsics (focal length, distortion)
   - Stable mounting system (minimize vibration/movement)
   - Adequate lighting (may need IR or high-speed cameras)

2. **Ball Identification System**
   - Unique ball IDs (QR codes, RFID, or visual markers)
   - Player-to-ball mapping database
   - Ball tracking across frame sequences

3. **Homography Calibration**
   - 4+ known ground control points on green
   - Perspective transform matrix (pixel → real-world feet)
   - Calibration validation and drift detection

4. **Real-Time Processing Pipeline**
   - Ball detection algorithm (circle detection, ML model)
   - Landing position extraction (freeze frame at apex)
   - Coordinate transformation (pixel → (x,y) in feet)
   - Integration with shot recording system

## Technical Requirements

### Camera Specifications
- **Resolution**: ≥1080p (higher for larger greens)
- **Frame Rate**: ≥60 fps (120+ fps for high-speed tracking)
- **Field of View**: Must cover entire green target area
- **Low Latency**: <100ms from landing to coordinate extraction

### Software Components Needed

#### 1. Ball Detection Module
```python
# Pseudocode - not yet implemented
def detect_ball_landing(frame):
    # Option A: Circle Hough Transform
    circles = cv2.HoughCircles(frame, ...)

    # Option B: ML object detection (YOLO, etc.)
    detections = ball_detector.detect(frame)

    # Option C: Background subtraction + blob detection
    fg_mask = bg_subtractor.apply(frame)
    blobs = detect_blobs(fg_mask)

    return ball_pixel_coords

def identify_ball(ball_region):
    # QR code, barcode, or visual pattern matching
    ball_id = decode_marker(ball_region)
    return ball_id
```

#### 2. Homography Calibration Module
```python
# Pseudocode - not yet implemented
def calibrate_homography():
    # User clicks 4 corner markers on green
    pixel_points = [(100, 50), (900, 50), (900, 700), (100, 700)]

    # Real-world coordinates in feet (relative to pin at origin)
    world_points = [(-10, -10), (10, -10), (10, 10), (-10, 10)]

    # Compute perspective transform
    H = cv2.getPerspectiveTransform(pixel_points, world_points)

    return H

def pixel_to_world(pixel_x, pixel_y, H):
    # Transform pixel coords to (x,y) feet from pin
    world_coords = H @ [pixel_x, pixel_y, 1]
    x_ft = world_coords[0] / world_coords[2]
    y_ft = world_coords[1] / world_coords[2]

    return (x_ft, y_ft)
```

#### 3. Integration with Simulator
```rust
// In src/camera/ (not yet created)
pub struct CameraSystem {
    homography: [[f64; 3]; 3],
    ball_id_map: HashMap<String, String>, // ball_id -> player_id
}

impl CameraSystem {
    pub fn capture_shot_landing(&self) -> Option<(f64, f64, String)> {
        // Returns (x_ft, y_ft, player_id) or None if no landing detected
        // This would interface with camera hardware via USB/network
        todo!("Hardware integration not yet implemented")
    }
}

// Integration point in player_session.rs
pub fn record_shot_with_camera(
    player: &mut Player,
    camera: &CameraSystem,
    hole: &Hole,
    wager: f64,
) -> ShotOutcome {
    // Wait for ball to land (with timeout)
    let (x_ft, y_ft, _) = camera.capture_shot_landing()
        .expect("Failed to detect ball landing");

    let miss_distance = (x_ft * x_ft + y_ft * y_ft).sqrt();
    let p_max = player.calculate_p_max_bvn(hole, ...); // Use 4D Kalman state
    let multiplier = hole.calculate_payout(miss_distance, p_max);

    // Store 2D coordinates
    ShotOutcome::new_bvn(x_ft, y_ft, multiplier, wager, hole.id, false)
}
```

## Current Blockers

1. **No Physical Setup**: No cameras, no green, no hardware
2. **No Ball ID System**: Cannot map balls to players
3. **No Calibration Tools**: No UI for homography setup
4. **No Real-Time Processing**: No computer vision pipeline
5. **Integration Testing**: Cannot test without hardware

## When to Implement

Camera integration should be implemented when:

1. ✅ **Software foundation complete** (Phase 9.1-9.4 done)
2. ❌ Physical venue/prototype exists
3. ❌ Hardware procurement budget approved
4. ❌ Computer vision expertise available (or consultant hired)
5. ❌ Integration testing environment set up

**Estimated Timeline**: 3-6 months after venue prototype construction begins

## Workaround for Now

Until camera integration is complete, use **simulated (x,y) coordinates**:

```rust
// Generate realistic (x,y) from BVN distribution
let (x_ft, y_ft) = bvn_random(mu_x, mu_y, sigma_x, sigma_y);
let is_fat_tail = rng.gen::<f64>() < 0.02;
if is_fat_tail {
    x_ft *= 3.0;
    y_ft *= 3.0;
}

// Use in simulations
let outcome = ShotOutcome::new_bvn(x_ft, y_ft, multiplier, wager, hole_id, is_fat_tail);
```

This allows full BVN pipeline testing without hardware.

## Alternative Approaches

If camera system proves too complex/expensive:

1. **Manual Entry**: Staff inputs (x,y) via touchscreen after each shot
2. **Laser Rangefinder**: Automated measurement from pin to ball
3. **Ground Sensors**: Pressure-sensitive mat under green
4. **Golfer Self-Report**: Trust system with spot-check verification

## Resources

- OpenCV Perspective Transform: https://docs.opencv.org/4.x/da/d6e/tutorial_py_geometric_transformations.html
- Ball Detection Tutorial: https://pyimagesearch.com/circle-detection/
- Homography Estimation: https://docs.opencv.org/4.x/d9/dab/tutorial_homography.html

---

**Next Review**: After Phase 9.1-9.4 fully integrated and tested in production simulations
**Owner**: TBD (requires CV expertise)
**Dependencies**: Venue construction, hardware budget approval
