#![allow(clippy::missing_safety_doc)] // You only live once, that's the motto - Drake

use core_data::numerics::Spark;

#[no_mangle]
pub unsafe extern "C" fn dreamcaller_return_two() -> i32 {
    let spark = Spark(123);
    spark.0 as i32
}
