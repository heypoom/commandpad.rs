import random
import socket
import time
from typing import List


def create_socket():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect(("localhost", 7878))
    return s


def message_set_tile_color(position: int, color: int):
    return bytearray([0x03, position, color])


def message_set_tile_rgb(position: int, r: int, g: int, b: int):
    return bytearray([0x04, position, r, g, b])


def message_set_grid(grid: List[int]):
    return bytearray([0x05] + grid)


def spec_rgb(position: int, r: int, g: int, b: int):
    return [3, position, r, g, b]


# for color in range(0, 255):
#     for position in range(11, 100):
#         s = create_socket()
#         s.send(message_set_tile_rgb(position, color, 0, color))


def rnd():
    return random.randint(0, 127)


def flatten(t):
    return [item for sublist in t for item in sublist]


def send_random():
    s = create_socket()

    specs = [spec_rgb(position, 0, rnd(), rnd()) for position in range(11, 100)]
    s.send(message_set_grid(flatten(specs)))


def to_position(x: int, y: int) -> int:
    return 101 - (10 * y) + x


test_grid = """
yyyyyyyyy
xxxxxxxxy
xooooooxy
xoxxxxoxy
xoxooxoxy
xoxooxoxy
xoxxxxoxy
xooooooxy
xxxxxxxxy
"""

grid_text_mapping = {"x": [0, 50, 50], "y": [0, 127, 0], "o": [0, 0, 127]}


def show_random():
    for i in range(1, 1000):
        time.sleep(1000 / 24 / 1000)
        send_random()


def print_text_grid(text):
    s = create_socket()
    specs = []

    for row, line in enumerate(text.split("\n")):
        for col, char in enumerate(list(line)):
            print(char)
            rgb_color = grid_text_mapping[char]
            position = to_position(col, row)
            print("({}, {}) = {}".format(col, row, position))

            specs += [3, position] + rgb_color

    print(specs)

    msg = message_set_grid(specs)
    print(msg)

    s.send(msg)


print_text_grid(test_grid)
print("ok")
