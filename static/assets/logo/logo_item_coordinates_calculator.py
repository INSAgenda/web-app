import math

def cos(x):
    return math.cos(math.radians(x))
def sin(x):
    return math.sin(math.radians(x))

angle = int(input("angle: "))   # 65
size = int(input("size: "))     # 75
length = int(input("length: ")) # 170

rh = sin(angle)*length+cos(angle)*size
rhp = sin(angle)*length
rhpp = cos(angle)*size
rw = cos(angle)*length+sin(angle)*size
rwp = cos(angle)*length
rwpp = sin(angle)*size

c1x = 50 + size/2
c1y = 400 + size/2
r1x = c1x - sin(angle)*size/2
r1y = c1y - cos(angle)*size/2 - rhp
c2x = r1x + rwp + sin(angle)*size/2-size/2
c2y = r1y + cos(angle)*size/2-size/2
r2x = c2x + size/2-sin(angle)*size/2
r2y = r1y
c3x = r2x + rwp + sin(angle)*size/2-size/2
c3y = c1y

r3x = (r1x*2 + rwpp - rwpp/3) / 2
r3y = (r1y*2 + rhpp - rhpp/3) / 2
r4x = (r2x*2 + rwpp - rwpp/3) / 2
r4y = r3y
c4x = c2x + size/2 - size/6
c4y = c2y + size/2 - size/6
r5x = (r2x*2 + rwpp - rwpp/6) / 2
r5y = (r2y*2 + rhpp - rhpp/6) / 2

print("C1:", c1x, c1y, "\nR1:", r1x, r1y, "\nC2:", c2x, c2y, "\nR2:", r2x, r2y, "\nC3:", c3x, c3y, "\nR3:", r3x, r3y, "\nR4:", r4x, r4y, "\nC4:", c4x, c4y, "\nR5:", r5x, r5y)