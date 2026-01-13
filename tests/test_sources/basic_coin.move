module 0x1::basic_coin {
    struct Coin has key, store {
        value: u64
    }

    struct Balance has key {
        coin: Coin
    }

    public fun mint(value: u64): Coin {
        Coin { value }
    }

    public fun burn(coin: Coin) {
        let Coin { value: _ } = coin;
    }

    public fun create_balance(recipient: &signer, value: u64) {
        let coin = mint(value);
        move_to(recipient, Balance { coin });
    }

    public fun get_balance(addr: address): u64 acquires Balance {
        let balance = borrow_global<Balance>(addr);
        balance.coin.value
    }

    public fun transfer(from: &signer, to: address, value: u64) acquires Balance {
        let from_addr = std::signer::address_of(from);
        let from_balance = borrow_global_mut<Balance>(from_addr);
        // Simplified Logic: just to test call graph and resource access
        let _val = from_balance.coin.value;
        call_internal_helper();
    }

    fn call_internal_helper() {
        // do nothing
    }
}
