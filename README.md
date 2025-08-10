# 3D Go Game

A complete implementation of the ancient game of Go extended into three dimensions, built with Rust and wgpu for high-performance graphics.

## Features

### ✅ Completed Features

- **3D Board Representation**: Full 19×19×19 board (configurable size)
- **Go Rules Implementation**: Complete game rules including:
  - Stone placement validation
  - Capture detection with liberty counting  
  - Turn-based gameplay (Black/White alternating)
  - Ko rule prevention
- **3D Graphics Engine**: 
  - Modern wgpu-based rendering pipeline
  - Phong shading with ambient, diffuse, and specular lighting
  - Instanced rendering for performance
- **Interactive Camera System**:
  - Orbit camera controls (mouse drag to rotate)
  - Zoom in/out with scroll wheel
  - WASD movement controls
- **3D Stone Placement**:
  - Mouse ray-casting for 3D position selection
  - Visual feedback with distinct black/white stone rendering
- **Performance Optimized**:
  - Efficient 3D collision detection
  - Separate rendering passes for different stone colors
  - Memory-efficient board representation using HashMaps

### 🚧 In Progress

- WASM version for web deployment

## Technical Architecture

### Core Components

1. **Game Logic** (`src/game/`)
   - `board.rs`: 3D board representation with efficient neighbor/group algorithms
   - `rules.rs`: Complete Go rule enforcement including captures and ko rule
   - `stone.rs`: Stone data structures and color management

2. **Rendering Engine** (`src/render/`)
   - `graphics.rs`: wgpu graphics pipeline with instanced rendering
   - `camera.rs`: 3D orbital camera system with smooth controls
   - `mesh.rs`: Procedural sphere and cube mesh generation
   - `shader.rs`: WGSL shader management
   - `shaders/basic.wgsl`: Vertex/fragment shaders with lighting

3. **Input System** (`src/input/`)
   - `mouse_picker.rs`: 3D ray-casting for mouse-to-world coordinate mapping

### Technology Stack

- **Language**: Rust (2021 edition)
- **Graphics**: wgpu 0.17 (WebGPU/Vulkan/DirectX/Metal)
- **Windowing**: winit 0.27
- **Math**: glam 0.24 (SIMD-optimized linear algebra)
- **Build**: Cargo with custom build configurations

## Controls

- **Mouse Drag**: Orbit camera around the board
- **Scroll Wheel**: Zoom in/out
- **WASD**: Move camera position  
- **Left Click**: Place stone at 3D grid position
- **Esc**: Exit game

## Building and Running

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Linux dependencies
sudo apt install cmake build-essential libfreetype6-dev libfontconfig1-dev
```

### Native Version

```bash
# Clone and build
git clone <repository>
cd 3dgo
cargo build --release

# Run the game
cargo run --bin go3d --release

# Run logic tests
cargo run --bin test_game
```

### Development

```bash
# Run with debug info
cargo run --bin go3d

# Check for errors
cargo check

# Run tests
cargo test
```

## Game Rules

The game follows traditional Go rules extended to 3D space:

1. **Objective**: Control the most territory by surrounding empty spaces
2. **Placement**: Stones must be placed on empty intersections
3. **Capture**: Stones/groups with no liberties (empty adjacent spaces) are captured
4. **Liberties**: Count empty spaces adjacent in all 6 directions (±x, ±y, ±z)
5. **Ko Rule**: Cannot immediately recapture to create identical board state
6. **Turn Order**: Black plays first, then alternating

### 3D-Specific Rules

- **Liberties**: Each position has up to 6 neighbors (not 4 as in 2D)
- **Groups**: Connected stones form groups across all 3 dimensions
- **Territory**: Surrounded empty spaces count as territory
- **Visualization**: Board rendered as 3D grid with spherical stones

## Performance Characteristics

- **Board Operations**: O(1) stone placement/lookup using HashMap
- **Group Detection**: O(n) breadth-first search where n = group size
- **Rendering**: Instanced rendering scales to thousands of stones
- **Memory**: Efficient sparse representation (only occupied positions stored)

## Screenshots

The game renders a 3D grid where players can place black and white stones. The camera system allows full 360-degree viewing of the board from any angle.

## Future Enhancements

- [ ] Web version via WebAssembly
- [ ] AI opponent with minimax/MCTS
- [ ] Network multiplayer
- [ ] Save/load game state
- [ ] Animated stone placement
- [ ] Territory visualization
- [ ] Sound effects
- [ ] Multiple board sizes
- [ ] Game replay system

## Architecture Decisions

### Why Rust + wgpu?

- **Performance**: Zero-cost abstractions with manual memory management
- **Safety**: Memory safety without runtime overhead
- **Cross-platform**: wgpu provides unified graphics API
- **WASM**: Excellent WebAssembly compilation support
- **Concurrency**: Built-in threading support for future AI/networking

### 3D Design Choices

- **Grid Representation**: 19×19×19 provides interesting strategic depth
- **Camera System**: Orbit camera allows easy inspection of all board layers
- **Stone Rendering**: Spheres provide clear visibility from all angles
- **Coordinate System**: Right-handed system with Y-up convention

This implementation demonstrates advanced 3D game development techniques while maintaining the strategic depth that makes Go compelling.