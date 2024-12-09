import math
from typing import List

type Coords = tuple[int, int]


def generate_grid_coords(cols=8, rows=8) -> List[int, int]:
    grid = []

    for row in range(0, rows):
        for col in range(0, cols):
            grid.append([row, col])

    return grid


def generate_nb_a(n):
    nbs = []

    for i in range(0, n):
        nbs.append(1 + math.floor(i / 2))

    return nbs


def generate_circle_from(start=[3, 3], radius=1):
    [x, y] = start

    pairs = []

    for s in range(0, radius):
        for c in range(0, 6):
            for d in range(0, radius - 1):
                if c == 0:
                    new_pairs = [x]


print(generate_nb_a(10))
