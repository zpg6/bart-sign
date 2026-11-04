# Classic BART red LED platform sign: font and layout (study B)

Independent measurement from photos. `#` = lit LED, `.` = dark. Rows top to bottom.
Column and row numbers are LED indices in the sign matrix, 0-based, origin top-left.

## 0. Which sign this is

Two different red LED signs show up in BART photos. Only the first one is in scope.

- **Classic sign (in scope).** Bezel label `PLATFORM 1` in white/gray sans-serif caps under the
  bottom-left corner of the LED field. Coarse pitch, 1-dot-wide strokes in the small font. Shown
  in the user's 16th St crops, San Bruno, SFO station, Ashby, North Berkeley, 24th St.
- **Newer full-matrix sign (out of scope).** Mixed-case `Platform 2` label on a tab, much finer
  pitch, smooth proportional font (Civic Center 2016 "new destination signs", Montgomery 2018
  `DUBLN PLSTN / 0 CAR TRAIN`, Pleasant Hill, Berryessa `SIGN DISCONNECTED`). Its text conventions
  are listed in section 6 only as history.

## 1. Sources I used

LED-resolvable (these carry most of the weight):

| Station photo (see source) | What it gives |
|---|---|
| Flickr 21588499273 "SFO BART Destination Sign" (2048 px original). The sign is showing random noise, so every LED position is visible | Matrix size by lattice fit: **176 columns x 48 rows** |
| Flickr 5426658254 "Destination display" (3648 px original, 2011). `LOOK FOR DESTINATION DISPLAY IN CAB...` with a dotted marquee border | Border sits on rows 0-1 and 46-47: **48 rows** confirmed; border blocks every 8 columns |
| Wikimedia Commons "New train at SFIA station, September 2019.JPG" (5184 px). Arriving screen `SAN FRANCISCO / ANTIOCH / 10-CAR, 3-DOOR` | Small and large font at LED level: pitch 5.45 px, stroke widths, glyph widths, spacing, line positions |
| Wikimedia Commons "San Bruno station 02.jpg" (5472 px). 4-line screen `MILL/CALTRN 4,19 MIN / 8-CAR, 2-DOOR / SF AIRPORT 12,39 MIN / 8-CAR. 2-DOOR` | Small-font row count (8), dot structure of M I N L T R / 1 2 3 4 9, comma vs clipped comma |

Shape-level only (LEDs not resolvable, used for proportions and positions):

- Two crops of the 16th St signs (the 4-line screen and the BERRYESSA arriving screen), same scene as
  Commons "Platform and Arrival Display at 16th St Mission Station, October 2021.jpg" (4032 px).
- Commons "BART train at Embarcadero station, September 2016.jpg" (`PITTSBURG / BAY POINT / 10 CAR TRAIN`),
  "Richmond-bound train at Downtown Berkeley station, June 2019" (`RICHMOND / 6 CARS - BOARD CENTER`),
  "Ashby station platform from fare mezzanine, March 2018" (`BART POLICE TIP:` scrolling, clipped at field bottom),
  "North Berkeley BART station 2016 1.jpg" (`BART TRAINS. THANK YOU FOR YOUR COOPERATION.`).

Method: crop, upscale (NEAREST and LANCZOS), use the blue channel (red channel is saturated),
fit the LED pitch from brightness modulation inside strokes, sample each glyph on a fitted lattice,
and cross-check widths and positions against the 16th St crops with a bloom correction.

## 2. Matrix

- **176 x 48 LEDs**, square pitch. 176 came from a lattice fit of all lit dots in the noise photo;
  edge columns are as fully populated as interior ones, so the edges are real (confidence: high, +/-1).
  48 rows agrees in three places: noise photo lattice, marquee border rows 0-1 / 46-47, and the
  4-line layout below that fills rows 0-47 exactly (high).
- 176 = 22 x 8 and the marquee border repeats every 8 columns, so the panel is probably built from
  8-column modules (medium).
- The bezel/housing extends well above the field. Black space above line 1 in the photos is housing, not unlit LEDs.

## 3. Small font

- **Cap height 8 rows.** No lowercase seen. Only the comma descends (2 rows).
- **Strokes are 1 dot wide**, both vertical and horizontal. Measured FWHM: stems 7-8 px, bars 6-8 px, at 5.45 px pitch.
  The photos look bold because of bloom; the dots are single.
