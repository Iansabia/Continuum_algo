# Camera Integration Plan

## Date: October 18, 2025

## Overview

This document outlines the camera-based ball tracking system that will capture (x,y) coordinates of ball resting positions, enabling the transition from Rayleigh distribution (1D radial distance) to Bivariate Normal distribution (2D coordinate-based modeling).

---

## System Architecture

### Hardware Requirements

**Primary Camera:**
- High-resolution camera (minimum 1920×1080, recommended 4K)
- Fixed mounting position with clear view of landing area
- Frame rate: 30+ FPS for ball detection
- Lighting: Consistent illumination (LED panels for indoor venues)

**Ball ID System:**
- Golf balls with unique serial markings (QR codes, barcodes, or alphanumeric IDs)
- Minimum marking size: 10mm diameter for reliable detection
- High-contrast markings (black on white or vice versa)

**Computing:**
- GPU-accelerated computer vision processing (NVIDIA GPU recommended)
- Real-time homography transformation pipeline
- Integration with simulator API via REST or WebSocket

### Software Pipeline

```
┌─────────────────┐
│  Camera Capture │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Ball Detection │ ← OpenCV contour detection
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   ID Reading    │ ← OCR/QR/Barcode reader
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Homography     │ ← Pixel → Real-world (X,Y)
│  Transformation │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Simulator API  │ ← POST /shots with (x,y,ball_id)
└─────────────────┘
```

---

## Coordinate System

### Physical Layout

```
              North (Y+)
                  ▲
                  │
                  │
                  │
    ──────────────┼──────────────► East (X+)
                  │
                  │
                  │
              South (Y-)

Origin: Pin/Hole center
X-axis: Lateral (positive = right of target line)
Y-axis: Distance (positive = long, negative = short)
```

### Homography Calibration

**Calibration Markers:**
- 4 corners of landing area with known real-world coordinates
- Example for 150-yard hole:
  - Marker A: (-30 ft, -20 ft) ← Bottom-left
  - Marker B: (+30 ft, -20 ft) ← Bottom-right
  - Marker C: (+30 ft, +20 ft) ← Top-right
  - Marker D: (-30 ft, +20 ft) ← Top-left

**Calibration Process:**
1. Capture image with all 4 markers visible
2. User clicks on each marker in pixel space
3. System computes 3×3 homography matrix H
4. Validation: Place test ball at known location, verify accuracy

**Transformation:**
```rust
// Pixel coordinates (u, v) → Real-world (X, Y)
[X]   [h11  h12  h13]   [u]
[Y] = [h21  h22  h23] × [v]
[1]   [h31  h32  h33]   [1]

// Normalize:
X_real = X / 1
Y_real = Y / 1
```

---

## Ball Detection Algorithm

### Step 1: Preprocessing

```python
# Pseudocode
image = capture_frame()
gray = convert_to_grayscale(image)
blurred = gaussian_blur(gray, kernel_size=5)
binary = adaptive_threshold(blurred)
```

### Step 2: Contour Detection

```python
contours = find_contours(binary)
candidates = []

for contour in contours:
    area = calculate_area(contour)
    circularity = calculate_circularity(contour)

    # Filter for golf ball size and shape
    if 50 < area < 500 and circularity > 0.8:
        candidates.append(contour)
```

### Step 3: Ball ID Reading

**Option A: QR Codes**
- Use OpenCV QRCodeDetector
- Fast and reliable
- Requires larger marking area (~15mm)

**Option B: OCR (Alphanumeric IDs)**
- Use Tesseract OCR or custom ML model
- Smaller marking possible (~10mm)
- May require better lighting

**Option C: Barcode**
- Use pyzbar or similar library
- Moderate marking size (~12mm)
- Good reliability in controlled lighting

### Step 4: Position Extraction

```python
# Get centroid of detected ball
moments = calculate_moments(contour)
pixel_x = moments['m10'] / moments['m00']
pixel_y = moments['m01'] / moments['m00']

# Apply homography
real_x, real_y = apply_homography(pixel_x, pixel_y, H_matrix)
```

