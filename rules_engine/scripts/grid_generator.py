# Example:
# python3 ./rules_engine/scripts/grid_generator.py --print-legend --min-paths 2
# --emoji --max-cost 2 --max-reward 3 --allow-backtracking 

import argparse
import sys
import time
from collections import deque
import random


def neighbors(r, c, rows, cols):
    for dr, dc in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        nr = r + dr
        nc = c + dc
        if 0 <= nr < rows and 0 <= nc < cols:
            yield nr, nc


def is_passable(ch):
    return ch != 'W'


def bfs_min_cost(grid, start, end):
    rows = len(grid)
    cols = len(grid[0])
    dq = deque()
    inf = 10 ** 12
    dist = [[inf] * cols for _ in range(rows)]
    sr, sc = start
    er, ec = end
    dist[sr][sc] = 0
    dq.append((sr, sc))
    while dq:
        r, c = dq.popleft()
        if (r, c) == (er, ec):
            return dist[r][c]
        for nr, nc in neighbors(r, c, rows, cols):
            if not is_passable(grid[nr][nc]):
                continue
            w = 1 if grid[nr][nc] == 'C' else 0
            nd = dist[r][c] + w
            if nd < dist[nr][nc]:
                dist[nr][nc] = nd
                if w == 0:
                    dq.appendleft((nr, nc))
                else:
                    dq.append((nr, nc))
    return None


def bfs_distances(grid, start):
    rows = len(grid)
    cols = len(grid[0])
    q = deque()
    dist = [[-1] * cols for _ in range(rows)]
    sr, sc = start
    if not is_passable(grid[sr][sc]):
        return dist
    dist[sr][sc] = 0
    q.append((sr, sc))
    while q:
        r, c = q.popleft()
        for nr, nc in neighbors(r, c, rows, cols):
            if dist[nr][nc] != -1:
                continue
            if not is_passable(grid[nr][nc]):
                continue
            dist[nr][nc] = dist[r][c] + 1
            q.append((nr, nc))
    return dist


def farthest_odd_cell_pair(grid):
    rows = len(grid)
    cols = len(grid[0])
    start = None
    for r in range(1, rows, 2):
        for c in range(1, cols, 2):
            if is_passable(grid[r][c]):
                start = (r, c)
                break
        if start:
            break
    if not start:
        return None, None
    d1 = bfs_distances(grid, start)
    a = start
    maxd = -1
    for r in range(1, rows, 2):
        for c in range(1, cols, 2):
            if d1[r][c] > maxd:
                maxd = d1[r][c]
                a = (r, c)
    d2 = bfs_distances(grid, a)
    b = a
    maxd = -1
    for r in range(1, rows, 2):
        for c in range(1, cols, 2):
            if d2[r][c] > maxd:
                maxd = d2[r][c]
                b = (r, c)
    return a, b


def has_path(grid, start, end):
    rows = len(grid)
    cols = len(grid[0])
    q = deque()
    seen = set()
    if not is_passable(grid[start[0]][start[1]]):
        return False
    q.append(start)
    seen.add(start)
    while q:
        r, c = q.popleft()
        if (r, c) == end:
            return True
        for nr, nc in neighbors(r, c, rows, cols):
            if (nr, nc) in seen:
                continue
            if not is_passable(grid[nr][nc]):
                continue
            seen.add((nr, nc))
            q.append((nr, nc))
    return False


def count_paths_at_least(grid, start, end, threshold):
    rows = len(grid)
    cols = len(grid[0])
    sys.setrecursionlimit(1000000)
    target = threshold
    count = [0]
    visited = [[False] * cols for _ in range(rows)]

    def dfs(r, c):
        if count[0] >= target:
            return
        if (r, c) == end:
            count[0] += 1
            return
        for nr, nc in neighbors(r, c, rows, cols):
            if not is_passable(grid[nr][nc]):
                continue
            if visited[nr][nc]:
                continue
            visited[nr][nc] = True
            dfs(nr, nc)
            visited[nr][nc] = False

    sr, sc = start
    visited[sr][sc] = True
    dfs(sr, sc)
    return count[0] >= target


