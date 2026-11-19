//! The one-line readouts the storage reports, as plain text.
//!
//! Upstream puts these numbers in a GUI; this port says them in chat and in the
//! block's feedback, so the wording and the grouping are part of what a user
//! sees. Pure, and tested here rather than in the glue.

use crate::runtime::{CAPACITY_1024K, Storage};

/// A count with thousands separators, as the feedback lines show it
/// (`1,048,576`).
pub fn grouped(amount: u64) -> String {
    let digits = amount.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// One line about a storage: what it holds, of what, and how much room is left.
pub fn summary(where_: &str, storage: &Storage) -> String {
    format!(
        "{where_}: {} / {} items in {} resources, {} free",
        grouped(storage.total()),
        grouped(storage.capacity()),
        storage.resources(),
        grouped(storage.free())
    )
}

/// The capacity of the slice's rung: the 1024k block and disk hold exactly
/// 1,048,576 items.
pub fn capacity() -> u64 {
    CAPACITY_1024K
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Resource;

    #[test]
    fn counts_are_grouped_in_threes() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1,000");
        assert_eq!(grouped(1_048_576), "1,048,576");
        assert_eq!(grouped(1_073_741_824), "1,073,741,824");
    }

    #[test]
    fn an_empty_storage_says_so_in_one_line() {
        let storage = Storage::new(capacity());
        assert_eq!(
            summary("storage", &storage),
            "storage: 0 / 1,048,576 items in 0 resources, 1,048,576 free"
        );
    }

    #[test]
    fn a_filled_storage_reports_its_resources_and_free_room() {
        let mut storage = Storage::new(capacity());
        storage.insert(Resource::new("minecraft:stone"), 1_048_000);
        storage.insert(Resource::new("minecraft:dirt"), 576);
        assert_eq!(
            summary("The disk", &storage),
            "The disk: 1,048,576 / 1,048,576 items in 2 resources, 0 free"
        );
    }
}
