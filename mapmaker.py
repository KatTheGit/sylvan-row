import io

tile_size = 10
map_path = "assets/maps/map_maker.map"
# W = wall
# U = unbreakable_wall
map_drawing = """
UUUUUUUUUUUUUUUUUUUUUUUUUUU
U.........................U
U.........................U
U.............WWW.........U
U..............WW.........U
U.....W...................U
U.....W...................U
U.....W..............WW...U
U.....W...WWWWWWW...WW....U
U.........W...............U
U.........................U
U................W........U
U.....WW...WWWWWWW...W....U
U....WW..............W....U
U....................W....U
U....................W....U
U..........WW.............U
U..........WWW............U
U.........................U
U.........................U
UUUUUUUUUUUUUUUUUUUUUUUUUUU
"""

map_file = ""
for y, line in enumerate(io.StringIO(map_drawing)):
    for x, char in enumerate(repr(line)):
        position = f"{x*tile_size-20}.0 {y*tile_size-20}.0\n"
        if char == 'W':
            map_file += "wall " + position
        if char == 'U':
            map_file += "unbreakablewall " + position
with open(map_path, 'w') as file:
    file.write(map_file)