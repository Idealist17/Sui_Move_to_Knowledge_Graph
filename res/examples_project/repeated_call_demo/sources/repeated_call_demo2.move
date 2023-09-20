module repeated_call_demo::repeated_call_demo2
{

    public fun demo2_f0(len:u8) {
        let i = 0;
        while (i < len) {
            i = i + 1;
        };
    }
}