- **Proportional widths** (columns): most letters and digits 5; `O` 6; `A` 7; `M` 7; `W` 7 (inferred);
  `I` is a 3-column cell with the stem in the middle; `1` is a 5-column cell (flag col 1, stem col 2);
  comma is a 5-column cell with its dots in cols 1-2; space 2.
- **Gap between characters: 3 blank columns.** So advance = width + 3 and a word space shows as
  3+2+3 = 8 blank columns. This was checked against every character position in `SF AIRPORT`,
  `DALY CITY`, `10-CAR, 2-DOOR` and `1,13 MIN` in the 16th St crop, and against `10-CAR, 3-DOOR` at LED level.
  `0` (5 wide) and `O` (6 wide) are different glyphs.
- Width check: `10-CAR, 2-DOOR` = 110 columns, `SF AIRPORT` = 75, `1,13 MIN` = 58.
- Hyphen is 5 wide on row 3. Comma = 2x2 block on rows 6-7 (cols 1-2), then one dot at row 8 col 2
  and one at row 9 col 1 (LED-level, SFO station photo).

## 4. Large font

- **Cap height 14 rows. Strokes 2 dots wide and bars 2 rows thick.** Corners are chamfered 1-2 dots (C, O, S, R).
  Sampled at thresholds 0.7/0.85/0.93 on the SFO station photo; at 0.85 the F, T and H bars are 2 rows and all stems 2 columns.
- Widths: 10 columns for most letters, `A` 11, `I` 2, `M` and `W` 12 (inferred). **Gap 2 columns.** Space 8.
- Checks: `SAN FRANCISCO` predicts 146 columns and measures 145.9. `ANTIOCH` predicts 75 and measures 74.9.
  In the 16th St crop `BERRYESSA` predicts 107; the advance between letters is 12 columns (71-72 px at 5.96 px/col).
- No large digits or punctuation were seen. The ones below are designed in the same style.

## 5. Screen layouts

### 5a. Four-line list (`SF AIRPORT 1,13 MIN`)

| Line | Rows | Content | Alignment |
|---|---|---|---|
| 1 | 0-7 | destination | left, first column 1 |
| 1 (right) | 0-7 | minutes, e.g. `1,13 MIN` | right, last lit column 174 |
| 2 | 12-19 (comma tail 20-21) | `10-CAR, 2-DOOR` | left, cell starts col 1 (the `1` flag lands on col 2) |
| 3 | 28-35 | destination + minutes | as line 1 |
| 4 | 40-47 | car line | as line 2 |

- Line pitch 12 rows inside a train block, 16 rows between blocks. Measured in the 16th St crop:
  12.5 / 15.7 / 12.3 rows with py = 6.04 px. In the San Bruno photo the line-1 to line-4 distance is 40 rows at the measured 10.4 px/row (high).
- **Minutes sit on the same rows as the destination.** In both photos they look lower, but that is
  image tilt (0.017 px/px). After tilt correction the offset is 0 to 0.8 row (medium-high).
- Horizontal margins: lit span `S`...`N` measures about 174 columns, so the margins are about
  1 column a side (medium, +/-2). I placed text at col 1 and right-aligned to col 174.

### 5b. Arriving, one big line (`BERRYESSA`)

| Element | Rows | Columns |
|---|---|---|
| big destination | 10-23 | centered: start = (176 - w) // 2. BERRYESSA w=107 -> cols 34-140 |
| car line | 38-45 (comma tail 46-47) | centered: `10-CAR, 2-DOOR` w=110 -> cols 33-142 |

### 5c. Arriving, two big lines (`SAN FRANCISCO / ANTIOCH`, SFO station 2019)

| Element | Rows |
|---|---|
| big line 1 | 2-15 |
| big line 2 | 19-32 |
| car line | 38-45 (tail 46-47) |

- Measured at LED level: big1 top to big2 top = 17.0 rows, big2 top to car top = 19.0 rows (high).
  The absolute offset (2) assumes the comma tail ends on row 47; it could be 1 (medium).
