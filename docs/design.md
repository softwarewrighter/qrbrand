# Design Document

## Overview

This document describes the architectural and implementation design of the qrbrand CLI tool for generating branded QR codes.

## System Architecture

### High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Input     │───▶│  QR Generation  │───▶│  Image Output   │
│   (clap)        │    │   Pipeline      │    │   (PNG File)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
        │                       │                       │
        ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  URL Validation │    │  Logo Overlay   │    │  Text Rendering │
│   (url crate)   │    │  (image crate)  │    │  (rusttype)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Component Design

#### 1. CLI Interface Component
**Purpose**: Parse and validate command-line arguments
**Implementation**: `clap` crate with derive macros
**Key Functions**:
- `Args::parse()` - Main argument parsing
- URL validation via `Url::parse()`
- Parameter range validation (logo_scale, size, etc.)

#### 2. QR Generation Component
**Purpose**: Generate QR code from URL
**Implementation**: `qrcode` crate with custom rendering
**Key Functions**:
- `render_qr_rgba()` - Core QR rendering logic
- Error correction level H (30%) for logo compatibility
- Integer pixel-per-module rendering for crisp edges

#### 3. Logo Overlay Component
**Purpose**: Overlay and position logo image
**Implementation**: `image` crate with alpha compositing
**Key Functions**:
- `overlay_logo_center()` - Main logo placement
- `resize_fit()` - Aspect-ratio-preserving resize
- `draw_rect()` - White plate drawing utility

#### 4. Text Rendering Component
**Purpose**: Render URL text below QR code
**Implementation**: `rusttype` crate with embedded font
**Key Functions**:
- `add_url_text_below()` - Text band addition
- `measure_text_width()` - Text measurement
- `draw_text_rgba()` - Anti-aliased text rendering
- `blend_over()` - Alpha blending for text

## Data Structures

### Command Line Arguments (`Args` struct)
```rust
struct Args {
    url: String,           // URL to encode
    image: Option<String>, // Optional logo path
    out: String,           // Output file path
    size: u32,             // QR size in pixels
    quiet: u32,            // Quiet zone in modules
    logo_scale: f32,       // Logo size fraction (0.05-0.35)
    logo_plate: bool,      // White plate behind logo
    logo_pad: f32,         // Plate padding fraction
    show_url: bool,        // Render URL text
}
```

### Image Processing Pipeline
1. **Input Validation**: Validate URL and file paths
2. **QR Generation**: Create base QR code image
3. **Optional Logo Overlay**: 
   - Load and resize logo
   - Draw white plate (if enabled)
   - Alpha composite onto QR
4. **Optional Text Rendering**:
   - Calculate text dimensions
   - Extend canvas height
   - Render text with embedded font
5. **Output**: Save as PNG file

## Algorithm Design

### QR Rendering Algorithm (`render_qr_rgba`)
```
Input: QR code, size, quiet_modules
Output: ImageBuffer<Rgba<u8>, Vec<u8>>

1. Calculate module_count = code.width()
2. Calculate total_modules = module_count + 2 * quiet_modules
3. Calculate pixels_per_module = floor(size / total_modules)
4. Validate pixels_per_module >= 2
5. Create white image of size (ppm * total_modules)
6. For each module (x, y):
   - If module is dark:
     - Calculate pixel coordinates with quiet zone offset
     - Fill ppm x ppm block with black
7. Return image
```

### Logo Overlay Algorithm (`overlay_logo_center`)
```
Input: QR image, logo_path, scale, plate, pad
Output: Modified QR image

1. Validate scale in range 0.05-0.35
2. Calculate target_logo_size = qr_width * scale
3. Load logo image
4. Resize logo preserving aspect ratio (resize_fit)
5. If plate enabled:
   - Calculate plate_size = logo_size * (1 + 2*pad)
   - Draw white rectangle at center
6. Alpha composite logo onto QR center
```

