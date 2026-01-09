# 贝塞尔曲线拟合库 - 完整使用指南

## 📚 文件清单

你需要创建以下文件结构：

```
bezier-dp-fit/
├── Cargo.toml                 # Rust项目配置
├── pyproject.toml             # Python打包配置
├── .gitignore                 # Git忽略文件
├── README.md                  # 项目说明
├── QUICKSTART.md              # 快速开始
├── src/                       # Rust源代码
│   ├── lib.rs                 # 库入口
│   ├── geometry/              # 几何模块
│   │   ├── mod.rs
│   │   ├── point.rs           # 点结构
│   │   └── bezier.rs          # 贝塞尔曲线
│   ├── fitting/               # 拟合模块
│   │   ├── mod.rs
│   │   └── fitter.rs          # 单段拟合器
│   ├── optimizer/             # 优化模块
│   │   ├── mod.rs
│   │   ├── config.rs          # 配置
│   │   └── dp.rs              # DP算法
│   └── python/                # Python绑定
│       ├── mod.rs
│       └── bindings.rs        # PyO3绑定
├── tests/                     # 测试
│   └── test_basic.rs
├── benches/                   # 性能测试
│   └── benchmark.rs
└── examples/                  # 示例
    └── example.py             # Python使用示例
```

---

## 🚀 三步安装

### 1. 安装 Rust

**Windows:**
- 访问 https://rustup.rs/
- 下载并运行 `rustup-init.exe`
- 按默认选项安装
- 重启终端

**macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

验证：
```bash
rustc --version
cargo --version
```

### 2. 安装 Python 工具

```bash
pip install maturin numpy
```

### 3. 编译安装

在项目根目录：
```bash
# 开发模式（推荐）
maturin develop --release

# 或构建wheel包
maturin build --release
pip install target/wheels/bezier_dp_fit-*.whl
```

---

## 💻 基础使用

### 最简单的例子

```python
from bezier_dp_fit import fit_curve_py as fit_curve

# 你的骨架点（有序的像素坐标）
points = [
    (10, 10), (11, 11), (12, 13), (13, 16), (14, 20),
    (15, 25), (16, 31), (17, 38), (18, 46), (19, 55),
    # ... 更多点
]

# 拟合
result = fit_curve(
    points=points,
    min_segment_len=30,
    max_segment_len=200,
    max_error=2.0
)

# 查看结果
print(f"分了 {result.num_segments} 段")
print(f"总误差: {result.total_error:.2f}")
print(f"SVG路径: {result.to_svg()}")
```

### 使用 Numpy 数组

```python
import numpy as np

# 从numpy数组输入
points = np.array([
    [0, 0], [1, 1], [2, 4], [3, 9], [4, 16]
], dtype=float)

result = fit_curve(points)
```

---

## 🎯 实际应用场景

### 场景1: 图像骨架矢量化

```python
from bezier_dp_fit import fit_curve_py as fit_curve
from skimage.morphology import skeletonize
from scipy import ndimage
import numpy as np
from PIL import Image

# 1. 读取图像并骨架化
img = Image.open('drawing.png').convert('L')
binary = np.array(img) < 128
skeleton = skeletonize(binary)

# 2. 提取骨架点（你需要实现轮廓跟踪）
# 这里简化为找到所有骨架像素
y_coords, x_coords = np.where(skeleton)
points = list(zip(x_coords, y_coords))

# 3. 拟合
result = fit_curve(
    points=points,
    min_segment_len=20,
    max_segment_len=150,
    max_error=1.5
)

# 4. 导出SVG
svg_content = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" 
     viewBox="0 0 {img.width} {img.height}">
  <path d="{result.to_svg()}" 
        fill="none" 
        stroke="black" 
        stroke-width="2"/>
</svg>'''

with open('vectorized.svg', 'w') as f:
    f.write(svg_content)
```

### 场景2: 路径简化

```python
# 简化GPS轨迹或鼠标轨迹
def simplify_path(raw_points, tolerance=5.0):
    """简化路径，保持形状"""
    result = fit_curve(
        points=raw_points,
        min_segment_len=10,
        max_segment_len=100,
        max_error=tolerance
    )
    return result.sample_points(points_per_segment=20)

# 原始轨迹：1000个点
gps_track = [...]  # 你的GPS数据

# 简化后：~50个点
simplified = simplify_path(gps_track, tolerance=10.0)
print(f"从 {len(gps_track)} 简化到 {len(simplified)} 个点")
```

