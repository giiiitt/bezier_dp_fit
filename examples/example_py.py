"""
Bezier curve fitting examples.
"""
import argparse
import os
import numpy as np
from bezier_dp_fit import fit_curve_py as fit_curve

try:
    import cv2
except Exception:  # optional dependency for image demo
    cv2 = None


def _ensure_cv2():
    if cv2 is None:
        raise RuntimeError(
            "opencv-python is required for image demo. "
            "Install with: pip install opencv-python"
        )


def read_skeleton(image_path):
    _ensure_cv2()
    img = cv2.imread(image_path, cv2.IMREAD_GRAYSCALE)
    if img is None:
        raise FileNotFoundError(f"cannot read image: {image_path}")
    return img, img.shape[1], img.shape[0]


def find_critical_points(skel):
    h, w = skel.shape
    criticals = []
    intersection_points = set()
    for y in range(1, h - 1):
        for x in range(1, w - 1):
            if skel[y, x] != 0:
                continue
            p = [
                skel[y - 1, x], skel[y - 1, x + 1], skel[y, x + 1], skel[y + 1, x + 1],
                skel[y + 1, x], skel[y + 1, x - 1], skel[y, x - 1], skel[y - 1, x - 1]
            ]
            b = [1 if v == 0 else 0 for v in p]
            transitions = 0
            for i in range(8):
                if b[i] == 0 and b[(i + 1) % 8] == 1:
                    transitions += 1
            if transitions == 2:
                continue
            pt = (x, y)
            criticals.append(pt)
            if sum(b) >= 3:
                intersection_points.add(pt)
    return criticals, intersection_points


def trace_paths(skel, criticals, intersection_points, min_path_len):
    """
    Build paths by walking between critical points.
    """
    criticals = set(criticals)

    def get_neighbors(x, y):
        for dy in [-1, 0, 1]:
            for dx in [-1, 0, 1]:
                if dx == 0 and dy == 0:
                    continue
                nx, ny = x + dx, y + dy
                if (0 <= nx < skel.shape[1]) and (0 <= ny < skel.shape[0]):
                    if skel[ny, nx] == 0:
                        yield (nx, ny)

    def group_neighbor_branches(neighbors):
        components = []
        remaining = set(neighbors)
        while remaining:
            start = remaining.pop()
            comp = [start]
            stack = [start]
            while stack:
                p = stack.pop()
                for q in list(remaining):
                    if max(abs(p[0] - q[0]), abs(p[1] - q[1])) <= 1:
                        remaining.remove(q)
                        comp.append(q)
                        stack.append(q)
            components.append(comp)
        return components

    visited_edges = set()
    paths = []

    # Trace paths between critical points.
    for start in criticals:
        for nbr in get_neighbors(*start):
            if (start, nbr) in visited_edges:
                continue
            path = [start, nbr]
            visited_edges.add((start, nbr))
            visited_edges.add((nbr, start))
            prev, cur = start, nbr

            reason = None
            while cur not in criticals:
                neighbors = [n for n in get_neighbors(*cur)]
                nbrs = [n for n in neighbors if n != prev and (cur, n) not in visited_edges]
                if len(nbrs) == 0:
                    reason = 'dead_end'
                    break
                critical_nbrs = [n for n in nbrs if n in criticals]
                if critical_nbrs:
                    next_pt = critical_nbrs[0]
                elif len(nbrs) > 1:
                    components = group_neighbor_branches(neighbors)
                    branches = len(components)
                    prev_comp = None
                    for comp in components:
                        if prev in comp:
                            prev_comp = comp
                            break
                    if prev in criticals and branches == 2 and prev_comp is not None:
                        other_comps = [c for c in components if c is not prev_comp]
                        single_comps = [c for c in other_comps if len(c) == 1]
                        if len(single_comps) == 1:
                            candidate = single_comps[0][0]
                            if candidate in nbrs:
                                next_pt = candidate
                            else:
                                reason = 'ambiguous'
                                break
                        else:
                            reason = 'ambiguous'
                            break
                    else:
                        reason = 'ambiguous'
                        break
                else:
                    next_pt = nbrs[0]

                path.append(next_pt)
                visited_edges.add((cur, next_pt))
                visited_edges.add((next_pt, cur))
                prev, cur = cur, next_pt

            if reason is None:
                if cur in criticals:
                    reason = 'at_critical'
                else:
                    reason = 'stopped'

            path_len = len(path)
            if path_len >= min_path_len:
                keep = True
            elif path_len < 2:
                keep = False
            else:
                keep = path[0] in intersection_points and path[-1] in intersection_points

            if keep:
                paths.append(path)

    # Trace closed loops without critical points.
    h, w = skel.shape
    for y in range(1, h - 1):
        for x in range(1, w - 1):
            if skel[y, x] != 0:
                continue
            if (x, y) in criticals:
                continue
            for nbr in get_neighbors(x, y):
                if ((x, y), nbr) in visited_edges:
                    continue
                start = (x, y)
                path = [start, nbr]
                path_set = {start, nbr}
                visited_edges.add((start, nbr))
                visited_edges.add((nbr, start))
                prev, cur = start, nbr
                while True:
                    if cur in criticals:
                        break
                    nbrs = [n for n in get_neighbors(*cur) if n != prev]
                    if not nbrs:
                        break
                    next_pt = None
                    for cand in nbrs:
                        if (cur, cand) not in visited_edges:
                            next_pt = cand
                            break
                    if next_pt is None:
                        close_pt = None
                        if start in nbrs:
                            close_pt = start
                        else:
                            for cand in nbrs:
                                if cand in path_set:
                                    close_pt = cand
                                    break
                        if close_pt is None:
                            break
                        visited_edges.add((cur, close_pt))
                        visited_edges.add((close_pt, cur))
                        path.append(close_pt)
                        if close_pt != start:
                            loop_start_idx = path.index(close_pt)
                            loop_path = path[loop_start_idx:]
                        else:
                            loop_path = path
                        if len(loop_path) >= min_path_len:
                            paths.append(loop_path)
                        break
                    if next_pt in path_set and next_pt != start:
                        visited_edges.add((cur, next_pt))
                        visited_edges.add((next_pt, cur))
                        path.append(next_pt)
                        loop_start_idx = path.index(next_pt)
                        loop_path = path[loop_start_idx:]
                        if len(loop_path) >= min_path_len:
                            paths.append(loop_path)
                        break
                    path.append(next_pt)
                    path_set.add(next_pt)
                    visited_edges.add((cur, next_pt))
                    visited_edges.add((next_pt, cur))
                    if next_pt == start:
                        if len(path) >= min_path_len:
                            paths.append(path)
                        break
                    prev, cur = cur, next_pt

    return paths