def enumerate_walls_between_odd_cells(grid):
    rows = len(grid)
    cols = len(grid[0])
    walls = []
    for r in range(1, rows, 2):
        for c in range(1, cols, 2):
            if r + 2 < rows and grid[r][c] != 'W' and grid[r + 2][c] != 'W':
                mr = r + 1
                mc = c
                if grid[mr][mc] == 'W':
                    walls.append((mr, mc))
            if c + 2 < cols and grid[r][c] != 'W' and grid[r][c + 2] != 'W':
                mr = r
                mc = c + 1
                if grid[mr][mc] == 'W':
                    walls.append((mr, mc))
    return walls


def carve_maze(rows, cols, rng):
    grid = [['W' for _ in range(cols)] for _ in range(rows)]
    cells = []
    for r in range(1, rows, 2):
        for c in range(1, cols, 2):
            cells.append((r, c))
    if not cells:
        return grid
    start = rng.choice(cells)
    stack = [start]
    visited = set([start])
    grid[start[0]][start[1]] = 'B'
    while stack:
        r, c = stack[-1]
        options = []
        for dr, dc in ((2, 0), (-2, 0), (0, 2), (0, -2)):
            nr = r + dr
            nc = c + dc
            if 1 <= nr < rows - 1 and 1 <= nc < cols - 1 and (nr, nc) not in visited:
                options.append((nr, nc, r + dr // 2, c + dc // 2))
        if options:
            nr, nc, mr, mc = rng.choice(options)
            grid[mr][mc] = 'B'
            grid[nr][nc] = 'B'
            visited.add((nr, nc))
            stack.append((nr, nc))
        else:
            stack.pop()
    return grid


def open_random_loops(grid, rng, openings):
    candidates = enumerate_walls_between_odd_cells(grid)
    rng.shuffle(candidates)
    opened = 0
    for mr, mc in candidates:
        if opened >= openings:
            break
        if grid[mr][mc] == 'W':
            grid[mr][mc] = 'B'
            opened += 1
    return opened


def choose_start_end(grid):
    a, b = farthest_odd_cell_pair(grid)
    return a, b


def assign_costs_and_rewards(grid, start, end, min_cost_required, max_rewards_allowed, max_cost_allowed, rng):
    rows = len(grid)
    cols = len(grid[0])
    sr, sc = start
    er, ec = end
    for r in range(rows):
        for c in range(cols):
            if (r, c) == (sr, sc) or (r, c) == (er, ec):
                continue
            if is_passable(grid[r][c]):
                grid[r][c] = 'C'
    grid[sr][sc] = 'S'
    grid[er][ec] = 'E'
    flips = max(1, (rows * cols) // 8)
    attempts = flips * 3
    for _ in range(attempts):
        r = rng.randrange(rows)
        c = rng.randrange(cols)
        if (r, c) == (sr, sc) or (r, c) == (er, ec):
            continue
        if grid[r][c] != 'C':
            continue
        grid[r][c] = 'B'
        mc = bfs_min_cost(grid, start, end)
        if mc is None or mc < min_cost_required:
            grid[r][c] = 'C'
    rewards_to_place = max_rewards_allowed
    reward_candidates_c = [(r, c) for r in range(rows) for c in range(cols) if grid[r][c] == 'C' and (r, c) != (sr, sc) and (r, c) != (er, ec)]
    reward_candidates_b = [(r, c) for r in range(rows) for c in range(cols) if grid[r][c] == 'B' and (r, c) != (sr, sc) and (r, c) != (er, ec)]
    rng.shuffle(reward_candidates_c)
    rng.shuffle(reward_candidates_b)
    for r, c in reward_candidates_c + reward_candidates_b:
        if rewards_to_place <= 0:
            break
        if grid[r][c] == 'R':
            continue
        prev = grid[r][c]
        grid[r][c] = 'R'
        mc = bfs_min_cost(grid, start, end)
        if mc is None or mc < min_cost_required:
            grid[r][c] = prev
        else:
            rewards_to_place -= 1

    if max_cost_allowed is not None:
        def count_c():
            return sum(1 for rr in range(rows) for cc in range(cols) if grid[rr][cc] == 'C')

        current_c = count_c()
        if current_c > max_cost_allowed:
            candidates = [(r, c) for r in range(rows) for c in range(cols) if grid[r][c] == 'C' and (r, c) != (sr, sc) and (r, c) != (er, ec)]
            rng.shuffle(candidates)
            for r, c in candidates:
                if current_c <= max_cost_allowed:
                    break
                grid[r][c] = 'B'
                mc = bfs_min_cost(grid, start, end)
                if mc is None or mc < min_cost_required:
                    grid[r][c] = 'C'
                else:
                    current_c -= 1
            if current_c > max_cost_allowed:
                return None

    return bfs_min_cost(grid, start, end)


def grid_to_strings(grid, unicode_border, use_color, use_emoji, cell_space):
    rows = len(grid)
    cols = len(grid[0])

    def colorize(s, color_code):
        if not use_color:
            return s
        return f"\x1b[{color_code}m{s}\x1b[0m"

    def render_tile(ch):
        if use_emoji:
            if ch == 'S':
                return colorize('🚩', '92')
            if ch == 'E':
                return colorize('🚪', '95')
            if ch == 'B':
                return colorize('⬜️', '37')
            if ch == 'R':
                return colorize('💎', '93')
            if ch == 'C':
                return colorize('⚔️', '91')
            if ch == 'W':
                return colorize('🧱', '90')
            return ch
        else:
            if ch == 'S':
                return colorize('S', '92')
            if ch == 'E':
                return colorize('E', '95')
            if ch == 'B':
                return colorize('B', '37')
            if ch == 'R':
                return colorize('R', '93')
            if ch == 'C':
                return colorize('C', '91')
            if ch == 'W':
                return colorize('W', '90')
            return ch

    sep = ' ' * max(1, cell_space)
    lines = [sep.join(render_tile(grid[r][c]) for c in range(cols)) for r in range(rows)]
    if not unicode_border:
        return lines
    content_width = max((len(line) for line in lines), default=cols)
    top = '┌' + ('─' * content_width) + '┐'
    bot = '└' + ('─' * content_width) + '┘'
    mid = ['│' + line.ljust(content_width) + '│' for line in lines]
    return [top] + mid + [bot]


def count_rewards(grid):
    return sum(1 for row in grid for ch in row if ch == 'R')


def try_generate(size, rng, min_paths, min_cost_required, max_rewards_allowed, max_cost_allowed, time_deadline, verbose):
    if time_deadline is not None and time.time() > time_deadline:
        return None
    n = size
    if n < 3:
        return None
    if n % 2 == 0:
        n += 1
    grid = carve_maze(n, n, rng)
    base_openings = max(1, (n * n) // 20)
    open_random_loops(grid, rng, base_openings)
    a, b = choose_start_end(grid)
    if not a or not b:
        return None
    if not has_path(grid, a, b):
        return None
    if not count_paths_at_least(grid, a, b, min_paths):
        extra_budget = max(1, (n * n) // 10)
        opened = open_random_loops(grid, rng, extra_budget)
        if opened == 0:
            return None
        a, b = choose_start_end(grid)
        if not a or not b:
            return None
        if not count_paths_at_least(grid, a, b, min_paths):
            return None
    mc = assign_costs_and_rewards(grid, a, b, min_cost_required, max_rewards_allowed, max_cost_allowed, rng)
    if mc is None or mc < min_cost_required:
        return None
    return grid


def count_paths_for_report(grid, start, end, cap):
    rows = len(grid)
    cols = len(grid[0])
    count = 0
    visited = [[False] * cols for _ in range(rows)]
    sys.setrecursionlimit(1000000)

    def dfs(r, c):
        nonlocal count
        if count >= cap:
            return
        if (r, c) == end:
            count += 1
            return
        for nr, nc in neighbors(r, c, rows, cols):
            if not is_passable(grid[nr][nc]):
                continue
            if visited[nr][nc]:
                continue
            visited[nr][nc] = True
            dfs(nr, nc)
            visited[nr][nc] = False

    visited[start[0]][start[1]] = True
    dfs(start[0], start[1])
    return count


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--max-rewards', type=int, default=3)
    parser.add_argument('--min-cost', type=int, default=2)
    parser.add_argument('--max-cost', type=int, default=None)
    parser.add_argument('--min-paths', type=int, default=2)
    parser.add_argument('--min-size', type=int, default=3)
    parser.add_argument('--max-size', type=int, default=15)
    parser.add_argument('--seed', type=int, default=None)
    parser.add_argument('--iterations', type=int, default=50000)
    parser.add_argument('--timeout', type=float, default=None)
    parser.add_argument('--print-legend', action='store_true')
    parser.add_argument('--unicode', action='store_true')
    parser.add_argument('--color', action='store_true')
    parser.add_argument('--emoji', action='store_true')
    parser.add_argument('--cell-space', type=int, default=1)
    parser.add_argument('--allow-backtracking', action='store_true')
    parser.add_argument('--verbose', action='store_true')
    args = parser.parse_args()

    rng = random.Random(args.seed)
    deadline = None
    if args.timeout is not None:
        deadline = time.time() + args.timeout

    attempts = 0
    success_grid = None
    chosen_size = None
    max_loop_size = args.max_size if args.max_size is not None else 99
    for size in range(max(3, args.min_size), max(args.min_size, max_loop_size) + 1):
        if size % 2 == 0:
            continue
        if args.timeout is not None and time.time() > deadline:
            break
        remaining_sizes = 0
        for s in range(size, max(args.min_size, args.max_size) + 1):
            if s % 2 == 1 and s >= max(3, args.min_size):
                remaining_sizes += 1
        per_size_budget = max(1, (args.iterations - attempts) // max(1, remaining_sizes))
        for _ in range(per_size_budget):
            if args.timeout is not None and time.time() > deadline:
                break
            if attempts >= args.iterations:
                break
            attempts += 1
            grid = try_generate(size, rng, args.min_paths, args.min_cost, args.max_rewards, args.max_cost, deadline, args.verbose)
            if grid is not None:
                success_grid = grid
                chosen_size = size
                break
        if success_grid is not None:
            break

    if success_grid is None:
        print('Failed to generate a dungeon satisfying constraints', file=sys.stderr)
        sys.exit(1)

    rows = len(success_grid)
    cols = len(success_grid[0])

    s = None
    e = None
    for r in range(rows):
        for c in range(cols):
            if success_grid[r][c] == 'S':
                s = (r, c)
            elif success_grid[r][c] == 'E':
                e = (r, c)

    lines = grid_to_strings(success_grid, args.unicode, args.color, args.emoji, args.cell_space)
    for line in lines:
        print(line)

    if args.print_legend:
        mc = bfs_min_cost(success_grid, s, e)
        cap = max(100, args.min_paths)
        path_count = count_paths_for_report(success_grid, s, e, cap)
        rewards_total = count_rewards(success_grid)
        print(f'size: {rows}x{cols}')
        print(f'assume_backtracking: {args.allow_backtracking}')
        if path_count >= cap:
            print(f'paths: >= {cap}')
        else:
            print(f'paths: {path_count} (simple)')
        print(f'min_cost: {mc}')
        print(f'rewards_total: {rewards_total}')


if __name__ == '__main__':
    main()