### 场景3: 动画路径

```python
# 生成平滑的动画路径
def create_animation_path(keyframes, smoothness=1.0):
    """从关键帧生成平滑路径"""
    result = fit_curve(
        points=keyframes,
        min_segment_len=3,
        max_segment_len=20,
        max_error=smoothness
    )
    
    # 高密度采样用于动画
    return result.sample_points(points_per_segment=50)

keyframes = [(0, 0), (100, 50), (200, 100), (300, 80)]
smooth_path = create_animation_path(keyframes)

# 在动画中逐帧使用 smooth_path 中的点
```

---

## ⚙️ 参数详解

### `min_segment_len` (最小段长)

控制最少要用多少个点拟合一段曲线。

- **用途**: 防止过度分段
- **太小**: 曲线太碎，SVG文件大
- **太大**: 丢失细节
- **推荐值**:
  - 低分辨率(512px): 15-25
  - 中等(1024px): 25-35  
  - 高分辨率(2048px+): 35-50

```python
# 粗糙拟合（快速，平滑）
result = fit_curve(points, min_segment_len=50)

# 精细拟合（慢，保留细节）
result = fit_curve(points, min_segment_len=15)
```

### `max_segment_len` (最大段长)

限制单段曲线最多包含多少个点。

- **用途**: 控制性能和内存
- **推荐**: `min_segment_len * 5` 到 `* 8`
- **注意**: 设太大没用，还占内存

```python
# 平衡设置
min_len = 30
result = fit_curve(
    points, 
    min_segment_len=min_len,
    max_segment_len=min_len * 6  # 180
)
```

### `max_error` (最大误差)

允许的最大拟合误差（像素为单位）。

- **用途**: 平衡精度和段数
- **值越小**: 越精确，但分段越多
- **推荐**:
  - 需要高精度: 0.5-1.5
  - 一般用途: 1.5-3.0
  - 平滑优先: 3.0-5.0

```python
# 高精度模式
result = fit_curve(points, max_error=1.0)
print(f"高精度: {result.num_segments} 段")

# 平滑模式
result = fit_curve(points, max_error=5.0)
print(f"平滑: {result.num_segments} 段")
```

---

## 🔧 高级用法

### 批量处理多条线

```python
def process_multiple_lines(lines_list):
    """批量处理多条线稿"""
    results = []
    
    for line_points in lines_list:
        if len(line_points) < 30:
            continue  # 跳过太短的线
            
        result = fit_curve(
            points=line_points,
            min_segment_len=25,
            max_segment_len=150,
            max_error=2.0
        )
        results.append(result)
    
    return results

# 使用
all_lines = [line1, line2, line3, ...]  # 多条线的点集
fitted = process_multiple_lines(all_lines)

# 合并到一个SVG
svg_paths = [r.to_svg() for r in fitted]
```

### 自适应参数

```python
def adaptive_fit(points):
    """根据点数自动调整参数"""
    n = len(points)
    
    if n < 100:
        config = (10, 50, 1.5)
    elif n < 1000:
        config = (25, 150, 2.0)
    else:
        config = (40, 200, 3.0)
    
    return fit_curve(
        points=points,
        min_segment_len=config[0],
        max_segment_len=config[1],
        max_error=config[2]
    )
```

### 获取详细信息

```python
result = fit_curve(points)

# 统计信息
print(f"总点数: {len(points)}")
print(f"分段数: {result.num_segments}")
print(f"压缩率: {result.num_segments / len(points) * 100:.1f}%")
print(f"总误差: {result.total_error:.2f}")
print(f"平均段长: {len(points) / result.num_segments:.1f}")

# 每段的控制点
for i, (p0, p1, p2) in enumerate(result.control_points()):
    print(f"段 {i+1}:")
    print(f"  起点: {p0}")
    print(f"  控制点: {p1}")
    print(f"  终点: {p2}")

# 采样验证
sampled = result.sample_points(points_per_segment=100)
print(f"采样了 {len(sampled)} 个点用于验证")
```

---

## 📊 性能优化建议

### 1. 大数据集分块处理

```python
def fit_large_dataset(points, chunk_size=2000):
    """分块处理大数据集"""
    results = []
    
    for i in range(0, len(points), chunk_size):
        chunk = points[i:i+chunk_size]
        result = fit_curve(chunk)
        results.append(result)
    
    return results
```

