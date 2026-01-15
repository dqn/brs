# BMS (Be-Music Source) Format Specification

This document summarizes the BMS file format specification for implementing a BMS player.

## Overview

BMS is a plain text script format for rhythm games, created by Urao Yane and NBK in 1998 for BM98 (a Beatmania simulator).

- Official specification: https://bm98.yaneu.com/bm98/bmsformat.html
- Comprehensive reference: https://hitkey.nekokan.dyndns.info/cmds.htm

## File Structure

```
# Header Field
#PLAYER 1
#TITLE Song Name
#ARTIST Artist
#BPM 150
#WAV01 kick.wav
...

# Main Data Field
#00101:0001000100010001
#00111:01020304
...
```

## Header Commands

| Command | Description | Example |
|---------|-------------|---------|
| `#PLAYER n` | 1=Single, 2=Two Play, 3=Double | `#PLAYER 1` |
| `#TITLE text` | Song title | `#TITLE My Song` |
| `#ARTIST text` | Artist name | `#ARTIST Composer` |
| `#GENRE text` | Genre classification | `#GENRE Techno` |
| `#BPM n` | Beats per minute (default: 130) | `#BPM 150` |
| `#PLAYLEVEL n` | Difficulty level | `#PLAYLEVEL 7` |
| `#RANK n` | Judge difficulty (0-3) | `#RANK 2` |
| `#TOTAL n` | Gauge increase rate | `#TOTAL 300` |
| `#WAVxx file` | Sound definition (xx: 01-ZZ) | `#WAV01 kick.wav` |
| `#BMPxx file` | Image definition (xx: 01-ZZ) | `#BMP01 bg.png` |
| `#BPMxx n` | Extended BPM value | `#BPM01 155.5` |
| `#STOPxx n` | Stop duration (1/192 notes) | `#STOP01 48` |
| `#LNTYPE n` | Long note type (1=RDM) | `#LNTYPE 1` |
| `#LNOBJ xx` | Long note end marker | `#LNOBJ ZZ` |

## Main Data Format

```
#XXXYY:ZZZZZZ...
```

- `XXX`: Measure number (000-999)
- `YY`: Channel number (00-ZZ in base-36)
- `ZZZZZZ`: Object sequence (pairs of base-36 digits)

### Object ID

- Range: 00-ZZ (base-36, 0-9 and A-Z)
- `00` = rest (no object)
- `01`-`ZZ` = reference to #WAVxx/#BMPxx etc.

### Sequence Length

The sequence is evenly divided across the measure. For example:

```
#00011:01020300        # 4 objects at 1/4, 2/4, 3/4, 4/4 (last is rest)
#00111:0102030405060708  # 8 objects (1/8 intervals)
```

## Channel Numbers

### System Channels (01-09)

| Channel | Description |
|---------|-------------|
| `01` | BGM (background keysounds) |
| `02` | Measure length (1.0 = 4/4) |
| `03` | BPM change (01-FF = 1-255) |
| `04` | BGA base layer |
| `05` | Extended objects |
| `06` | BGA poor layer |
| `07` | BGA overlay layer |
| `08` | Extended BPM (via #BPMxx) |
| `09` | Stop sequence (via #STOPxx) |

### Player 1 Channels (BMS 5-key)

| Channel | Key |
|---------|-----|
| `11` | Key 1 |
| `12` | Key 2 |
| `13` | Key 3 |
| `14` | Key 4 |
| `15` | Key 5 |
| `16` | Scratch |
| `17` | Free zone |

### Player 1 Channels (BME 7-key)

| Channel | Key |
|---------|-----|
| `18` | Key 6 |
| `19` | Key 7 |

### Player 2 Channels

| Channel | Key |
|---------|-----|
| `21-27` | Player 2 keys (mirrors 11-17) |
| `28-29` | Player 2 keys 6-7 |

### Invisible Channels

| Channel | Description |
|---------|-------------|
| `31-39` | Player 1 invisible (sound only) |
| `41-49` | Player 2 invisible |

### Long Note Channels (RDM-type)

| Channel | Description |
|---------|-------------|
| `51-59` | Player 1 long notes |
| `61-69` | Player 2 long notes |

Start point: first non-00 object
End point: second non-00 object

### Landmine Channels

| Channel | Description |
|---------|-------------|
| `D1-D9` | Player 1 mines |
| `E1-E9` | Player 2 mines |

## File Extensions

| Extension | Description |
|-----------|-------------|
| `.bms` | Standard 5-key format |
| `.bme` | 7-key extension |
| `.bml` | Long note format |
| `.pms` | Pop'n Music 9-key format |

## Timing Calculation

### BPM to Milliseconds

```
ms_per_beat = 60000 / BPM
ms_per_measure = ms_per_beat * 4 * measure_length
```

### Position to Time

For a note at measure M, position P (0.0-1.0):

1. Sum milliseconds for all complete measures before M
2. Add `P * ms_per_measure` for current measure
3. Handle BPM changes and stops along the way

### Stop Sequences

Stop duration in 1/192 of a whole note:

```
stop_ms = (stop_value / 192) * 4 * ms_per_beat
```

## Long Note Types

### RDM-type (#LNTYPE 1)

- Channels 51-59 for P1, 61-69 for P2
- First non-00 = start, second non-00 = end
- Default in most implementations

### LNOBJ-type

- Uses regular note channels (11-19, etc.)
- Note with #LNOBJ-defined ID marks end
- Previous note in same channel is start

## References

- [BMS Format Specification (Official)](https://bm98.yaneu.com/bm98/bmsformat.html)
- [BMS Command Memo (Comprehensive)](https://hitkey.nekokan.dyndns.info/cmds.htm)
- [bmspec (Executable Specification)](https://github.com/bemusic/bmspec)
- [Wikipedia: Be-Music Source](https://en.wikipedia.org/wiki/Be-Music_Source)
