# Classic BART red LED platform sign: font and layout

This covers the red dot-matrix destination signs BART installed around 2000. They hang over the platforms in black housings with a "PLATFORM n" label printed bottom-left. Everything below comes from LED-level measurements on photos. Each claim says whether I saw it or inferred it.

The short version. The small font is 8 LEDs tall and proportional, with a 3-column gap between glyph boxes. Digits sit in fixed 5-column cells. The arriving-train font is 14 LEDs tall with strokes 2 LEDs thick and a 2-column gap. Text spans 174 columns by 45 rows. Line 4 sits on the bottom LED row, so its comma loses the tail and reads as a period.

## Reference photos

Station photos, best first. The working copies are not in this repo; each entry names its source.

1. Wikimedia Commons, "Platform and Arrival Display at 16th St Mission Station, October 2021.jpg", User:Graue, CC BY-SA 4.0, 3840 px thumbnail of the 4032 px original. This is the photo the user wants reproduced, with both screens in it. The LEDs bloom into each other, about 3.3 px per LED, so I recovered single LEDs with the half-max run method described below.
2. Commons, "SSF BART southbound tracks sign.JPG", BrokenSphere, May 2008, CC BY-SA. Same hardware. Individual LEDs are sharp at about 7 px per LED and the unlit LEDs show as dark-red columns. It shows the older text format: "SF AIRPORT / 8 MIN / 4 CAR TRAIN / MILLBRAE/CALTRN / 11 MIN / 8 CAR TRAIN". Most of the small-font shapes come from this photo.
3. bart.gov news photo IMG_2147.JPG from the August 2019 article on the new door-count messages. The bottom sign on the rack is a classic sign at about 11 px per LED, very sharp. It shows "SF  RICHMOND / 8,10 MIN / 10-CAR, 2-DOOR / SF  ANTIOCH / 11,26 MIN / 4:27 PM Train Canceled". I took 6, 7, H, the colon and the comma from it. It also shows lowercase exists.
4. bart.gov photo DSS SIGN_23, May 2021. A second classic sign showing "SF DALY CITY / 3,22 MIN / 10-CAR, 3-DOOR / RICHMOND / 15,45 MIN / 8-CAR. 2-DOOR". Line 4 shows a period here too.
5. Commons, Pi.1415926535, March 2018. This is a counter-example. It is the newer, denser downtown sign with a mixed-case "Platform 2" label and a different bold font. Do not copy glyphs from it.
6. Working images, not in this repo: the 2021 left sign resampled to 10 px per LED with perspective removed, a deskewed crop of the arriving sign, and two comparisons that put the photo above a render of the glyph tables in this file. They line up column for column.

Rate limiting on Commons originals meant I used the 3840 px and 1920 px thumbnails. The 3840 px one is about 95% of the original resolution. Flickr and Reddit searches turned up nothing better than these.

## How I measured

On the sharp photos, SSF 2008 and the 2019 lab photo, I deskewed the crop, fitted an LED lattice by maximising brightness at lattice points, and sampled each LED. I also read the zoomed crops by eye.

The 2021 photo is overexposed, so every lit LED is a white blob about 5.4 px wide at half maximum, against a 3.0 to 3.3 px pitch. For it I measured the half-max edges of every lit run along each LED row and converted run lengths to LED counts with `n = (run - w)/pitch + 1`. A 2-column gap shows clearly. A 1-column gap is almost invisible, which is why the big font looks like touching letters. To set the pitch and column origin I used text whose column layout the SSF photo had already fixed, "SF AIRPORT" being 75 columns. Pixel to column was fitted with a quadratic to absorb perspective. Line 2 residuals were under 0.4 column everywhere.

As a cross-check I rendered the glyph tables at the measured positions and compared them with the photo. The comparison images are described in item 6 above.

## LED matrix size

What I measured, in LED units, with text column 0 as the first column of left-aligned text and row 0 as the top row of line 1:

- Text spans columns 0 to 173, so 174 columns. "SF AIRPORT" starts at column 0 and the N of "MIN" ends at column 173. Measured on both minute lines of the 2021 photo. The SSF photo gives about 174 too.
- Text spans rows 0 to 44, so 45 rows.
- Horizontal and vertical pitch match within about 3% in every photo, so the LEDs sit on a square grid.

