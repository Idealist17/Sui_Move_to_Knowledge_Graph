
module feature_test::structs {
    use sui::object::{Self, UID};
    use sui::transfer;
    use sui::tx_context::{TxContext};

    /// A struct representing a resource (key ability)
    struct ResourceStruct has key {
        id: UID,
        value: u64
    }

    /// A struct representing data (store, copy, drop abilities)
    struct DataStruct has store, copy, drop {
        value: u64
    }

    // --- Packs / Unpacks (Must be inside module) ---

    public fun create_data(val: u64): DataStruct {
        DataStruct { value: val }
    }

    public fun destroy_data(d: DataStruct): u64 {
        let DataStruct { value } = d;
        value
    }

    public fun create_and_transfer(ctx: &mut TxContext, value: u64) {
        let r = ResourceStruct {
            id: object::new(ctx),
            value
        };
        transfer::transfer(r, sui::tx_context::sender(ctx));
    }

    // --- Acquires (via generic calls likely, or dynamic fields) ---
    // SUI doesn't use `move_to` / `borrow_global` on the same address model often.
    // However, the scanner looks for `Operation::MoveTo`, `BorrowGlobal`, `Exists`.
    // These opcodes are generated for standard Move global storage ops.
    // If we want to force them, we need to use a non-object struct or standard move patterns if SUI allows.
    // Actually, SUI *does* allow `borrow_global` if you have `key` struct but it's weird with objects.
    // Let's try to just use `exists` which is harmless.
    
    // NOTE: SUI compiler might reject `exists<ResourceStruct>(addr)`.
    // Let's assume we just want to see `Calls` and `Packs`/`Unpacks` primarily.
    // If we can't easily generate `Acquires` in idiomatic SUI, that's okay, 
    // as long as the tool supports it when present.
    // 
    // But let's try one `exists` check.
    
    /*
    public fun check_exists(addr: address): bool {
        exists<ResourceStruct>(addr)
    }
    */
    // Commented out to avoid compilation error if SUI strict mode bans it. 
    // We will focus on Packs/Unpacks/Calls.
}

module feature_test::logic {
    use feature_test::structs::{Self};

    public fun test_interactions() {
        let d = structs::create_data(10);
        let _ = structs::destroy_data(d);
    }
}