- The single big line is centered on the two-line slot: rows 10-23 vs. 2-32. From the 16th St crop, the
  field top edge is visible as a faint step at y = 97 and the field is 295 px tall (6.15 px/row):
  big top = row 10.2, car baseline = row 46.1, big-top-to-car-top = 28.1 rows (medium, +/-1).
- Each line is centered on its own. `ANTIOCH` measured about 1.5 columns right of exact center; in
  the BERRYESSA crop the big and small lines end on the same pixel column. Expect +/-1-2 columns of firmware rounding.

### 5d. Line 4 `CAR.` instead of `CAR,` is a real sign quirk

Line 4 sits on rows 40-47 and its baseline is the last row of the matrix, so the comma's two
descender rows (48-49) do not exist and it is clipped to its 2x2 block, which reads as a period.
Evidence: line 2 in the same photo shows the full comma with its tail. Both 16th St and San Bruno
show the same `CAR,` / `CAR.` pattern on two different signs on different dates. In the San Bruno
blue-channel zoom, line 4 is exactly the top 2x2 of the line-2 comma, and the 16th St photo matches. Confidence: high. To reproduce it, draw line 4's comma
and let the bottom edge clip it; do not substitute a period.

### 5e. Bezel label

- Text `PLATFORM n`, caps, bold grotesque sans (looks like Helvetica Bold / Arial Bold), printed on
  the black bezel. It is not LEDs.
- Position: top of the caps about 1-1.5 LED rows below the bottom of the field (u2: 9 px under the
  line-4 baseline center at 6.04 px/row). Left edge about 1.5 columns right of the first lit column.
  Cap height about 5.5 LED pitches (34 px in u2). Width of `PLATFORM 1` about 48 LED columns.
- Color: neutral light gray/white paint. It photographs as warm gray under the red glow: u3 median
  RGB (138,112,91), 90th percentile (161,126,115); daylight photos show near-white. For a screen
  mock that matches the photos use roughly #9C9C9C-#B4B4B4 (my suggestion, not measured paint).

## 6. Text conventions

- **Destinations as displayed** (classic sign, all caps): `SF AIRPORT`, `DALY CITY`, `MILL/CALTRN`
  (Millbrae/Caltrain, 11 chars, slash used), `BERRYESSA`, `RICHMOND`, `PITTSBURG / BAY POINT` (2 big lines),
  `SAN FRANCISCO / ANTIOCH` (2 big lines, arriving screen at SFO). Long names abbreviate on the
  4-line screen so the destination fits left of the minutes (about 110 columns, roughly 12-14 chars).
  On the arriving screen they split over two big lines.
  Abbreviations seen on the newer sign (same data feed, may differ): `DUBLN PLSTN`, `PTS/BAY PT`, `P. HILL LTD`, `SF/24TH STREET`.
- **Minutes**: up to two values, comma, no space, then space + `MIN`: `1,13 MIN`, `5,11 MIN`, `4,19 MIN`,
  `12,39 MIN`. Right-aligned to the right margin on the destination's row. Inside the minutes string the
  digits look tabular (the `1` keeps its 5-column cell), which is why `1,13` looks loosely spaced.
- **Train at the platform**: the sign switches to the arriving screen (big destination centered and the car
  line under it, no minutes). I did not find a classic-sign photo with `LEAVING`, `ARRIVING` or `0 MIN` text.
  BART's real-time feed reports `Leaving` for the departing train; mapping that to the arriving screen is my inference.
- **Car line**: `<n>-CAR, <d>-DOOR` (since the 3-door Fleet of the Future, about 2019). Older formats:
  `10 CAR TRAIN` (Embarcadero 2016, classic sign), `6 CARS - BOARD CENTER` (Downtown Berkeley 2019 arriving),
  `0 CAR TRAIN` (a known glitch, Commons "BART destination sign with zero car train").
- **Alignment summary**: 4-line screen = destination and car line left, minutes right. Arriving screen =
  every line centered. Service messages: `LOOK FOR DESTINATION DISPLAY...` (2011) is centered, uses a
  different 5-row font and a marquee border; `BART POLICE TIP:` (Ashby 2018) scrolls vertically and is clipped by the field edges.
- Color: red LEDs only, no amber or other colors on the classic sign.

## 7. Confidence summary