Margins outside the text are less certain. In SSF 2008 one unlit LED column is visible left of the S, and the unlit area above line 1 is about 2 to 3 rows. In the lab photo the window edge sits about 2.4 pitches outside the text on the left, right and top. At the bottom the line 4 glyphs touch the bezel lip. The clipped comma, covered further down, says there is no LED row below row 44.

My best guess for the physical matrix is 176 x 48: 11 x 3 modules of 16 x 16, with text origin at matrix column 1, row 3. Treat that as a guess. If you only draw the text block, use 174 x 45, which I did measure. Centring in arriving mode points to the same centre either way, column 86.5 in text coordinates. The margins look symmetric left and right.

On a 480 x 320 TFT, 174 columns gives a 2.76 px pitch and 176 columns gives 2.73 px. Both are non-integer, so pick a layout. One choice is a 2.7 px pitch with round dots about 2 px across.

## Small font, 4-line mode

- Cap height is 8 LEDs, not 7. Every capital, digit and slash is 8 rows tall.
- The font is proportional. Most letters are 5 wide. A, M, V and W are 7. O and Q are 6. I is a single column centred in a 3-column box. Slash is 4.
- Every glyph box is followed by 3 blank columns. That is wide, and it is why the sign looks airy.
- A space is 2 columns. Between words that gives 3 + 2 + 3 = 8 blank columns. I measured this on "SF AIRPORT", on "13 MIN" and on "11 MIN". The lab sign's "SF  RICHMOND" has 13 blank columns, which matches two spaces exactly.
- Digits are tabular. Every digit has a 5-column box and the "1" sits in it with its stem at index 2, so "11" has a wide gap. The 1 has a one-LED flag at row 1 and no base serif.
- The zero is a plain oval with no slash or dot. It is identical to the letter O except that it is 5 wide instead of 6.
- The comma uses a 5-column box with ink in box columns 1 and 2. That makes the space after "CAR," and inside "1,13" wider than it looks. Ink rows 6 and 7 are a 2x2 block, row 8 is the right dot, row 9 is the left dot, so it descends 2 rows below the baseline. The lab photo shows this sharply and the 2021 photo agrees.
- The hyphen is 5 LEDs on row 3, in a 5-column box.
- The colon is 2x2 dots on rows 1 to 2 and rows 5 to 6, in a 2-column box.
- Some shapes to get right: A has a single-LED apex 2 rows tall and a crossbar on row 5. M has a 2-row notch. N steps its diagonal 2 rows per column. R and P close their bowls on row 4, while B, E, F and H have the middle bar on row 3. The 7 has a dot below its top-left corner, so it reads "#####" then "#...#". The 4 is open with its stem at column 3. The 2 is curved at the top and the 3 is flat-topped.

### Vertical layout

Line tops are at rows 0, 11, 26 and 37, so lines occupy rows 0-7, 11-18, 26-33 and 37-44. The gaps are 3, 7 and 3 rows. The 2008 SSF sign uses the same spacing, and the 2021 photo matches it to within 0.1 row: line pitches of 35.7, 49.2 and 35.7 px give an 11:15:11 ratio. The minutes share rows with lines 1 and 3. Lines 2 and 4 carry only the car text.

### Horizontal layout

- The destination is left-aligned at column 0.
- The minutes are right-aligned so the N of "MIN" ends at column 173. "1,13 MIN" fills columns 116 to 173 and "5,11 MIN" does the same. "15,45 MIN" would start at column 108.
- The car line is left-aligned at column 0 and runs to column 109 for "10-CAR, 2-DOOR".

### Line 4 "CAR." is a clipped comma, not a typo

Both the 16th St 2021 sign ("10-CAR. 3-DOOR") and the Lake Merritt 2021 sign ("8-CAR. 2-DOOR") show a period on line 4 and a normal comma on line 2. I measured the line 4 mark. It is the 2x2 block on rows 6 and 7 of the comma, at exactly the comma's columns 51 and 52, and nothing appears below it. The comma's tail would fall on rows 45 and 46, past the bottom of the LED field. Line 4 sits on the bottom LED row; the lab photo shows line 4 touching the bezel lip and the SSF window edge in the same place. On the arriving screen the car line sits 2 rows higher and its comma tail is visible, which fits.

So draw "10-CAR, 3-DOOR" on line 4 and clip anything past row 44. Hard-coding a period gives the same result. Clipping also handles any other descender correctly.

## Arriving-train mode