### 2. 预处理点集

```python
def preprocess_points(points, min_distance=2.0):
    """移除距离太近的点"""
    if len(points) < 2:
        return points
    
    filtered = [points[0]]
    for p in points[1:]:
        last = filtered[-1]
        dist = ((p[0]-last[0])**2 + (p[1]-last[1])**2)**0.5
        if dist >= min_distance:
            filtered.append(p)
    
    return filtered

# 使用
clean_points = preprocess_points(raw_points)
result = fit_curve(clean_points)
```

---

## 🐛 调试和验证

### 可视化对比

```python
import matplotlib.pyplot as plt

def visualize_fit(original_points, result):
    """可视化拟合结果"""
    # 原始点
    orig_x = [p[0] for p in original_points]
    orig_y = [p[1] for p in original_points]
    
    # 拟合曲线采样
    fitted = result.sample_points(points_per_segment=50)
    fit_x = [p[0] for p in fitted]
    fit_y = [p[1] for p in fitted]
    
    plt.figure(figsize=(12, 6))
    plt.plot(orig_x, orig_y, 'r.', label='原始点', markersize=2)
    plt.plot(fit_x, fit_y, 'b-', label='拟合曲线', linewidth=2)
    
    # 控制点
    for i, (p0, p1, p2) in enumerate(result.control_points()):
        plt.plot([p0[0], p1[0], p2[0]], [p0[1], p1[1], p2[1]], 
                'g--', alpha=0.5)
        plt.plot(p1[0], p1[1], 'go', markersize=5)
    
    plt.legend()
    plt.axis('equal')
    plt.grid(True, alpha=0.3)
    plt.title(f'{result.num_segments} 段, 误差 {result.total_error:.2f}')
    plt.show()

# 使用
result = fit_curve(points)
visualize_fit(points, result)
```

---

## ❓ 常见问题

### Q: 为什么我的结果分段很多？

A: 可能的原因：
- `max_error` 设置太小
- `min_segment_len` 太小
- 原始点很嘈杂（先预处理）

解决：
```python
# 增大容忍度
result = fit_curve(points, max_error=3.0)  # 原来是1.0

# 增加最小段长
result = fit_curve(points, min_segment_len=40)  # 原来是20
```

### Q: 性能很慢怎么办？

A: 检查：
1. 是否使用了 `--release` 编译
2. 点数是否>10000（考虑分块）
3. `max_segment_len` 是否太大

```python
# 快速模式
result = fit_curve(
    points,
    min_segment_len=50,  # 增大
    max_segment_len=200,  # 限制
    max_error=3.0  # 放宽
)
```

### Q: 如何处理多条分离的线？

A: 需要先分离再分别拟合：

```python
def fit_multiple_contours(skeleton_image):
    """处理多条轮廓"""
    from skimage import measure
    
    # 找到所有连通区域
    labeled = measure.label(skeleton_image)
    
    results = []
    for region_id in range(1, labeled.max() + 1):
        # 提取单条线的点
        y, x = np.where(labeled == region_id)
        points = list(zip(x, y))
        
        # 排序点（按连续性）
        points = order_points(points)  # 你需要实现
        
        # 拟合
        result = fit_curve(points)
        results.append(result)
    
    return results
```

---

## 📖 进阶主题

### 使用 Rust API（如果你想在Rust中用）

```rust
use bezier_dp_fit::{Point2D, FitConfig, fit_curve};

fn main() {
    let points: Vec<Point2D> = vec![
        Point2D::new(0.0, 0.0),
        Point2D::new(10.0, 10.0),
        // ...
    ];

    let config = FitConfig::new(30, 200, 2.0);
    let result = fit_curve(&points, &config);

    println!("Segments: {}", result.num_segments);
    println!("Error: {}", result.total_error);
    println!("SVG: {}", result.to_svg_path());
}
```

### 性能测试

```bash
# 运行benchmark
cargo bench

# 运行测试
cargo test

# 查看生成的报告
open target/criterion/report/index.html
```

---

## 🎓 学习资源

- **贝塞尔曲线**: https://javascript.info/bezier-curve
- **动态规划**: https://www.geeksforgeeks.org/dynamic-programming/
- **PyO3文档**: https://pyo3.rs/
- **Rust学习**: https://doc.rust-lang.org/book/

---

**祝你使用愉快！遇到问题随时查阅本文档。** 🚀