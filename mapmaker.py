import io
import random

tile_size = 10
map_path = "assets/maps/map_maker.map"
# W = wall
# U = unbreakable_wall
# I = Water, edge
# L = Water, full
map_drawing = """
UUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUU
U.................................U
U.................................U
U..................WWWW...........U
U.....................W...........U
U.....................W...........U
U..........W..........W...........U
U..........W..........W...........U
U..........WU......UWWW...........U
U.................................U
U.................................U
U............................W....U
U....UWW........WUW..........U....U
U....U..........WUW........WWU....U
U....W............................U
U.................................U
U.................................U
U...........WWWU......UW..........U
U...........W..........W..........U
U...........W..........W..........U
U...........W.....................U
U...........W.....................U
U...........WWWW..................U
U.................................U
U.................................U
UUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUU
"""
map_drawing = """
.IIIII.......................II......
ILLLLLL..............................
.LLLLLUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUU
..U.................................U
..U.................................UU
..U..............................UWWUU
..U............WII..II..............UUW
..U............WLL..LLW....WW.......UWW
..U......W.....W...........WII......UU
..U......WWU...U...........ILL......UU
..U.................................U
..U...................UW............U
..U.................................UW
..U.........U.......................UW
..U....U....W....IIII.....W...WU....U
..U....UW...W.....LLLI....W....U....U
..U.......................U.........UU
..U.................................UWW
..U............WU...................UWW
..U.................................UW
..U......III...........U...UWW......UW
..U......LLW...........W.....W......UU
..U.......WW....WII..IIW............U
..U..............LL..LLW............U
..UWWU..............................U
..U.................................U
..U.................................U.III
.IUUUUUUUIIIIIIIIIIIIIIIIIUUUUUUUUUUUILLLIII
.LIIIIIIILLLLLLLLLLLUWLLLLIIIIIIIIIIILLLLLLLII
.LLLLLLLLLLLUWULLLLLIILLLLLLLLLLLLLLLLLLLLLLLLI
.LLLLLLLLLLLIIILLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLI
.LLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLL
"""

map_y_offset = -40
map_x_offset = -40

map_file = ""
for y, line in enumerate(io.StringIO(map_drawing)):
    for x, char in enumerate(repr(line)):
        position = f"{x*tile_size + map_x_offset}.0 {y*tile_size + map_y_offset}.0\n"
        if char == 'W':
            map_file += "wall " + position
        if char == 'U':
            map_file += "unbreakablewall " + position
        if char == 'I':
            map_file += "water1 " + position
        if char == 'L':
            map_file += "water2 " + position

        #bg_tiles = ["grass-1", "grass-2", "grass-3", "grass-4", "grass-5"]
        #bg_tile = random.choice(bg_tiles)
        #if (x + y) % 2 == 1:
        #    bg_tile += "-b"
        #map_file += f"{bg_tile} {position}"
with open(map_path, 'w') as file:
    file.write(map_file)