The screen shows the destination in the big font with the small-font car line below it. Both are centred horizontally. Coordinates below use the 4-line text grid. I tied the two signs together with the printed PLATFORM label, which sits in the same place on every housing.

- The big line occupies rows 7 to 20. Top-row centre to car-line top-row centre is 86.8 px at 3.08 px per row, which is 28.2 rows.
- The car line occupies rows 35 to 42, 2 rows above line 4's position. Its comma tail lands on rows 43 and 44 and is visible in the photo.
- "BERRYESSA" covers columns 34 to 140, which is 107 columns. "10-CAR, 2-DOOR" covers columns 31 to 140, which is 110 columns. Both right edges land on column 140. Centring with floor((174 - width)/2) gives 33 and 32, so the photo is within 1 column of plain centring. I would use plain centring. If you want an exact match to this one photo, use 34 and 31.
- I have no photo of a big destination that needs a "/". "SFO/MILLBRAE" in the big font comes to 135 columns and still fits. I don't know what the sign does with longer names.

## Big font, arriving mode

- It is 14 LEDs tall. The E bars measured 8.4 px at half max, with centres 18.25 and 18.7 px apart. That fits rows 0-1, 6-7 and 12-13 of a 14-row glyph at 3.08 px per row. A 15-row glyph does not fit the bar spacing.
- Every stroke is 2 LEDs thick.
- Letters are 10 columns wide and A is 11. BERRYESSA comes to 9 glyphs and 107 columns, which matches the measured 106.9.
- The gap between letters is 2 columns.
- Round letters cut their corners. B starts with rows of 8 and 9 LEDs, O and S with 6 and 8, and a 3-wide step sits at each bowl join.
- A has a 3-wide apex 3 rows tall and a crossbar on rows 8 and 9. R's bowl closes lower than B's top bowl, on rows 7 and 8, and its leg steps out over rows 9 to 11. The S terminals reach down to row 4 and up to row 9.
- I observed B, E, R, Y, S and A. Everything else in the big table is my inference in the same style. The big space width is a guess of 4.

## Text conventions seen on classic signs

- Destinations are in caps. I saw "SF AIRPORT", "DALY CITY", "SF DALY CITY" at Lake Merritt in 2021, "RICHMOND", "BERRYESSA", "SF RICHMOND", "SF ANTIOCH" and "MILLBRAE/CALTRN" in 2008. The newer non-classic sign abbreviates harder, with "DUBLN PLSTN" and "P. HILL LTD".
- The minutes are digits joined by commas with no space, then a space and "MIN": "1,13 MIN", "3,22 MIN", "15,45 MIN". In 2008 one value was shown, as in "8 MIN".
- Since the 2019 update the car line reads "10-CAR, 2-DOOR" and "8-CAR, 2-DOOR", with a hyphen, a comma and a space. Before that it read "4 CAR TRAIN".
- When a train arrives, the destination is shown big and centred with the car line under it.
- A cancelled train puts a mixed-case message on its car line, "4:27 PM Train Canceled", so lowercase glyphs exist. I did not tabulate them.
- I found no photo of "LEAVING", "ARRIVING", "NOW", a clock, an idle "BART" screen or a scrolling advisory on a classic sign. BART's 2021 release says the old signs rotated scrolling messages, but I have no picture of one. Treat these states as unknown.
- Fit to width: "SFO/MILLBRAE" is 95 columns. Next to "15,45 MIN", which starts at column 108, a destination should end by about column 99, leaving 8 blank columns. "PITTSBURG/BAY POINT" is 144 columns and "DUBLIN/PLEASANTON" is 135, so on a line with minutes the real sign must abbreviate them. I don't know its exact abbreviations for Lafayette's destinations.

## Color

- In the 2021 iPhone photo the lit LEDs are saturated white cores, average 252,243,244, with deep red halos, median 218,3,36. Unlit areas near text read about 110,9,17 from the glow, and 40,20,16 away from text. In SSF 2008 the lit LEDs read pink, 240,45,101, from camera white balance.
- Red AlInGaP LEDs at 625 to 630 nm look pure red to orange-red to the eye. For the TFT I'd use a lit dot of about 255,40,24, which is #FF2818, with an optional 1 px glow of about 120,10,8 if you have the pixels.
- Unlit LEDs show faintly as dark red dots behind the red filter window in SSF 2008. Draw them at about 45,10,10 on a background of about 12,4,4.

