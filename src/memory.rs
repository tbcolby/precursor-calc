//! Memory registers and storage

/// Memory registers (10 like TI-85)
pub struct Memory {
    registers: [f64; 10],
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub fn new() -> Self {
        Self {
            registers: [0.0; 10],
        }
    }

    /// Store value in register (0-9)
    pub fn store(&mut self, register: usize, value: f64) -> bool {
        if register < 10 {
            self.registers[register] = value;
            true
        } else {
            false
        }
    }

    /// Recall value from register (0-9)
    pub fn recall(&self, register: usize) -> Option<f64> {
        if register < 10 {
            Some(self.registers[register])
        } else {
            None
        }
    }

    /// Add to register (M+)
    pub fn add(&mut self, register: usize, value: f64) -> bool {
        if register < 10 {
            self.registers[register] += value;
            true
        } else {
            false
        }
    }

    /// Subtract from register (M-)
    pub fn subtract(&mut self, register: usize, value: f64) -> bool {
        if register < 10 {
            self.registers[register] -= value;
            true
        } else {
            false
        }
    }

    /// Clear a register
    pub fn clear(&mut self, register: usize) -> bool {
        if register < 10 {
            self.registers[register] = 0.0;
            true
        } else {
            false
        }
    }

    /// Clear all registers
    pub fn clear_all(&mut self) {
        self.registers = [0.0; 10];
    }

    /// Check if any register is non-zero (for indicator)
    pub fn has_stored_value(&self) -> bool {
        self.registers.iter().any(|&v| v != 0.0)
    }

    /// Get all registers
    pub fn get_all(&self) -> &[f64; 10] {
        &self.registers
    }

    /// Set all registers (for loading from storage)
    pub fn set_all(&mut self, values: [f64; 10]) {
        self.registers = values;
    }

    /// Get register labels with values for display
    pub fn get_display_list(&self) -> [(usize, f64); 10] {
        let mut result = [(0usize, 0.0f64); 10];
        for (i, &v) in self.registers.iter().enumerate() {
            result[i] = (i, v);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_recall() {
        let mut mem = Memory::new();
        mem.store(0, 42.0);
        assert_eq!(mem.recall(0), Some(42.0));
        assert_eq!(mem.recall(1), Some(0.0));
    }

    #[test]
    fn test_add_subtract() {
        let mut mem = Memory::new();
        mem.store(0, 10.0);
        mem.add(0, 5.0);
        assert_eq!(mem.recall(0), Some(15.0));
        mem.subtract(0, 3.0);
        assert_eq!(mem.recall(0), Some(12.0));
    }

    #[test]
    fn test_has_stored() {
        let mut mem = Memory::new();
        assert!(!mem.has_stored_value());
        mem.store(5, 1.0);
        assert!(mem.has_stored_value());
    }
}
