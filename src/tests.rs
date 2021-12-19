#[cfg(test)]
mod tests {
    use crate::fluid::FluidCube;

    #[test]
    fn test_constrain_lower_bound() {
        let var: u32 = 5;
        assert!(FluidCube::constrain(var, 10, 20) == 10);
    }
    #[test]
    fn test_constrain_upper_bound() {
        let var: u32 = 25;
        assert!(FluidCube::constrain(var, 10, 20) == 20);
    }
}