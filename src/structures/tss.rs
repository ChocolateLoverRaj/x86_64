//! Provides a type for the task state segment structure.

use crate::VirtAddr;
use core::mem::offset_of;

/// In 64-bit mode the TSS holds information that is not
/// directly related to the task-switch mechanism,
/// but is used for stack switching when an interrupt or exception occurs.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct TaskStateSegment<const N: usize> {
    reserved_1: u32,
    /// The full 64-bit canonical forms of the stack pointers (RSP) for privilege levels 0-2.
    /// The stack pointers used when a privilege level change occurs from a lower privilege level to a higher one.
    pub privilege_stack_table: [VirtAddr; 3],
    reserved_2: u64,
    /// The full 64-bit canonical forms of the interrupt stack table (IST) pointers.
    /// The stack pointers used when an entry in the Interrupt Descriptor Table has an IST value other than 0.
    pub interrupt_stack_table: [VirtAddr; 7],
    reserved_3: u64,
    reserved_4: u16,
    /// The 16-bit offset to the I/O permission bit map from the 64-bit TSS base.
    iomap_base: u16,
    /// The I/O Permission bitmap, where each bit corresponds to a `u16` port address, and a value of `0` means that Ring 3 is allowed to access the port and `1` means it's not allowed.
    pub iomap: [u8; N],
    iomap_last_byte: u8,
}

impl<const N: usize> TaskStateSegment<N> {
    /// Creates a new TSS with zeroed privilege and interrupt stack table and a I/O-Permission Bitmap with all bits set to `1` (deny).
    ///
    /// As we always set the TSS segment limit to
    /// `size_of::<TaskStateSegment>() - 1`, this means that `iomap_base` is
    /// initialized to `size_of::<TaskStateSegment>()`.
    #[inline]
    pub const fn new() -> Self {
        TaskStateSegment {
            privilege_stack_table: [VirtAddr::zero(); 3],
            interrupt_stack_table: [VirtAddr::zero(); 7],
            iomap_base: offset_of!(Self, iomap) as u16,
            iomap: [u8::MAX; N],
            iomap_last_byte: u8::MAX,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
        }
    }

    /// Consumes access to self, returning a pointer guaranteeing that the TSS will stay there forever (static lifetime), and a reference to modify the iomap
    pub const fn ready_to_activate(
        &'static mut self,
    ) -> (ReadyTssPointer<N>, &'static mut [u8; N]) {
        (
            ReadyTssPointer(self),
            // Safety: a u8 array does not require any alignment
            unsafe { &mut *(&raw mut self.iomap) },
        )
    }
}

/// Used to be sure that the TSS pointer points to a static TSS (so the pointer will not become invalid)
#[derive(Debug)]
pub struct ReadyTssPointer<const N: usize>(pub(crate) *mut TaskStateSegment<N>);

impl<const N: usize> Default for TaskStateSegment<N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn check_tss_size() {
        // Per the SDM, the minimum size of a TSS is 0x68 bytes, giving a
        // minimum limit of 0x67.
        // But because we have the last byte of iomap, that increases the size by 1 byte
        assert_eq!(size_of::<TaskStateSegment<0>>(), 0x69);
    }
}
