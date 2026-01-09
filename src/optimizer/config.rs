use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitConfig {
    pub min_segment_len: usize,
    pub max_segment_len: usize,
    pub max_error: f64,
}

impl Default for FitConfig {
    fn default() -> Self {
        Self {
            min_segment_len: 30,
            max_segment_len: 200,
            max_error: 2.0,
        }
    }
}

impl FitConfig {
    pub fn new(min_segment_len: usize, max_segment_len: usize, max_error: f64) -> Self {
        // 验证参数
        assert!(min_segment_len >= 3, "min_segment_len must be at least 3");
        assert!(max_segment_len >= min_segment_len, 
                "max_segment_len must be >= min_segment_len");
        assert!(max_error > 0.0, "max_error must be positive");
        
        Self {
            min_segment_len,
            max_segment_len,
            max_error,
        }
    }
    
    /// 创建配置，自动修正无效参数
    pub fn new_clamped(min_segment_len: usize, max_segment_len: usize, max_error: f64) -> Self {
        let min_len = min_segment_len.max(3);
        let max_len = max_segment_len.max(min_len);
        let error = max_error.max(0.1);
        
        Self {
            min_segment_len: min_len,
            max_segment_len: max_len,
            max_error: error,
        }
    }
}