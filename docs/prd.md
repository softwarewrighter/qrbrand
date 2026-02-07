# Product Requirements Document (PRD)

## Overview

**qrbrand** is a personal vibe-coded Rust CLI tool for generating branded QR codes with logo overlays and URL text rendering. The tool enables developers and content creators to generate professional-looking QR codes for marketing materials, presentations, and digital content.

## Problem Statement

Creating branded QR codes typically requires:
1. Using online QR code generators with privacy concerns
2. Manual image editing software for logo overlays
3. Separate tools for adding URL text below QR codes
4. No control over error correction levels for logo overlays

## Solution

A command-line tool that:
- Generates QR codes from URLs with high error correction
- Overlays logos/images in the center with proper scaling
- Adds white plates behind logos for better scan reliability
- Renders URL text below QR codes using embedded fonts
- Provides full control over QR size, quiet zones, and logo parameters

## Target Users

1. **Developers** who need to generate QR codes programmatically
2. **Content Creators** who want branded QR codes for marketing
3. **Presenters** who need QR codes in slides and handouts
4. **Technical Users** who prefer CLI tools over web interfaces

## Core Requirements

### Must Have
- [x] Generate QR codes from URLs
- [x] Support PNG output format
- [x] Configurable QR size and quiet zones
- [x] Logo overlay with aspect ratio preservation
- [x] White plate behind logos for scan reliability
- [x] URL text rendering below QR codes
- [x] Embedded font for cross-platform consistency
- [x] Command-line interface with help documentation
- [x] Error handling for invalid URLs and files

### Should Have
- [ ] Support for multiple output formats (JPG, SVG)
- [ ] Batch processing of multiple URLs
- [ ] Custom color schemes for QR codes
- [ ] Gradient or styled logo overlays
- [ ] Configurable text font and style

### Could Have
- [ ] Web interface wrapper
- [ ] API server mode
- [ ] QR code scanning/decoding
- [ ] Template system for consistent branding
- [ ] Integration with CI/CD pipelines

### Won't Have (for now)
- GUI application
- Mobile app
- Cloud storage integration
- Social media auto-posting

## User Stories

### As a developer, I want to:
- Generate QR codes from command line for automation scripts
- Embed QR generation in build processes
- Have consistent output across different operating systems
- Control exact pixel dimensions for print materials

### As a content creator, I want to:
- Add my logo to QR codes for brand consistency
- Ensure QR codes remain scannable with logo overlays
- Add readable URL text for users who can't scan
- Generate multiple QR codes with consistent styling

### As a presenter, I want to:
- Generate high-resolution QR codes for slides
- Have QR codes that work even when projected
- Add branding to QR codes in presentations
- Quickly regenerate QR codes if URLs change

## Success Metrics

1. **Usability**: Users can generate a branded QR code with 3 commands or less
2. **Reliability**: Generated QR codes scan successfully 99% of the time
3. **Performance**: Generate QR codes in under 2 seconds for standard sizes
4. **Adoption**: 100+ downloads/clones in first month
5. **Satisfaction**: Positive feedback from technical users

## Technical Constraints

1. **Font Dependency**: Requires DejaVuSans.ttf in assets directory
2. **Memory Usage**: Large QR codes (4096px+) require significant RAM
3. **Platform Support**: Cross-platform but requires Rust toolchain
4. **File Formats**: Currently PNG-only for output

## Non-Functional Requirements

### Performance
- Generate 1024px QR code in < 1 second
- Handle up to 4096px QR codes without crashing
- Memory usage < 500MB for largest supported size

### Reliability
- 99.9% successful QR code generation
- All generated QR codes must be scannable
- Graceful error handling for invalid inputs

### Security
- No network calls during generation
- No external dependencies beyond Rust crates
- Local file operations only

### Maintainability
- Clean, documented Rust code
- Comprehensive test coverage
- Simple dependency tree
- Regular dependency updates

## Dependencies

### External
- Rust and Cargo toolchain
- DejaVu Sans font (included in assets)

### Internal (Crates)
- clap: Command line parsing
- qrcode: QR code generation
- image: Image manipulation
- anyhow: Error handling
- url: URL validation
- rusttype: Font rendering

## Future Considerations

1. **WebAssembly Port**: Could enable browser-based generation
2. **Library Mode**: Expose core functionality as a library
3. **Plugin System**: Allow custom logo processors
4. **Template System**: Save and reuse branding configurations
5. **CI/CD Integration**: Generate QR codes during build processes

## Version History

### v0.1.0 (Current)
- Initial release with core functionality
- Basic CLI interface
- Logo overlay with white plates
- URL text rendering with embedded font

### Planned v0.2.0
- Multiple output formats (JPG, SVG)
- Custom color schemes
- Batch processing
- Improved error messages

### Planned v1.0.0
- Stable API
- Comprehensive documentation
- Performance optimizations
- Extended test coverage