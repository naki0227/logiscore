/// データの長さから約200 tickを目標にv1 Denseの密度を算出する。
pub(crate) fn calculate_optimal_density(data_length: usize) -> u8 {
    if data_length == 0 {
        return 1;
    }
    data_length.div_ceil(200).clamp(1, 255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_respects_boundaries() {
        assert_eq!(calculate_optimal_density(0), 1);
        assert_eq!(calculate_optimal_density(200), 1);
        assert_eq!(calculate_optimal_density(201), 2);
        assert_eq!(calculate_optimal_density(usize::MAX), 255);
    }
}