def save_svg(path_strings, width, height, out_path, stroke_width=1.0):
    header = (
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}">\n'
    )
    body = []
    for d in path_strings:
        body.append(
            f'  <path d="{d}" fill="none" stroke="black" stroke-width="{stroke_width}"/>\n'
        )
    footer = "</svg>\n"
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(header)
        f.writelines(body)
        f.write(footer)


def run_image_demo(args):
    skel, w, h = read_skeleton(args.image)
    criticals, intersection_points = find_critical_points(skel)
    paths = trace_paths(skel, criticals, intersection_points, args.min_path_len)
    if not paths:
        print("no paths found")
        return

    path_strings = []
    total_segments = 0
    total_control_points = 0
    total_points = 0

    for pts in paths:
        if len(pts) < 3:
            continue
        closed = len(pts) > 1 and pts[0] == pts[-1]
        max_error = args.max_error_closed if closed else args.max_error
        result = fit_curve(
            points=pts,
            min_segment_len=args.min_segment_len,
            max_segment_len=args.max_segment_len,
            max_error=max_error,
        )
        path_strings.append(result.to_svg())
        total_segments += result.num_segments
        total_control_points += len(result.control_points())
        total_points += len(pts)

    if not path_strings:
        print("no valid paths to export")
        return

    out_path = args.output
    save_svg(path_strings, w, h, out_path, stroke_width=args.stroke_width)
    avg_len = total_points / max(len(paths), 1)
    print(f"paths: {len(paths)}")
    print(f"critical_points: {len(criticals)} intersections: {len(intersection_points)}")
    print(f"points_total: {total_points} avg_path_len: {avg_len:.2f}")
    print(f"segments_total: {total_segments} control_points_total: {total_control_points}")
    print(f"saved: {os.path.abspath(out_path)}")


def run_basic_demo():
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

    print(f"分段数 {result.num_segments}")
    print(f"总误差 {result.total_error:.2f}")
    print(f"\nSVG路径:\n{result.to_svg()}")

    print("\n控制点")
    for i, cp in enumerate(result.control_points()):
        print(f"段{i+1}: {cp}")


def main():
    parser = argparse.ArgumentParser(description="Bezier DP Fit examples")
    parser.add_argument("--image", type=str, help="input skeleton image")
    parser.add_argument("--output", type=str, default="output.svg", help="output svg")
    parser.add_argument("--min-path-len", type=int, default=5, help="min path length")
    parser.add_argument("--min-segment-len", type=int, default=3, help="min segment len")
    parser.add_argument("--max-segment-len", type=int, default=20000, help="max segment len")
    parser.add_argument("--max-error", type=float, default=2.2, help="max error for open paths")
    parser.add_argument("--max-error-closed", type=float, default=0.5, help="max error for closed paths")
    parser.add_argument("--stroke-width", type=float, default=1.0, help="svg stroke width")
    args = parser.parse_args()

    if args.image:
        run_image_demo(args)
        return
    run_basic_demo()


if __name__ == "__main__":
    main()