| Item | Value | Confidence |
|---|---|---|
| Matrix | 176 x 48 | high (+/-1 col) |
| Small cap height | 8 rows (+2 descender for comma) | high |
| Small stroke | 1 dot | high (LED level) |
| Small gap / space | 3 / 2 | high (fits 4 lines of text to within 0.5 col) |
| Small widths | 5 typical, O 6, A 7, M 7, I cell 3, 1 cell 5, comma cell 5 | high for observed, medium for M |
| Large cap / stroke / width / gap / space | 14 rows / 2 dots / 10 (A 11) / 2 / 8 | high (LED level, two width checks) |
| 4-line rows | 0-7, 12-19, 28-35, 40-47 | high |
| Arriving 2-line rows | 2-15, 19-32, 38-45 | high relative, medium absolute (+/-1) |
| Arriving 1-line rows | 10-23, 38-45 | medium (+/-1) |
| Horizontal margins in 4-line mode | about 1 col | medium (+/-2) |
| Line-4 comma clipping | real quirk | high |
| Glyph pixels for INFERRED letters | designed to match observed style | low-medium |

During the study, previews were rendered from the tables below (not in this repo).
The renders look thinner and more widely spaced than the photos because they have no bloom. Each lit LED in the
photos glows about 1.2-1.5 pitches wide.

## 8. Glyphs

Small glyphs are 8 rows. The comma has 10 rows (rows 8-9 are below the baseline).
Large glyphs are 14 rows.

### Small font glyphs