---

## API Integration

### Endpoint: POST /api/shots

**Request Format:**
```json
{
  "ball_id": "A7B3C2",
  "x_ft": 5.2,
  "y_ft": -3.8,
  "timestamp": "2025-10-18T14:32:17Z",
  "camera_confidence": 0.95,
  "hole_id": 4
}
```

**Response:**
```json
{
  "shot_id": "uuid-1234",
  "player_id": "player-A7B3C2",
  "wager": 10.00,
  "multiplier": 1.85,
  "payout": 18.50,
  "miss_distance_ft": 6.42,
  "current_sigma_x": 12.3,
  "current_sigma_y": 8.7,
  "current_mu_x": 0.8,
  "current_mu_y": -1.2
}
```

### Internal Processing

When simulator receives (x, y) coordinates:

1. **Calculate radial distance:** `d = sqrt(x² + y²)` for payout calculation
2. **Store (x, y) in ShotRecord** for Kalman update
3. **Update 4D Kalman filter:** `[μ_x, μ_y, σ_x, σ_y]`
4. **Calculate P_max** using 2D BVN integration
5. **Determine payout** based on distance and P_max
6. **Return results** including updated skill parameters

---

## Accuracy Requirements

### Target Accuracy

- **Position Accuracy:** ±2 inches (±0.17 ft) at 95% confidence
- **Ball ID Read Rate:** >99% success rate
- **Processing Latency:** <500ms from ball landing to API response

### Validation Tests

**Test 1: Known Position Verification**
- Place ball at 10 known positions
- Measure captured (x,y) vs. actual
- Calculate RMSE, must be <2 inches

**Test 2: Edge Case Handling**
- Ball partially obscured → Flag for manual review
- Multiple balls in frame → Match by ID
- Poor lighting → Request recalibration

**Test 3: Repeatability**
- Same ball, same position, 10 captures
- Standard deviation must be <1 inch

---

## Error Handling

### Camera Failures

| Error | Detection | Response |
|-------|-----------|----------|
| No ball detected | Timeout (5 sec) | Request re-shot or manual entry |
| ID unreadable | OCR confidence <70% | Use backup ID method or manual |
| Multiple balls | Count >1 | Match by expected player rotation |
| Homography invalid | Calibration age >7 days | Require recalibration |

### Fallback Mode

If camera system fails:
1. **Manual Entry:** Operator inputs (x,y) or radial distance
2. **Developer Mode:** Use simulated shots (testing only)
3. **Degraded Mode:** Revert to 1D Rayleigh (lose bias detection)

---

## Calibration Schedule

### Initial Setup
- Full calibration with 4 corner markers
- 10-ball validation test
- Document homography matrix H

### Regular Maintenance
- **Daily:** Quick validation (1 test ball at known position)
- **Weekly:** Full 10-ball validation test
- **Monthly:** Complete recalibration if drift detected
- **After any camera movement:** Immediate recalibration required

### Calibration Drift Detection

Monitor validation test results:
- If RMSE increases >0.5 inches from baseline → Warning
- If RMSE increases >1.0 inch from baseline → Force recalibration
- If any single measurement error >3 inches → Immediate recalibration

---

## Implementation Phases

### Phase 1: Prototype (Weeks 1-2)
- Set up camera hardware
- Implement basic ball detection
- Test homography transformation with manual markers
- Validate accuracy with 10 test shots

### Phase 2: ID System (Weeks 3-4)
- Implement ball ID reading (QR/OCR/Barcode)
- Test ID read rate with different lighting conditions
- Integrate with player database (ball_id → player_id)

### Phase 3: API Integration (Week 5)
- Create POST /api/shots endpoint
- Implement (x,y) storage in database
- Update simulator to accept coordinate-based shots
- Test end-to-end flow