## PLATFORM label on the bezel

- The text is "PLATFORM 1" or "PLATFORM 2" in a bold grotesque, Helvetica Bold or Arial Bold, all caps. It is printed on the black bezel below the LED field, not drawn with LEDs.
- Its left edge lines up with text column 0, measured at -0.04 columns.
- The cap height is about 5.9 LED pitches. "PLATFORM 1" is about 49.5 pitches wide and "PLATFORM 2" about 50.5.
- In text-row units its top edge is at about row 45.6 and its bottom at about row 51.5. That puts it about 1.6 pitches below the centre of line 4's bottom LED row.
- The color is light to medium neutral gray. The 2021 photo shows it as 152,121,111 away from the LEDs and 171,87,87 under the LED glow, warmed by the red light. In SSF 2008 it photographed white under flash. I'd use about 150,145,140 on a bezel of about 20,16,16.

## Glyph tables

Rust-ready. `#` is a lit LED and `.` is dark. Each line is one LED row. The width in the box is what the sign advances, and the inter-glyph gap goes after it: 3 columns for small glyphs, 2 for big ones. A label of OBSERVED means I read the LED pattern from a photo. Observed from a bloomed photo only means I recovered it from the overexposed 2021 picture and the shape is right but single LEDs at diagonal steps are less certain. INFERRED means I made it up in the same style.

Sources: SSF08 is the SSF 2008 photo, 16th21 the 16th St 2021 photo, lab19 the bart.gov 2019 lab photo.

### Small font: 10 rows, 8 cap rows plus 2 descender rows, baseline below row 7

Rows 8 and 9 are empty for everything except the comma. On line 4 the last LED row is row 7 of the glyph, so rows 8 and 9 clip.