`A` w=7 OBSERVED (LED level, SFO-station 2019 photo)
```
...#...
...#...
..#.#..
..#.#..
.#...#.
.#####.
#.....#
#.....#
```
`B` w=5 INFERRED
```
####.
#...#
#...#
####.
#...#
#...#
#...#
####.
```
`C` w=5 OBSERVED (LED level, SFO-station 2019 photo)
```
.###.
#...#
#....
#....
#....
#....
#...#
.###.
```
`D` w=5 OBSERVED (LED level, SFO-station 2019 photo)
```
####.
#...#
#...#
#...#
#...#
#...#
#...#
####.
```
`E` w=5 INFERRED
```
#####
#....
#....
####.
#....
#....
#....
#####
```
`F` w=5 OBSERVED shape (16th St); exact dots partly inferred
```
#####
#....
#....
####.
#....
#....
#....
#....
```
`G` w=5 INFERRED
```
.###.
#...#
#....
#....
#..##
#...#
#...#
.###.
```
`H` w=5 INFERRED
```
#...#
#...#
#...#
#####
#...#
#...#
#...#
#...#
```
`I` w=3 OBSERVED shape (16th St, San Bruno (width from spacing)); exact dots partly inferred
```
.#.
.#.
.#.
.#.
.#.
.#.
.#.
.#.
```
`J` w=5 INFERRED
```
....#
....#
....#
....#
....#
....#
#...#
.###.
```
`K` w=5 INFERRED
```
#...#
#..#.
#.#..
##...
##...
#.#..
#..#.
#...#
```
`L` w=5 OBSERVED shape (16th St, San Bruno); exact dots partly inferred
```
#....
#....
#....
#....
#....
#....
#....
#####
```
`M` w=7 OBSERVED shape (16th St, San Bruno); exact dots partly inferred
```
#.....#
##...##
#.#.#.#
#..#..#
#.....#
#.....#
#.....#
#.....#
```
`N` w=5 OBSERVED shape (16th St, San Bruno); exact dots partly inferred
```
#...#
##..#
##..#
#.#.#
#.#.#
#..##
#..##
#...#
```
`O` w=6 OBSERVED (LED level, SFO-station 2019 photo)
```
.####.
#....#
#....#
#....#
#....#
#....#
#....#
.####.
```
`P` w=5 OBSERVED shape (16th St); exact dots partly inferred
```
####.
#...#
#...#
#...#
####.
#....
#....
#....
```
`Q` w=5 INFERRED
```
.###.
#...#
#...#
#...#
#...#
#.#.#
#..#.
.##.#
```
`R` w=5 OBSERVED (LED level, SFO-station 2019 photo)
```
####.
#...#
#...#
#...#
####.
#..#.
#...#
#...#
```
`S` w=5 OBSERVED shape (16th St, San Bruno); exact dots partly inferred
```
.###.
#...#
#....
.###.
....#
....#
#...#
.###.
```
`T` w=5 OBSERVED shape (16th St, San Bruno); exact dots partly inferred
```
#####
..#..
..#..
..#..
..#..
..#..
..#..
..#..
```
`U` w=5 INFERRED
```
#...#
#...#
#...#
#...#
#...#
#...#
#...#
.###.
```
`V` w=5 INFERRED
```
#...#
#...#
#...#
.#.#.
.#.#.
.#.#.
..#..
..#..
```
`W` w=7 INFERRED
```
#.....#
#.....#
#.....#
#..#..#
#..#..#
#.#.#.#
##...##
#.....#
```
`X` w=5 INFERRED
```
#...#
#...#
.#.#.
..#..
..#..
.#.#.
#...#
#...#
```
`Y` w=5 OBSERVED shape (16th St); exact dots partly inferred
```
#...#
#...#
.#.#.
..#..
..#..
..#..
..#..
..#..
```
`Z` w=5 INFERRED
```
#####
....#
...#.
..#..
..#..
.#...
#....
#####
```
`0` w=5 OBSERVED (LED level, SFO-station 2019 photo)
```
.###.
#...#
#...#
#...#
#...#
#...#
#...#
.###.
```
`1` w=5 OBSERVED (LED level, SFO-station 2019 photo)
```
..#..
.##..
..#..
..#..
..#..
..#..
..#..
..#..
```
`2` w=5 OBSERVED shape (San Bruno, 16th St); exact dots partly inferred
```
.###.
#...#
....#
...#.
..#..
.#...
#....
#####
```
`3` w=5 OBSERVED (LED level, SFO-station 2019 photo)
```
#####
...#.
..#..
...#.
....#
....#
#...#
.###.
```
`4` w=5 OBSERVED shape (San Bruno); exact dots partly inferred
```
...#.
..##.
.#.#.
#..#.
#####
...#.
...#.
...#.
```
`5` w=5 OBSERVED shape (16th St); exact dots partly inferred
```
#####
#....
#....
####.
....#
....#
#...#
.###.
```
`6` w=5 INFERRED
```
..##.
.#...
#....
####.
#...#
#...#
#...#
.###.
```
`7` w=5 INFERRED
```
#####
....#
...#.
..#..
..#..
.#...
.#...
.#...
```
`8` w=5 OBSERVED shape (San Bruno); exact dots partly inferred
```
.###.
#...#
#...#
.###.
#...#
#...#
#...#
.###.
```
`9` w=5 OBSERVED shape (San Bruno); exact dots partly inferred
```
.###.
#...#
#...#
#...#
.####
....#
...#.
.##..
```
`space` w=2 OBSERVED shape (16th St (width from spacing)); exact dots partly inferred
```
..
..
..
..
..
..
..
..
```
`comma` w=5 OBSERVED (LED level, SFO-station 2019 photo)
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
`hyphen` w=5 OBSERVED (LED level, SFO-station 2019 photo)
```
.....
.....
.....
#####
.....
.....
.....
.....
```
`slash` w=4 OBSERVED shape (San Bruno); exact dots partly inferred
```
...#
...#
..#.
..#.
.#..
.#..
#...
#...
```
`period` w=5 OBSERVED shape (16th St + San Bruno (= clipped comma)); exact dots partly inferred
```
.....
.....
.....
.....
.....
.....
.##..
.##..
```
`colon` w=4 INFERRED
```
....
.##.
.##.
....
....
.##.
.##.
....
```
`apostrophe` w=5 INFERRED
```
.##..
.##..
..#..
.#...
.....
.....
.....
.....
```
`ampersand` w=5 INFERRED
```
.##..
#..#.
#..#.
.##..
#.#.#
#..#.
#..#.
.##.#
```

### Large font glyphs