### Phase 4: Production Deployment (Weeks 6-7)
- Install at first venue
- Run parallel testing (camera vs. manual entry)
- Tune detection parameters for production environment
- Document operational procedures

### Phase 5: Monitoring & Optimization (Ongoing)
- Track calibration drift over time
- Optimize detection algorithm for speed
- Implement automated calibration checks
- Add analytics dashboard for camera system health

---

## Cost Estimate

### Hardware (per venue)
- High-res camera: $300-$800
- Mounting hardware: $100-$200
- Lighting improvements: $200-$500
- Computing (if dedicated): $500-$1,500

**Total Hardware:** $1,100 - $3,000 per venue

### Software Development
- Computer vision pipeline: 40-60 hours
- API integration: 20-30 hours
- Testing & validation: 30-40 hours
- Documentation: 10-15 hours

**Total Development:** 100-145 hours @ $100-150/hr = $10,000-$22,000 (one-time)

### Ongoing Costs
- Ball ID markings: $0.50-$2.00 per ball
- Calibration maintenance: 2 hours/month per venue
- Support & troubleshooting: 5-10 hours/month across all venues

---

## Benefits Over Manual Entry

### Accuracy
- **Manual:** ±6-12 inches (operator measurement error)
- **Camera:** ±2 inches (validated accuracy)
- **Improvement:** 3-6× more accurate

### Speed
- **Manual:** 10-30 seconds per shot (measure, record, input)
- **Camera:** <1 second (automatic)
- **Improvement:** 10-30× faster

### Bias Detection
- **Manual:** Only radial distance captured
- **Camera:** Full (x,y) enables systematic bias analysis
- **New Feature:** Players can see tendencies (e.g., "You miss 4 ft right on average")

### Data Quality
- **Manual:** Human error, rounding, disputes
- **Camera:** Objective, precise, auditable
- **Trust:** Reduces player complaints about measurements

---

## Security Considerations

### Ball ID Spoofing
- **Risk:** Players swap balls to use another's inflated P_max
- **Mitigation:**
  - Check player rotation order
  - Flag if same ball_id used by multiple players in short time
  - Video review for disputes

### Camera Tampering
- **Risk:** Players obstruct camera or manipulate markers
- **Mitigation:**
  - Secure camera mounting (tamper-evident)
  - Automated calibration drift detection
  - Alert on sudden accuracy degradation

### Privacy
- **Risk:** Camera captures player images
- **Mitigation:**
  - Crop to landing area only (exclude people)
  - Store only ball position data, not images
  - Post privacy policy at venue

---

## Success Metrics

### Technical Performance
- **Position Accuracy:** <2 inch RMSE (measured weekly)
- **ID Read Rate:** >99% (measured per session)
- **Uptime:** >98% (excluding scheduled maintenance)
- **Calibration Stability:** <0.5 inch drift per week

### Business Impact
- **Throughput:** 3-5× more shots per hour (vs. manual)
- **Player Satisfaction:** >90% prefer camera to manual (survey)
- **Dispute Rate:** <1% of shots disputed (vs. 5-10% manual)
- **Operational Cost:** <$50/month per venue (maintenance)

---

## Next Steps

1. ✅ Document camera integration plan (this file)
2. ⏳ Source camera hardware and test equipment
3. ⏳ Implement ball detection prototype
4. ⏳ Validate homography transformation accuracy
5. ⏳ Integrate with simulator API (requires BVN implementation)
6. ⏳ Deploy at pilot venue
7. ⏳ Collect 1000+ shots for validation
8. ⏳ Roll out to additional venues

---

## Related Documentation

- See **BVN_MIGRATION.md** for mathematical model changes required to use (x,y) data
- See **continuum_checklist.md** Phase 5 for camera integration tasks
- See **WEB_INTERFACE_PLAN.md** Section 8.3 for bias visualization features

---

**Prepared By:** Engineering Team
**Classification:** Internal - Technical Specification
**Status:** 🟡 PLANNING PHASE
**Target Completion:** Q1 2026
