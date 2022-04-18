# reg-enum-macro-rs : simple rust helper macro for enum definitions

```rust
use reg_enum_macro_rs::reg_enum;

reg_enum!(T, u16, {
    cycle = 0,
    time = 1,
    next = [2, 2, 31],
});

fn main() {
    let x = T::cycle;
    println!("{:?} = {}", x, x.to_u16());
    let y = T::from_u16(34);
    println!("{:?}", y);
    println!("{:?} = {}", T::next8, T::next8.to_u16());
}
```