`A` w=11 OBSERVED (LED level, SFO-station 2019 photo)
```
....###....
....###....
....###....
...#####...
...##.##...
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
`B` w=10 OBSERVED 16th St BERRYESSA (shape only); dots inferred
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
`C` w=10 OBSERVED (LED level, SFO-station 2019 photo)
```
..######..
.########.
###....###
##......##
##......##
##........
##........
##........
##........
##......##
##......##
###....###
.########.
..######..
```
`D` w=10 INFERRED
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
`E` w=10 OBSERVED 16th St BERRYESSA (shape only); dots inferred
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
`F` w=10 OBSERVED (LED level, SFO-station 2019 photo)
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
`G` w=10 INFERRED
```
..######..
.########.
###....###
##......##
##........
##........
##...#####
##...#####
##......##
##......##
##......##
###....###
.########.
..######..
```
`H` w=10 OBSERVED (LED level, SFO-station 2019 photo)
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
`I` w=2 OBSERVED (LED level, SFO-station 2019 photo)
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
`J` w=10 INFERRED
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
`K` w=10 INFERRED
```
##......##
##.....##.
##....##..
##...##...
##..##....
##.##.....
####......
#####.....
##.##.....
##..##....
##...##...
##....##..
##.....##.
##......##
```
`L` w=10 INFERRED
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
`M` w=12 INFERRED
```
##........##
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
```
`N` w=10 OBSERVED (LED level, SFO-station 2019 photo)
```
###.....##
###.....##
####....##
####....##
##.##...##
##.##...##
##..##..##
##..##..##
##...##.##
##...##.##
##....####
##....####
##.....###
##.....###
```
`O` w=10 OBSERVED (LED level, SFO-station 2019 photo)
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
`P` w=10 INFERRED
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
`Q` w=10 INFERRED
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
##..##..##
##...####.
###...###.
.########.
..#####.##
```
`R` w=10 OBSERVED (LED level, SFO-station 2019 photo)
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
##....###.
##.....###
##......##
##......##
##......##
```
`S` w=10 OBSERVED (LED level, SFO-station 2019 photo)
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
`T` w=10 OBSERVED (LED level, SFO-station 2019 photo)
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
`U` w=10 INFERRED
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
`V` w=10 INFERRED
```
##......##
##......##
##......##
##......##
.##....##.
.##....##.
.##....##.
..##..##..
..##..##..
..##..##..
...####...
...####...
....##....
....##....
```
`W` w=12 INFERRED
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
`X` w=10 INFERRED
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
`Y` w=10 OBSERVED 16th St BERRYESSA (shape only); dots inferred
```
##......##
##......##
.##....##.
.##....##.
..##..##..
..##..##..
...####...
....##....
....##....
....##....
....##....
....##....
....##....
....##....
```
`Z` w=10 INFERRED
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
`0` w=10 INFERRED
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
`1` w=10 INFERRED
```
....##....
...###....
..####....
.##.##....
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
`2` w=10 INFERRED
```
..######..
.########.
###....###
##......##
........##
.......###
......###.
....####..
..####....
.###......
###.......
##........
##########
##########
```
`3` w=10 INFERRED
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
`4` w=10 INFERRED
```
......##..
.....###..
....####..
...##.##..
..##..##..
.##...##..
##....##..
##....##..
##########
##########
......##..
......##..
......##..
......##..
```
`5` w=10 INFERRED
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
`6` w=10 INFERRED
```
...#####..
..######..
.###......
###.......
##........
########..
#########.
##.....###
##......##
##......##
##......##
###....###
.########.
..######..
```
`7` w=10 INFERRED
```
##########
##########
........##
.......###
......###.
.....###..
....###...
....##....
...##.....
...##.....
...##.....
...##.....
...##.....
...##.....
```
`8` w=10 INFERRED
```
..######..
.########.
###....###
##......##
##......##
###....###
.########.
.########.
###....###
##......##
##......##
###....###
.########.
..######..
```
`9` w=10 INFERRED
```
..######..
.########.
###....###
##......##
##......##
##......##
###.....##
.#########
..########
........##
.......###
......###.
..######..
..#####...
```
`slash` w=10 INFERRED
```
........##
........##
.......##.
.......##.
......##..
.....##...
.....##...
....##....
...##.....
...##.....
..##......
.##.......
.##.......
##........
```
`hyphen` w=8 INFERRED
```
........
........
........
........
........
........
########
########
........
........
........
........
........
........
```
`space` w=8 INFERRED
```
........
........
........
........
........
........
........
........
........
........
........
........
........
........
```
