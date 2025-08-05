import io

tile_size = 10
map_path = "assets/maps/map_maker.map"
# W = wall
# U = unbreakable_wall
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