### Text Rendering Algorithm (`add_url_text_below`)
```
Input: QR image, url_text
Output: Extended image with text

1. Load embedded DejaVuSans font
2. Calculate band_height = max(qr_height * 0.18, 120px)
3. Create new image with height = qr_height + band_height
4. Copy QR to top of new image
5. Calculate max_text_width = qr_width - margins
6. Find font_size that fits text within max_width
   - Start with font_size = band_height * 0.35
   - While text_width > max_width and font_size > 14:
     - font_size *= 0.92
     - Recalculate text_width
7. Calculate text position (centered in band)
8. Render text with anti-aliasing (draw_text_rgba)
9. Return extended image
```

## Error Handling Design

### Error Types
1. **Input Errors**: Invalid URLs, missing files, invalid parameters
2. **Processing Errors**: Image loading failures, memory allocation failures
3. **Output Errors**: File write permissions, disk space

### Error Propagation
- Use `anyhow::Result<T>` for consistent error handling
- Contextual error messages with `with_context()`
- Early returns on fatal errors
- User-friendly error messages

### Recovery Strategies
1. **Input Validation**: Validate all inputs before processing
2. **Resource Management**: Use RAII patterns for image data
3. **Cleanup**: Remove partial output files on failure
4. **Fallbacks**: Use default values where safe

## Performance Considerations

### Memory Usage
- **QR Code**: `size² * 4` bytes (RGBA)
- **Logo**: Variable based on input size
- **Text Band**: Additional `qr_width * band_height * 4` bytes
- **Peak Memory**: ~2x QR size during composition

### CPU Optimization
- **Integer Arithmetic**: Use integer math for pixel coordinates
- **Loop Unrolling**: Manual pixel filling for QR modules
- **Font Caching**: Embedded font loaded once per process
- **Image Operations**: Use optimized `image` crate operations

### I/O Optimization
- **Single Pass**: Process images in memory, single file write
- **Buffer Reuse**: Reuse image buffers where possible
- **Lazy Loading**: Load logo only when needed

## Security Design

### Input Sanitization
1. **URL Validation**: Use `url` crate for proper URL parsing
2. **Path Validation**: Check file existence and permissions
3. **Parameter Validation**: Validate numeric ranges (logo_scale, etc.)

### Resource Limits
1. **Maximum Size**: Practical limit of 4096px to prevent memory exhaustion
2. **File Size Limits**: Reject extremely large input images
3. **Memory Bounds**: Check allocation success

### Privacy Considerations
1. **No Network Calls**: All processing local
2. **No Telemetry**: No data collection
3. **File Operations**: Only read input files, write output file

## Testing Strategy

### Unit Tests
- Test individual functions in isolation
- Mock external dependencies where needed
- Cover edge cases and error conditions

### Integration Tests
- Test complete CLI workflow
- Verify file input/output
- Test error handling paths

### Property Tests
- Verify QR codes are always scannable
- Test logo scaling preserves aspect ratio
- Verify text fits within bounds

### Performance Tests
- Measure generation time for different sizes
- Track memory usage patterns
- Verify scalability

## Deployment Design

### Build Process
```
cargo build --release
```
- Single static binary
- No external runtime dependencies
- Embedded font included

### Distribution
- **Source**: GitHub repository
- **Binary**: Release assets for major platforms
- **Package Managers**: Potential Cargo install

### Installation
1. **From Source**: `cargo install --path .`
2. **From Binary**: Copy to PATH
3. **Development**: `cargo run -- [args]`

## Maintenance Considerations

### Code Organization
- Single file for simplicity
- Clear function separation
- Comprehensive comments

### Dependency Management
- Minimal dependency tree
- Regular `cargo update`
- Security vulnerability monitoring

### Documentation
- Inline code documentation
- README with usage examples
- Architecture documentation (this document)

### Evolution Path
1. **Refactoring**: Split into modules if file grows
2. **Features**: Add new output formats, color schemes
3. **Performance**: Optimize hot paths as needed
4. **Compatibility**: Maintain Rust edition updates