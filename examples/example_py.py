"""
贝塞尔曲线拟合示例
"""
import numpy as np
from bezier_dp_fit import fit_curve_py as fit_curve

# 示例1: 使用列表
print("=== 示例1: 列表输入 ===")
points = [
    (10, 1), (10, 2), (11, 3), (12, 4), (13, 5),
    (14, 6), (15, 7), (16, 8), (17, 9), (18, 10),
    (19, 11), (20, 12), (21, 13), (22, 14), (23, 15),
    (24, 16), (25, 17), (26, 18), (27, 19), (28, 20),
    (29, 21), (30, 22), (31, 23), (32, 24), (33, 25),
    (34, 26), (35, 27), (36, 28), (37, 29), (38, 30),
    (39, 31), (40, 32), (41, 33), (42, 34), (43, 35),
]

result = fit_curve(
    points=points,
    min_segment_len=10,
    max_segment_len=50,
    max_error=2.0
)

print(f"分段数: {result.num_segments}")
print(f"总误差: {result.total_error:.2f}")
print(f"\nSVG路径:\n{result.to_svg()}")

print("\n控制点:")
for i, cp in enumerate(result.control_points()):
    print(f"段{i+1}: {cp}")

# 示例2: 使用numpy数组
print("\n=== 示例2: Numpy数组输入 ===")
np_points = np.array([
    [0, 0], [1, 1], [2, 4], [3, 9], [4, 16],
    [5, 25], [6, 36], [7, 49], [8, 64], [9, 81],
    [10, 100], [11, 121], [12, 144], [13, 169], [14, 196],
    [15, 225], [16, 256], [17, 289], [18, 324], [19, 361],
    [20, 400], [21, 441], [22, 484], [23, 529], [24, 576],
    [25, 625], [26, 676], [27, 729], [28, 784], [29, 841],
    [30, 900],
], dtype=float)

result2 = fit_curve(
    points=np_points,
    min_segment_len=8,
    max_segment_len=40,
    max_error=5.0
)

print(f"分段数: {result2.num_segments}")
print(f"总误差: {result2.total_error:.2f}")

# 示例3: 采样拟合结果
print("\n=== 示例3: 采样曲线 ===")
sampled = result.sample_points(points_per_segment=20)
print(f"采样了 {len(sampled)} 个点")
print(f"前5个点: {sampled[:5]}")

# 示例4: 导出为SVG文件
print("\n=== 示例4: 导出SVG ===")
svg_content = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <!-- 原始点 -->
  <g fill="red" opacity="0.5">
    {''.join(f'<circle cx="{p[0]}" cy="{p[1]}" r="0.5"/>' for p in points[:20])}
  </g>
  
  <!-- 拟合曲线 -->
  <path d="{result.to_svg()}" 
        fill="none" 
        stroke="blue" 
        stroke-width="0.5"/>
</svg>'''

with open('output.svg', 'w') as f:
    f.write(svg_content)
print("已保存到 output.svg")

# 示例5: JSON导出
print("\n=== 示例5: JSON导出 ===")
json_str = result.to_json()
print(json_str[:200] + "...")  # 只显示前200字符