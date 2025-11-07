import io
import random

# just edit the python file itself

tile_size = 1 # Dont change this
map_path = "assets/maps/map1-cages.map"
# W = wall
# U = unbreakable_wall
# I = Water, edge
# L = Water, full
map_drawing = """
.......UUUUUUUUUUUUUUUUUUUUUUUU....
.......U............U.........U....
.......U............W.........U....
.......U............W.........U....
...UUUUU......UWW...WWWWWU....UUUUU
...U..............................U
...U..............................U
...U..............................U
UUUU.......W......................U
U..........UWW....W....UWW........U
U......................U..........U
U.................................U
U.................................U
U....W.......................U....U
U....U...........O...........U....U
U....U.......................W....U
U.................................U
U.................................U
U..........U......................U
U........WWU....W....WWU..........U
U......................W.......UUUU
U..............................U...
U..............................U...
U..............................U...
UUUUU....UWWWWW...WWU......UUUUU...
....U.........W............U.......
....U.........W............U.......
....U.........U............U.......
....UUUUUUUUUUUUUUUUUUUUUUUU.......
"""
map_drawing = """
.......UUUUUUUUUUUUUUUUUUUUUUUU....
.......U............U.........U....
.......U............W.........U....
.......U............W.........U....
...UUUUU......UWW...WWWWWU...WUUUUU
...U.........................W....U
...U.........................W....U
...U.........................W....U
UUUUWW.....W.................W....U
U....W.....UWW....W....UWW...W....U
U....W.................U.....W....U
U....W.......................W....U
U....W.......................W....U
U....W.......................U....U
U....U...........O...........U....U
U....U.......................W....U
U....W.......................W....U
U....W.......................W....U
U....W.....U.................W....U
U....W...WWU....W....WWU.....W....U
U....W.................W.....WWUUUU
U....W.........................U...
U....W.........................U...
U....W.........................U...
UUUUUW...UWWWWW...WWU......UUUUU...
....U.........W............U.......
....U.........W............U.......
....U.........U............U.......
....UUUUUUUUUUUUUUUUUUUUUUUU.......
"""
map_y_offset = -1
map_x_offset = -1
map_file = ""
for y, line in enumerate(io.StringIO(map_drawing)):
    for x, char in enumerate(repr(line)):
        position = f"{(x+ map_x_offset)*tile_size}.0 {(y + map_y_offset)*tile_size}.0\n"
        if char == 'W':
            map_file += "wall " + position
        if char == 'U':
            map_file += "unbreakablewall " + position
        if char == 'I':
            map_file += "water1 " + position
        if char == 'L':
            map_file += "water2 " + position
        if char == 'O':
            map_file += "orb " + position

        #bg_tiles = ["grass-1", "grass-2", "grass-3", "grass-4", "grass-5"]
        #bg_tile = random.choice(bg_tiles)
        #if (x + y) % 2 == 1:
        #    bg_tile += "-b"
        #map_file += f"{bg_tile} {position}"
with open(map_path, 'w') as file:
    file.write(map_file)