#### `A`  width 7  OBSERVED (SSF08 (LEDs resolved), 16th21, lab19)
```
...#...
...#...
..#.#..
..#.#..
.#...#.
.#####.
#.....#
#.....#
.......
.......
```
#### `B`  width 5  OBSERVED (SSF08)
```
####.
#...#
#...#
####.
#...#
#...#
#...#
####.
.....
.....
```
#### `C`  width 5  OBSERVED (SSF08, 16th21, lab19)
```
.###.
#...#
#....
#....
#....
#....
#...#
.###.
.....
.....
```
#### `D`  width 5  OBSERVED (bloomed photo only) (16th21 (bloomed), lab19 (DOOR, RICHMOND))
```
####.
#...#
#...#
#...#
#...#
#...#
#...#
####.
.....
.....
```
#### `E`  width 5  OBSERVED (SSF08)
```
#####
#....
#....
####.
#....
#....
#....
#####
.....
.....
```
#### `F`  width 5  OBSERVED (SSF08, 16th21, lab19)
```
#####
#....
#....
####.
#....
#....
#....
#....
.....
.....
```
#### `G`  width 5  INFERRED
```
.###.
#...#
#....
#....
#.###
#...#
#...#
.###.
.....
.....
```
#### `H`  width 5  OBSERVED (lab19 (RICHMOND, ANTIOCH))
```
#...#
#...#
#...#
#####
#...#
#...#
#...#
#...#
.....
.....
```
#### `I`  width 3  OBSERVED (SSF08, 16th21, lab19)
```
.#.
.#.
.#.
.#.
.#.
.#.
.#.
.#.
...
...
```
#### `J`  width 5  INFERRED
```
....#
....#
....#
....#
....#
....#
#...#
.###.
.....
.....
```
#### `K`  width 5  INFERRED
```
#...#
#..#.
#.#..
##...
##...
#.#..
#..#.
#...#
.....
.....
```
#### `L`  width 5  OBSERVED (SSF08, 16th21)
```
#....
#....
#....
#....
#....
#....
#....
#####
.....
.....
```
#### `M`  width 7  OBSERVED (SSF08, 16th21, lab19)
```
#.....#
##...##
##...##
#.#.#.#
#.#.#.#
#..#..#
#..#..#
#.....#
.......
.......
```
#### `N`  width 5  OBSERVED (SSF08, 16th21, lab19)
```
#...#
##..#
##..#
#.#.#
#.#.#
#..##
#..##
#...#
.....
.....
```
#### `O`  width 6  OBSERVED (SSF08, 16th21, lab19)
```
.####.
#....#
#....#
#....#
#....#
#....#
#....#
.####.
......
......
```
#### `P`  width 5  OBSERVED (SSF08, lab19)
```
####.
#...#
#...#
#...#
####.
#....
#....
#....
.....
.....
```
#### `Q`  width 6  INFERRED
```
.####.
#....#
#....#
#....#
#....#
#..#.#
#...#.
.###.#
......
......
```
#### `R`  width 5  OBSERVED (SSF08, 16th21, lab19)
```
####.
#...#
#...#
#...#
####.
#..#.
#...#
#...#
.....
.....
```
#### `S`  width 5  OBSERVED (SSF08, 16th21)
```
.###.
#...#
#....
.###.
....#
....#
#...#
.###.
.....
.....
```
#### `T`  width 5  OBSERVED (SSF08, 16th21)
```
#####
..#..
..#..
..#..
..#..
..#..
..#..
..#..
.....
.....
```
#### `U`  width 5  INFERRED
```
#...#
#...#
#...#
#...#
#...#
#...#
#...#
.###.
.....
.....
```
#### `V`  width 7  INFERRED
```
#.....#
#.....#
.#...#.
.#...#.
..#.#..
..#.#..
...#...
...#...
.......
.......
```
#### `W`  width 7  INFERRED
```
#.....#
#..#..#
#..#..#
#.#.#.#
#.#.#.#
##...##
##...##
#.....#
.......
.......
```
#### `X`  width 5  INFERRED
```
#...#
#...#
.#.#.
..#..
..#..
.#.#.
#...#
#...#
.....
.....
```
#### `Y`  width 5  OBSERVED (bloomed photo only) (16th21 (bloomed; rows 2-3 slightly ambiguous))
```
#...#
#...#
.#.#.
.#.#.
..#..
..#..
..#..
..#..
.....
.....
```
#### `Z`  width 5  INFERRED
```
#####
....#
...#.
..#..
..#..
.#...
#....
#####
.....
.....
```
#### `0`  width 5  OBSERVED (16th21, lab19)
```
.###.
#...#
#...#
#...#
#...#
#...#
#...#
.###.
.....
.....
```
#### `1`  width 5  OBSERVED (SSF08, 16th21, lab19)
```
..#..
.##..
..#..
..#..
..#..
..#..
..#..
..#..
.....
.....
```
#### `2`  width 5  OBSERVED (16th21, lab19)
```
.###.
#...#
....#
...#.
..#..
.#...
#....
#####
.....
.....
```
#### `3`  width 5  OBSERVED (bloomed photo only) (16th21 (bloomed; rows 1-3 diagonal uncertain))
```
#####
...#.
..#..
...#.
....#
....#
#...#
.###.
.....
.....
```
#### `4`  width 5  OBSERVED (SSF08, lab19)
```
...#.
..##.
.#.#.
#..#.
#####
...#.
...#.
...#.
.....
.....
```
#### `5`  width 5  OBSERVED (bloomed photo only) (16th21 (bloomed))
```
#####
#....
#....
####.
....#
....#
#...#
.###.
.....
.....
```
#### `6`  width 5  OBSERVED (lab19)
```
.###.
#...#
#....
####.
#...#
#...#
#...#
.###.
.....
.....
```
#### `7`  width 5  OBSERVED (lab19)
```
#####
#...#
....#
...#.
...#.
..#..
..#..
..#..
.....
.....
```
#### `8`  width 5  OBSERVED (SSF08)
```
.###.
#...#
#...#
.###.
#...#
#...#
#...#
.###.
.....
.....
```
#### `9`  width 5  INFERRED
```
.###.
#...#
#...#
#...#
.####
....#
#...#
.###.
.....
.....
```
#### `space`  width 2  OBSERVED (SSF08, 16th21, lab19 (width from spacing))
```
..
..
..
..
..
..
..
..
..
..
```
#### `,`  width 5  OBSERVED (lab19 (sharp), 16th21)
```
.....
.....
.....
.....
.....
.....
.##..
.##..
..#..
.#...
```
#### `-`  width 5  OBSERVED (16th21, lab19)
```
.....
.....
.....
#####
.....
.....
.....
.....
.....
.....
```
#### `/`  width 4  OBSERVED (SSF08)
```
...#
...#
..#.
..#.
.#..
.#..
#...
#...
....
....
```
#### `.`  width 5  OBSERVED (bloomed photo only) (16th21 + Lake Merritt 21 line 4 (= comma with rows 8-9 clipped))
```
.....
.....
.....
.....
.....
.....
.##..
.##..
.....
.....
```
#### `:`  width 2  OBSERVED (lab19)
```
..
##
##
..
..
##
##
..
..
..
```
#### `'`  width 2  INFERRED
```
.#
.#
#.
..
..
..
..
..
..
..
```
#### `&`  width 5  INFERRED
```
.##..
#..#.
#..#.
.##..
#.#.#
#..#.
#..#.
.##.#
.....
.....
```

