# 🚀 零基础快速开始指南

**完全不懂Rust？没关系！** 跟着这个教程，10分钟内跑起来。

---

## 📋 准备清单

- [ ] Windows 10/11, macOS, 或 Linux
- [ ] Python 3.8+
- [ ] 网络连接（下载工具）

---

## 第1步：安装 Rust（5分钟）

### Windows 用户

1. 打开 https://rustup.rs/
2. 点击下载 `rustup-init.exe`
3. 运行安装程序
4. 看到选项时，**直接按回车**（选默认）
5. 等待安装完成（会下载约400MB）
6. **关闭当前终端，重新打开**

验证：
```powershell
rustc --version
# 应该显示: rustc 1.x.x
```

### macOS/Linux 用户

打开终端，运行：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

按回车选默认，然后：
```bash
source $HOME/.cargo/env
rustc --version
```

---

## 第2步：下载项目代码

有两种方式：

### 方式A: 使用git
```bash
git clone <你的仓库地址>
cd bezier-dp-fit
```

### 方式B: 手动创建

创建文件夹 `bezier-dp-fit`，然后把我给你的所有代码文件放进去，保持这个结构：

```
bezier-dp-fit/
├── Cargo.toml
├── pyproject.toml
├── src/
│   ├── lib.rs
│   ├── geometry/
│   │   ├── mod.rs
│   │   ├── point.rs
│   │   └── bezier.rs
│   ├── fitting/
│   │   ├── mod.rs
│   │   └── fitter.rs
│   ├── optimizer/
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   └── dp.rs
│   └── python/
│       ├── mod.rs
│       └── bindings.rs
└── examples/
    └── example.py
```

---

## 第3步：安装 Python 工具

```bash
pip install maturin numpy
```

如果你用的是 conda：
```bash
conda install -c conda-forge maturin numpy
```

---

## 第4步：编译和安装（最关键！）

打开终端，**进入项目文件夹**：
```bash
cd bezier-dp-fit
```

然后运行：
```bash
maturin develop --release
```

你会看到：
```
🍹 Building a mixed python/rust project
🔗 Found pyo3 bindings
🐍 Found CPython 3.x at ...
📦 Built wheel ...
✨ Installed bezier-dp-fit-0.1.0
```

**这一步会比较慢（1-3分钟），第一次编译Rust需要下载依赖。**

---

## 第5步：测试是否成功

### 快速测试
```bash
python -c "import bezier_dp_fit; print('✅ 安装成功！')"
```

### 完整测试

创建文件 `test.py`：
```python
from bezier_dp_fit import fit_curve_py as fit_curve

# 简单的直线
points = [(i, i*2) for i in range(50)]

result = fit_curve(points, min_segment_len=10, max_segment_len=50, max_error=2.0)

print(f"✅ 成功！分了 {result.num_segments} 段")
print(f"总误差: {result.total_error:.2f}")
print(f"SVG: {result.to_svg()[:50]}...")
```

运行：
```bash
python test.py
```

看到这个就成功了：
```
✅ 成功！分了 2 段
总误差: 0.15
SVG: M 0.00 0.00 Q 12.50 25.00, 25.00 50.00 Q 37...
```

---

## 第6步：运行完整示例

```bash
python examples/example.py
```

会生成 `output.svg` 文件，用浏览器打开可以看到拟合结果！

---

## 🎯 现在开始用你自己的数据

```python
from bezier_dp_fit import fit_curve_py as fit_curve

# 替换成你的骨架点
my_points = [
    (10, 20), (11, 21), (12, 23), # ...
    # 从你的图像骨架中提取的坐标
]

# 拟合
result = fit_curve(
    points=my_points,
    min_segment_len=30,      # 根据你的图像分辨率调整
    max_segment_len=200,
    max_error=2.0            # 越小越精确，但分段越多
)

# 使用结果
svg_path = result.to_svg()
print(svg_path)

# 或者获取控制点自己处理
for p0, p1, p2 in result.control_points():
    print(f"贝塞尔曲线: {p0} -> {p1} -> {p2}")
```

---

## ❓ 遇到问题？

### 问题1: `maturin: command not found`

**解决:**
```bash
pip install --user maturin
# 然后把 ~/.local/bin 加入PATH（Linux/Mac）
# 或重启终端（Windows）
```

### 问题2: `error: linker 'link.exe' not found` (Windows)

**原因:** 缺少C++编译器

**解决:** 安装 Visual Studio Build Tools
1. 下载: https://visualstudio.microsoft.com/downloads/
2. 选择 "Tools for Visual Studio"
3. 下载 "Build Tools for Visual Studio 2022"
4. 安装时勾选 "Desktop development with C++"
5. 重启后重新运行 `maturin develop --release`

### 问题3: 编译很慢

**正常的！** 第一次编译Rust会下载和编译所有依赖，可能需要3-5分钟。

后续修改后重新编译只需要10-30秒。

### 问题4: `ImportError` 找不到模块

**检查:**
```bash
# 1. 确认你在正确的Python环境
which python   # 或 where python (Windows)

# 2. 确认安装到了这个环境
pip list | grep bezier

# 3. 重新安装
cd bezier-dp-fit
maturin develop --release
```

### 问题5: 运行时出错 "points must be ..."

**原因:** 输入格式不对

**正确格式:**
```python
# ✅ 对
points = [(1.0, 2.0), (3.0, 4.0)]
points = [[1, 2], [3, 4]]
points = np.array([[1, 2], [3, 4]])

# ❌ 错
points = [1, 2, 3, 4]  # 不是点的列表
points = [(1,), (2,)]  # 每个点需要2个坐标
```

---

## 🎓 下一步

- 阅读 `README.md` 了解参数调优
- 看 `examples/example.py` 学习更多用法
- 修改 `src/` 下的Rust代码（如果你想定制）
- 运行 `cargo test` 执行单元测试

---

## 💡 小贴士

- 修改代码后，运行 `maturin develop --release` 重新编译
- `--release` 很重要，影响性能10倍以上
- 第一次编译慢，之后就快了
- 遇到问题先 `cargo clean`，然后重新编译

---

**完成！现在你有一个高性能的贝塞尔拟合库了！🎉**