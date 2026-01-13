# Capability Audit Report for Security Knowledge Graph

This document details the audit results of `MoveScanner-v1` against the 6 critical capabilities required for building a Security Knowledge Graph.

## Summary Table

| Capability | Status | Notes |
| :--- | :--- | :--- |
| **1. Deep Type Resolution** | **[SUPPORTED]** | Uses `globalize_signature` to recursively resolve types, including generic instantiations (e.g., `Coin<SUI>`). |
| **2. Ability Extraction** | **[MISSING]** | Struct abilities (Key/Store/Drop/Copy) are NOT extracted when creating `StructData`. |
| **3. Object Lifecycle** | **[SUPPORTED]** | Explicitly handles `Pack`, `Unpack`, `PackGeneric`, and `UnpackGeneric` ops in bytecode generation. |
| **4. Entry Function ID** | **[SUPPORTED]** | Identifies `entry` modifier and visibility attributes directly from `FunctionDefinition`. |
| **5. Cross-Module Call Graph**| **[SUPPORTED]** | Builds a graph where nodes utilize `QualifiedId` (ModuleID + FunID), covering external calls. |
| **6. Compiler Dependency** | **[SUPPORTED]** | Depends on official `move-binary-format`, `move-model`, etc. |

## Detailed Findings

### 1. Deep Type Resolution (Generic Instantiation)
*   **Status**: **[SUPPORTED]**
*   **Evidence**: `src/move_ir/utils.rs` (lines 103-132)
*   **Analysis**: The function `globalize_signature` handles `SignatureToken::StructInstantiation`. It recursively converts type parameters, ensuring that types like `Coin<SUI>` are distinguished from `Coin<USDC>` by preserving the full type hierarchy in the `Type::Struct` enum.

### 2. Ability Extraction (Key/Store/Drop/Copy)
*   **Status**: **[MISSING]**
*   **Evidence**: `src/move_ir/utils.rs` (lines 526-560)
*   **Analysis**: The function `create_move_struct_data` constructs the `StructData` object used for analysis. However, it **ignores** the `abilities` field present in the `StructHandle` of the `CompiledModule`.
*   **Fix Required**:
    1.  Modify `src/move_ir/utils.rs`: Update `create_move_struct_data` to read `module.struct_handle_at(...).abilities`.
    2.  Update `StructData` instantiation to include these abilities (assuming the `StructData` definition in the used `move-model` version supports it, or extend it if it's a local wrapper).

### 3. Object Lifecycle Tracking (Pack/Unpack)
*   **Status**: **[SUPPORTED]**
*   **Evidence**: `src/move_ir/generate_bytecode.rs` (lines 811-955)
*   **Analysis**: The generator explicitly matches `MoveBytecode::Pack`, `PackGeneric`, `Unpack`, and `UnpackGeneric`. It converts these into `Operation::Pack` and `Operation::Unpack` in the intermediate representation, preserving the type index.

### 4. Entry Function Identification
*   **Status**: **[SUPPORTED]**
*   **Evidence**: `src/move_ir/utils.rs` (lines 305-309) & `src/move_ir/packages.rs`
*   **Analysis**: The code explicitly checks `func_define.is_entry` and extracts visibility modifiers to distinguish between `entry`, `public`, and private functions.

### 5. Cross-Module Call Graph
*   **Status**: **[SUPPORTED]**
*   **Evidence**: `src/move_ir/generate_bytecode.rs` (lines 1594-1642, `build_call_graph`)
*   **Analysis**: The graph nodes are initialized using `cm.function_handles()`, which includes all functions referenced in the module (both internal and imported). The edges are built by scanning `Bytecode::Call` instructions. The use of `QualifiedId` ensures that calls to external modules are correctly represented as distinct nodes in the graph.

### 6. Dependency on Move Compiler
*   **Status**: **[SUPPORTED]**
*   **Evidence**: `Cargo.toml`
*   **Analysis**: The project uses the official (forked) Move compiler crates:
    *   `move-binary-format`
    *   `move-model`
    *   `move-stackless-bytecode`
    *   `move-core-types`
