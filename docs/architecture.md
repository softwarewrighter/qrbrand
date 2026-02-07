# Architecture

## Overview

qrbrand is a Rust CLI tool for generating QR codes with logo overlays and URL text rendering. The architecture follows a simple, single-binary design with modular functions for different QR generation features.

## System Components

### 1. CLI Interface (`main.rs`)
- **Command Line Parsing**: Uses `clap` crate for argument parsing
- **Argument Validation**: Validates URLs, file paths, and parameter ranges
- **Error Handling**: Uses `anyhow` for consistent error propagation

### 2. QR Code Generation
- **QR Rendering**: `render_qr_rgba()` function generates QR codes with configurable size and quiet zones
- **Error Correction**: Uses high error correction level (EcLevel::H) for logo overlays
- **Pixel-perfect rendering**: Maintains crisp module boundaries by using integer pixels per module

### 3. Logo Overlay System
- **Image Processing**: `overlay_logo_center()` handles logo resizing and placement
- **Aspect Ratio Preservation**: `resize_fit()` maintains logo proportions
- **White Plate Feature**: Optional white background plate for better scan reliability
- **Alpha Compositing**: Uses `imageops::overlay()` for proper alpha blending

### 4. Text Rendering System
- **Font Management**: Embeds DejaVuSans.ttf for consistent cross-platform text rendering
- **Text Measurement**: `measure_text_width()` calculates text dimensions
- **Text Drawing**: `draw_text_rgba()` renders anti-aliased text with alpha blending
- **Dynamic Sizing**: Automatically adjusts font size to fit available space

### 5. Image Processing Pipeline
1. URL validation and QR code generation
2. Optional logo overlay with white plate
3. Optional URL text rendering below QR
4. PNG file output

## Dependencies

### Core Dependencies
- `clap`: Command line argument parsing
- `qrcode`: QR code generation
- `image`: Image manipulation and I/O
- `anyhow`: Error handling
- `url`: URL parsing and validation
- `rusttype`: Font rendering

### Font Dependency
- **DejaVuSans.ttf**: Embedded font file in `assets/` directory
- Required for URL text rendering feature
- Included in repository for consistent behavior

## Data Flow

```
Input (CLI args)
    ↓
URL Validation
    ↓
QR Code Generation
    ↓
[Optional] Logo Overlay
    ↓
[Optional] URL Text Rendering
    ↓
Image Composition
    ↓
PNG File Output
```

## Key Design Decisions

### 1. Single Binary Design
- All functionality in one executable
- No external configuration files needed
- Simple deployment and usage

### 2. Embedded Font
- Uses embedded DejaVuSans.ttf to avoid OS font dependencies
- Ensures consistent text rendering across platforms
- Increases binary size but improves reliability

### 3. High Error Correction
- Uses EcLevel::H (30% error correction)
- Essential for logo overlays that obscure QR modules
- Improves scan reliability with logo overlays

### 4. Integer Pixel Rendering
- Maintains crisp QR module boundaries
- Prevents anti-aliasing artifacts in QR codes
- Ensures optimal scan reliability

### 5. Modular Function Design
- Each major feature in separate, testable functions
- Clear separation of concerns
- Easy to extend with new features

## File Structure

```
qrbrand/
├── src/
│   └── main.rs          # All application logic
├── assets/
│   └── DejaVuSans.ttf   # Embedded font
├── Cargo.toml           # Dependencies and metadata
├── Cargo.lock           # Dependency versions
└── docs/                # Documentation
```

## Performance Considerations

- **Memory Usage**: Processes images in memory; large QR codes (4096px+) may use significant RAM
- **CPU Usage**: Image resizing and text rendering are CPU-intensive operations
- **I/O**: Single file read/write operations; minimal disk I/O

## Limitations

- Requires DejaVuSans.ttf in assets directory
- Text rendering only supports single-line URLs
- Logo overlay assumes roughly square logos work best
- Maximum QR size limited by available memory