# Bezier DP Fit - 高性能贝塞尔曲线拟合库

用动态规划(DP)算法拟合二次贝塞尔曲线，专为像素骨架线稿矢量化设计。

## ✨ 特性

- 🚀 **极快**: Rust实现，并行计算，1000点<10ms
- 🎯 **最优解**: DP保证全局最优分段
- 🐍 **Python友好**: 原生支持numpy数组
- 📐 **灵活配置**: 可调整段长、误差容忍度
- 📦 **零依赖**: Python端只需numpy

---

## 📦 安装环境

### 第一步：安装 Rust

**Windows:**
1. 下载 https://rustup.rs/
2. 运行安装程序，按默认选项
3. 重启终端

**macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

验证安装：
```bash
rustc --version
cargo --version
```

### 第二步：安装 Python 构建工具

```bash
pip install maturin
```

---

## 🔧 编译和安装

### 方式1: 开发模式（推荐用于测试）

在项目根目录运行：

```bash
# 编译并安装到当前Python环境（可编辑模式）
maturin develop --release

# 如果遇到问题，先清理再编译
cargo clean
maturin develop --release
```

### 方式2: 构建wheel包

```bash
# 构建wheel包
maturin build --release

# 安装生成的wheel
pip install target/wheels/bezier_dp_fit-*.whl
```

---

## 🚀 快速开始

### Python 使用

```python
from bezier_dp_fit import fit_curve_py as fit_curve

# 输入：骨架点集（有序）
points = [
    (10, 1), (10, 2), (11, 3), (12, 4), # ...
    # 你的骨架坐标
]

# 拟合
result = fit_curve(
    points=points,
    min_segment_len=30,    # 最小段长（像素）
    max_segment_len=200,   # 最大段长
    max_error=2.0          # 最大允许误差
)

# 查看结果
print(f"分了 {result.num_segments} 段")
print(f"总误差: {result.total_error:.2f}")

# 获取SVG路径
svg_path = result.to_svg()
print(svg_path)  # "M 10 1 Q 15 5, 20 2 Q ..."

# 获取控制点
for segment in result.control_points():
    p0, p1, p2 = segment
    print(f"起点{p0}, 控制点{p1}, 终点{p2}")
```

### Numpy 数组输入

```python
import numpy as np

points = np.array([
    [0, 0], [1, 1], [2, 4], # ...
], dtype=float)

result = fit_curve(points)
```

---

## 📖 API 文档

### `fit_curve_py(points, min_segment_len=30, max_segment_len=200, max_error=2.0)`

**参数:**
- `points`: 点集
  - 列表: `[(x1,y1), (x2,y2), ...]`
  - Numpy数组: `shape=(N, 2)`
- `min_segment_len`: 最小段长（像素），默认30
- `max_segment_len`: 最大段长（像素），默认200
- `max_error`: 最大允许误差，默认2.0

**返回:** `FitResult` 对象

### `FitResult` 对象

**属性:**
- `num_segments`: 分段数量（只读）
- `total_error`: 总误差（只读）

**方法:**
- `to_svg()`: 返回SVG路径字符串
- `control_points()`: 返回控制点列表 `[[(x0,y0), (x1,y1), (x2,y2)], ...]`
- `sample_points(n)`: 采样n个点/段，返回 `[(x,y), ...]`
- `to_json()`: 导出为JSON字符串

---

## 🎨 完整示例

```python
from bezier_dp_fit import fit_curve_py as fit_curve
import numpy as np

# 1. 准备数据（你的骨架提取代码）
skeleton_points = [...]  # 从图像中提取

# 2. 拟合
result = fit_curve(
    points=skeleton_points,
    min_segment_len=20,
    max_segment_len=150,
    max_error=1.5
)

# 3. 导出SVG
svg = f'''
<svg viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
  <path d="{result.to_svg()}" 
        fill="none" 
        stroke="black" 
        stroke-width="2"/>
</svg>
'''

with open('output.svg', 'w') as f:
    f.write(svg)

print(f"✅ 完成！分了{result.num_segments}段，误差{result.total_error:.2f}")
```

---

## ⚙️ 参数调优指南

### `min_segment_len` (最小段长)

- **太小 (<20)**: 过度分段，SVG文件大
- **合适 (20-40)**: 平衡性能和精度
- **太大 (>50)**: 细节丢失

**推荐值:**
- 低分辨率图像(512px): 15-25
- 中等分辨率(1024px): 25-35
- 高分辨率(2048px+): 30-50

### `max_segment_len` (最大段长)

- **太小**: 限制性能提升
- **合适**: 5-8倍 min_segment_len
- **太大**: 无额外效果，增加内存

**推荐值:** `min * 6` 到 `min * 8`

### `max_error` (最大误差)

- **严格 (<1.0)**: 高精度，更多分段
- **平衡 (1.5-3.0)**: 大多数情况适用
- **宽松 (>4.0)**: 平滑但细节少

---

## 🔍 故障排查

### 问题1: `maturin: command not found`

```bash
pip install --upgrade maturin
# 或
pip install maturin --user
```

### 问题2: 编译失败 "linker not found"

**Windows:** 安装 Visual Studio Build Tools
**Linux:** `sudo apt install build-essential`
**macOS:** `xcode-select --install`

### 问题3: `ImportError: No module named 'bezier_dp_fit'`

确保在正确的Python环境中：
```bash
which python  # 查看当前Python路径
maturin develop --release  # 重新安装
python -c "import bezier_dp_fit; print('OK')"
```

### 问题4: 性能不理想

- 增大 `min_segment_len`
- 放宽 `max_error`
- 检查点数量（>10000点建议分块处理）

---

## 📊 性能参考

测试环境: AMD Ryzen 7, 16GB RAM

| 点数 | 配置 | 时间 | 分段数 |
|-----|------|------|--------|
| 100 | 默认 | <1ms | 3-5 |
| 1000 | 默认 | 5-10ms | 20-35 |
| 5000 | 默认 | 50-100ms | 90-150 |
| 10000 | 默认 | 200-400ms | 180-300 |

---

## 📁 项目结构

```
bezier-dp-fit/
├── Cargo.toml          # Rust配置
├── pyproject.toml      # Python配置
├── src/
│   ├── lib.rs          # 库入口
│   ├── geometry/       # 几何结构
│   ├── fitting/        # 拟合算法
│   ├── optimizer/      # DP优化器
│   └── python/         # Python绑定
├── examples/
│   └── example.py      # 使用示例
└── README.md
```

---

## 🤝 贡献

欢迎提Issue和PR！

## 📄 License

MIT License