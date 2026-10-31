"""Performance de l'analyse : durée (médiane, maximum sur N passes) et mémoire au pic (échantillonnée toutes les 20 ms)."""
import subprocess
import sys
import time
from pathlib import Path

import psutil

sys.stdout.reconfigure(encoding="utf-8")
HERE = Path(__file__).parent
EXE = HERE / "target" / "release" / "spike-import.exe"
FIX = HERE / "fixtures"


def measure(name, passes):
    p = subprocess.Popen([str(EXE), "bench", str(FIX / name), str(passes)], stdout=subprocess.PIPE, text=True)
    ps, peak = psutil.Process(p.pid), 0
    while p.poll() is None:
        try:
            peak = max(peak, ps.memory_info().rss)
        except psutil.Error:
            pass
        time.sleep(0.02)
    med, mx = p.stdout.read().split()
    return float(med), float(mx), peak / 1e6


if __name__ == "__main__":
    for name, passes in (("perf-2000.xlsx", 10), ("limit-5000.xlsx", 10), ("size-4mb-ok.xlsx", 5), ("reference.xlsx", 10)):
        med, mx, peak = measure(name, passes)
        size = (FIX / name).stat().st_size / 1048576
        print(f"{name:<18} {size:5.2f} Mo : {passes} passes, médiane {med:7.1f} ms, maximum {mx:7.1f} ms, mémoire au pic {peak:6.0f} Mo")
