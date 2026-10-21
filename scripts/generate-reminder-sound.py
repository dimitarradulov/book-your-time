"""BYT's original synthesized chime uses no third-party recordings.

Run with python3 scripts/generate-reminder-sound.py from the repository root.
"""

import math
from pathlib import Path
import struct
import wave


SAMPLE_RATE = 44_100
DURATION = 0.72
NOTES = ((0.0, 0.24, 1046.5), (0.32, 0.36, 1318.5))


def sample(time):
    value = 0.0
    for start, duration, frequency in NOTES:
        elapsed = time - start
        if 0 <= elapsed < duration:
            attack = min(elapsed / 0.004, 1.0)
            release = min((duration - elapsed) / 0.03, 1.0)
            envelope = attack * release * math.exp(-7 * elapsed)
            tone = math.sin(math.tau * frequency * elapsed)
            tone += 0.25 * math.sin(math.tau * frequency * 2 * elapsed)
            value += 0.65 * envelope * tone
    return round(value * 32767)


destination = Path(__file__).resolve().parents[1] / "src-tauri/sounds/reminder.wav"
destination.parent.mkdir(parents=True, exist_ok=True)
with wave.open(str(destination), "wb") as sound:
    sound.setnchannels(1)
    sound.setsampwidth(2)
    sound.setframerate(SAMPLE_RATE)
    sound.writeframes(b"".join(
        struct.pack("<h", sample(index / SAMPLE_RATE))
        for index in range(round(DURATION * SAMPLE_RATE))
    ))