### Big font: 14 rows, strokes 2 LEDs thick, gap 2

I observed B, E, R, Y, S and A in "BERRYESSA". Everything else is inferred.

#### `A`  width 11  OBSERVED (bloomed photo only) (16th21 BERRYESSA (bloomed, resolved via half-max runs))
```
....###....
....###....
....###....
...#####...
...#####...
...##.##...
..##...##..
..##...##..
..#######..
.#########.
.##.....##.
.##.....##.
##.......##
##.......##
```
#### `B`  width 10  OBSERVED (bloomed photo only) (16th21)
```
########..
#########.
##.....###
##......##
##......##
##.....###
#########.
#########.
##.....###
##......##
##......##
##.....###
#########.
########..
```
#### `C`  width 10  INFERRED
```
..######..
.########.
###....###
##......##
##........
##........
##........
##........
##........
##........
##......##
###....###
.########.
..######..
```
#### `D`  width 10  INFERRED
```
########..
#########.
##.....###
##......##
##......##
##......##
##......##
##......##
##......##
##......##
##......##
##.....###
#########.
########..
```
#### `E`  width 10  OBSERVED (bloomed photo only) (16th21)
```
##########
##########
##........
##........
##........
##........
########..
########..
##........
##........
##........
##........
##########
##########
```
#### `F`  width 10  INFERRED
```
##########
##########
##........
##........
##........
##........
########..
########..
##........
##........
##........
##........
##........
##........
```
#### `G`  width 10  INFERRED
```
..######..
.########.
###....###
##......##
##........
##........
##........
##...#####
##...#####
##......##
##......##
###....###
.########.
..######..
```
#### `H`  width 10  INFERRED
```
##......##
##......##
##......##
##......##
##......##
##......##
##########
##########
##......##
##......##
##......##
##......##
##......##
##......##
```
#### `I`  width 2  INFERRED
```
##
##
##
##
##
##
##
##
##
##
##
##
##
##
```
#### `J`  width 10  INFERRED
```
........##
........##
........##
........##
........##
........##
........##
........##
........##
........##
##......##
###....###
.########.
..######..
```
#### `K`  width 10  INFERRED
```
##......##
##.....##.
##....##..
##...##...
##..##....
##.##.....
####......
####......
##.##.....
##..##....
##...##...
##....##..
##.....##.
##......##
```
#### `L`  width 10  INFERRED
```
##........
##........
##........
##........
##........
##........
##........
##........
##........
##........
##........
##........
##########
##########
```
#### `M`  width 12  INFERRED
```
###......###
####....####
##.##..##.##
##..####..##
##...##...##
##........##
##........##
##........##
##........##
##........##
##........##
##........##
##........##
##........##
```
#### `N`  width 10  INFERRED
```
###.....##
####....##
####....##
##.##...##
##.##...##
##..##..##
##..##..##
##..##..##
##...##.##
##...##.##
##....####
##....####
##.....###
##......##
```
#### `O`  width 10  INFERRED
```
..######..
.########.
###....###
##......##
##......##
##......##
##......##
##......##
##......##
##......##
##......##
###....###
.########.
..######..
```
#### `P`  width 10  INFERRED
```
########..
#########.
##.....###
##......##
##......##
##......##
##.....###
#########.
########..
##........
##........
##........
##........
##........
```
#### `Q`  width 10  INFERRED
```
..######..
.########.
###....###
##......##
##......##
##......##
##......##
##......##
##......##
##...##.##
##....####
###...####
.#########
..#####.##
```
#### `R`  width 10  OBSERVED (bloomed photo only) (16th21)
```
########..
#########.
##.....###
##......##
##......##
##......##
##.....###
#########.
#########.
##....###.
##.....###
##......##
##......##
##......##
```
#### `S`  width 10  OBSERVED (bloomed photo only) (16th21)
```
..######..
.########.
###....###
##......##
##......##
####......
..####....
....#####.
......####
##......##
##......##
###....###
.########.
..######..
```
#### `T`  width 10  INFERRED
```
##########
##########
....##....
....##....
....##....
....##....
....##....
....##....
....##....
....##....
....##....
....##....
....##....
....##....
```
#### `U`  width 10  INFERRED
```
##......##
##......##
##......##
##......##
##......##
##......##
##......##
##......##
##......##
##......##
##......##
###....###
.########.
..######..
```
#### `V`  width 10  INFERRED
```
##......##
##......##
##......##
##......##
.##....##.
.##....##.
..##..##..
..##..##..
..##..##..
...####...
...####...
...####...
....##....
....##....
```
#### `W`  width 12  INFERRED
```
##........##
##........##
##........##
##........##
##........##
##........##
##........##
##........##
##...##...##
##..####..##
##.##..##.##
####....####
###......###
##........##
```
#### `X`  width 10  INFERRED
```
##......##
##......##
.##....##.
.##....##.
..##..##..
...####...
....##....
....##....
...####...
..##..##..
.##....##.
.##....##.
##......##
##......##
```
#### `Y`  width 10  OBSERVED (bloomed photo only) (16th21)
```
##......##
##......##
.##....##.
.##....##.
..##..##..
..######..
...####...
...####...
....##....
....##....
....##....
....##....
....##....
....##....
```
#### `Z`  width 10  INFERRED
```
##########
##########
........##
.......##.
......##..
.....##...
....##....
...##.....
..##......
.##.......
##........
##........
##########
##########
```
#### `0`  width 10  INFERRED
```
..######..
.########.
###....###
##......##
##......##
##......##
##......##
##......##
##......##
##......##
##......##
###....###
.########.
..######..
```
#### `1`  width 6  INFERRED
```
..##..
.###..
####..
..##..
..##..
..##..
..##..
..##..
..##..
..##..
..##..
..##..
..##..
..##..
```
#### `2`  width 10  INFERRED
```
..######..
.########.
###....###
##......##
........##
.......###
......###.
.....###..
....###...
...###....
..###.....
.###......
##########
##########
```
#### `3`  width 10  INFERRED
```
##########
##########
......###.
.....###..
....###...
...######.
...#######
........##
........##
........##
##......##
###....###
.########.
..######..
```
#### `4`  width 10  INFERRED
```
.....####.
....#####.
...###.##.
..###..##.
.###...##.
###....##.
##.....##.
##########
##########
.......##.
.......##.
.......##.
.......##.
.......##.
```
#### `5`  width 10  INFERRED
```
##########
##########
##........
##........
##........
########..
#########.
.......###
........##
........##
##......##
###....###
.########.
..######..
```
#### `6`  width 10  INFERRED
```
..######..
.########.
###....###
##........
##........
########..
#########.
###....###
##......##
##......##
##......##
###....###
.########.
..######..
```
#### `7`  width 10  INFERRED
```
##########
##########
........##
.......###
......###.
.....###..
....###...
....##....
....##....
....##....
....##....
....##....
....##....
....##....
```
#### `8`  width 10  INFERRED
```
..######..
.########.
###....###
##......##
###....###
.########.
.########.
###....###
##......##
##......##
##......##
###....###
.########.
..######..
```
#### `9`  width 10  INFERRED
```
..######..
.########.
###....###
##......##
##......##
###....###
.#########
..########
........##
........##
##......##
###....###
.########.
..######..
```
#### `/`  width 8  INFERRED
```
......##
......##
.....##.
.....##.
....##..
....##..
...##...
...##...
..##....
..##....
.##.....
.##.....
##......
##......
```
#### `space`  width 4  INFERRED
```
....
....
....
....
....
....
....
....
....
....
....
....
....
....
```

## What I am unsure of

- The physical matrix size. 174 x 45 is the text extent I measured. 176 x 48 is a guess at the panel.
- The arriving screen's left edge, plus or minus 1 column: 34 and 31 measured against 33 and 32 from plain centring.
- Small Y rows 2 and 3, small 3 rows 1 to 3, and the bowl joins of big B and R. All of these come from bloomed pixels.
- Every glyph marked INFERRED, especially the big font digits and the small G, J, K, Q, U, V, W, X, Z, 9, apostrophe and ampersand.
- States I have no photo of: leaving or arriving text, a clock, an idle screen, advisories.
